use crate::include::core::SkPathTypes::SkPathVerb;
use crate::src::core::SkPathEnums::*;
use crate::src::core::SkPoint::SkPoint;
pub struct SkPathPriv;

// Source: src/core/SkPathPriv.cpp:21-23
impl SkPathPriv {
    pub fn GenIDChangeListenersCount(path: &crate::src::core::SkPath::SkPath) -> i32 {
        path.fPathData.genIDChangeListenerCount()
    }
}
// Source: src/core/SkPathPriv.cpp:25-31
impl SkPathPriv {
    pub fn AddGenIDChangeListener(
        path: &crate::src::core::SkPath::SkPath,
        listener: Option<
            std::sync::Arc<dyn crate::src::core::SkIDChangeListener::SkIDChangeListener>,
        >,
    ) {
        if !crate::src::core::SkPathData::SkPathDataRef::ptr_eq(
            &path.fPathData,
            crate::src::core::SkPath::SkPath::PeekErrorSingleton(),
        ) {
            path.fPathData.addGenIDChangeListener(listener);
        }
    }
}
// Source: src/core/SkPathPriv.cpp:81-83
fn rect_make_dir(dx: f32, dy: f32) -> i32 {
    (dx != 0.0) as i32 | (((dx > 0.0 || dy > 0.0) as i32) << 1)
}

// Source: src/core/SkPathPriv.cpp:87-143
fn trivial_rect(pts: &[SkPoint], vbs: &[SkPathVerb]) -> Option<RectContour> {
    use crate::include::core::SkPathTypes::SkPathDirection;
    const VERBS: [SkPathVerb; 5] = [
        SkPathVerb::kMove,
        SkPathVerb::kLine,
        SkPathVerb::kLine,
        SkPathVerb::kLine,
        SkPathVerb::kClose,
    ];
    if pts.len() != 4 || vbs != VERBS {
        return None;
    }
    let v0 = pts[1] - pts[0];
    let v1 = pts[2] - pts[1];
    let v2 = pts[3] - pts[2];
    let v3 = pts[0] - pts[3];
    let orthogonal =
        |a: SkPoint, b: SkPoint| ((a.fX == 0.0) ^ (b.fX == 0.0)) & ((a.fY == 0.0) ^ (b.fY == 0.0));
    if !(((v0.fX == 0.0) ^ (v0.fY == 0.0))
        & orthogonal(v0, v1)
        & orthogonal(v1, v2)
        & orthogonal(v2, v3))
    {
        return None;
    }
    let rect =
        crate::src::core::SkRect::SkRect::MakeLTRB(pts[0].fX, pts[0].fY, pts[2].fX, pts[2].fY)
            .makeSorted();
    let dir = if SkPoint::cross_product(v0, v1) > 0.0 {
        SkPathDirection::kCW
    } else {
        SkPathDirection::kCCW
    };
    Some(RectContour {
        fRect: rect,
        fIsClosed: true,
        fDirection: dir,
        fPointsConsumed: pts.len(),
        fVerbsConsumed: vbs.len(),
    })
}

