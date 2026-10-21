// Borrow adaptation: stack storage and metadata are owned together; raw() lends the view
// rather than storing C++ self-pointers that would be invalidated by a Rust move.
use crate::include::core::SkPathTypes::*;
use crate::src::core::SkPathEnums::*;
use crate::src::core::SkPathMakers::*;
use crate::src::core::SkPathRaw::SkPathRaw;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRRect::SkRRect;
use crate::src::core::SkRect::SkRect;
struct Metadata {
    npoints: usize,
    verbs: &'static [SkPathVerb],
    conics: &'static [f32],
    bounds: SkRect,
    fill_type: SkPathFillType,
    convexity: SkPathConvexity,
    segment_mask: u8,
}
impl Metadata {
    fn raw<'a>(&self, storage: &'a [SkPoint]) -> SkPathRaw<'a> {
        SkPathRaw {
            fPoints: &storage[..self.npoints],
            fVerbs: self.verbs,
            fConics: self.conics,
            fBounds: self.bounds,
            fFillType: self.fill_type,
            fConvexity: self.convexity,
            fSegmentMask: self.segment_mask,
        }
    }
}

// Source: src/core/SkPathRawShapes.h:26-30
pub struct Rect {
    pub fStorage: [SkPoint; 4],
    meta: Metadata,
}
impl Rect {
    #[inline]
    pub fn raw(&self) -> SkPathRaw<'_> {
        self.meta.raw(&self.fStorage)
    }
}

// Source: src/core/SkPathRawShapes.h:32-36
pub struct Oval {
    pub fStorage: [SkPoint; 9],
    meta: Metadata,
}
impl Oval {
    #[inline]
    pub fn raw(&self) -> SkPathRaw<'_> {
        self.meta.raw(&self.fStorage)
    }
}

// Source: src/core/SkPathRawShapes.h:38-45
pub struct RRect {
    pub fStorage: [SkPoint; 13],
    meta: Metadata,
}
impl RRect {
    #[inline]
    pub fn raw(&self) -> SkPathRaw<'_> {
        self.meta.raw(&self.fStorage)
    }
}

// Source: src/core/SkPathRawShapes.h:47-49
pub struct Triangle<'a> {
    view: SkPathRaw<'a>,
}
impl Triangle<'_> {
    #[inline]
    pub fn raw(&self) -> SkPathRaw<'_> {
        self.view
    }
}

// Source: src/core/SkPathRawShapes.cpp:14-14
pub const kDefFillType: SkPathFillType = SkPathFillType::kWinding;

// Source: src/core/SkPathRawShapes.cpp:16-22
pub const gRectVerbs: [SkPathVerb; 5] = [
    SkPathVerb::kMove,
    SkPathVerb::kLine,
    SkPathVerb::kLine,
    SkPathVerb::kLine,
    SkPathVerb::kClose,
];

// Source: src/core/SkPathRawShapes.cpp:24-24
pub const gRectSegMask: u8 = kLine_SkPathSegmentMask as u8;

// Source: src/core/SkPathRawShapes.cpp:26-44
fn set_as_rect(storage: &mut [SkPoint], r: &SkRect, dir: SkPathDirection, index: u32) -> Metadata {
    crate::SkASSERT!(@source b"src/core/SkPathRawShapes.cpp\0".as_ptr(),28; @condition b"storage.size() >= 4\0".as_ptr(); storage.len() >= 4);
    let meta = Metadata {
        npoints: 4,
        verbs: &gRectVerbs,
        conics: &[],
        bounds: *r,
        fill_type: kDefFillType,
        convexity: SkPathDirection_ToConvexity(dir),
        segment_mask: gRectSegMask,
    };
    let mut iter = SkPath_RectPointIterator::new(r, dir, index);
    storage[0] = iter.current();
    storage[1] = iter.next();
    storage[2] = iter.next();
    storage[3] = iter.next();
    meta
}

