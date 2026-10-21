// Copyright 2011 Google Inc. BSD-style license.
use crate::src::core::SkEdge::{SkCubicEdge, SkEdge, SkQuadraticEdge, Type};
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::{SkIRect, SkRect};
// One pinned allocator keeps its 512 embedded bytes at a stable address. All
// attempted edges share its native Fibonacci block/packed-footer allocation.
// Source: src/core/SkEdgeBuilder.h:34-36
pub struct SkBasicEdgeBuilder {
    // Source: src/core/SkEdgeBuilder.h:30-30
    // UNVERIFIED: Rust drops fields forward; native drops arena before pointer list.
    fAlloc: core::pin::Pin<Box<crate::src::core::SkArenaAlloc::SkSTArenaAlloc<512>>>,
    fList: Vec<*mut SkEdge>,
}
// Source: src/core/SkEdgeBuilder.h:38-42
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Combine {
    kNo_Combine,
    kPartial_Combine,
    kTotal_Combine,
}
impl SkBasicEdgeBuilder {
    // Source: src/core/SkEdgeBuilder.h:57-61
    pub fn new() -> Self {
        Self {
            fList: Vec::new(),
            fAlloc: Box::pin(crate::src::core::SkArenaAlloc::SkSTArenaAlloc::new(512)),
        }
    }
    /// Safety: replaced pointers must remain live, concrete correctly tagged
    /// SkEdge allocations for all later builder and scan operations.
    pub unsafe fn edgeList(&mut self) -> &mut [*mut SkEdge] {
        &mut self.fList
    }
    // Source: src/core/SkEdgeBuilder.h:66-69
    pub unsafe fn allocEdges(&mut self, _n: usize, _sizeof_edge: *mut usize) -> *mut u8 {
        crate::SkDEBUGFAIL!(@source b"src/core/SkEdgeBuilder.h\0".as_ptr(), 67; b"Not implemented\0".as_ptr());
        core::ptr::null_mut()
    }
    // Source: src/core/SkEdgeBuilder.h:76-79
    pub unsafe fn addPolyLine(
        &mut self,
        _pts: *const SkPoint,
        _edge: *mut u8,
        _edge_ptr: *mut *mut u8,
    ) -> Combine {
        crate::SkDEBUGFAIL!(@source b"src/core/SkEdgeBuilder.h\0".as_ptr(), 77; b"Not implemented\0".as_ptr());
        Combine::kNo_Combine
    }
    // Source: src/core/SkEdgeBuilder.cpp:25-66
    fn combineVertical(edge: &SkEdge, last: &mut SkEdge) -> Combine {
        use Combine::*;
        if last.fEdgeType != Type::kLine || last.fDxDy != 0 || edge.fX != last.fX {
            return kNo_Combine;
        }
        if edge.fWinding == last.fWinding {
            if edge.fLastY + 1 == last.fFirstY {
                last.fFirstY = edge.fFirstY;
                return kPartial_Combine;
            }
            if edge.fFirstY == last.fLastY + 1 {
                last.fLastY = edge.fLastY;
                return kPartial_Combine;
            }
            return kNo_Combine;
        }
        if edge.fFirstY == last.fFirstY {
            if edge.fLastY == last.fLastY {
                return kTotal_Combine;
            }
            if edge.fLastY < last.fLastY {
                last.fFirstY = edge.fLastY + 1;
                return kPartial_Combine;
            }
            last.fFirstY = last.fLastY + 1;
            last.fLastY = edge.fLastY;
            last.fWinding = edge.fWinding;
            return kPartial_Combine;
        }
        if edge.fLastY == last.fLastY {
            if edge.fFirstY > last.fFirstY {
                last.fLastY = edge.fFirstY - 1;
                return kPartial_Combine;
            }
            last.fLastY = last.fFirstY - 1;
            last.fFirstY = edge.fFirstY;
            last.fWinding = edge.fWinding;
            return kPartial_Combine;
        }
        kNo_Combine
    }
}
// Source: src/core/SkEdgeBuilder.cpp:120-125
fn is_vertical(edge: &SkEdge) -> bool {
    edge.fDxDy == 0 && edge.fEdgeType == Type::kLine
}
impl SkBasicEdgeBuilder {
    // Source: src/core/SkEdgeBuilder.cpp:137-150
    pub fn addLine<const EVEN: bool>(&mut self, pts: &[SkPoint; 2]) {
        let edge = unsafe { self.fAlloc.as_mut().arena().make(SkEdge::new()) };
        let edge = unsafe { &mut *edge };
        if edge.setLine::<EVEN>(&pts[0], &pts[1], None) {
            let combine = if is_vertical(edge) && !self.fList.is_empty() {
                unsafe { Self::combineVertical(edge, &mut **self.fList.last().unwrap()) }
            } else {
                Combine::kNo_Combine
            };
            match combine {
                Combine::kTotal_Combine => {
                    self.fList.pop();
                }
                Combine::kPartial_Combine => {}
                Combine::kNo_Combine => self.fList.push(edge as *mut SkEdge),
            }
        }
    }
    // Source: src/core/SkEdgeBuilder.cpp:166-171
    pub fn addQuad<const EVEN: bool>(&mut self, pts: &[SkPoint; 3]) {
        let edge = unsafe { self.fAlloc.as_mut().arena().make(SkQuadraticEdge::new()) };
        let edge = unsafe { &mut *edge };
        if edge.setQuadratic::<EVEN>(pts) {
            self.fList.push(&mut edge.edge);
        }
    }
    // Source: src/core/SkEdgeBuilder.cpp:179-184
    pub fn addCubic<const EVEN: bool>(&mut self, pts: &[SkPoint; 4]) {
        let edge = unsafe { self.fAlloc.as_mut().arena().make(SkCubicEdge::new()) };
        let edge = unsafe { &mut *edge };
        if edge.setCubic::<EVEN>(pts) {
            self.fList.push(&mut edge.edge);
        }
    }
    // Source: src/core/SkEdgeBuilder.cpp:207-209
    pub fn recoverClip(&self, src: &SkIRect) -> SkRect {
        SkRect {
            fLeft: src.fLeft as f32,
            fTop: src.fTop as f32,
            fRight: src.fRight as f32,
            fBottom: src.fBottom as f32,
        }
    }
}