// Source: src/core/SkPathPriv.cpp:145-297
impl SkPathPriv {
    pub fn IsRectContour(
        points: &[SkPoint],
        verbs: &[SkPathVerb],
        segment_mask: u32,
        allow_partial: bool,
    ) -> Option<RectContour> {
        use crate::include::core::SkPathTypes::SkPathDirection;
        if segment_mask != 1 || points.len() < 4 || verbs.len() < 4 {
            return None;
        }
        if let Some(rect) = trivial_rect(points, verbs) {
            return Some(rect);
        }
        let mut curr = 0usize;
        let mut corners = 0usize;
        let mut line_start = SkPoint::make(0.0, 0.0);
        let mut first = None;
        let mut last = None;
        let mut first_corner = SkPoint::default();
        let mut third_corner = SkPoint::default();
        let mut pi = 0;
        let mut save = None;
        let mut directions = [-1i32; 5];
        let mut closed_or_moved = false;
        let mut auto_close = false;
        let mut insert_close = false;
        while curr < verbs.len() && (!allow_partial || !auto_close) {
            let verb = if insert_close {
                SkPathVerb::kClose
            } else {
                verbs[curr]
            };
            match verb {
                SkPathVerb::kClose | SkPathVerb::kLine => {
                    if verb == SkPathVerb::kClose {
                        save = Some(pi);
                        auto_close = true;
                        insert_close = false;
                    } else {
                        last = Some(pi);
                    }
                    let line_end = if verb == SkPathVerb::kClose {
                        points[first.unwrap()]
                    } else {
                        let p = points[pi];
                        pi += 1;
                        p
                    };
                    let delta = line_end - line_start;
                    if delta.fX != 0.0 && delta.fY != 0.0 {
                        return None;
                    }
                    if !delta.is_finite() {
                        return None;
                    }
                    if line_start != line_end {
                        let next = rect_make_dir(delta.fX, delta.fY);
                        if corners == 0 {
                            directions[0] = next;
                            corners = 1;
                            closed_or_moved = false;
                            line_start = line_end;
                        } else {
                            if closed_or_moved {
                                return None;
                            }
                            if !(auto_close && next == directions[0]) {
                                closed_or_moved = auto_close;
                                if directions[corners - 1] == next {
                                    if corners == 3 && verb == SkPathVerb::kLine {
                                        third_corner = line_end;
                                    }
                                    line_start = line_end;
                                } else {
                                    directions[corners] = next;
                                    corners += 1;
                                    match corners {
                                        2 => first_corner = line_start,
                                        3 => {
                                            if (directions[0] ^ directions[2]) != 2 {
                                                return None;
                                            }
                                            third_corner = line_end;
                                        }
                                        4 => {
                                            if (directions[1] ^ directions[3]) != 2 {
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
                SkPathVerb::kQuad | SkPathVerb::kConic | SkPathVerb::kCubic => return None,
                SkPathVerb::kMove => {
                    if allow_partial && !auto_close && directions[0] >= 0 {
                        insert_close = true;
                        curr -= 1;
                        continue;
                    }
                    if corners == 0 {
                        first = Some(pi);
                    } else {
                        let close = points[first.unwrap()] - points[last.unwrap()];
                        if close.fX != 0.0 && close.fY != 0.0 {
                            return None;
                        }
                    }
                    line_start = points[pi];
                    pi += 1;
                    closed_or_moved = true;
                }
            }
            curr += 1;
        }
        if corners < 3 || corners > 4 {
            return None;
        }
        let close = points[first.unwrap()] - points[last.unwrap()];
        if close.fX != 0.0 && close.fY != 0.0 {
            return None;
        }
        let rect = crate::src::core::SkRect::SkRect::MakeLTRB(
            first_corner.fX,
            first_corner.fY,
            third_corner.fX,
            third_corner.fY,
        )
        .makeSorted();
        Some(RectContour {
            fRect: rect,
            fIsClosed: auto_close,
            fDirection: if directions[0] == ((directions[1] + 1) & 3) {
                SkPathDirection::kCW
            } else {
                SkPathDirection::kCCW
            },
            fPointsConsumed: save.unwrap_or(0),
            fVerbsConsumed: curr,
        })
    }
}

// Source: src/core/SkPathPriv.cpp:299-344
impl SkPathPriv {
    pub fn IsNestedFillRects(
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        rects: Option<&mut [crate::src::core::SkRect::SkRect; 2]>,
        dirs: Option<&mut [crate::include::core::SkPathTypes::SkPathDirection; 2]>,
    ) -> bool {
        let Some(first) =
            Self::IsRectContour(raw.fPoints, raw.fVerbs, raw.fSegmentMask as u32, true)
        else {
            return false;
        };
        let pts = &raw.fPoints[first.fPointsConsumed..];
        let verbs = &raw.fVerbs[first.fVerbsConsumed..];
        if let Some(second) = Self::IsRectContour(pts, verbs, raw.fSegmentMask as u32, false) {
            if first.fRect.contains(&second.fRect) {
                if let Some(out) = rects {
                    *out = [first.fRect, second.fRect];
                }
                if let Some(out) = dirs {
                    *out = [first.fDirection, second.fDirection];
                }
                return true;
            }
            if second.fRect.contains(&first.fRect) {
                if let Some(out) = rects {
                    *out = [second.fRect, first.fRect];
                }
                if let Some(out) = dirs {
                    *out = [second.fDirection, first.fDirection];
                }
                return true;
            }
        }
        false
    }
}
// Source: src/core/SkPathPriv.cpp:346-360
impl SkPathPriv {
    pub fn DrawArcIsConvex(
        sweepAngle: f32,
        arcType: crate::include::core::SkArc::Type,
        isFillNoPathEffect: bool,
    ) -> bool {
        if isFillNoPathEffect && sweepAngle.abs() >= 360.0 {
            return true;
        }
        if arcType == crate::include::core::SkArc::Type::kWedge {
            return sweepAngle.abs() <= 180.0;
        }
        sweepAngle.abs() <= 360.0
    }
}
// Source: src/core/SkPathPriv.cpp:362-416
impl SkPathPriv {
    pub fn CreateDrawArcPath(
        arc: &crate::include::core::SkArc::SkArc,
        isFillNoPathEffect: bool,
    ) -> crate::src::core::SkPath::SkPath {
        let oval = arc.fOval;
        let mut start = arc.fStartAngle;
        let mut sweep = arc.fSweepAngle;
        crate::SkASSERT!(@source b"src/core/SkPathPriv.cpp\0".as_ptr(),365; @condition b"!oval.isEmpty()\0".as_ptr(); !oval.isEmpty());
        debug_assert!(sweep != 0.0);
        if sweep.abs() > 3600.0 {
            sweep = 3600.0f32.copysign(sweep) + sweep % 360.0;
        }
        let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
        builder.setFillType(crate::include::core::SkPathTypes::SkPathFillType::kWinding);
        builder.setIsVolatile(true);
        if isFillNoPathEffect && sweep.abs() >= 360.0 {
            builder.addOval(
                &oval,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                1,
            );
            debug_assert!(Self::DrawArcIsConvex(
                sweep,
                crate::include::core::SkArc::Type::kArc,
                isFillNoPathEffect
            ));
            return builder.detach(None);
        }
        if arc.isWedge() {
            builder.moveTo(oval.center());
        }
        let first_dir = if sweep > 0.0 {
            SkPathFirstDirection::kCW
        } else {
            SkPathFirstDirection::kCCW
        };
        let convex = Self::DrawArcIsConvex(sweep, arc.fType, isFillNoPathEffect);
        let mut force_move = !arc.isWedge();
        while sweep <= -360.0 {
            builder.arcTo(&oval, start, -180.0, force_move);
            start -= 180.0;
            builder.arcTo(&oval, start, -180.0, false);
            start -= 180.0;
            force_move = false;
            sweep += 360.0;
        }
        while sweep >= 360.0 {
            builder.arcTo(&oval, start, 180.0, force_move);
            start += 180.0;
            builder.arcTo(&oval, start, 180.0, false);
            start += 180.0;
            force_move = false;
            sweep -= 360.0;
        }
        builder.arcTo(&oval, start, sweep, force_move);
        if arc.isWedge() {
            builder.close();
        }
        let path = builder.detach(None);
        let convexity = if convex {
            SkPathFirstDirection_ToConvexity(first_dir)
        } else {
            SkPathConvexity::kConcave
        };
        path.setConvexity(convexity);
        path
    }
}
// Source: src/core/SkPathPriv.cpp:420-420
fn sign(x: f32) -> i32 {
    (x < 0.0) as i32
}

// Source: src/core/SkPathPriv.cpp:421-421
const kValueNeverReturnedBySign: i32 = 2;

// Source: src/core/SkPathPriv.cpp:423-430
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DirChange {
    kUnknown_DirChange,
    kLeft_DirChange,
    kRight_DirChange,
    kStraight_DirChange,
    kBackwards_DirChange,
    kInvalid_DirChange,
}

// Source: src/core/SkPathPriv.cpp:436-436
impl Convexicator {
    fn getFirstDirection(&self) -> SkPathFirstDirection {
        self.fFirstDirection
    }
}

// Source: src/core/SkPathPriv.cpp:438-441
impl Convexicator {
    fn setMovePt(&mut self, pt: SkPoint) {
        self.fFirstPt = pt;
        self.fLastPt = pt;
        self.fExpectedDir = DirChange::kInvalid_DirChange;
    }
}

// Source: src/core/SkPathPriv.cpp:443-457
impl Convexicator {
    fn addPt(&mut self, pt: SkPoint) -> bool {
        if self.fLastPt == pt {
            return true;
        }
        if self.fFirstPt == self.fLastPt
            && self.fExpectedDir == DirChange::kInvalid_DirChange
            && self.fLastVec.equals(0.0, 0.0)
        {
            self.fLastVec = pt - self.fLastPt;
            self.fFirstVec = self.fLastVec;
        } else if !self.addVec(pt - self.fLastPt) {
            return false;
        }
        self.fLastPt = pt;
        true
    }
}

// Source: src/core/SkPathPriv.cpp:459-498
impl Convexicator {
    fn IsConcaveBySign(points: &[SkPoint]) -> bool {
        if points.len() <= 3 {
            return false;
        }
        let mut curr_pt = points[0];
        let first_pt = curr_pt;
        let mut dxes = 0;
        let mut dyes = 0;
        let mut last_sx = kValueNeverReturnedBySign;
        let mut last_sy = kValueNeverReturnedBySign;
        for outer in 0..2 {
            let input = if outer == 0 {
                &points[1..]
            } else {
                core::slice::from_ref(&first_pt)
            };
            for &point in input {
                let vec = point - curr_pt;
                if !vec.is_zero() {
                    if !vec.is_finite() {
                        return true;
                    }
                    let sx = sign(vec.fX);
                    let sy = sign(vec.fY);
                    dxes += (sx != last_sx) as i32;
                    dyes += (sy != last_sy) as i32;
                    if dxes > 3 || dyes > 3 {
                        return true;
                    }
                    last_sx = sx;
                    last_sy = sy;
                }
                curr_pt = point;
                if outer != 0 {
                    break;
                }
            }
        }
        false
    }
}

// Source: src/core/SkPathPriv.cpp:500-505
impl Convexicator {
    fn close(&mut self) -> bool {
        self.addPt(self.fFirstPt) && self.addVec(self.fFirstVec)
    }
}

// Source: src/core/SkPathPriv.cpp:507-509
impl Convexicator {
    fn isFinite(&self) -> bool {
        self.fIsFinite
    }
}

// Source: src/core/SkPathPriv.cpp:511-513
impl Convexicator {
    fn reversals(&self) -> i32 {
        self.fReversals
    }
}

// Source: src/core/SkPathPriv.cpp:516-525
impl Convexicator {
    fn directionChange(&self, cur_vec: SkPoint) -> DirChange {
        let cross = SkPoint::cross_product(self.fLastVec, cur_vec);
        if !cross.is_finite() {
            return DirChange::kUnknown_DirChange;
        }
        if cross == 0.0 {
            return if self.fLastVec.dot(cur_vec) < 0.0 {
                DirChange::kBackwards_DirChange
            } else {
                DirChange::kStraight_DirChange
            };
        }
        if cross > 0.0 {
            DirChange::kRight_DirChange
        } else {
            DirChange::kLeft_DirChange
        }
    }
}

// Source: src/core/SkPathPriv.cpp:527-558
impl Convexicator {
    fn addVec(&mut self, cur_vec: SkPoint) -> bool {
        let dir = self.directionChange(cur_vec);
        match dir {
            DirChange::kLeft_DirChange | DirChange::kRight_DirChange => {
                if self.fExpectedDir == DirChange::kInvalid_DirChange {
                    self.fExpectedDir = dir;
                    self.fFirstDirection = if dir == DirChange::kRight_DirChange {
                        SkPathFirstDirection::kCW
                    } else {
                        SkPathFirstDirection::kCCW
                    };
                } else if dir != self.fExpectedDir {
                    self.fFirstDirection = SkPathFirstDirection::kUnknown;
                    return false;
                }
                self.fLastVec = cur_vec;
            }
            DirChange::kStraight_DirChange => {}
            DirChange::kBackwards_DirChange => {
                self.fLastVec = cur_vec;
                self.fReversals += 1;
                return self.fReversals < 3;
            }
            DirChange::kUnknown_DirChange => {
                self.fIsFinite = false;
                return false;
            }
            DirChange::kInvalid_DirChange => {
                eprintln!("Use of invalid direction change flag");
                std::process::abort();
            }
        }
        true
    }
}

// Source: src/core/SkPathPriv.cpp:560-569
struct Convexicator {
    fFirstPt: SkPoint,
    fFirstVec: SkPoint,
    fLastPt: SkPoint,
    fLastVec: SkPoint,
    fExpectedDir: DirChange,
    fFirstDirection: SkPathFirstDirection,
    fReversals: i32,
    fIsFinite: bool,
}
impl Default for Convexicator {
    fn default() -> Self {
        Self {
            fFirstPt: SkPoint::make(0.0, 0.0),
            fFirstVec: SkPoint::make(0.0, 0.0),
            fLastPt: SkPoint::make(0.0, 0.0),
            fLastVec: SkPoint::make(0.0, 0.0),
            fExpectedDir: DirChange::kInvalid_DirChange,
            fFirstDirection: SkPathFirstDirection::kUnknown,
            fReversals: 0,
            fIsFinite: true,
        }
    }
}

// Source: src/core/SkPathPriv.cpp:572-582
fn trim_trailing_moves<'a, 'b>(pts: &mut &'a [SkPoint], vbs: &mut &'b [SkPathVerb]) {
    let mut vb_count = vbs.len();
    while vb_count > 0 && vbs[vb_count - 1] == SkPathVerb::kMove {
        vb_count -= 1;
    }
    let delta = vbs.len() - vb_count;
    if delta != 0 {
        crate::SkASSERT!(@source b"src/core/SkPathPriv.cpp\0".as_ptr(),578; @condition b"pts.size() >= delta\0".as_ptr(); pts.len() >= delta);
        *pts = &pts[..pts.len() - delta];
        *vbs = &vbs[..vbs.len() - delta];
    }
}

// Source: src/core/SkPathPriv.cpp:584-657
impl SkPathPriv {
    pub fn ComputeConvexity(
        mut points: &[SkPoint],
        mut vbs: &[SkPathVerb],
        conic_weights: &[f32],
    ) -> SkPathConvexity {
        debug_assert!(crate::src::core::SkRect::SkRect::Bounds(points).is_some());
        trim_trailing_moves(&mut points, &mut vbs);
        if vbs.is_empty() {
            return SkPathConvexity::kConvex_Degenerate;
        }
        if Convexicator::IsConcaveBySign(points) {
            return SkPathConvexity::kConcave;
        }
        let mut contour_count = 0;
        let mut needs_close = false;
        let mut state = Convexicator::default();
        let mut iter = crate::src::core::SkPathIter::SkPathIter::new(points, vbs, conic_weights);
        while let Some(rec) = iter.next() {
            let verb = rec.fVerb;
            let pts = rec.fPoints;
            if contour_count == 0 {
                if verb == SkPathVerb::kMove {
                    state.setMovePt(pts[0]);
                } else {
                    contour_count += 1;
                    needs_close = true;
                }
            }
            if contour_count == 1 {
                if verb == SkPathVerb::kClose || verb == SkPathVerb::kMove {
                    if !state.close() {
                        return SkPathConvexity::kConcave;
                    }
                    needs_close = false;
                    contour_count += 1;
                } else {
                    let count = Self::PtsInVerb(verb as u32);
                    crate::SkASSERT!(@source b"src/core/SkPathPriv.cpp\0".as_ptr(),631; @condition b"count > 0\0".as_ptr(); count > 0);
                    for i in 1..=count {
                        if !state.addPt(pts[i]) {
                            return SkPathConvexity::kConcave;
                        }
                    }
                }
            } else if verb != SkPathVerb::kMove {
                return SkPathConvexity::kConcave;
            }
        }
        if needs_close && !state.close() {
            return SkPathConvexity::kConcave;
        }
        let first_dir = state.getFirstDirection();
        if first_dir == SkPathFirstDirection::kUnknown && state.reversals() >= 3 {
            return SkPathConvexity::kConcave;
        }
        SkPathFirstDirection_ToConvexity(first_dir)
    }
}

// Source: src/core/SkPathPriv.cpp:662-681
fn cross_prod(p0: SkPoint, p1: SkPoint, p2: SkPoint) -> f32 {
    let mut cross = SkPoint::cross_product(p1 - p0, p2 - p0);
    if cross == 0.0 {
        let (x0, y0) = (p0.fX as f64, p0.fY as f64);
        let (x1, y1) = (p1.fX as f64, p1.fY as f64);
        let (x2, y2) = (p2.fX as f64, p2.fY as f64);
        cross = ((x1 - x0) * (y2 - y0) - (y1 - y0) * (x2 - x0)) as f32;
    }
    cross
}
// Source: src/core/SkPathPriv.cpp:684-696
fn find_max_y(pts: &[SkPoint]) -> usize {
    debug_assert!(!pts.is_empty());
    let mut max = pts[0].fY;
    let mut index = 0;
    for i in 1..pts.len() {
        if pts[i].fY > max {
            max = pts[i].fY;
            index = i;
        }
    }
    index
}
// Source: src/core/SkPathPriv.cpp:698-710
fn find_diff_pt(pts: &[SkPoint], index: usize, inc: usize) -> usize {
    let mut i = index;
    loop {
        i = (i + inc) % pts.len();
        if i == index || pts[index] != pts[i] {
            break;
        }
    }
    i
}
// Source: src/core/SkPathPriv.cpp:716-738
fn find_min_max_x_at_y(pts: &[SkPoint], index: usize) -> (usize, usize) {
    let y = pts[index].fY;
    let mut min = pts[index].fX;
    let mut max = min;
    let mut min_index = index;
    let mut max_index = index;
    for i in index + 1..pts.len() {
        if pts[i].fY != y {
            break;
        }
        let x = pts[i].fX;
        if x < min {
            min = x;
            min_index = i;
        } else if x > max {
            max = x;
            max_index = i;
        }
    }
    (min_index, max_index)
}
// Source: src/core/SkPathPriv.cpp:740-742
fn crossToDir(cross: f32) -> SkPathFirstDirection {
    if cross > 0.0 {
        SkPathFirstDirection::kCW
    } else {
        SkPathFirstDirection::kCCW
    }
}
// Source: src/core/SkPathPriv.cpp:744-762
struct DirectionContourIter<'a> {
    points: &'a [SkPoint],
    verbs: &'a [SkPathVerb],
    conics: &'a [f32],
    point_index: usize,
    verb_index: usize,
    conic_index: usize,
    count: usize,
    done: bool,
    contour_counter: usize,
}
// Source: src/core/SkPathPriv.cpp:764-774
impl<'a> DirectionContourIter<'a> {
    fn new(points: &'a [SkPoint], verbs: &'a [SkPathVerb], conics: &'a [f32]) -> Self {
        let mut iter = Self {
            points,
            verbs,
            conics,
            point_index: 0,
            verb_index: 0,
            conic_index: 0,
            count: 0,
            done: false,
            contour_counter: 0,
        };
        iter.next();
        iter
    }
}
// Source: src/core/SkPathPriv.cpp:776-815
impl DirectionContourIter<'_> {
    fn next(&mut self) {
        if self.verb_index >= self.verbs.len() {
            self.done = true;
        }
        if self.done {
            return;
        }
        self.point_index += self.count;
        debug_assert_eq!(self.verbs[self.verb_index], SkPathVerb::kMove);
        let mut count = 1;
        let mut i = self.verb_index + 1;
        while i < self.verbs.len() {
            match self.verbs[i] {
                SkPathVerb::kMove => break,
                SkPathVerb::kLine => count += 1,
                SkPathVerb::kConic => {
                    self.conic_index += 1;
                    count += 2;
                }
                SkPathVerb::kQuad => count += 2,
                SkPathVerb::kCubic => count += 3,
                SkPathVerb::kClose => {}
            }
            i += 1;
        }
        self.count = count;
        self.verb_index = i;
        self.contour_counter += 1;
    }
}
// Source: src/core/SkPathPriv.cpp:825-895
impl SkPathPriv {
    pub fn ComputeFirstDirection(
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
    ) -> SkPathFirstDirection {
        let mut iter = DirectionContourIter::new(raw.fPoints, raw.fVerbs, raw.fConics);
        let mut ymax = raw.fBounds.fTop;
        let mut ymax_cross = 0.0;
        while !iter.done {
            let n = iter.count;
            if n >= 3 {
                let pts = &iter.points[iter.point_index..iter.point_index + n];
                let index = find_max_y(pts);
                if !(pts[index].fY < ymax) {
                    let mut cross = 0.0;
                    let mut try_cross = true;
                    if pts[(index + 1) % n].fY == pts[index].fY {
                        let (min, max) = find_min_max_x_at_y(pts, index);
                        if min != max {
                            debug_assert_eq!(pts[min].fY, pts[index].fY);
                            debug_assert_eq!(pts[max].fY, pts[index].fY);
                            debug_assert!(pts[min].fX <= pts[max].fX);
                            cross = (min as i32 - max as i32) as f32;
                            try_cross = false;
                        }
                    }
                    if try_cross {
                        let prev = find_diff_pt(pts, index, n - 1);
                        if prev == index {
                            iter.next();
                            continue;
                        }
                        let next = find_diff_pt(pts, index, 1);
                        crate::SkASSERT!(@source b"src/core/SkPathPriv.cpp\0".as_ptr(),876; @condition b"next != index\0".as_ptr(); (next)!=(index));
                        cross = cross_prod(pts[prev], pts[index], pts[next]);
                        if cross == 0.0
                            && pts[prev].fY == pts[index].fY
                            && pts[next].fY == pts[index].fY
                        {
                            cross = pts[index].fX - pts[next].fX;
                        }
                    }
                    if cross != 0.0 {
                        ymax = pts[index].fY;
                        ymax_cross = cross;
                    }
                }
            }
            iter.next();
        }
        if ymax_cross != 0.0 {
            crossToDir(ymax_cross)
        } else {
            SkPathFirstDirection::kUnknown
        }
    }
}
// Source: src/core/SkPathPriv.cpp:899-901
fn poly_eval2(a: f32, b: f32, c: f32, t: f32) -> f32 {
    (a * t + b) * t + c
}

// Source: src/core/SkPathPriv.cpp:903-905
fn poly_eval3(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    ((a * t + b) * t + c) * t + d
}

// Source: src/core/SkPathPriv.cpp:907-911
fn between(a: f32, b: f32, c: f32) -> bool {
    debug_assert!(
        ((a <= b && b <= c) || (a >= b && b >= c)) == ((a - b) * (c - b) <= 0.0)
            || (nearly_zero(a) && nearly_zero(b) && nearly_zero(c))
    );
    (a - b) * (c - b) <= 0.0
}

// Source: src/core/SkPathPriv.cpp:913-920
fn eval_cubic_pts(c0: f32, c1: f32, c2: f32, c3: f32, t: f32) -> f32 {
    let a = c3 + 3.0 * (c1 - c2) - c0;
    let b = 3.0 * (c2 - c1 - c1 + c0);
    let c = 3.0 * (c1 - c0);
    let d = c0;
    poly_eval3(a, b, c, d, t)
}

// Source: src/core/SkPathPriv.cpp:922-932
fn find_minmax<const N: usize>(pts: &[SkPoint]) -> (f32, f32) {
    let mut min = pts[0].fX;
    let mut max = min;
    for p in &pts[1..N] {
        min = if p.fX < min { p.fX } else { min };
        max = if max < p.fX { p.fX } else { max };
    }
    (min, max)
}

// Source: src/core/SkPathPriv.cpp:934-940
fn checkOnCurve(x: f32, y: f32, start: SkPoint, end: SkPoint) -> bool {
    if start.fY == end.fY {
        between(start.fX, x, end.fX) && x != end.fX
    } else {
        x == start.fX && y == start.fY
    }
}

// Source: src/core/SkPathPriv.cpp:942-986
fn winding_mono_cubic(pts: &[SkPoint; 4], x: f32, y: f32, on: &mut i32) -> i32 {
    let mut y0 = pts[0].fY;
    let mut y3 = pts[3].fY;
    let mut dir = 1;
    if y0 > y3 {
        core::mem::swap(&mut y0, &mut y3);
        dir = -1;
    }
    if y < y0 || y > y3 {
        return 0;
    }
    if checkOnCurve(x, y, pts[0], pts[3]) {
        *on += 1;
        return 0;
    }
    if y == y3 {
        return 0;
    }
    let (min, max) = find_minmax::<4>(pts);
    if x < min {
        return 0;
    }
    if x > max {
        return dir;
    }
    let mut t = 0.0;
    if !SkCubicClipper::ChopMonoAtY(pts, y, &mut t) {
        return 0;
    }
    let xt = eval_cubic_pts(pts[0].fX, pts[1].fX, pts[2].fX, pts[3].fX, t);
    if nearly_equal(xt, x) {
        if x != pts[3].fX || y != pts[3].fY {
            *on += 1;
            return 0;
        }
    }
    if xt < x {
        dir
    } else {
        0
    }
}

// Source: src/core/SkPathPriv.cpp:988-996
fn winding_cubic(pts: &[SkPoint], x: f32, y: f32, on: &mut i32) -> i32 {
    let mut dst = [SkPoint::default(); 10];
    let n = SkChopCubicAtYExtrema(pts.try_into().unwrap(), &mut dst);
    let mut w = 0;
    for i in 0..=n {
        w += winding_mono_cubic((&dst[3 * i..3 * i + 4]).try_into().unwrap(), x, y, on);
    }
    w
}

// Source: src/core/SkPathPriv.cpp:998-1006
fn conic_eval_numerator(src: &[f32; 3], w: f32, t: f32) -> f64 {
    debug_assert!(t >= 0.0 && t <= 1.0);
    let src2w = src[1] * w;
    let c = src[0];
    let a = src[2] - 2.0 * src2w + c;
    let b = 2.0 * (src2w - c);
    poly_eval2(a, b, c, t) as f64
}

// Source: src/core/SkPathPriv.cpp:1009-1014
fn conic_eval_denominator(w: f32, t: f32) -> f64 {
    let b = 2.0 * (w - 1.0);
    let c = 1.0;
    let a = -b;
    poly_eval2(a, b, c, t) as f64
}

// Source: src/core/SkPathPriv.cpp:1016-1064
fn winding_mono_conic(conic: &SkConic, x: f32, y: f32, on: &mut i32) -> i32 {
    let pts = &conic.fPts;
    let mut y0 = pts[0].fY;
    let mut y2 = pts[2].fY;
    let mut dir = 1;
    if y0 > y2 {
        core::mem::swap(&mut y0, &mut y2);
        dir = -1;
    }
    if y < y0 || y > y2 {
        return 0;
    }
    if checkOnCurve(x, y, pts[0], pts[2]) {
        *on += 1;
        return 0;
    }
    if y == y2 {
        return 0;
    }
    let mut roots = [0.0; 2];
    let mut a = pts[2].fY;
    let mut b = pts[1].fY * conic.fW - y * conic.fW + y;
    let mut c = pts[0].fY;
    a += c - 2.0 * b;
    b -= c;
    c -= y;
    let n = SkFindUnitQuadRoots(a, 2.0 * b, c, &mut roots);
    crate::SkASSERT!(@source b"src/core/SkPathPriv.cpp\0".as_ptr(),1046; @condition b"n <= 1\0".as_ptr(); n <= 1);
    let xt = if n == 0 {
        pts[(1 - dir) as usize].fX
    } else {
        let t = roots[0];
        (conic_eval_numerator(&[pts[0].fX, pts[1].fX, pts[2].fX], conic.fW, t)
            / conic_eval_denominator(conic.fW, t)) as f32
    };
    if nearly_equal(xt, x) {
        if x != pts[2].fX || y != pts[2].fY {
            *on += 1;
            return 0;
        }
    }
    if xt < x {
        dir
    } else {
        0
    }
}

// Source: src/core/SkPathPriv.cpp:1066-1076
fn is_mono_quad(y0: f32, y1: f32, y2: f32) -> bool {
    if y0 == y1 {
        return true;
    }
    if y0 < y1 {
        y1 <= y2
    } else {
        y1 >= y2
    }
}

// Source: src/core/SkPathPriv.cpp:1078-1090
fn winding_conic(pts: &[SkPoint], x: f32, y: f32, weight: f32, on: &mut i32) -> i32 {
    let conic = SkConic::new(pts.try_into().unwrap(), weight);
    let mut chopped = [SkConic::default(); 2];
    let mono = is_mono_quad(pts[0].fY, pts[1].fY, pts[2].fY) || !conic.chopAtYExtrema(&mut chopped);
    let mut w = winding_mono_conic(if mono { &conic } else { &chopped[0] }, x, y, on);
    if !mono {
        w += winding_mono_conic(&chopped[1], x, y, on);
    }
    w
}

// Source: src/core/SkPathPriv.cpp:1092-1145
fn winding_mono_quad(pts: &[SkPoint], x: f32, y: f32, on: &mut i32) -> i32 {
    let mut y0 = pts[0].fY;
    let mut y2 = pts[2].fY;
    let mut dir = 1;
    if y0 > y2 {
        core::mem::swap(&mut y0, &mut y2);
        dir = -1;
    }
    if y < y0 || y > y2 {
        return 0;
    }
    if checkOnCurve(x, y, pts[0], pts[2]) {
        *on += 1;
        return 0;
    }
    if y == y2 {
        return 0;
    }
    if false {
        if pts[0].fX > x && pts[1].fX > x && pts[2].fX > x {
            return 0;
        }
    }
    let mut roots = [0.0; 2];
    let n = SkFindUnitQuadRoots(
        pts[0].fY - 2.0 * pts[1].fY + pts[2].fY,
        2.0 * (pts[1].fY - pts[0].fY),
        pts[0].fY - y,
        &mut roots,
    );
    crate::SkASSERT!(@source b"src/core/SkPathPriv.cpp\0".as_ptr(),1124; @condition b"n <= 1\0".as_ptr(); n <= 1);
    let xt = if n == 0 {
        pts[(1 - dir) as usize].fX
    } else {
        let t = roots[0];
        let c = pts[0].fX;
        let a = pts[2].fX - 2.0 * pts[1].fX + c;
        let b = 2.0 * (pts[1].fX - c);
        poly_eval2(a, b, c, t)
    };
    if nearly_equal(xt, x) {
        if x != pts[2].fX || y != pts[2].fY {
            *on += 1;
            return 0;
        }
    }
    if xt < x {
        dir
    } else {
        0
    }
}

// Source: src/core/SkPathPriv.cpp:1147-1161
fn winding_quad(pts: &[SkPoint], x: f32, y: f32, on: &mut i32) -> i32 {
    let mut storage = [SkPoint::default(); 5];
    let mut span = pts;
    let mut n = 0;
    if !is_mono_quad(pts[0].fY, pts[1].fY, pts[2].fY) {
        n = SkChopQuadAtYExtrema(pts.try_into().unwrap(), &mut storage);
        span = &storage;
    }
    let mut w = winding_mono_quad(span, x, y, on);
    if n > 0 {
        w += winding_mono_quad(&span[2..], x, y, on);
    }
    w
}

// Source: src/core/SkPathPriv.cpp:1163-1201
fn winding_line(pts: &[SkPoint], x: f32, y: f32, on: &mut i32) -> i32 {
    let x0 = pts[0].fX;
    let mut y0 = pts[0].fY;
    let x1 = pts[1].fX;
    let mut y1 = pts[1].fY;
    let dy = y1 - y0;
    let mut dir = 1;
    if y0 > y1 {
        core::mem::swap(&mut y0, &mut y1);
        dir = -1;
    }
    if y < y0 || y > y1 {
        return 0;
    }
    if checkOnCurve(x, y, pts[0], pts[1]) {
        *on += 1;
        return 0;
    }
    if y == y1 {
        return 0;
    }
    let cross = (x1 - x0) * (y - pts[0].fY) - dy * (x - x0);
    if cross == 0.0 {
        if x != x1 || y != pts[1].fY {
            *on += 1;
        }
        dir = 0;
    } else if SkScalarSignAsInt(cross) == dir {
        dir = 0;
    }
    dir
}

// Source: src/core/SkPathPriv.cpp:1203-1229
fn tangent_cubic(pts: &[SkPoint], x: f32, y: f32, tangents: &mut Vec<SkPoint>) {
    if !between(pts[0].fY, y, pts[1].fY)
        && !between(pts[1].fY, y, pts[2].fY)
        && !between(pts[2].fY, y, pts[3].fY)
    {
        return;
    }
    if !between(pts[0].fX, x, pts[1].fX)
        && !between(pts[1].fX, x, pts[2].fX)
        && !between(pts[2].fX, x, pts[3].fX)
    {
        return;
    }
    let mut dst = [SkPoint::default(); 10];
    let n = SkChopCubicAtYExtrema(pts.try_into().unwrap(), &mut dst);
    for i in 0..=n {
        let c: &[SkPoint; 4] = (&dst[3 * i..3 * i + 4]).try_into().unwrap();
        let mut t = 0.0;
        if !SkCubicClipper::ChopMonoAtY(c, y, &mut t) {
            continue;
        }
        let xt = eval_cubic_pts(c[0].fX, c[1].fX, c[2].fX, c[3].fX, t);
        if !nearly_equal(x, xt) {
            continue;
        }
        let mut tangent = SkPoint::default();
        SkEvalCubicAt(c, t, None, Some(&mut tangent), None);
        tangents.push(tangent);
    }
}

// Source: src/core/SkPathPriv.cpp:1231-1256
fn tangent_conic(pts: &[SkPoint], x: f32, y: f32, w: f32, tangents: &mut Vec<SkPoint>) {
    if !between(pts[0].fY, y, pts[1].fY) && !between(pts[1].fY, y, pts[2].fY) {
        return;
    }
    if !between(pts[0].fX, x, pts[1].fX) && !between(pts[1].fX, x, pts[2].fX) {
        return;
    }
    let mut roots = [0.0; 2];
    let mut a = pts[2].fY;
    let mut b = pts[1].fY * w - y * w + y;
    let mut c = pts[0].fY;
    a += c - 2.0 * b;
    b -= c;
    c -= y;
    let n = SkFindUnitQuadRoots(a, 2.0 * b, c, &mut roots);
    for &t in &roots[..n] {
        let xt = (conic_eval_numerator(&[pts[0].fX, pts[1].fX, pts[2].fX], w, t)
            / conic_eval_denominator(w, t)) as f32;
        if !nearly_equal(x, xt) {
            continue;
        }
        let conic = SkConic::new(pts.try_into().unwrap(), w);
        tangents.push(conic.evalTangentAt(t));
    }
}

// Source: src/core/SkPathPriv.cpp:1258-1282
fn tangent_quad(pts: &[SkPoint], x: f32, y: f32, tangents: &mut Vec<SkPoint>) {
    if !between(pts[0].fY, y, pts[1].fY) && !between(pts[1].fY, y, pts[2].fY) {
        return;
    }
    if !between(pts[0].fX, x, pts[1].fX) && !between(pts[1].fX, x, pts[2].fX) {
        return;
    }
    let mut roots = [0.0; 2];
    let n = SkFindUnitQuadRoots(
        pts[0].fY - 2.0 * pts[1].fY + pts[2].fY,
        2.0 * (pts[1].fY - pts[0].fY),
        pts[0].fY - y,
        &mut roots,
    );
    for &t in &roots[..n] {
        let c = pts[0].fX;
        let a = pts[2].fX - 2.0 * pts[1].fX + c;
        let b = 2.0 * (pts[1].fX - c);
        let xt = poly_eval2(a, b, c, t);
        if !nearly_equal(x, xt) {
            continue;
        }
        tangents.push(SkEvalQuadTangentAt(pts.try_into().unwrap(), t));
    }
}

// Source: src/core/SkPathPriv.cpp:1284-1304
fn tangent_line(pts: &[SkPoint], x: f32, y: f32, tangents: &mut Vec<SkPoint>) {
    let y0 = pts[0].fY;
    let y1 = pts[1].fY;
    if !between(y0, y, y1) {
        return;
    }
    let x0 = pts[0].fX;
    let x1 = pts[1].fX;
    if !between(x0, x, x1) {
        return;
    }
    let dx = x1 - x0;
    let dy = y1 - y0;
    if !nearly_equal((x - x0) * dy, dx * (y - y0)) {
        return;
    }
    tangents.push(SkPoint::make(dx, dy));
}

// Source: src/core/SkPathPriv.cpp:1306-1308
fn contains_inclusive(r: &crate::src::core::SkRect::SkRect, p: SkPoint) -> bool {
    r.fLeft <= p.fX && p.fX <= r.fRight && r.fTop <= p.fY && p.fY <= r.fBottom
}

// Source: src/core/SkPathPriv.cpp:1310-1393
// Adapted Source: src/core/SkPathPriv.cpp:1356-1356
impl SkPathPriv {
    pub fn Contains(raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>, p: SkPoint) -> bool {
        use crate::include::core::SkPathTypes::{SkPathFillType, SkPathFillType_IsInverse};
        let ft = raw.fillType();
        let inverse = SkPathFillType_IsInverse(ft);
        if raw.empty() {
            return inverse;
        }
        if !contains_inclusive(&raw.bounds(), p) {
            return inverse;
        }
        let mut w = 0;
        let mut on = 0;
        let mut iter = SkPathEdgeIter::new(raw);
        while let Some(rec) = iter.next() {
            match rec.fEdge {
                Edge::kLine => w += winding_line(rec.fPts, p.fX, p.fY, &mut on),
                Edge::kQuad => w += winding_quad(rec.fPts, p.fX, p.fY, &mut on),
                Edge::kConic => {
                    w += winding_conic(rec.fPts, p.fX, p.fY, rec.fConicWeight.unwrap(), &mut on)
                }
                Edge::kCubic => w += winding_cubic(rec.fPts, p.fX, p.fY, &mut on),
            }
        }
        let even_odd = ft == SkPathFillType::kEvenOdd || ft == SkPathFillType::kInverseEvenOdd;
        if even_odd {
            w &= 1;
        }
        if w != 0 {
            return !inverse;
        }
        if on <= 1 {
            return (on != 0) ^ inverse;
        }
        if (on & 1) != 0 || even_odd {
            return ((on & 1) != 0) ^ inverse;
        }
        // Rust Vec replaces the source's heap-only SkTDArray; its allocation-growth policy is adapted.
        let mut tangents = Vec::new();
        let mut iter = SkPathEdgeIter::new(raw);
        while let Some(rec) = iter.next() {
            let old = tangents.len();
            match rec.fEdge {
                Edge::kLine => tangent_line(rec.fPts, p.fX, p.fY, &mut tangents),
                Edge::kQuad => tangent_quad(rec.fPts, p.fX, p.fY, &mut tangents),
                Edge::kConic => tangent_conic(
                    rec.fPts,
                    p.fX,
                    p.fY,
                    rec.fConicWeight.unwrap(),
                    &mut tangents,
                ),
                Edge::kCubic => tangent_cubic(rec.fPts, p.fX, p.fY, &mut tangents),
            }
            if tangents.len() > old {
                let last = tangents.len() - 1;
                let tangent = tangents[last];
                if nearly_zero(SkPointPriv::LengthSqd(tangent)) {
                    tangents.remove(last);
                } else {
                    for i in 0..last {
                        let test = tangents[i];
                        if nearly_zero(test.cross(tangent))
                            && SkScalarSignAsInt(tangent.fX * test.fX) <= 0
                            && SkScalarSignAsInt(tangent.fY * test.fY) <= 0
                        {
                            tangents.remove(last);
                            tangents.swap_remove(i);
                            break;
                        }
                    }
                }
            }
        }
        !tangents.is_empty() ^ inverse
    }
}

// Source: src/core/SkPathPriv.cpp:1397-1408
impl SkPathPriv {
    pub fn IsAxisAligned(pts: &[SkPoint]) -> bool {
        for pair in pts.windows(2) {
            if pair[0].fX != pair[1].fX && pair[0].fY != pair[1].fY {
                return false;
            }
        }
        true
    }
}

// Source: src/core/SkPathPriv.cpp:1410-1439
impl SkPathPriv {
    pub fn TransformConvexity(
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        pts: &[SkPoint],
        mut convexity: SkPathConvexity,
    ) -> SkPathConvexity {
        if matrix.isIdentity() || pts.is_empty() {
            return convexity;
        }
        if SkPathConvexity_IsConvex(convexity) {
            if !matrix.isScaleTranslate() || !Self::IsAxisAligned(pts) {
                convexity = SkPathConvexity::kUnknown;
            } else {
                let det =
                    matrix.getScaleX() * matrix.getScaleY() - matrix.getSkewX() * matrix.getSkewY();
                if det < 0.0 {
                    convexity = SkPathConvexity_OppositeConvexDirection(convexity);
                } else if det > 0.0 {
                } else {
                    convexity = SkPathConvexity::kConvex_Degenerate;
                }
            }
        }
        convexity
    }
}

// Source: src/core/SkPathPriv.cpp:1443-1453
fn compute_quad_extremas(src: &[SkPoint; 3], extrema: &mut [SkPoint; 5]) -> usize {
    use crate::src::core::SkGeometry::*;
    let mut ts = [0.0; 2];
    let mut n = SkFindQuadExtrema(src[0].fX, src[1].fX, src[2].fX, &mut ts[0]);
    n += SkFindQuadExtrema(src[0].fY, src[1].fY, src[2].fY, &mut ts[n]);
    debug_assert!(n <= 2);
    for i in 0..n {
        extrema[i] = SkEvalQuadAt(src, ts[i]);
    }
    extrema[n] = src[2];
    n + 1
}

// Source: src/core/SkPathPriv.cpp:1455-1466
fn compute_conic_extremas(src: &[SkPoint; 3], w: f32, extrema: &mut [SkPoint; 5]) -> usize {
    let conic = crate::src::core::SkGeometry::SkConic::new(src, w);
    let mut ts = [0.0; 2];
    let mut n = conic.findXExtrema(&mut ts[0]) as usize;
    n += conic.findYExtrema(&mut ts[n]) as usize;
    debug_assert!(n <= 2);
    for i in 0..n {
        extrema[i] = conic.evalAt(ts[i]);
    }
    extrema[n] = src[2];
    n + 1
}

// Source: src/core/SkPathPriv.cpp:1468-1478
fn compute_cubic_extremas(src: &[SkPoint; 4], extrema: &mut [SkPoint; 5]) -> usize {
    use crate::src::core::SkGeometry::*;
    let mut ts = [0.0; 4];
    let mut pair = [0.0; 2];
    let mut n = SkFindCubicExtrema(src[0].fX, src[1].fX, src[2].fX, src[3].fX, &mut pair);
    ts[..n].copy_from_slice(&pair[..n]);
    let count = SkFindCubicExtrema(src[0].fY, src[1].fY, src[2].fY, src[3].fY, &mut pair);
    ts[n..n + count].copy_from_slice(&pair[..count]);
    n += count;
    debug_assert!(n <= 4);
    for i in 0..n {
        SkEvalCubicAt(src, ts[i], Some(&mut extrema[i]), None, None);
    }
    extrema[n] = src[3];
    n + 1
}

// Source: src/core/SkPathPriv.cpp:1480-1526
impl SkPathPriv {
    pub fn ComputeTightBounds(
        points: &[SkPoint],
        verbs: &[SkPathVerb],
        weights: &[f32],
    ) -> crate::src::core::SkRect::SkRect {
        use crate::src::core::SkRect::SkRect;
        if verbs.is_empty() {
            return SkRect::MakeEmpty();
        }
        let (mut left, mut top, mut right, mut bottom) =
            (points[0].fX, points[0].fY, points[0].fX, points[0].fY);
        for rec in Self::IterateSlices(verbs, points, weights) {
            let mut extrema = [SkPoint::default(); 5];
            let pts = rec.fPoints;
            let count = match rec.fVerb {
                SkPathVerb::kMove => {
                    extrema[0] = pts[0];
                    1
                }
                SkPathVerb::kLine => {
                    extrema[0] = pts[1];
                    1
                }
                SkPathVerb::kQuad => compute_quad_extremas(pts.try_into().unwrap(), &mut extrema),
                SkPathVerb::kConic => {
                    compute_conic_extremas(pts.try_into().unwrap(), rec.fConicWeight, &mut extrema)
                }
                SkPathVerb::kCubic => compute_cubic_extremas(pts.try_into().unwrap(), &mut extrema),
                SkPathVerb::kClose => 0,
            };
            for p in &extrema[..count] {
                left = p.fX.min(left);
                top = p.fY.min(top);
                right = p.fX.max(right);
                bottom = p.fY.max(bottom);
            }
        }
        SkRect::MakeLTRB(left, top, right, bottom)
    }
}

use crate::include::core::SkScalar::{
    SK_ScalarNearlyZero, SkScalarNearlyEqual, SkScalarNearlyZero, SkScalarSignAsInt,
};
use crate::src::core::SkCubicClipper::SkCubicClipper;
use crate::src::core::SkGeometry::{
    SkChopCubicAtYExtrema, SkChopQuadAtYExtrema, SkConic, SkEvalCubicAt, SkEvalQuadTangentAt,
    SkFindUnitQuadRoots,
};
use crate::src::core::SkPointPriv::SkPointPriv;
fn nearly_equal(a: f32, b: f32) -> bool {
    SkScalarNearlyEqual(a, b, SK_ScalarNearlyZero)
}
fn nearly_zero(x: f32) -> bool {
    SkScalarNearlyZero(x, SK_ScalarNearlyZero)
}

// Source: src/core/SkPathPriv.cpp:1528-1546
impl SkPathPriv {
    pub fn FindLastMoveToIndex(verbs: &[SkPathVerb], point_count: usize) -> i32 {
        if verbs.is_empty() {
            debug_assert_eq!(point_count, 0);
            return -1;
        }
        crate::SkASSERT!(@source b"src/core/SkPathPriv.cpp\0".as_ptr(),1533; @condition b"verbs[0] == SkPathVerb::kMove\0".as_ptr(); (verbs[0])==(SkPathVerb::kMove));
        debug_assert!(point_count > 0);
        let mut index = point_count as i32 - 1;
        for &v in verbs.iter().rev() {
            if v == SkPathVerb::kMove {
                break;
            }
            index -= Self::PtsInVerb(v as u32) as i32;
        }
        debug_assert!(index >= 0);
        index
    }
}

// Source: src/core/SkPathPriv.cpp:1548-1612
impl SkPathPriv {
    pub fn TransformDirAndStart(
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        is_rrect: bool,
        dir: crate::include::core::SkPathTypes::SkPathDirection,
        mut start: u32,
    ) -> (crate::include::core::SkPathTypes::SkPathDirection, u32) {
        use crate::include::core::SkPathTypes::SkPathDirection;
        if matrix.isIdentity() {
            return (dir, start);
        }
        let mut input = start;
        let mut ccw = dir == SkPathDirection::kCCW;
        let mut rm = 0;
        if is_rrect {
            rm = input & 1;
            input /= 2;
        }
        let (anti, top_neg, same);
        if matrix.getScaleX() != 0.0 {
            anti = 0;
            if matrix.getScaleX() > 0.0 {
                top_neg = 0;
                same = if matrix.getScaleY() > 0.0 { 1 } else { 0 };
            } else {
                top_neg = 2;
                same = if matrix.getScaleY() > 0.0 { 0 } else { 1 };
            }
        } else {
            anti = 1;
            if matrix.getSkewX() > 0.0 {
                top_neg = 0;
                same = if matrix.getSkewY() > 0.0 { 1 } else { 0 };
            } else {
                top_neg = 2;
                same = if matrix.getSkewY() > 0.0 { 0 } else { 1 };
            }
        }
        if same != anti {
            start = input.wrapping_add(4).wrapping_sub(top_neg | anti) % 4;
            debug_assert!(start < 4);
            if is_rrect {
                start = 2 * start + rm;
            }
        } else {
            ccw = !ccw;
            start = (6u32 + (top_neg | anti)).wrapping_sub(input) % 4;
            debug_assert!(start < 4);
            if is_rrect {
                start = 2 * start + if rm != 0 { 0 } else { 1 };
            }
        }
        (
            if ccw {
                SkPathDirection::kCCW
            } else {
                SkPathDirection::kCW
            },
            start,
        )
    }
}

// Source: src/core/SkPathPriv.cpp:1614-1667
impl SkPathPriv {
    pub fn DeduceRRectFromContour(
        bounds: &crate::src::core::SkRect::SkRect,
        pts: &[SkPoint],
        verbs: &[SkPathVerb],
    ) -> crate::src::core::SkRRect::SkRRect {
        debug_assert!(!verbs.is_empty());
        debug_assert_eq!(verbs[0], SkPathVerb::kMove);
        let mut radii = [SkPoint::make(0.0, 0.0); 4];
        let mut i = 0;
        for &v in verbs {
            match v {
                SkPathVerb::kMove => {
                    debug_assert_eq!(i, 0);
                    i += 1;
                }
                SkPathVerb::kLine => {
                    let delta = pts[i] - pts[i - 1];
                    debug_assert!(delta.fX == 0.0 || delta.fY == 0.0);
                    i += 1;
                }
                SkPathVerb::kQuad | SkPathVerb::kCubic => debug_assert!(false),
                SkPathVerb::kConic => {
                    let v10 = pts[i] - pts[i - 1];
                    let v21 = pts[i + 1] - pts[i];
                    let delta;
                    if v10.fX != 0.0 {
                        debug_assert!(v21.fX == 0.0 && v10.fY == 0.0);
                        delta = SkPoint::make(v10.fX.abs(), v21.fY.abs());
                    } else if v10.fY == 0.0 {
                        debug_assert!(v21.fX == 0.0 || v21.fY == 0.0);
                        delta = SkPoint::make(v21.fX.abs(), v21.fY.abs());
                    } else {
                        debug_assert_eq!(v21.fY, 0.0);
                        delta = SkPoint::make(v21.fX.abs(), v10.fY.abs());
                    }
                    let corner = if pts[i].fX == bounds.fLeft {
                        if pts[i].fY == bounds.fTop {
                            0
                        } else {
                            3
                        }
                    } else if pts[i].fY == bounds.fTop {
                        1
                    } else {
                        2
                    };
                    debug_assert!(radii[corner].fX == 0.0 && radii[corner].fY == 0.0);
                    radii[corner] = delta;
                    i += 2;
                }
                SkPathVerb::kClose => {}
            }
        }
        let mut rrect = crate::src::core::SkRRect::SkRRect::default();
        rrect.setRectRadii(bounds, &radii);
        rrect
    }
}
// Source: src/core/SkPathPriv.h:44-46
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RRectAsEnum {
    kRect,
    kOval,
    kRRect,
}

// Source: src/core/SkPathPriv.h:47-55
impl SkPathPriv {
    #[inline]
    pub fn SimplifyRRect(
        rr: &crate::src::core::SkRRect::SkRRect,
        start_index: u32,
    ) -> (RRectAsEnum, u32) {
        if rr.isRect() || rr.isEmpty() {
            return (RRectAsEnum::kRect, start_index.wrapping_add(1) / 2);
        }
        if rr.isOval() {
            return (RRectAsEnum::kOval, start_index / 2);
        }
        (RRectAsEnum::kRRect, start_index)
    }
}

// Source: src/core/SkPathPriv.h:73-73
impl SkPathPriv {
    pub const kW0PlaneDistance: f32 = 1.0 / (1 << 14) as f32;
}

// Source: src/core/SkPathPriv.h:75-78
impl SkPathPriv {
    #[inline]
    pub fn AsFirstDirection(
        dir: crate::include::core::SkPathTypes::SkPathDirection,
    ) -> SkPathFirstDirection {
        if dir == crate::include::core::SkPathTypes::SkPathDirection::kCW {
            SkPathFirstDirection::kCW
        } else {
            SkPathFirstDirection::kCCW
        }
    }
}
// Source: src/core/SkPathPriv.h:84-89
impl SkPathPriv {
    #[inline]
    pub fn OppositeFirstDirection(dir: SkPathFirstDirection) -> SkPathFirstDirection {
        const OPPOSITE: [SkPathFirstDirection; 3] = [
            SkPathFirstDirection::kCCW,
            SkPathFirstDirection::kCW,
            SkPathFirstDirection::kUnknown,
        ];
        OPPOSITE[dir as usize]
    }
}
// Source: src/core/SkPathPriv.h:100-120
impl SkPathPriv {
    #[inline]
    pub fn IsClosedSingleContour(verbs: &[SkPathVerb]) -> bool {
        if verbs.is_empty() {
            return false;
        }
        let mut move_count = 0;
        for (i, verb) in verbs.iter().enumerate() {
            match verb {
                SkPathVerb::kMove => {
                    move_count += 1;
                    if move_count > 1 {
                        return false;
                    }
                }
                SkPathVerb::kClose => return i == verbs.len() - 1,
                _ => {}
            }
        }
        false
    }
}
// Source: src/core/SkPathPriv.h:122-124
impl SkPathPriv {
    #[inline]
    pub fn IsClosedSingleContourPath(path: &crate::src::core::SkPath::SkPath) -> bool {
        Self::IsClosedSingleContour(path.verbs())
    }
}
// Source: src/core/SkPathPriv.h:184-184
pub struct Iterate<'a> {
    // Source: src/core/SkPathPriv.h:204-207
    fVerbsBegin: &'a [SkPathVerb],
    fVerbsEnd: &'a [SkPathVerb],
    fPoints: &'a [SkPoint],
    fWeights: &'a [f32],
}

// Source: src/core/SkPathPriv.h:187-194
impl SkPathPriv {
    #[inline]
    pub fn Iterate(path: &crate::src::core::SkPath::SkPath) -> Iterate<'_> {
        let mut result = Self::IterateSlices(path.verbs(), path.points(), path.conicWeights());
        if !path.isFinite() {
            result.fVerbsBegin = result.fVerbsEnd;
        }
        result
    }
}

// Source: src/core/SkPathPriv.h:195-200
impl SkPathPriv {
    #[inline]
    pub fn IterateSlices<'a>(
        verbs: &'a [SkPathVerb],
        points: &'a [SkPoint],
        weights: &'a [f32],
    ) -> Iterate<'a> {
        Iterate {
            fVerbsBegin: verbs,
            fVerbsEnd: &verbs[verbs.len()..],
            fPoints: points,
            fWeights: weights,
        }
    }
}

// Source: src/core/SkPathPriv.h:201-201
impl<'a> Iterate<'a> {
    #[inline]
    pub fn begin(&self) -> crate::src::core::SkPath::RangeIter<'a> {
        crate::src::core::SkPath::RangeIter::new(self.fVerbsBegin, self.fPoints, self.fWeights)
    }
}

// Source: src/core/SkPathPriv.h:202-202
impl<'a> Iterate<'a> {
    #[inline]
    pub fn end(&self) -> crate::src::core::SkPath::RangeIter<'a> {
        crate::src::core::SkPath::RangeIter::new(self.fVerbsEnd, &[], &[])
    }
}