// Source: src/core/SkPathRawShapes.cpp:48-55
pub const gOvalVerbs: [SkPathVerb; 6] = [
    SkPathVerb::kMove,
    SkPathVerb::kConic,
    SkPathVerb::kConic,
    SkPathVerb::kConic,
    SkPathVerb::kConic,
    SkPathVerb::kClose,
];

// Source: src/core/SkPathRawShapes.cpp:57-57
pub const gOvalSegMask: u8 = kConic_SkPathSegmentMask as u8;

// Source: src/core/SkPathRawShapes.cpp:59-64
pub const gFourQuarterCircleConics: [f32; 4] =
    [crate::include::core::SkScalar::SK_ScalarRoot2Over2; 4];

// Source: src/core/SkPathRawShapes.cpp:66-86
fn set_as_oval(storage: &mut [SkPoint], r: &SkRect, dir: SkPathDirection, index: u32) -> Metadata {
    crate::SkASSERT!(@source b"src/core/SkPathRawShapes.cpp\0".as_ptr(),68; @condition b"storage.size() >= 9\0".as_ptr(); storage.len() >= 9);
    let meta = Metadata {
        npoints: 9,
        verbs: &gOvalVerbs,
        conics: &gFourQuarterCircleConics,
        bounds: *r,
        fill_type: kDefFillType,
        convexity: SkPathDirection_ToConvexity(dir),
        segment_mask: gOvalSegMask,
    };
    let mut oval_iter = SkPath_OvalPointIterator::new(r, dir, index);
    let mut rect_iter = SkPath_RectPointIterator::new(
        r,
        dir,
        index + if dir == SkPathDirection::kCW { 0 } else { 1 },
    );
    storage[0] = oval_iter.current();
    for i in 0..4 {
        storage[i * 2 + 1] = rect_iter.next();
        storage[i * 2 + 2] = oval_iter.next();
    }
    meta
}

// Source: src/core/SkPathRawShapes.cpp:90-97
pub const gRRectVerbs_LineStart: [SkPathVerb; 10] = [
    SkPathVerb::kMove,
    SkPathVerb::kLine,
    SkPathVerb::kConic,
    SkPathVerb::kLine,
    SkPathVerb::kConic,
    SkPathVerb::kLine,
    SkPathVerb::kConic,
    SkPathVerb::kLine,
    SkPathVerb::kConic,
    SkPathVerb::kClose,
];

// Source: src/core/SkPathRawShapes.cpp:99-106
pub const gRRectVerbs_ConicStart: [SkPathVerb; 9] = [
    SkPathVerb::kMove,
    SkPathVerb::kConic,
    SkPathVerb::kLine,
    SkPathVerb::kConic,
    SkPathVerb::kLine,
    SkPathVerb::kConic,
    SkPathVerb::kLine,
    SkPathVerb::kConic,
    SkPathVerb::kClose,
];

// Source: src/core/SkPathRawShapes.cpp:108-108
pub const gRRectSegMask: u8 = (kLine_SkPathSegmentMask | kConic_SkPathSegmentMask) as u8;

// Source: src/core/SkPathRawShapes.cpp:110-161
fn set_as_rrect(
    storage: &mut [SkPoint],
    rrect: &SkRRect,
    dir: SkPathDirection,
    index: u32,
) -> Metadata {
    let starts_with_conic = index & 1 == (dir == SkPathDirection::kCW) as u32;
    let npoints = 13 - starts_with_conic as usize;
    let bounds = rrect.getBounds();
    crate::SkASSERT!(@source b"src/core/SkPathRawShapes.cpp\0".as_ptr(),118; @condition b"storage.size() >= npoints\0".as_ptr(); storage.len() >= npoints);
    let meta = Metadata {
        npoints,
        verbs: if starts_with_conic {
            &gRRectVerbs_ConicStart
        } else {
            &gRRectVerbs_LineStart
        },
        conics: &gFourQuarterCircleConics,
        bounds: *bounds,
        fill_type: kDefFillType,
        convexity: SkPathDirection_ToConvexity(dir),
        segment_mask: gRRectSegMask,
    };
    let mut rrect_iter = SkPath_RRectPointIterator::new(rrect, dir, index);
    let rect_start_index = index / 2 + if dir == SkPathDirection::kCW { 0 } else { 1 };
    let mut rect_iter = SkPath_RectPointIterator::new(bounds, dir, rect_start_index);
    storage[0] = rrect_iter.current();
    if starts_with_conic {
        for i in 0..3 {
            storage[i * 3 + 1] = rect_iter.next();
            storage[i * 3 + 2] = rrect_iter.next();
            storage[i * 3 + 3] = rrect_iter.next();
        }
        storage[10] = rect_iter.next();
        storage[11] = rrect_iter.next();
    } else {
        for i in 0..4 {
            storage[i * 3 + 1] = rrect_iter.next();
            storage[i * 3 + 2] = rect_iter.next();
            storage[i * 3 + 3] = rrect_iter.next();
        }
    }
    meta
}