macro_rules! impl_shared_build {
    ($builder:ident) => {
        impl $builder {
            // Source: src/core/SkEdgeBuilder.cpp:220-268
            pub fn buildPoly<const EVEN: bool>(
                &mut self,
                raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
                iclip: Option<&SkIRect>,
                canCullToTheRight: bool,
            ) -> usize {
                use crate::src::core::SkLineClipper::SkLineClipper;
                use crate::src::core::SkPathPriv::{Edge, SkPathEdgeIter};
                let mut maxEdgeCount = raw.fPoints.len();
                if iclip.is_some() {
                    maxEdgeCount =
                        match maxEdgeCount.checked_mul(SkLineClipper::kMaxClippedLineSegments) {
                            Some(count) => count,
                            None => return 0,
                        };
                }
                let _ = maxEdgeCount;
                let mut iter = SkPathEdgeIter::new(raw);
                if let Some(iclip) = iclip {
                    let clip = self.recoverClip(iclip);
                    while let Some(e) = iter.next() {
                        match e.fEdge {
                            Edge::kLine => {
                                let pts: &[SkPoint; 2] = e.fPts.try_into().unwrap();
                                let mut lines = [SkPoint::default(); SkLineClipper::kMaxPoints];
                                let lineCount = SkLineClipper::ClipLine(
                                    pts,
                                    &clip,
                                    &mut lines,
                                    canCullToTheRight,
                                );
                                crate::SkASSERT!(@source b"src/core/SkEdgeBuilder.cpp\0".as_ptr(), 242; @condition b"lineCount <= SkLineClipper::kMaxClippedLineSegments\0".as_ptr(); lineCount <= SkLineClipper::kMaxClippedLineSegments);
                                for i in 0..lineCount {
                                    self.addLine::<EVEN>((&lines[i..i + 2]).try_into().unwrap());
                                }
                            }
                            _ => crate::SkDEBUGFAIL!(@source b"src/core/SkEdgeBuilder.cpp\0".as_ptr(), 249; b"unexpected verb\0".as_ptr()),
                        }
                    }
                } else {
                    while let Some(e) = iter.next() {
                        match e.fEdge {
                            Edge::kLine => self.addLine::<EVEN>(e.fPts.try_into().unwrap()),
                            _ => crate::SkDEBUGFAIL!(@source b"src/core/SkEdgeBuilder.cpp\0".as_ptr(), 261; b"unexpected verb\0".as_ptr()),
                        }
                    }
                }
                // The Rust list accessor supplies the original fList.begin() pointer;
                // keeping another pointer would become stale if Vec reallocates.
                self.fList.len()
            }
        }

        impl $builder {
            // Source: src/core/SkEdgeBuilder.cpp:270-299
            pub fn build<const EVEN: bool>(
                &mut self,
                raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
                iclip: Option<&SkIRect>,
                canCullToTheRight: bool,
            ) -> usize {
                use crate::include::core::SkPathTypes::SkPathVerb;
                if let Some(iclip) = iclip {
                    let clip = self.recoverClip(iclip);
                    let mut isFinite = true;
                    crate::src::core::SkEdgeClipper::SkEdgeClipper::ClipPath(
                        raw,
                        &clip,
                        canCullToTheRight,
                        |clipper, _| {
                            let mut pts = [SkPoint::default(); 4];
                            while let Some(verb) = clipper.next(&mut pts) {
                                let count = match verb {
                                    SkPathVerb::kLine => 2,
                                    SkPathVerb::kQuad => 3,
                                    SkPathVerb::kCubic => 4,
                                    _ => 0,
                                };
                                if !pts[..count]
                                    .iter()
                                    .all(|p| p.fX.is_finite() && p.fY.is_finite())
                                {
                                    isFinite = false;
                                    return;
                                }
                                match verb {
                                    SkPathVerb::kLine => {
                                        self.addLine::<EVEN>((&pts[..2]).try_into().unwrap())
                                    }
                                    SkPathVerb::kQuad => {
                                        self.addQuad::<EVEN>((&pts[..3]).try_into().unwrap())
                                    }
                                    SkPathVerb::kCubic => self.addCubic::<EVEN>(&pts),
                                    _ => {}
                                }
                            }
                        },
                    );
                    return if isFinite { self.fList.len() } else { 0 };
                }
                self.buildUnclipped::<EVEN>(raw)
            }
        }

        impl $builder {
            // Source: src/core/SkEdgeBuilder.cpp:301-344
            /// The native unclipped branch of build; clipped curves require SkEdgeClipper.
            pub fn buildUnclipped<const EVEN: bool>(
                &mut self,
                raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
            ) -> usize {
                use crate::src::core::SkGeometry::{
                    SkAutoConicToQuads, SkChopCubicAtYExtrema, SkChopQuadAtYExtrema,
                };
                use crate::src::core::SkPathPriv::{Edge, SkPathEdgeIter};
                let mut iter = SkPathEdgeIter::new(raw);
                let mut quadder = SkAutoConicToQuads::new();
                const CONIC_TOL: f32 = 0.25;
                let mut monoY = [SkPoint::default(); 10];
                let mut monoX = [SkPoint::default(); 5];
                let mut handle_quad = |builder: &mut Self, pts: &[SkPoint; 3]| {
                    let n = SkChopQuadAtYExtrema(pts, &mut monoX);
                    for i in 0..=n {
                        builder.addQuad::<EVEN>((&monoX[i * 2..i * 2 + 3]).try_into().unwrap());
                    }
                };
                while let Some(e) = iter.next() {
                    match e.fEdge {
                        Edge::kLine => self.addLine::<EVEN>(e.fPts.try_into().unwrap()),
                        Edge::kQuad => handle_quad(self, e.fPts.try_into().unwrap()),
                        Edge::kConic => {
                            let quadPts = quadder.computeQuads(
                                e.fPts.try_into().unwrap(),
                                e.fConicWeight.unwrap(),
                                CONIC_TOL,
                            );
                            // Native countQuads equals (point count - 1) / 2, allowing
                            // the lending slice to retain its borrow without copying.
                            for pts in quadPts.windows(3).step_by(2) {
                                handle_quad(self, pts.try_into().unwrap());
                            }
                        }
                        Edge::kCubic => {
                            let n = SkChopCubicAtYExtrema(e.fPts.try_into().unwrap(), &mut monoY);
                            for i in 0..=n {
                                self.addCubic::<EVEN>(
                                    (&monoY[i * 3..i * 3 + 4]).try_into().unwrap(),
                                );
                            }
                        }
                    }
                }
                self.fList.len()
            }
        }

        impl $builder {
            // Source: src/core/SkEdgeBuilder.cpp:346-364
            pub fn buildEdges<const EVEN: bool>(
                &mut self,
                raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
                shiftedClip: Option<&SkIRect>,
            ) -> usize {
                let canCullToTheRight = !raw.isKnownToBeConvex();
                let count = if raw.segmentMasks()
                    == crate::include::core::SkPathTypes::kLine_SkPathSegmentMask
                {
                    self.buildPoly::<EVEN>(raw, shiftedClip, canCullToTheRight)
                } else {
                    self.build::<EVEN>(raw, shiftedClip, canCullToTheRight)
                };
                if !canCullToTheRight {
                    crate::SkASSERT!(@source b"src/core/SkEdgeBuilder.cpp\0".as_ptr(), 361; @condition b"count != 1\0".as_ptr(); count != 1);
                }
                count
            }
            // Source: src/core/SkEdgeBuilder.cpp:366-371
            pub fn buildPathEdges<const EVEN: bool>(
                &mut self,
                path: &crate::src::core::SkPath::SkPath,
                shiftedClip: Option<&SkIRect>,
            ) -> usize {
                use crate::src::core::SkPathEnums::SkResolveConvexity;
                if let Some(raw) =
                    crate::src::core::SkPathPriv::SkPathPriv::Raw(path, SkResolveConvexity::kYes)
                {
                    return self.buildEdges::<EVEN>(&raw, shiftedClip);
                }
                0
            }
        }
    };
}
impl_shared_build!(SkBasicEdgeBuilder);