// Rust for-loop protocol only: delegate to the original borrowed RangeIter; no path retain,
// array copy, lazy cache or replacement iteration algorithm is introduced.
impl<'a> IntoIterator for Iterate<'a> {
    type Item = crate::src::core::SkPath::RangeRec<'a>;
    type IntoIter = crate::src::core::SkPath::RangeIter<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}
impl<'a> IntoIterator for &Iterate<'a> {
    type Item = crate::src::core::SkPath::RangeRec<'a>;
    type IntoIter = crate::src::core::SkPath::RangeIter<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}

// Source: src/core/SkPathPriv.h:217-219
impl SkPathPriv {
    #[inline]
    pub fn IsOval(
        path: &crate::src::core::SkPath::SkPath,
    ) -> Option<crate::include::private::SkPathRef::SkPathOvalInfo> {
        path.getOvalInfo()
    }
}
// Source: src/core/SkPathPriv.h:223-225
impl SkPathPriv {
    #[inline]
    pub fn IsRRect(
        path: &crate::src::core::SkPath::SkPath,
    ) -> Option<crate::include::private::SkPathRef::SkPathRRectInfo> {
        path.getRRectInfo()
    }
}
// Source: src/core/SkPathPriv.h:235-244
impl SkPathPriv {
    #[inline]
    pub fn TooBigForMath(bounds: &crate::src::core::SkRect::SkRect) -> bool {
        const SCALE: f32 = 0.25;
        const MAX: f32 = f32::MAX * SCALE;
        !(bounds.fLeft >= -MAX
            && bounds.fTop >= -MAX
            && bounds.fRight <= MAX
            && bounds.fBottom <= MAX)
    }
}
// Source: src/core/SkPathPriv.h:247-260
impl SkPathPriv {
    #[inline]
    pub fn PtsInIter(verb: u32) -> usize {
        const COUNTS: [u8; 7] = [1, 2, 3, 3, 4, 0, 0];
        debug_assert!((verb as usize) < COUNTS.len());
        COUNTS[verb as usize] as usize
    }
}
// Source: src/core/SkPathPriv.h:262-262
impl SkPathPriv {
    #[inline]
    pub fn PtsInIterVerb(verb: SkPathVerb) -> usize {
        Self::PtsInIter(verb as u32)
    }
}
// Source: src/core/SkPathPriv.h:266-279
impl SkPathPriv {
    #[inline]
    pub fn PtsInVerb(verb: u32) -> usize {
        const TABLE: [u8; 7] = [1, 1, 2, 2, 3, 0, 0];
        debug_assert!((verb as usize) < TABLE.len());
        TABLE[verb as usize] as usize
    }
}