// Source: src/core/SkPathRawShapes.cpp:165-167
impl Rect {
    pub fn new(r: &SkRect, dir: SkPathDirection, index: u32) -> Self {
        let mut storage = [SkPoint::make(0.0, 0.0); 4];
        let meta = set_as_rect(&mut storage, r, dir, index);
        Self {
            fStorage: storage,
            meta,
        }
    }
}

// Source: src/core/SkPathRawShapes.cpp:169-171
impl Oval {
    pub fn new(r: &SkRect, dir: SkPathDirection, index: u32) -> Self {
        let mut storage = [SkPoint::make(0.0, 0.0); 9];
        let meta = set_as_oval(&mut storage, r, dir, index);
        Self {
            fStorage: storage,
            meta,
        }
    }
}

// Source: src/core/SkPathRawShapes.cpp:173-185
impl RRect {
    pub fn new(rrect: &SkRRect, dir: SkPathDirection, index: u32) -> Self {
        let mut storage = [SkPoint::make(0.0, 0.0); 13];
        let bounds = rrect.getBounds();
        let meta = if rrect.isRect() || rrect.isEmpty() {
            set_as_rect(&mut storage, bounds, dir, (index + 1) / 2)
        } else if rrect.isOval() {
            set_as_oval(&mut storage, bounds, dir, index / 2)
        } else {
            set_as_rrect(&mut storage, rrect, dir, index)
        };
        Self {
            fStorage: storage,
            meta,
        }
    }
}

// Source: src/core/SkPathRawShapes.cpp:189-194
pub const gTriangle_Verbs: [SkPathVerb; 4] = [
    SkPathVerb::kMove,
    SkPathVerb::kLine,
    SkPathVerb::kLine,
    SkPathVerb::kClose,
];

// Source: src/core/SkPathRawShapes.cpp:196-204
fn tri_to_convexity(pts: &[SkPoint]) -> SkPathConvexity {
    let u = pts[1] - pts[0];
    let v = pts[2] - pts[1];
    let cross = u.fX * v.fY - u.fY * v.fX;
    if cross > 0.0 {
        SkPathConvexity::kConvex_CW
    } else if cross < 0.0 {
        SkPathConvexity::kConvex_CCW
    } else {
        SkPathConvexity::kConvex_Degenerate
    }
}

// Source: src/core/SkPathRawShapes.cpp:205-212
impl<'a> Triangle<'a> {
    pub fn new(three_points: &'a [SkPoint], bounds: &SkRect) -> Self {
        let result = Self {
            view: SkPathRaw {
                fPoints: three_points,
                fVerbs: &gTriangle_Verbs,
                fConics: &[],
                fBounds: *bounds,
                fFillType: SkPathFillType::kDefault,
                fConvexity: tri_to_convexity(three_points),
                fSegmentMask: kLine_SkPathSegmentMask as u8,
            },
        };
        crate::SkASSERT!(@source b"src/core/SkPathRawShapes.cpp\0".as_ptr(),210; @condition b"threePoints.size() == 3\0".as_ptr(); (three_points.len())==(3));
        debug_assert_eq!(SkRect::Bounds(three_points).unwrap(), *bounds);
        result
    }
}