// Source: src/core/SkEdgeBuilder.h:82-98
pub struct SkAnalyticEdgeBuilder {
    // Source: src/core/SkEdgeBuilder.h:30-30
    // Native arena dies before the nonowning pointer-list allocation.
    fAlloc: core::pin::Pin<Box<crate::src::core::SkArenaAlloc::SkSTArenaAlloc<512>>>,
    fList: Vec<*mut crate::src::core::SkAnalyticEdge::SkAnalyticEdge>,
}
impl SkAnalyticEdgeBuilder {
    pub fn new() -> Self {
        Self {
            fList: Vec::new(),
            fAlloc: Box::pin(crate::src::core::SkArenaAlloc::SkSTArenaAlloc::new(512)),
        }
    }
    /// Safety: every replaced pointer must remain a live, correctly tagged
    /// analytic edge throughout later builder and scan operations.
    pub unsafe fn analyticEdgeList(
        &mut self,
    ) -> &mut [*mut crate::src::core::SkAnalyticEdge::SkAnalyticEdge] {
        &mut self.fList
    }
    // Source: src/core/SkEdgeBuilder.cpp:68-118
    fn combineVertical(
        edge: &crate::src::core::SkAnalyticEdge::SkAnalyticEdge,
        last: &mut crate::src::core::SkAnalyticEdge::SkAnalyticEdge,
    ) -> Combine {
        use crate::src::core::SkAnalyticEdge::Type;
        use Combine::*;
        let approximately_equal = |a: i32, b: i32| a.wrapping_sub(b).wrapping_abs() < 0x100;
        if last.fEdgeType != Type::kLine || last.fDX != 0 || edge.fX != last.fX {
            return kNo_Combine;
        }
        if edge.fWinding == last.fWinding {
            if edge.fLowerY == last.fUpperY {
                last.fUpperY = edge.fUpperY;
                last.fY = last.fUpperY;
                return kPartial_Combine;
            }
            if approximately_equal(edge.fUpperY, last.fLowerY) {
                last.fLowerY = edge.fLowerY;
                return kPartial_Combine;
            }
            return kNo_Combine;
        }
        if approximately_equal(edge.fUpperY, last.fUpperY) {
            if approximately_equal(edge.fLowerY, last.fLowerY) {
                return kTotal_Combine;
            }
            if edge.fLowerY < last.fLowerY {
                last.fUpperY = edge.fLowerY;
                last.fY = last.fUpperY;
                return kPartial_Combine;
            }
            last.fUpperY = last.fLowerY;
            last.fY = last.fUpperY;
            last.fLowerY = edge.fLowerY;
            last.fWinding = edge.fWinding;
            return kPartial_Combine;
        }
        if approximately_equal(edge.fLowerY, last.fLowerY) {
            if edge.fUpperY > last.fUpperY {
                last.fLowerY = edge.fUpperY;
                return kPartial_Combine;
            }
            last.fLowerY = last.fUpperY;
            last.fUpperY = edge.fUpperY;
            last.fY = last.fUpperY;
            last.fWinding = edge.fWinding;
            return kPartial_Combine;
        }
        kNo_Combine
    }
    // Source: src/core/SkEdgeBuilder.cpp:127-132
    fn is_vertical(edge: &crate::src::core::SkAnalyticEdge::SkAnalyticEdge) -> bool {
        edge.fDX == 0 && edge.fEdgeType == crate::src::core::SkAnalyticEdge::Type::kLine
    }
    // Source: src/core/SkEdgeBuilder.cpp:151-165
    pub fn addLine<const EVEN: bool>(&mut self, pts: &[SkPoint; 2]) {
        let edge = unsafe {
            self.fAlloc
                .as_mut()
                .arena()
                .make(crate::src::core::SkAnalyticEdge::SkAnalyticEdge::new())
        };
        let edge = unsafe { &mut *edge };
        if edge.setLine::<EVEN>(&pts[0], &pts[1]) {
            let combine = if Self::is_vertical(edge) && !self.fList.is_empty() {
                unsafe { Self::combineVertical(edge, &mut **self.fList.last().unwrap()) }
            } else {
                Combine::kNo_Combine
            };
            match combine {
                Combine::kTotal_Combine => {
                    self.fList.pop();
                }
                Combine::kPartial_Combine => {}
                Combine::kNo_Combine => self.fList.push(edge),
            }
        }
    }
    // Source: src/core/SkEdgeBuilder.cpp:172-177
    pub fn addQuad<const EVEN: bool>(&mut self, pts: &[SkPoint; 3]) {
        let edge = unsafe {
            self.fAlloc
                .as_mut()
                .arena()
                .make(crate::src::core::SkAnalyticEdge::SkAnalyticQuadraticEdge::new())
        };
        let edge = unsafe { &mut *edge };
        if edge.setQuadratic::<EVEN>(pts) {
            self.fList.push(&mut edge.edge);
        }
    }
    // Source: src/core/SkEdgeBuilder.cpp:185-190
    pub fn addCubic<const EVEN: bool>(&mut self, pts: &[SkPoint; 4]) {
        let edge = unsafe {
            self.fAlloc
                .as_mut()
                .arena()
                .make(crate::src::core::SkAnalyticEdge::SkAnalyticCubicEdge::new())
        };
        let edge = unsafe { &mut *edge };
        if edge.setCubic::<EVEN>(pts) {
            self.fList.push(&mut edge.edge);
        }
    }
    // Source: src/core/SkEdgeBuilder.cpp:194-204
    /// The previous edge corresponds to edgePtr[-1] when edgePtr is after the
    /// original contiguous buffer start; None corresponds to its first entry.
    pub fn addPolyLine<const EVEN: bool>(
        &mut self,
        pts: &[SkPoint; 2],
        edge: &mut crate::src::core::SkAnalyticEdge::SkAnalyticEdge,
        previous: Option<&mut crate::src::core::SkAnalyticEdge::SkAnalyticEdge>,
    ) -> Combine {
        if edge.setLine::<EVEN>(&pts[0], &pts[1]) {
            if Self::is_vertical(edge) {
                if let Some(last) = previous {
                    return Self::combineVertical(edge, last);
                }
            }
            return Combine::kNo_Combine;
        }
        Combine::kPartial_Combine
    }
    // Source: src/core/SkEdgeBuilder.cpp:210-212
    pub fn recoverClip(&self, src: &SkIRect) -> SkRect {
        SkRect {
            fLeft: src.fLeft as f32,
            fTop: src.fTop as f32,
            fRight: src.fRight as f32,
            fBottom: src.fBottom as f32,
        }
    }
    // Source: src/core/SkEdgeBuilder.cpp:214-217
    pub fn allocEdges(
        &mut self,
        n: usize,
    ) -> (*mut crate::src::core::SkAnalyticEdge::SkAnalyticEdge, usize) {
        let size = core::mem::size_of::<crate::src::core::SkAnalyticEdge::SkAnalyticEdge>();
        let allocation = unsafe {
            self.fAlloc
                .as_mut()
                .arena()
                .makeArrayDefault::<crate::src::core::SkAnalyticEdge::SkAnalyticEdge>(n)
        };
        (allocation, size)
    }
}
impl_shared_build!(SkAnalyticEdgeBuilder);