// Source: src/core/SkPathPriv.h:281-281
impl SkPathPriv {
    #[inline]
    pub fn PtsInVerbEnum(verb: SkPathVerb) -> usize {
        Self::PtsInVerb(verb as u32)
    }
}
// Source: src/core/SkPathPriv.h:285-292
impl SkPathPriv {
    #[inline]
    pub fn AllPointsEq(pts: &[SkPoint]) -> bool {
        for i in 1..pts.len() {
            if pts[0] != pts[i] {
                return false;
            }
        }
        true
    }
}

// Source: src/core/SkPathPriv.h:294-300
pub struct RectContour {
    pub fRect: crate::src::core::SkRect::SkRect,
    pub fIsClosed: bool,
    pub fDirection: crate::include::core::SkPathTypes::SkPathDirection,
    pub fPointsConsumed: usize,
    pub fVerbsConsumed: usize,
}

// Source: src/core/SkPathPriv.h:320-324
impl SkPathPriv {
    #[inline]
    pub fn IsNestedFillRectsPath(
        path: &crate::src::core::SkPath::SkPath,
        rects: Option<&mut [crate::src::core::SkRect::SkRect; 2]>,
        dirs: Option<&mut [crate::include::core::SkPathTypes::SkPathDirection; 2]>,
    ) -> bool {
        if let Some(raw) = Self::Raw(path, SkResolveConvexity::kNo) {
            Self::IsNestedFillRects(&raw, rects, dirs)
        } else {
            false
        }
    }
}
// Source: src/core/SkPathPriv.h:327-329
impl SkPathPriv {
    #[inline]
    pub fn IsInverseFillType(fill: crate::include::core::SkPathTypes::SkPathFillType) -> bool {
        (fill as i32 & 2) != 0
    }
}
// Source: src/core/SkPathPriv.h:348-350
impl SkPathPriv {
    #[inline]
    pub fn GetConvexity(path: &crate::src::core::SkPath::SkPath) -> SkPathConvexity {
        path.getConvexity()
    }
}
// Source: src/core/SkPathPriv.h:351-353
impl SkPathPriv {
    #[inline]
    pub fn GetConvexityOrUnknown(path: &crate::src::core::SkPath::SkPath) -> SkPathConvexity {
        path.getConvexityOrUnknown()
    }
}
// Source: src/core/SkPathPriv.h:354-356
impl SkPathPriv {
    #[inline]
    pub fn SetConvexity(path: &crate::src::core::SkPath::SkPath, c: SkPathConvexity) {
        path.setConvexity(c);
    }
}
// Source: src/core/SkPathPriv.h:357-360
impl SkPathPriv {
    #[inline]
    pub fn ForceComputeConvexity(path: &crate::src::core::SkPath::SkPath) {
        path.setConvexity(SkPathConvexity::kUnknown);
        path.isConvex();
    }
}
// Source: src/core/SkPathPriv.h:362-364
impl SkPathPriv {
    #[inline]
    pub fn GetDataConvexityOrUnknown(
        data: &crate::src::core::SkPathData::SkPathData,
    ) -> SkPathConvexity {
        data.getConvexityOrUnknown()
    }
}
// Source: src/core/SkPathPriv.h:366-368
impl SkPathPriv {
    #[inline]
    pub fn ReverseAddPath(
        builder: &mut crate::src::core::SkPathBuilder::SkPathBuilder,
        path: &crate::src::core::SkPath::SkPath,
    ) {
        builder.privateReverseAddPath(path);
    }
}
// Source: src/core/SkPathPriv.h:370-372
impl SkPathPriv {
    #[inline]
    pub fn ReversePathTo(
        builder: &mut crate::src::core::SkPathBuilder::SkPathBuilder,
        path: &crate::src::core::SkPath::SkPath,
    ) {
        builder.privateReversePathTo(path);
    }
}
// Source: src/core/SkPathPriv.h:374-378
impl SkPathPriv {
    #[inline]
    pub fn ReversePath(
        path: &crate::src::core::SkPath::SkPath,
    ) -> crate::src::core::SkPath::SkPath {
        let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
        builder.privateReverseAddPath(path);
        builder.detach(None)
    }
}
// Source: src/core/SkPathPriv.h:380-382
impl SkPathPriv {
    #[inline]
    pub fn GetVerbs(builder: &crate::src::core::SkPathBuilder::SkPathBuilder) -> &[SkPathVerb] {
        builder.verbs()
    }
}
// Source: src/core/SkPathPriv.h:384-386
impl SkPathPriv {
    #[inline]
    pub fn CountVerbs(builder: &crate::src::core::SkPathBuilder::SkPathBuilder) -> usize {
        builder.verbs().len()
    }
}
// Source: src/core/SkPathPriv.h:388-390
impl SkPathPriv {
    #[inline]
    pub fn Raw(
        path: &crate::src::core::SkPath::SkPath,
        rc: SkResolveConvexity,
    ) -> Option<crate::src::core::SkPathRaw::SkPathRaw<'_>> {
        path.raw(rc)
    }
}

// Source: src/core/SkPathPriv.h:392-414
impl SkPathPriv {
    #[inline]
    pub fn RawBuilder(
        builder: &crate::src::core::SkPathBuilder::SkPathBuilder,
        rc: SkResolveConvexity,
    ) -> Option<crate::src::core::SkPathRaw::SkPathRaw<'_>> {
        let bounds = builder.computeFiniteBounds()?;
        let mut convexity = builder.fConvexity;
        if convexity == SkPathConvexity::kUnknown && rc == SkResolveConvexity::kYes {
            convexity =
                Self::ComputeConvexity(&builder.fPts, &builder.fVerbs, &builder.fConicWeights);
        }
        Some(crate::src::core::SkPathRaw::SkPathRaw {
            fPoints: builder.points(),
            fVerbs: builder.verbs(),
            fConics: builder.conicWeights(),
            fBounds: bounds,
            fFillType: builder.fillType(),
            fConvexity: convexity,
            fSegmentMask: builder.fSegmentMask as u8,
        })
    }
}

// Source: src/core/SkPathPriv.h:418-432
impl SkPathPriv {
    #[inline]
    pub fn TrimmedBounds(
        mut pts: &[crate::src::core::SkPoint::SkPoint],
        vbs: &[crate::include::core::SkPathTypes::SkPathVerb],
    ) -> Option<crate::src::core::SkRect::SkRect> {
        if vbs.len() > 1
            && vbs.last() == Some(&crate::include::core::SkPathTypes::SkPathVerb::kMove)
        {
            debug_assert!(!pts.is_empty());
            if !pts.last().unwrap().is_finite() {
                return None;
            }
            pts = &pts[..pts.len() - 1];
        }
        crate::src::core::SkRect::SkRect::Bounds(pts)
    }
}