// Source: src/core/SkEdgeBuilder.h:23-54
// Native inherited protocol for the two actual final owners. The associated
// Edge projects their real concrete edge storage; no provider or second arena
// is created. Existing build algorithms remain single implementations on those
// owners. EVEN retains the existing translation's compile-time edge precision.
pub trait SkEdgeBuilder<const EVEN: bool> {
    type Edge;
    // Source: src/core/SkEdgeBuilder.h:25-26
    fn buildEdges(
        &mut self,
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        clip: Option<&SkIRect>,
    ) -> usize;
    fn buildPathEdges(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        clip: Option<&SkIRect>,
    ) -> usize;
    // Source: src/core/SkEdgeBuilder.h:48-49
    /// Safety: successful storage is arena-owned and cannot outlive this owner;
    /// a native unsuccessful allocation may leave size uninitialized.
    unsafe fn allocEdges(
        &mut self,
        n: usize,
        size: &mut core::mem::MaybeUninit<usize>,
    ) -> *mut Self::Edge;
    fn recoverClip(&self, clip: &SkIRect) -> SkRect;
    // Source: src/core/SkEdgeBuilder.h:51-54
    fn addLine(&mut self, points: &[SkPoint; 2]);
    fn addQuad(&mut self, points: &[SkPoint; 3]);
    fn addCubic(&mut self, points: &[SkPoint; 4]);
    /// Safety: edge/previous project actual disjoint native objects; previous
    /// is exactly the predecessor selected by the original edge cursor.
    unsafe fn addPolyLine(
        &mut self,
        points: &[SkPoint; 2],
        edge: &mut Self::Edge,
        previous: Option<&mut Self::Edge>,
    ) -> Combine;
}
// Source: src/core/SkEdgeBuilder.h:57-79
// Existing concrete methods implement the actual native final subclass. These
// are type/dispatch adapters only: no duplicate native algorithm credit.
impl<const EVEN: bool> SkEdgeBuilder<EVEN> for SkBasicEdgeBuilder {
    type Edge = SkEdge;
    fn buildEdges(
        &mut self,
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        clip: Option<&SkIRect>,
    ) -> usize {
        SkBasicEdgeBuilder::buildEdges::<EVEN>(self, raw, clip)
    }
    fn buildPathEdges(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        clip: Option<&SkIRect>,
    ) -> usize {
        SkBasicEdgeBuilder::buildPathEdges::<EVEN>(self, path, clip)
    }
    unsafe fn allocEdges(
        &mut self,
        n: usize,
        size: &mut core::mem::MaybeUninit<usize>,
    ) -> *mut SkEdge {
        unsafe { SkBasicEdgeBuilder::allocEdges(self, n, size.as_mut_ptr()).cast() }
    }
    fn recoverClip(&self, clip: &SkIRect) -> SkRect {
        SkBasicEdgeBuilder::recoverClip(self, clip)
    }
    fn addLine(&mut self, points: &[SkPoint; 2]) {
        SkBasicEdgeBuilder::addLine::<EVEN>(self, points);
    }
    fn addQuad(&mut self, points: &[SkPoint; 3]) {
        SkBasicEdgeBuilder::addQuad::<EVEN>(self, points);
    }
    fn addCubic(&mut self, points: &[SkPoint; 4]) {
        SkBasicEdgeBuilder::addCubic::<EVEN>(self, points);
    }
    unsafe fn addPolyLine(
        &mut self,
        points: &[SkPoint; 2],
        edge: &mut SkEdge,
        previous: Option<&mut SkEdge>,
    ) -> Combine {
        let mut previous = previous.map_or(core::ptr::null_mut(), |p| p as *mut SkEdge as *mut u8);
        unsafe {
            SkBasicEdgeBuilder::addPolyLine(
                self,
                points.as_ptr(),
                (edge as *mut SkEdge).cast(),
                &mut previous,
            )
        }
    }
}
// Source: src/core/SkEdgeBuilder.h:82-97
impl<const EVEN: bool> SkEdgeBuilder<EVEN> for SkAnalyticEdgeBuilder {
    type Edge = crate::src::core::SkAnalyticEdge::SkAnalyticEdge;
    fn buildEdges(
        &mut self,
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        clip: Option<&SkIRect>,
    ) -> usize {
        SkAnalyticEdgeBuilder::buildEdges::<EVEN>(self, raw, clip)
    }
    fn buildPathEdges(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        clip: Option<&SkIRect>,
    ) -> usize {
        SkAnalyticEdgeBuilder::buildPathEdges::<EVEN>(self, path, clip)
    }
    unsafe fn allocEdges(
        &mut self,
        n: usize,
        size: &mut core::mem::MaybeUninit<usize>,
    ) -> *mut Self::Edge {
        let (pointer, element_size) = SkAnalyticEdgeBuilder::allocEdges(self, n);
        size.write(element_size);
        pointer
    }
    fn recoverClip(&self, clip: &SkIRect) -> SkRect {
        SkAnalyticEdgeBuilder::recoverClip(self, clip)
    }
    fn addLine(&mut self, points: &[SkPoint; 2]) {
        SkAnalyticEdgeBuilder::addLine::<EVEN>(self, points);
    }
    fn addQuad(&mut self, points: &[SkPoint; 3]) {
        SkAnalyticEdgeBuilder::addQuad::<EVEN>(self, points);
    }
    fn addCubic(&mut self, points: &[SkPoint; 4]) {
        SkAnalyticEdgeBuilder::addCubic::<EVEN>(self, points);
    }
    unsafe fn addPolyLine(
        &mut self,
        points: &[SkPoint; 2],
        edge: &mut Self::Edge,
        previous: Option<&mut Self::Edge>,
    ) -> Combine {
        SkAnalyticEdgeBuilder::addPolyLine::<EVEN>(self, points, edge, previous)
    }
}