// Source: src/core/SkPathPriv.h:440-449
pub struct SkPathEdgeIter<'a> {
    pub(crate) points: &'a [SkPoint],
    pub(crate) verbs: &'a [SkPathVerb],
    pub(crate) conics: &'a [f32],
    pub(crate) verb_index: usize,
    pub(crate) point_index: usize,
    pub(crate) move_index: usize,
    pub(crate) conic_index: isize,
    pub(crate) scratch: [SkPoint; 2],
    pub(crate) needs_close_line: bool,
    pub(crate) next_is_new_contour: bool,
    pub(crate) is_conic: bool,
}

// Source: src/core/SkPathPriv.h:455-458
impl SkPathEdgeIter<'_> {
    #[inline]
    pub fn conicWeight(&self) -> f32 {
        debug_assert!(self.is_conic);
        self.conics[self.conic_index as usize]
    }
}

// Source: src/core/SkPathPriv.h:460-466
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    kLine = 1,
    kQuad = 2,
    kConic = 3,
    kCubic = 4,
}

// Source: src/core/SkPathPriv.h:468-470
impl SkPathEdgeIter<'_> {
    #[inline]
    pub fn EdgeToVerb(edge: Edge) -> SkPathVerb {
        match edge {
            Edge::kLine => SkPathVerb::kLine,
            Edge::kQuad => SkPathVerb::kQuad,
            Edge::kConic => SkPathVerb::kConic,
            Edge::kCubic => SkPathVerb::kCubic,
        }
    }
}

// Source: src/core/SkPathPriv.h:473-480
pub struct EdgeResult<'a> {
    pub fPts: &'a [SkPoint],
    pub fEdge: Edge,
    pub fIsNewContour: bool,
    // Captured from the same iterator step so the lending result can carry the weight safely.
    pub fConicWeight: Option<f32>,
}

// Source: src/core/SkPathPriv.h:482-535
impl SkPathEdgeIter<'_> {
    #[inline]
    pub fn next(&mut self) -> Option<EdgeResult<'_>> {
        loop {
            debug_assert!(self.verb_index <= self.verbs.len());
            if self.verb_index == self.verbs.len() {
                if self.needs_close_line {
                    return Some(self.close_line());
                }
                return None;
            }
            self.is_conic = false;
            let verb = self.verbs[self.verb_index];
            self.verb_index += 1;
            match verb {
                SkPathVerb::kMove => {
                    if self.needs_close_line {
                        self.scratch = [
                            self.points[self.point_index - 1],
                            self.points[self.move_index],
                        ];
                        self.needs_close_line = false;
                        self.next_is_new_contour = true;
                        self.move_index = self.point_index;
                        self.point_index += 1;
                        return Some(EdgeResult {
                            fPts: &self.scratch,
                            fEdge: Edge::kLine,
                            fIsNewContour: false,
                            fConicWeight: None,
                        });
                    }
                    self.move_index = self.point_index;
                    self.point_index += 1;
                    self.next_is_new_contour = true;
                }
                SkPathVerb::kClose => {
                    if self.needs_close_line {
                        return Some(self.close_line());
                    }
                }
                _ => {
                    let v = verb as usize;
                    let pts_count = (v + 2) / 2;
                    let cws_count = (v & (v - 1)) / 2;
                    self.needs_close_line = true;
                    self.point_index += pts_count;
                    self.conic_index += cws_count as isize;
                    self.is_conic = verb == SkPathVerb::kConic;
                    debug_assert_eq!(self.is_conic, cws_count > 0);
                    let is_new = self.next_is_new_contour;
                    self.next_is_new_contour = false;
                    let edge = match verb {
                        SkPathVerb::kLine => Edge::kLine,
                        SkPathVerb::kQuad => Edge::kQuad,
                        SkPathVerb::kConic => Edge::kConic,
                        SkPathVerb::kCubic => Edge::kCubic,
                        _ => unreachable!(),
                    };
                    return Some(EdgeResult {
                        fPts: &self.points[self.point_index - (pts_count + 1)..self.point_index],
                        fEdge: edge,
                        fIsNewContour: is_new,
                        fConicWeight: self
                            .is_conic
                            .then(|| self.conics[self.conic_index as usize]),
                    });
                }
            }
        }
    }
    #[inline]
    fn close_line(&mut self) -> EdgeResult<'_> {
        self.scratch = [
            self.points[self.point_index - 1],
            self.points[self.move_index],
        ];
        self.needs_close_line = false;
        self.next_is_new_contour = true;
        EdgeResult {
            fPts: &self.scratch,
            fEdge: Edge::kLine,
            fIsNewContour: false,
            fConicWeight: None,
        }
    }
}

// Source: src/core/SkPathPriv.h:37-40
const _: () = {
    assert!(
        crate::include::core::SkPathTypes::SkPathFillType::kWinding as u8 == 0,
        "fill_type_mismatch"
    );
    assert!(
        crate::include::core::SkPathTypes::SkPathFillType::kEvenOdd as u8 == 1,
        "fill_type_mismatch"
    );
    assert!(
        crate::include::core::SkPathTypes::SkPathFillType::kInverseWinding as u8 == 2,
        "fill_type_mismatch"
    );
    assert!(
        crate::include::core::SkPathTypes::SkPathFillType::kInverseEvenOdd as u8 == 3,
        "fill_type_mismatch"
    );
};
