// Copyright 2017 Google LLC. BSD-style license.
use crate::src::core::{
    SkPoint::SkPoint,
    SkPointPriv::SkPointPriv,
    SkRect::SkRect,
    SkRectPriv::SkRectPriv,
    SkScalar::{SK_ScalarNearlyZero, SkScalarNearlyZero, SkScalarRoundToInt},
    SkTInternalLList::{Linked, Links, SkTInternalLList},
};
use core::{mem::MaybeUninit, ptr::null_mut};
// Native AutoSTMalloc/AutoSTArray POD storage has 64 inline elements. This
// language-only owner keeps that threshold and does not relocate active nodes.
struct AutoBuffer<T> {
    inline: [MaybeUninit<T>; 64],
    heap: Option<Box<[MaybeUninit<T>]>>,
}
impl<T> AutoBuffer<T> {
    fn new(n: usize) -> Self {
        Self {
            inline: core::array::from_fn(|_| MaybeUninit::uninit()),
            heap: if n > 64 {
                Some((0..n).map(|_| MaybeUninit::uninit()).collect())
            } else {
                None
            },
        }
    }
    fn as_mut_ptr(&mut self) -> *mut T {
        match self.heap.as_mut() {
            Some(h) => h.as_mut_ptr().cast(),
            None => self.inline.as_mut_ptr().cast(),
        }
    }
}
// Source: src/utils/SkPolyUtils.cpp:36-38
#[derive(Clone, Copy, Default)]
struct OffsetSegment {
    fP0: SkPoint,
    fV: SkPoint,
}
// Source: src/utils/SkPolyUtils.cpp:40-40
const kCrossTolerance: f32 = SK_ScalarNearlyZero * SK_ScalarNearlyZero;
// Source: src/utils/SkPolyUtils.cpp:46-54
fn compute_side(p0: SkPoint, v: SkPoint, p: SkPoint) -> i32 {
    let perpDot = v.cross(p - p0);
    if !(perpDot.abs() <= kCrossTolerance) {
        if perpDot > 0.0 {
            1
        } else {
            -1
        }
    } else {
        0
    }
}
// Source: src/utils/SkPolyUtils.h:78-78
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkPolyUtils.cpp:57-75
pub fn SkGetPolygonWinding(polygonVerts: &[SkPoint]) -> i32 {
    if polygonVerts.len() < 3 {
        return 0;
    }
    let mut quadArea = 0.0;
    let mut v0 = polygonVerts[1] - polygonVerts[0];
    for curr in 2..polygonVerts.len() {
        let v1 = polygonVerts[curr] - polygonVerts[0];
        quadArea += v0.cross(v1);
        v0 = v1;
    }
    if quadArea.abs() <= kCrossTolerance {
        return 0;
    }
    if quadArea > 0.0 {
        1
    } else {
        -1
    }
}
// Source: src/utils/SkPolyUtils.cpp:78-88
fn compute_offset_vector(
    p0: SkPoint,
    p1: SkPoint,
    offset: f32,
    side: i32,
    vector: &mut SkPoint,
) -> bool {
    crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 80; @condition b"side == -1 || side == 1\0".as_ptr(); side==-1||side==1);
    let mut perp = SkPoint::make(p0.fY - p1.fY, p1.fX - p0.fX);
    if !perp.setLength(offset * side as f32) {
        return false;
    }
    *vector = perp;
    true
}
// Source: src/utils/SkPolyUtils.cpp:91-94
fn outside_interval(numer: f32, denom: f32, denomPositive: bool) -> bool {
    (denomPositive && (numer < 0.0 || numer > denom))
        || (!denomPositive && (numer > 0.0 || numer < denom))
}
// Source: src/utils/SkPolyUtils.cpp:97-99
fn zero_length(v: SkPoint, vdotv: f32) -> bool {
    !(v.fX.is_finite() && v.fY.is_finite() && vdotv != 0.0)
}
// Source: src/utils/SkPolyUtils.cpp:106-194
fn compute_intersection(
    s0: OffsetSegment,
    s1: OffsetSegment,
    p: &mut SkPoint,
    s: &mut f32,
    t: &mut f32,
) -> bool {
    let v0 = s0.fV;
    let v1 = s1.fV;
    let w = s1.fP0 - s0.fP0;
    let mut denom = v0.cross(v1);
    let denomPositive = denom > 0.0;
    let (mut sNumer, mut tNumer);
    if denom.abs() <= kCrossTolerance {
        if !(w.cross(v0).abs() <= kCrossTolerance) || !(w.cross(v1).abs() <= kCrossTolerance) {
            return false;
        }
        let v0dotv0 = v0.dot(v0);
        if zero_length(v0, v0dotv0) {
            let v1dotv1 = v1.dot(v1);
            if zero_length(v1, v1dotv1) {
                if !SkPointPriv::CanNormalize(w.fX, w.fY) {
                    *p = s0.fP0;
                    *s = 0.0;
                    *t = 0.0;
                    return true;
                } else {
                    return false;
                }
            }
            tNumer = v1.dot(-w);
            denom = v1dotv1;
            if outside_interval(tNumer, denom, true) {
                return false;
            }
            sNumer = 0.0;
        } else {
            sNumer = v0.dot(w);
            denom = v0dotv0;
            tNumer = 0.0;
            if outside_interval(sNumer, denom, true) {
                let v1dotv1 = v1.dot(v1);
                if zero_length(v1, v1dotv1) {
                    return false;
                }
                let oldSNumer = sNumer;
                sNumer = v0.dot(w + v1);
                tNumer = denom;
                if outside_interval(sNumer, denom, true) {
                    if sNumer * oldSNumer > 0.0 {
                        return false;
                    }
                    sNumer = 0.0;
                    tNumer = v1.dot(-w);
                    denom = v1dotv1;
                }
            }
        }
    } else {
        sNumer = w.cross(v1);
        if outside_interval(sNumer, denom, denomPositive) {
            return false;
        }
        tNumer = w.cross(v0);
        if outside_interval(tNumer, denom, denomPositive) {
            return false;
        }
    }
    let localS = sNumer / denom;
    let localT = tNumer / denom;
    *p = s0.fP0 + v0 * localS;
    *s = localS;
    *t = localT;
    true
}
// Source: src/utils/SkPolyUtils.h:87-87
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkPolyUtils.cpp:196-250
pub fn SkIsConvexPolygon(polygonVerts: &[SkPoint]) -> bool {
    let n = polygonVerts.len();
    if n < 3 {
        return false;
    }
    let mut lastPerpDot = 0.0;
    let (mut xSignChangeCount, mut ySignChangeCount) = (0, 0);
    let (mut currIndex, mut nextIndex) = (0, 1);
    let mut v0 = polygonVerts[currIndex] - polygonVerts[n - 1];
    let (mut lastVx, mut lastVy) = (v0.fX, v0.fY);
    let mut v1 = polygonVerts[nextIndex] - polygonVerts[currIndex];
    for i in 0..n {
        if !polygonVerts[i].isFinite() {
            return false;
        }
        let perpDot = v0.cross(v1);
        if lastPerpDot * perpDot < 0.0 {
            return false;
        }
        if perpDot != 0.0 {
            lastPerpDot = perpDot;
        }
        if lastVx * v1.fX < 0.0 {
            xSignChangeCount += 1;
        }
        if lastVy * v1.fY < 0.0 {
            ySignChangeCount += 1;
        }
        if xSignChangeCount > 2 || ySignChangeCount > 2 {
            return false;
        }
        currIndex = nextIndex;
        nextIndex = (currIndex + 1) % n;
        if v1.fX != 0.0 {
            lastVx = v1.fX;
        }
        if v1.fY != 0.0 {
            lastVy = v1.fY;
        }
        v0 = v1;
        v1 = polygonVerts[nextIndex] - polygonVerts[currIndex];
    }
    true
}
// Source: src/utils/SkPolyUtils.cpp:253-259
#[derive(Clone, Copy)]
struct OffsetEdge {
    fPrev: *mut Self,
    fNext: *mut Self,
    fOffset: OffsetSegment,
    fIntersection: SkPoint,
    fTValue: f32,
    fIndex: u16,
    fEnd: u16,
}
impl OffsetEdge {
    // Source: src/utils/SkPolyUtils.cpp:261-266
    fn init(&mut self, start: u16, end: u16) {
        self.fIntersection = self.fOffset.fP0;
        self.fTValue = -f32::MAX;
        self.fIndex = start;
        self.fEnd = end;
    }
    // Source: src/utils/SkPolyUtils.cpp:269-282
    fn checkIntersection(&self, that: &Self, p: &mut SkPoint, s: &mut f32, t: &mut f32) -> bool {
        if self.fEnd == that.fIndex {
            let p1 = self.fOffset.fP0 + self.fOffset.fV;
            if SkPointPriv::EqualsWithinTolerance(p1, that.fOffset.fP0) {
                *p = p1;
                *s = 1.0;
                *t = 0.0;
                return true;
            }
        }
        compute_intersection(self.fOffset, that.fOffset, p, s, t)
    }
    // Source: src/utils/SkPolyUtils.cpp:287-311
    fn computeCrossingDistance(&self, that: &Self) -> f32 {
        let v0 = self.fOffset.fV;
        let v1 = that.fOffset.fV;
        let denom = v0.cross(v1);
        if denom.abs() <= kCrossTolerance {
            return f32::MAX;
        }
        let w = that.fOffset.fP0 - self.fOffset.fP0;
        let mut localS = w.cross(v1) / denom;
        if localS < 0.0 {
            localS = -localS;
        } else {
            localS -= 1.0;
        }
        localS *= localS.abs();
        localS *= v0.dot(v0);
        localS
    }
}
// Source: src/utils/SkPolyUtils.cpp:315-322
unsafe fn remove_node(node: *mut OffsetEdge, head: &mut *mut OffsetEdge) {
    unsafe {
        (*(*node).fPrev).fNext = (*node).fNext;
        (*(*node).fNext).fPrev = (*node).fPrev;
        if node == *head {
            *head = if (*node).fNext == node {
                null_mut()
            } else {
                (*node).fNext
            };
        }
    }
}
// Source: src/utils/SkPolyUtils.h:31-32
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkPolyUtils.cpp:338-485
pub fn SkInsetConvexPolygon(
    input: &[SkPoint],
    inset: f32,
    insetPolygon: &mut Vec<SkPoint>,
) -> bool {
    unsafe {
        let n = input.len();
        if n < 3 || n > u16::MAX as usize {
            return false;
        }
        if inset < -SK_ScalarNearlyZero || !inset.is_finite() {
            return false;
        }
        if inset <= SK_ScalarNearlyZero {
            insetPolygon.extend_from_slice(input);
            return true;
        }
        let winding = SkGetPolygonWinding(input);
        if winding == 0 {
            return false;
        }
        let mut storage = AutoBuffer::<OffsetEdge>::new(n);
        let edges = storage.as_mut_ptr();
        let mut prev = n - 1;
        for curr in 0..n {
            let next = (curr + 1) % n;
            if !input[curr].isFinite() {
                return false;
            }
            if compute_side(input[prev], input[curr] - input[prev], input[next]) * winding < 0 {
                return false;
            }
            let v = input[next] - input[curr];
            let mut perp = SkPoint::make(-v.fY, v.fX);
            perp.setLength(inset * winding as f32);
            edges.add(curr).write(OffsetEdge {
                fPrev: edges.add(prev),
                fNext: edges.add(next),
                fOffset: OffsetSegment {
                    fP0: input[curr] + perp,
                    fV: v,
                },
                fIntersection: input[curr] + perp,
                fTValue: -f32::MAX,
                fIndex: 0,
                fEnd: 0,
            });
            prev = curr;
        }
        let mut head = edges;
        let mut curr = head;
        let mut prev = (*curr).fPrev;
        let mut count = n as i32;
        let mut iterations = 0u32;
        let maxIterations = (n as u32).wrapping_mul(n as u32);
        while !head.is_null() && prev != curr {
            iterations = iterations.wrapping_add(1);
            if iterations > maxIterations {
                return false;
            }
            let (mut s, mut t) = (0.0, 0.0);
            let mut intersection = SkPoint::default();
            if compute_intersection(
                (*prev).fOffset,
                (*curr).fOffset,
                &mut intersection,
                &mut s,
                &mut t,
            ) {
                if s < (*prev).fTValue {
                    remove_node(prev, &mut head);
                    count -= 1;
                    prev = (*prev).fPrev;
                } else if (*curr).fTValue > -f32::MAX
                    && SkPointPriv::EqualsWithinToleranceWith(
                        intersection,
                        (*curr).fIntersection,
                        1.0e-6,
                    )
                {
                    break;
                } else {
                    (*curr).fIntersection = intersection;
                    (*curr).fTValue = t;
                    prev = curr;
                    curr = (*curr).fNext;
                }
            } else {
                let side = winding
                    * compute_side((*curr).fOffset.fP0, (*curr).fOffset.fV, (*prev).fOffset.fP0);
                if side < 0
                    && side
                        == winding
                            * compute_side(
                                (*curr).fOffset.fP0,
                                (*curr).fOffset.fV,
                                (*prev).fOffset.fP0 + (*prev).fOffset.fV,
                            )
                {
                    remove_node(prev, &mut head);
                    count -= 1;
                    prev = (*prev).fPrev;
                } else {
                    remove_node(curr, &mut head);
                    count -= 1;
                    curr = (*curr).fNext;
                }
            }
        }
        insetPolygon.clear();
        if head.is_null() {
            return false;
        }
        if count >= 0 {
            insetPolygon.reserve(count as usize);
        }
        let mut currIndex = 0;
        insetPolygon.push((*head).fIntersection);
        curr = (*head).fNext;
        while curr != head {
            if !SkPointPriv::EqualsWithinToleranceWith(
                (*curr).fIntersection,
                insetPolygon[currIndex],
                0.01,
            ) {
                insetPolygon.push((*curr).fIntersection);
                currIndex += 1;
            }
            curr = (*curr).fNext;
        }
        if currIndex >= 1
            && SkPointPriv::EqualsWithinToleranceWith(
                insetPolygon[0],
                insetPolygon[currIndex],
                0.01,
            )
        {
            insetPolygon.pop();
        }
        SkIsConvexPolygon(insetPolygon)
    }
}
// Source: src/utils/SkPolyUtils.h:66-67
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkPolyUtils.cpp:490-522
pub fn SkComputeRadialSteps(
    v1: &SkPoint,
    v2: &SkPoint,
    offset: f32,
    rotSin: &mut f32,
    rotCos: &mut f32,
    n: &mut i32,
) -> bool {
    let rCos = v1.dot(*v2);
    if !rCos.is_finite() {
        return false;
    }
    let rSin = v1.cross(*v2);
    if !rSin.is_finite() {
        return false;
    }
    let theta = rSin.atan2(rCos);
    let floatSteps = (offset * theta * 0.25).abs();
    if floatSteps >= u16::MAX as f32 {
        return false;
    }
    let steps = SkScalarRoundToInt(floatSteps);
    let dTheta = if steps > 0 { theta / steps as f32 } else { 0.0 };
    *rotSin = dTheta.sin();
    *rotCos = dTheta.cos();
    if steps > 0 && (*rotSin == 0.0 || *rotCos == 1.0) {
        return false;
    }
    *n = steps;
    true
}
// Source: src/utils/SkPolyUtils.cpp:527-529
fn left(p0: SkPoint, p1: SkPoint) -> bool {
    p0.fX < p1.fX || (!(p0.fX > p1.fX) && p0.fY > p1.fY)
}
// Source: src/utils/SkPolyUtils.cpp:532-534
fn right(p0: SkPoint, p1: SkPoint) -> bool {
    p0.fX > p1.fX || (!(p0.fX < p1.fX) && p0.fY < p1.fY)
}
// Source: src/utils/SkPolyUtils.cpp:542-546
#[derive(Clone, Copy)]
struct Vertex {
    fPosition: SkPoint,
    fIndex: u16,
    fPrevIndex: u16,
    fNextIndex: u16,
    fFlags: u16,
}
impl Vertex {
    // Source: src/utils/SkPolyUtils.cpp:537-539
    fn Left(a: &Self, b: &Self) -> bool {
        left(a.fPosition, b.fPosition)
    }
}
// Source: src/utils/SkPolyUtils.cpp:557-560
const kPrevLeft_VertexFlag: u16 = 0x1;
const kNextLeft_VertexFlag: u16 = 0x2;
// Source: src/utils/SkPolyUtils.cpp:688-694
#[derive(Clone, Copy)]
struct ActiveEdge {
    fSegment: OffsetSegment,
    fIndex0: u16,
    fIndex1: u16,
    fChild: [*mut Self; 2],
    fAbove: *mut Self,
    fBelow: *mut Self,
    fRed: i32,
}
impl ActiveEdge {
    // Source: src/utils/SkPolyUtils.cpp:555-555
    fn empty() -> Self {
        Self {
            fSegment: OffsetSegment::default(),
            fIndex0: 0,
            fIndex1: 0,
            fChild: [null_mut(); 2],
            fAbove: null_mut(),
            fBelow: null_mut(),
            fRed: 0,
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:556-565
    fn new(p0: SkPoint, v: SkPoint, index0: u16, index1: u16) -> Self {
        Self {
            fSegment: OffsetSegment { fP0: p0, fV: v },
            fIndex0: index0,
            fIndex1: index1,
            fChild: [null_mut(); 2],
            fAbove: null_mut(),
            fBelow: null_mut(),
            fRed: 1,
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:570-621
    fn aboveIfLeft(&self, that: &Self) -> bool {
        let p0 = self.fSegment.fP0;
        let q0 = that.fSegment.fP0;
        crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 573; @condition b"p0.fX <= q0.fX\0".as_ptr(); p0.fX<=q0.fX);
        let mut d = q0 - p0;
        let v = self.fSegment.fV;
        let w = that.fSegment.fV;
        if self.fIndex0 != that.fIndex0 {
            let cross = d.cross(v);
            if cross > kCrossTolerance {
                return true;
            } else if cross < -kCrossTolerance {
                return false;
            }
        } else if self.fIndex1 == that.fIndex1 {
            return false;
        }
        d += w;
        let cross = d.cross(v);
        if cross > kCrossTolerance {
            return true;
        } else if cross < -kCrossTolerance {
            return false;
        }
        if p0.fX < q0.fX {
            return p0.fY >= q0.fY;
        } else if p0.fY > q0.fY {
            return true;
        } else if p0.fY < q0.fY {
            return false;
        }
        let p1 = p0 + v;
        let q1 = q0 + w;
        if p1.fX < q1.fX {
            p1.fY >= q1.fY
        } else {
            p1.fY > q1.fY
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:624-632
    fn above(&self, that: &Self) -> bool {
        if right(self.fSegment.fP0, that.fSegment.fP0) {
            !that.aboveIfLeft(self)
        } else {
            self.aboveIfLeft(that)
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:634-671
    fn intersect_segment(&self, q0: SkPoint, w: SkPoint, index0: u16, index1: u16) -> bool {
        if self.fIndex0 == index0
            || self.fIndex1 == index0
            || self.fIndex0 == index1
            || self.fIndex1 == index1
        {
            return false;
        }
        let p0 = self.fSegment.fP0;
        let v = self.fSegment.fV;
        let p1 = p0 + v;
        let q1 = q0 + w;
        crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 652; @condition b"q0.fX <= p1.fX + SK_ScalarNearlyZero\0".as_ptr(); q0.fX<=p1.fX+SK_ScalarNearlyZero);
        if p0.fX < q0.fX {
            if q1.fX < p1.fX {
                compute_side(p0, v, q0) * compute_side(p0, v, q1) < 0
            } else {
                compute_side(p0, v, q0) * compute_side(q0, w, p1) > 0
            }
        } else {
            if p1.fX < q1.fX {
                compute_side(q0, w, p0) * compute_side(q0, w, p1) < 0
            } else {
                compute_side(q0, w, p0) * compute_side(p0, v, q1) > 0
            }
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:673-675
    fn intersect(&self, edge: &Self) -> bool {
        self.intersect_segment(
            edge.fSegment.fP0,
            edge.fSegment.fV,
            edge.fIndex0,
            edge.fIndex1,
        )
    }
    // Source: src/utils/SkPolyUtils.cpp:677-682
    fn lessThan(&self, that: &Self) -> bool {
        crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 678; @condition b"!this->above(this)\0".as_ptr(); !self.above(self));
        crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 679; @condition b"!that->above(that)\0".as_ptr(); !that.above(that));
        crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 680; @condition b"!(this->above(that) && that->above(this))\0".as_ptr(); !(self.above(that)&&that.above(self)));
        self.above(that)
    }
    // Source: src/utils/SkPolyUtils.cpp:684-686
    fn equals(&self, index0: u16, index1: u16) -> bool {
        self.fIndex0 == index0 && self.fIndex1 == index1
    }
}
// Source: src/utils/SkPolyUtils.cpp:1080-1083
// Heap allocation contains exactly maxEdges POD entries, as in sk_malloc_throw;
// nodes are never individually deleted or relocated while the tree is active.
struct ActiveEdgeList {
    fTreeHead: ActiveEdge,
    fAllocation: Box<[MaybeUninit<ActiveEdge>]>,
    fCurrFree: usize,
    fMaxFree: usize,
}
impl ActiveEdgeList {
    // Source: src/utils/SkPolyUtils.cpp:699-703
    fn new(maxEdges: usize) -> Self {
        Self {
            fTreeHead: ActiveEdge::empty(),
            fAllocation: (0..maxEdges).map(|_| MaybeUninit::uninit()).collect(),
            fCurrFree: 0,
            fMaxFree: maxEdges,
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:709-824
    unsafe fn insert(&mut self, p0: SkPoint, p1: SkPoint, index0: u16, index1: u16) -> bool {
        unsafe {
            let v = p1 - p0;
            if !v.isFinite() {
                return false;
            }
            if self.fTreeHead.fChild[1].is_null() {
                let root = self.allocate(p0, v, index0, index1);
                self.fTreeHead.fChild[1] = root;
                crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 717; @condition b"root\0".as_ptr(); !root.is_null());
                if root.is_null() {
                    return false;
                }
                (*root).fRed = 0;
                return true;
            }
            let mut top = core::ptr::addr_of_mut!(self.fTreeHead);
            let mut grandparent: *mut ActiveEdge = null_mut();
            let mut parent: *mut ActiveEdge = null_mut();
            let mut curr = (*top).fChild[1];
            let (mut dir, mut last) = (0usize, 0usize);
            let mut pred: *mut ActiveEdge = null_mut();
            let mut succ: *mut ActiveEdge = null_mut();
            loop {
                if curr.is_null() {
                    if (!pred.is_null() && (*pred).intersect_segment(p0, v, index0, index1))
                        || (!succ.is_null() && (*succ).intersect_segment(p0, v, index0, index1))
                    {
                        return false;
                    }
                    curr = self.allocate(p0, v, index0, index1);
                    (*parent).fChild[dir] = curr;
                    crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 746; @condition b"curr\0".as_ptr(); !curr.is_null());
                    if curr.is_null() {
                        return false;
                    }
                    (*curr).fAbove = pred;
                    (*curr).fBelow = succ;
                    if !pred.is_null() {
                        if (*pred).fSegment.fP0 == (*curr).fSegment.fP0
                            && (*pred).fSegment.fV == (*curr).fSegment.fV
                        {
                            return false;
                        }
                        (*pred).fBelow = curr;
                    }
                    if !succ.is_null() {
                        if (*succ).fSegment.fP0 == (*curr).fSegment.fP0
                            && (*succ).fSegment.fV == (*curr).fSegment.fV
                        {
                            return false;
                        }
                        (*succ).fAbove = curr;
                    }
                    if Self::IsRed(parent) {
                        let dir2 = ((*top).fChild[1] == grandparent) as usize;
                        (*top).fChild[dir2] = if curr == (*parent).fChild[last] {
                            Self::SingleRotation(grandparent, 1 - last)
                        } else {
                            Self::DoubleRotation(grandparent, 1 - last)
                        };
                    }
                    break;
                } else if Self::IsRed((*curr).fChild[0]) && Self::IsRed((*curr).fChild[1]) {
                    (*curr).fRed = 1;
                    (*(*curr).fChild[0]).fRed = 0;
                    (*(*curr).fChild[1]).fRed = 0;
                    if Self::IsRed(parent) {
                        let dir2 = ((*top).fChild[1] == grandparent) as usize;
                        (*top).fChild[dir2] = if curr == (*parent).fChild[last] {
                            Self::SingleRotation(grandparent, 1 - last)
                        } else {
                            Self::DoubleRotation(grandparent, 1 - last)
                        };
                    }
                }
                last = dir;
                let side = if (*curr).fIndex0 == index0 {
                    compute_side((*curr).fSegment.fP0, (*curr).fSegment.fV, p1)
                } else {
                    compute_side((*curr).fSegment.fP0, (*curr).fSegment.fV, p0)
                };
                if side == 0 {
                    return false;
                }
                dir = (side < 0) as usize;
                if dir == 0 {
                    succ = curr;
                } else {
                    pred = curr;
                }
                if !grandparent.is_null() {
                    top = grandparent;
                }
                grandparent = parent;
                parent = curr;
                curr = (*curr).fChild[dir];
            }
            (*self.fTreeHead.fChild[1]).fRed = 0;
            #[cfg(sk_debug)]
            {
                Self::VerifyTree(self.fTreeHead.fChild[1]);
            }
            true
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:827-884
    unsafe fn replace(
        &mut self,
        p0: SkPoint,
        p1: SkPoint,
        p2: SkPoint,
        index0: u16,
        index1: u16,
        index2: u16,
    ) -> bool {
        unsafe {
            if self.fTreeHead.fChild[1].is_null() {
                return false;
            }
            let v = p2 - p1;
            let mut curr = core::ptr::addr_of_mut!(self.fTreeHead);
            let mut found: *mut ActiveEdge = null_mut();
            let mut dir = 1;
            while !(*curr).fChild[dir].is_null() {
                curr = (*curr).fChild[dir];
                if (*curr).equals(index0, index1) {
                    found = curr;
                    break;
                } else {
                    let side = if (*curr).fIndex1 == index1 {
                        compute_side((*curr).fSegment.fP0, (*curr).fSegment.fV, p0)
                    } else {
                        compute_side((*curr).fSegment.fP0, (*curr).fSegment.fV, p1)
                    };
                    if side == 0 {
                        return false;
                    }
                    dir = (side < 0) as usize;
                }
            }
            if found.is_null() {
                return false;
            }
            let pred = (*found).fAbove;
            let succ = (*found).fBelow;
            if !pred.is_null()
                && ((*pred).intersect(&*found) || (*pred).intersect_segment(p1, v, index1, index2))
            {
                return false;
            }
            if !succ.is_null()
                && ((*succ).intersect(&*found) || (*succ).intersect_segment(p1, v, index1, index2))
            {
                return false;
            }
            (*found).fSegment.fP0 = p1;
            (*found).fSegment.fV = v;
            (*found).fIndex0 = index1;
            (*found).fIndex1 = index2;
            #[cfg(sk_debug)]
            {
                Self::VerifyTree(self.fTreeHead.fChild[1]);
            }
            true
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:886-995
    unsafe fn remove(&mut self, p0: SkPoint, p1: SkPoint, index0: u16, index1: u16) -> bool {
        unsafe {
            if self.fTreeHead.fChild[1].is_null() {
                return false;
            }
            let mut curr = core::ptr::addr_of_mut!(self.fTreeHead);
            let mut parent: *mut ActiveEdge = null_mut();
            let mut grandparent: *mut ActiveEdge = null_mut();
            let mut found: *mut ActiveEdge = null_mut();
            let mut dir = 1;
            while !(*curr).fChild[dir].is_null() {
                let last = dir;
                grandparent = parent;
                parent = curr;
                curr = (*curr).fChild[dir];
                if (*curr).equals(index0, index1) {
                    found = curr;
                    dir = 0;
                } else {
                    let side = if (*curr).fIndex1 == index1 {
                        compute_side((*curr).fSegment.fP0, (*curr).fSegment.fV, p0)
                    } else {
                        compute_side((*curr).fSegment.fP0, (*curr).fSegment.fV, p1)
                    };
                    if side == 0 {
                        return false;
                    }
                    dir = (side < 0) as usize;
                }
                if !Self::IsRed(curr) && !Self::IsRed((*curr).fChild[dir]) {
                    if Self::IsRed((*curr).fChild[1 - dir]) {
                        let rotated = Self::SingleRotation(curr, dir);
                        (*parent).fChild[last] = rotated;
                        parent = rotated;
                    } else {
                        let sibling = (*parent).fChild[1 - last];
                        if !sibling.is_null() {
                            if !Self::IsRed((*sibling).fChild[1 - last])
                                && !Self::IsRed((*sibling).fChild[last])
                            {
                                (*parent).fRed = 0;
                                (*sibling).fRed = 1;
                                (*curr).fRed = 1;
                            } else {
                                let dir2 = ((*grandparent).fChild[1] == parent) as usize;
                                if Self::IsRed((*sibling).fChild[last]) {
                                    (*grandparent).fChild[dir2] =
                                        Self::DoubleRotation(parent, last);
                                } else if Self::IsRed((*sibling).fChild[1 - last]) {
                                    (*grandparent).fChild[dir2] =
                                        Self::SingleRotation(parent, last);
                                }
                                (*curr).fRed = 1;
                                (*(*grandparent).fChild[dir2]).fRed = 1;
                                (*(*(*grandparent).fChild[dir2]).fChild[0]).fRed = 0;
                                (*(*(*grandparent).fChild[dir2]).fChild[1]).fRed = 0;
                            }
                        }
                    }
                }
            }
            if !found.is_null() {
                let mut pred = (*found).fAbove;
                let succ = (*found).fBelow;
                if (!pred.is_null() && (*pred).intersect(&*found))
                    || (!succ.is_null() && (*succ).intersect(&*found))
                {
                    return false;
                }
                if found != curr {
                    (*found).fSegment = (*curr).fSegment;
                    (*found).fIndex0 = (*curr).fIndex0;
                    (*found).fIndex1 = (*curr).fIndex1;
                    (*found).fAbove = (*curr).fAbove;
                    pred = (*found).fAbove;
                } else {
                    if !succ.is_null() {
                        (*succ).fAbove = pred;
                    }
                }
                if !pred.is_null() {
                    (*pred).fBelow = (*curr).fBelow;
                }
                let childIndex = ((*parent).fChild[1] == curr) as usize;
                (*parent).fChild[childIndex] = (*curr).fChild[(*curr).fChild[0].is_null() as usize];
                (*curr).fAbove = 0xdeadbeefusize as *mut ActiveEdge;
                (*curr).fBelow = 0xdeadbeefusize as *mut ActiveEdge;
                if !self.fTreeHead.fChild[1].is_null() {
                    (*self.fTreeHead.fChild[1]).fRed = 0;
                }
            }
            if !self.fTreeHead.fChild[1].is_null() {
                (*self.fTreeHead.fChild[1]).fRed = 0;
            }
            #[cfg(sk_debug)]
            {
                Self::VerifyTree(self.fTreeHead.fChild[1]);
            }
            true
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:999-1006
    unsafe fn allocate(
        &mut self,
        p0: SkPoint,
        v: SkPoint,
        index0: u16,
        index1: u16,
    ) -> *mut ActiveEdge {
        if self.fCurrFree >= self.fMaxFree {
            return null_mut();
        }
        let p = unsafe {
            self.fAllocation
                .as_mut_ptr()
                .add(self.fCurrFree)
                .cast::<ActiveEdge>()
        };
        self.fCurrFree += 1;
        unsafe { p.write(ActiveEdge::new(p0, v, index0, index1)) };
        p
    }
    // Source: src/utils/SkPolyUtils.cpp:1011-1013
    unsafe fn IsRed(node: *mut ActiveEdge) -> bool {
        !node.is_null() && unsafe { (*node).fRed != 0 }
    }
    // Source: src/utils/SkPolyUtils.cpp:1015-1025
    unsafe fn SingleRotation(node: *mut ActiveEdge, dir: usize) -> *mut ActiveEdge {
        unsafe {
            let tmp = (*node).fChild[1 - dir];
            (*node).fChild[1 - dir] = (*tmp).fChild[dir];
            (*tmp).fChild[dir] = node;
            (*node).fRed = 1;
            (*tmp).fRed = 0;
            tmp
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:1027-1031
    unsafe fn DoubleRotation(node: *mut ActiveEdge, dir: usize) -> *mut ActiveEdge {
        unsafe {
            (*node).fChild[1 - dir] = Self::SingleRotation((*node).fChild[1 - dir], 1 - dir);
            Self::SingleRotation(node, dir)
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:1034-1078
    // Native in-file assertion helper, invoked only at the original SkDEBUGCODE
    // outlets; this is not a source audit or an added diagnostic feature.
    unsafe fn VerifyTree(tree: *mut ActiveEdge) -> i32 {
        unsafe {
            if tree.is_null() {
                return 1;
            }
            let left = (*tree).fChild[0];
            let right = (*tree).fChild[1];
            if Self::IsRed(tree) && (Self::IsRed(left) || Self::IsRed(right)) {
                crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1044; @condition b"false\0".as_ptr(); false);
                return 0;
            }
            if !(*tree).fAbove.is_null() {
                crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1050; @condition b"tree->fAbove->fBelow == tree\0".as_ptr(); (*(*tree).fAbove).fBelow==tree);
                crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1051; @condition b"tree->fAbove->lessThan(tree)\0".as_ptr(); (*(*tree).fAbove).lessThan(&*tree));
            }
            if !(*tree).fBelow.is_null() {
                crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1054; @condition b"tree->fBelow->fAbove == tree\0".as_ptr(); (*(*tree).fBelow).fAbove==tree);
                crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1055; @condition b"tree->lessThan(tree->fBelow)\0".as_ptr(); (*tree).lessThan(&*(*tree).fBelow));
            }
            if (!left.is_null() && (*tree).lessThan(&*left))
                || (!right.is_null() && (*right).lessThan(&*tree))
            {
                crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1060; @condition b"false\0".as_ptr(); false);
                return 0;
            }
            let leftCount = Self::VerifyTree(left);
            let rightCount = Self::VerifyTree(right);
            if leftCount != 0 && rightCount != 0 {
                if leftCount != rightCount {
                    crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1071; @condition b"false\0".as_ptr(); false);
                    return 0;
                }
                if Self::IsRed(tree) {
                    leftCount
                } else {
                    leftCount + 1
                }
            } else {
                0
            }
        }
    }
}
// Source: src/utils/SkPolyUtils.cpp:704-707
impl Drop for ActiveEdgeList {
    fn drop(&mut self) {
        self.fTreeHead.fChild[1] = null_mut();
    }
}
// Source: src/utils/SkPolyUtils.h:98-98
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkPolyUtils.cpp:1092-1173
pub fn SkIsSimplePolygon(polygon: &[SkPoint]) -> bool {
    unsafe {
        let n = polygon.len();
        if n < 3 {
            return false;
        }
        if SkIsConvexPolygon(polygon) {
            return true;
        }
        if n > 2048 {
            return false;
        }
        // The original priority queue supplies strict-comparator heap event ordering.
        let mut vertexQueue = crate::src::core::SkTDPQueue::SkTDPQueue::new(n, Vertex::Left);
        for i in 0..n {
            if !polygon[i].isFinite() {
                return false;
            }
            let mut v = Vertex {
                fPosition: polygon[i],
                fIndex: i as u16,
                fPrevIndex: ((i + n - 1) % n) as u16,
                fNextIndex: ((i + 1) % n) as u16,
                fFlags: 0,
            };
            if polygon[v.fPrevIndex as usize] == polygon[v.fNextIndex as usize] {
                return false;
            }
            if left(polygon[v.fPrevIndex as usize], polygon[i]) {
                v.fFlags |= kPrevLeft_VertexFlag;
            }
            if left(polygon[v.fNextIndex as usize], polygon[i]) {
                v.fFlags |= kNextLeft_VertexFlag;
            }
            vertexQueue.insert(v);
        }
        let mut sweepLine = ActiveEdgeList::new(n);
        while vertexQueue.count() > 0 {
            let v = *vertexQueue.peek();
            if v.fFlags == 0 {
                if !sweepLine.insert(
                    v.fPosition,
                    polygon[v.fPrevIndex as usize],
                    v.fIndex,
                    v.fPrevIndex,
                ) {
                    break;
                }
                if !sweepLine.insert(
                    v.fPosition,
                    polygon[v.fNextIndex as usize],
                    v.fIndex,
                    v.fNextIndex,
                ) {
                    break;
                }
            } else if v.fFlags == (kPrevLeft_VertexFlag | kNextLeft_VertexFlag) {
                if !sweepLine.remove(
                    polygon[v.fPrevIndex as usize],
                    v.fPosition,
                    v.fPrevIndex,
                    v.fIndex,
                ) {
                    break;
                }
                if !sweepLine.remove(
                    polygon[v.fNextIndex as usize],
                    v.fPosition,
                    v.fNextIndex,
                    v.fIndex,
                ) {
                    break;
                }
            } else {
                if v.fFlags & kPrevLeft_VertexFlag != 0 {
                    if !sweepLine.replace(
                        polygon[v.fPrevIndex as usize],
                        v.fPosition,
                        polygon[v.fNextIndex as usize],
                        v.fPrevIndex,
                        v.fIndex,
                        v.fNextIndex,
                    ) {
                        break;
                    }
                } else {
                    crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1161; @condition b"v.fFlags & kNextLeft_VertexFlag\0".as_ptr(); v.fFlags&kNextLeft_VertexFlag!=0);
                    if !sweepLine.replace(
                        polygon[v.fNextIndex as usize],
                        v.fPosition,
                        polygon[v.fPrevIndex as usize],
                        v.fNextIndex,
                        v.fIndex,
                        v.fPrevIndex,
                    ) {
                        break;
                    }
                }
            }
            vertexQueue.pop();
        }
        vertexQueue.count() == 0
    }
}
// Source: src/utils/SkPolyUtils.cpp:1178-1184
unsafe fn setup_offset_edge(curr: *mut OffsetEdge, p0: SkPoint, p1: SkPoint, start: u16, end: u16) {
    unsafe {
        curr.write(OffsetEdge {
            fPrev: null_mut(),
            fNext: null_mut(),
            fOffset: OffsetSegment {
                fP0: p0,
                fV: p1 - p0,
            },
            fIntersection: p0,
            fTValue: -f32::MAX,
            fIndex: start,
            fEnd: end,
        });
    }
}
// Source: src/utils/SkPolyUtils.cpp:1186-1193
fn is_reflex_vertex(
    input: &[SkPoint],
    winding: i32,
    offset: f32,
    prev: usize,
    curr: usize,
    next: usize,
) -> bool {
    let side = compute_side(input[prev], input[curr] - input[prev], input[next]);
    (side * winding) as f32 * offset < 0.0
}
// Source: src/utils/SkPolyUtils.h:49-50
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkPolyUtils.cpp:1195-1468
pub fn SkOffsetSimplePolygon(
    input: &[SkPoint],
    bounds: &SkRect,
    offset: f32,
    output: &mut Vec<SkPoint>,
    mut polygonIndices: Option<&mut Vec<i32>>,
) -> bool {
    unsafe {
        let n = input.len();
        if n < 3 || n >= u16::MAX as usize || !offset.is_finite() {
            return false;
        }
        let w = SkRectPriv::HalfWidth(bounds).abs();
        let h = SkRectPriv::HalfHeight(bounds).abs();
        let minimum = if h < w { h } else { w };
        if offset > minimum {
            return false;
        }
        if SkScalarNearlyZero(offset, SK_ScalarNearlyZero) {
            for i in 0..n {
                output.push(input[i]);
                if let Some(indices) = polygonIndices.as_deref_mut() {
                    indices.push(i as i32);
                }
            }
            return true;
        }
        let winding = SkGetPolygonWinding(input);
        if winding == 0 {
            return false;
        }
        let mut normalStorage = AutoBuffer::<SkPoint>::new(n);
        let normals = normalStorage.as_mut_ptr();
        let mut numEdges = 0u32;
        for curr in 0..n {
            let prev = (curr + n - 1) % n;
            let next = (curr + 1) % n;
            if !input[curr].isFinite() {
                return false;
            }
            let mut normal = SkPoint::default();
            if !compute_offset_vector(input[curr], input[next], offset, winding, &mut normal) {
                return false;
            }
            normals.add(curr).write(normal);
            if curr > 0 && is_reflex_vertex(input, winding, offset, prev, curr, next) {
                let (mut sin, mut cos, mut steps) = (0.0, 0.0, 0);
                if !SkComputeRadialSteps(
                    &*normals.add(prev),
                    &*normals.add(curr),
                    offset,
                    &mut sin,
                    &mut cos,
                    &mut steps,
                ) {
                    return false;
                }
                numEdges += steps.max(1) as u32;
            }
            numEdges += 1;
        }
        if is_reflex_vertex(input, winding, offset, n - 1, 0, 1) {
            let (mut sin, mut cos, mut steps) = (0.0, 0.0, 0);
            if !SkComputeRadialSteps(
                &*normals.add(n - 1),
                &*normals,
                offset,
                &mut sin,
                &mut cos,
                &mut steps,
            ) {
                return false;
            }
            numEdges += steps.max(1) as u32;
        }
        if numEdges > i32::MAX as u32 {
            return false;
        }
        let mut storage = AutoBuffer::<OffsetEdge>::new(numEdges as usize);
        let edges = storage.as_mut_ptr();
        let mut prevEdge: *mut OffsetEdge = null_mut();
        let mut count = 0usize;
        for currIndex in 0..n {
            let prevIndex = (currIndex + n - 1) % n;
            let nextIndex = (currIndex + 1) % n;
            if is_reflex_vertex(input, winding, offset, prevIndex, currIndex, nextIndex) {
                let (mut sin, mut cos, mut steps) = (0.0, 0.0, 0);
                let mut prevNormal = *normals.add(prevIndex);
                if !SkComputeRadialSteps(
                    &prevNormal,
                    &*normals.add(currIndex),
                    offset,
                    &mut sin,
                    &mut cos,
                    &mut steps,
                ) {
                    return false;
                }
                for _ in 0..steps - 1 {
                    let currNormal = SkPoint::make(
                        prevNormal.fX * cos - prevNormal.fY * sin,
                        prevNormal.fY * cos + prevNormal.fX * sin,
                    );
                    let curr = edges.add(count);
                    count += 1;
                    setup_offset_edge(
                        curr,
                        input[currIndex] + prevNormal,
                        input[currIndex] + currNormal,
                        currIndex as u16,
                        currIndex as u16,
                    );
                    prevNormal = currNormal;
                    (*curr).fPrev = prevEdge;
                    if !prevEdge.is_null() {
                        (*prevEdge).fNext = curr;
                    }
                    prevEdge = curr;
                }
                let curr = edges.add(count);
                count += 1;
                setup_offset_edge(
                    curr,
                    input[currIndex] + prevNormal,
                    input[currIndex] + *normals.add(currIndex),
                    currIndex as u16,
                    currIndex as u16,
                );
                (*curr).fPrev = prevEdge;
                if !prevEdge.is_null() {
                    (*prevEdge).fNext = curr;
                }
                prevEdge = curr;
            }
            let curr = edges.add(count);
            count += 1;
            setup_offset_edge(
                curr,
                input[currIndex] + *normals.add(currIndex),
                input[nextIndex] + *normals.add(currIndex),
                currIndex as u16,
                nextIndex as u16,
            );
            (*curr).fPrev = prevEdge;
            if !prevEdge.is_null() {
                (*prevEdge).fNext = curr;
            }
            prevEdge = curr;
        }
        crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1338; @condition b"prevEdge\0".as_ptr(); !prevEdge.is_null());
        (*prevEdge).fNext = edges;
        (*edges).fPrev = prevEdge;
        crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1343; @condition b"edgeData.size() == (int)numEdges\0".as_ptr(); (count) == (numEdges as usize));
        let mut head = edges;
        let mut curr = head;
        let mut count = numEdges;
        let mut iterations = 0u64;
        let maxIterations = numEdges as u64 * numEdges as u64;
        while !head.is_null() && prevEdge != curr && count > 0 {
            iterations += 1;
            if iterations > maxIterations {
                return false;
            }
            let (mut s, mut t) = (0.0, 0.0);
            let mut intersection = SkPoint::default();
            if (*prevEdge).checkIntersection(&*curr, &mut intersection, &mut s, &mut t) {
                if s < (*prevEdge).fTValue {
                    remove_node(prevEdge, &mut head);
                    count -= 1;
                    prevEdge = (*prevEdge).fPrev;
                } else if (*curr).fTValue > -f32::MAX
                    && SkPointPriv::EqualsWithinToleranceWith(
                        intersection,
                        (*curr).fIntersection,
                        1.0e-6,
                    )
                {
                    break;
                } else {
                    (*curr).fIntersection = intersection;
                    (*curr).fTValue = t;
                    (*curr).fIndex = (*prevEdge).fEnd;
                    prevEdge = curr;
                    curr = (*curr).fNext;
                }
            } else {
                let prevPrev = (*prevEdge).fPrev;
                let currNext = (*curr).fNext;
                let dist0 = (*curr).computeCrossingDistance(&*prevPrev);
                let dist1 = (*prevEdge).computeCrossingDistance(&*currNext);
                if dist0 < 0.0 && dist1 < 0.0 {
                    let mut p1 = (*prevPrev).fOffset.fP0 + (*prevPrev).fOffset.fV;
                    let prevSame = SkPointPriv::EqualsWithinTolerance(p1, (*prevEdge).fOffset.fP0);
                    p1 = (*curr).fOffset.fP0 + (*curr).fOffset.fV;
                    let currSame = SkPointPriv::EqualsWithinTolerance(p1, (*currNext).fOffset.fP0);
                    if currSame && !prevSame {
                        remove_node(curr, &mut head);
                        curr = currNext;
                        count -= 1;
                        continue;
                    } else if prevSame && !currSame {
                        remove_node(prevEdge, &mut head);
                        prevEdge = prevPrev;
                        count -= 1;
                        continue;
                    }
                }
                if dist0 < dist1 {
                    remove_node(prevEdge, &mut head);
                    prevEdge = prevPrev;
                } else {
                    remove_node(curr, &mut head);
                    curr = currNext;
                }
                count -= 1;
            }
        }
        output.clear();
        if head.is_null() || count == 0 || count >= u16::MAX as u32 {
            return false;
        }
        output.reserve(count as usize);
        let mut currIndex = 0;
        output.push((*head).fIntersection);
        if let Some(indices) = polygonIndices.as_deref_mut() {
            indices.push((*head).fIndex as i32);
        }
        curr = (*head).fNext;
        while curr != head {
            if !SkPointPriv::EqualsWithinToleranceWith(
                (*curr).fIntersection,
                output[currIndex],
                0.01,
            ) {
                output.push((*curr).fIntersection);
                if let Some(indices) = polygonIndices.as_deref_mut() {
                    indices.push((*curr).fIndex as i32);
                }
                currIndex += 1;
            }
            curr = (*curr).fNext;
        }
        if currIndex >= 1
            && SkPointPriv::EqualsWithinToleranceWith(output[0], output[currIndex], 0.01)
        {
            output.pop();
            if let Some(indices) = polygonIndices.as_deref_mut() {
                indices.pop();
            }
        }
        let offsetWinding = SkGetPolygonWinding(output) as f32;
        winding as f32 * offsetWinding > 0.0 && SkIsSimplePolygon(output)
    }
}
// Source: src/utils/SkPolyUtils.h:51-51
// Real nullable default output argument; no alternate offset implementation.
pub fn SkOffsetSimplePolygon_no_indices(
    input: &[SkPoint],
    bounds: &SkRect,
    offset: f32,
    output: &mut Vec<SkPoint>,
) -> bool {
    SkOffsetSimplePolygon(input, bounds, offset, output, None)
}
// Source: src/utils/SkPolyUtils.cpp:1472-1481
#[derive(Clone, Copy, PartialEq, Eq)]
enum VertexType {
    kConvex,
    kReflex,
}
struct TriangulationVertex {
    links: Links<Self>,
    fPosition: SkPoint,
    fVertexType: VertexType,
    fIndex: u16,
    fPrevIndex: u16,
    fNextIndex: u16,
}
unsafe impl Linked for TriangulationVertex {
    unsafe fn links(node: *mut Self) -> *mut Links<Self> {
        unsafe { core::ptr::addr_of_mut!((*node).links) }
    }
}
// Source: src/utils/SkPolyUtils.cpp:1484-1493
fn compute_triangle_bounds(p0: SkPoint, p1: SkPoint, p2: SkPoint, bounds: &mut SkRect) {
    fn vmin(a: f32, b: f32) -> f32 {
        if a < b {
            a
        } else {
            b
        }
    }
    fn vmax(a: f32, b: f32) -> f32 {
        if a > b {
            a
        } else {
            b
        }
    }
    fn smin(a: f32, b: f32) -> f32 {
        if b < a {
            b
        } else {
            a
        }
    }
    fn smax(a: f32, b: f32) -> f32 {
        if a < b {
            b
        } else {
            a
        }
    }
    let min = [
        vmin(p0.fX, p1.fX),
        vmin(p0.fY, p1.fY),
        vmin(p0.fX, p2.fX),
        vmin(p0.fY, p2.fY),
    ];
    let max = [
        vmax(p0.fX, p1.fX),
        vmax(p0.fY, p1.fY),
        vmax(p0.fX, p2.fX),
        vmax(p0.fY, p2.fY),
    ];
    bounds.setLTRB(
        smin(min[0], min[2]),
        smin(min[1], min[3]),
        smax(max[0], max[2]),
        smax(max[1], max[3]),
    );
}
// Source: src/utils/SkPolyUtils.cpp:1497-1520
fn point_in_triangle(p0: SkPoint, p1: SkPoint, p2: SkPoint, p: SkPoint) -> bool {
    let v0 = p1 - p0;
    let v1 = p2 - p1;
    let n = v0.cross(v1);
    let w0 = p - p0;
    if n * v0.cross(w0) < SK_ScalarNearlyZero {
        return false;
    }
    let w1 = p - p1;
    if n * v1.cross(w1) < SK_ScalarNearlyZero {
        return false;
    }
    let v2 = p0 - p2;
    let w2 = p - p2;
    if n * v2.cross(w2) < SK_ScalarNearlyZero {
        return false;
    }
    true
}
// Source: src/utils/SkPolyUtils.cpp:1607-1613
struct ReflexHash {
    fBounds: SkRect,
    fHCount: i32,
    fVCount: i32,
    fNumVerts: i32,
    fGridConversion: SkPoint,
    fGrid: Vec<SkTInternalLList<TriangulationVertex>>,
}
impl ReflexHash {
    fn new() -> Self {
        Self {
            fBounds: SkRect::default(),
            fHCount: 0,
            fVCount: 0,
            fNumVerts: 0,
            fGridConversion: SkPoint::default(),
            fGrid: Vec::new(),
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:1525-1553
    fn init(&mut self, bounds: SkRect, vertexCount: i32) -> bool {
        self.fBounds = bounds;
        self.fNumVerts = 0;
        let width = bounds.width();
        let height = bounds.height();
        if !width.is_finite() || !height.is_finite() {
            return false;
        }
        let hCount = crate::src::core::SkFloatingPoint::sk_ieee_float_divide(
            vertexCount as f32 * width,
            height,
        )
        .sqrt();
        if !hCount.is_finite() {
            return false;
        }
        self.fHCount = SkScalarRoundToInt(hCount).min(vertexCount).max(1);
        self.fVCount = vertexCount / self.fHCount;
        self.fGridConversion = SkPoint::make(
            crate::src::core::SkFloatingPoint::sk_ieee_float_divide(
                self.fHCount as f32 - 0.001,
                width,
            ),
            crate::src::core::SkFloatingPoint::sk_ieee_float_divide(
                self.fVCount as f32 - 0.001,
                height,
            ),
        );
        if !self.fGridConversion.isFinite() {
            return false;
        }
        self.fGrid.resize_with(
            (self.fHCount * self.fVCount) as usize,
            SkTInternalLList::new,
        );
        for cell in &mut self.fGrid {
            cell.reset();
        }
        true
    }
    // Source: src/utils/SkPolyUtils.cpp:1555-1559
    unsafe fn add(&mut self, v: *mut TriangulationVertex) {
        let index = unsafe { self.hash(v) };
        unsafe { self.fGrid[index].addToTail(v) };
        self.fNumVerts += 1;
    }
    // Source: src/utils/SkPolyUtils.cpp:1561-1565
    unsafe fn remove(&mut self, v: *mut TriangulationVertex) {
        let index = unsafe { self.hash(v) };
        unsafe { self.fGrid[index].remove(v) };
        self.fNumVerts -= 1;
    }
    // Source: src/utils/SkPolyUtils.cpp:1567-1597
    unsafe fn checkTriangle(
        &self,
        p0: SkPoint,
        p1: SkPoint,
        p2: SkPoint,
        ignoreIndex0: u16,
        ignoreIndex1: u16,
    ) -> bool {
        unsafe {
            if self.fNumVerts == 0 {
                return false;
            }
            let mut bounds = SkRect::default();
            compute_triangle_bounds(p0, p1, p2, &mut bounds);
            let h0 = ((bounds.fLeft - self.fBounds.fLeft) * self.fGridConversion.fX) as i32;
            let h1 = ((bounds.fRight - self.fBounds.fLeft) * self.fGridConversion.fX) as i32;
            let v0 = ((bounds.fTop - self.fBounds.fTop) * self.fGridConversion.fY) as i32;
            let v1 = ((bounds.fBottom - self.fBounds.fTop) * self.fGridConversion.fY) as i32;
            for v in v0..=v1 {
                for h in h0..=h1 {
                    let i = (v * self.fHCount + h) as usize;
                    let mut iter = self.fGrid[i].begin();
                    while !iter.get().is_null() {
                        let r = &*iter.get();
                        if r.fIndex != ignoreIndex0
                            && r.fIndex != ignoreIndex1
                            && point_in_triangle(p0, p1, p2, r.fPosition)
                        {
                            return true;
                        }
                        iter.next();
                    }
                }
            }
            false
        }
    }
    // Source: src/utils/SkPolyUtils.cpp:1600-1605
    unsafe fn hash(&self, vert: *mut TriangulationVertex) -> usize {
        unsafe {
            let h = (((*vert).fPosition.fX - self.fBounds.fLeft) * self.fGridConversion.fX) as i32;
            let v = (((*vert).fPosition.fY - self.fBounds.fTop) * self.fGridConversion.fY) as i32;
            crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1603; @condition b"v*fHCount + h >= 0\0".as_ptr(); v*self.fHCount+h>=0);
            (v * self.fHCount + h) as usize
        }
    }
}
// Source: src/utils/SkPolyUtils.cpp:1617-1630
unsafe fn reclassify_vertex(
    p: *mut TriangulationVertex,
    polygon: &[SkPoint],
    winding: i32,
    hash: &mut ReflexHash,
    convex: &mut SkTInternalLList<TriangulationVertex>,
) {
    unsafe {
        if (*p).fVertexType == VertexType::kReflex {
            let v0 = (*p).fPosition - polygon[(*p).fPrevIndex as usize];
            let v1 = polygon[(*p).fNextIndex as usize] - (*p).fPosition;
            if winding as f32 * v0.cross(v1) > SK_ScalarNearlyZero * SK_ScalarNearlyZero {
                (*p).fVertexType = VertexType::kConvex;
                hash.remove(p);
                (*p).links = Links::default();
                convex.addToTail(p);
            }
        }
    }
}
// Source: src/utils/SkPolyUtils.h:111-112
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkPolyUtils.cpp:1632-1771
pub fn SkTriangulateSimplePolygon(
    polygon: &[SkPoint],
    indexMap: &mut [u16],
    triangleIndices: &mut Vec<u16>,
) -> bool {
    unsafe {
        let n = polygon.len();
        if n < 3 || n >= u16::MAX as usize {
            return false;
        }
        let mut bounds = SkRect::default();
        if !bounds.setBoundsCheck(polygon) {
            return false;
        }
        let winding = SkGetPolygonWinding(polygon);
        if winding == 0 {
            return false;
        }
        let mut storage = AutoBuffer::<TriangulationVertex>::new(n);
        let vertices = storage.as_mut_ptr();
        let mut prev = n - 1;
        let mut v0 = polygon[0] - polygon[prev];
        for curr in 0..n {
            let next = (curr + 1) % n;
            let v1 = polygon[next] - polygon[curr];
            let vertexType =
                if winding as f32 * v0.cross(v1) > SK_ScalarNearlyZero * SK_ScalarNearlyZero {
                    VertexType::kConvex
                } else {
                    VertexType::kReflex
                };
            vertices.add(curr).write(TriangulationVertex {
                links: Links::default(),
                fPosition: polygon[curr],
                fVertexType: vertexType,
                fIndex: curr as u16,
                fPrevIndex: prev as u16,
                fNextIndex: next as u16,
            });
            prev = curr;
            v0 = v1;
        }
        // Pin just the native stack list identity used by debug intrusive back-links.
        let mut convexStorage = core::pin::pin!(SkTInternalLList::<TriangulationVertex>::new());
        let convex = convexStorage.as_mut().get_unchecked_mut();
        let mut hash = ReflexHash::new();
        if !hash.init(bounds, n as i32) {
            return false;
        }
        prev = n - 1;
        for curr in 0..n {
            if (*vertices.add(curr)).fVertexType == VertexType::kConvex {
                let next = (curr + 1) % n;
                if (*vertices.add(prev)).fVertexType == VertexType::kReflex
                    || (*vertices.add(next)).fVertexType == VertexType::kReflex
                {
                    convex.addToHead(vertices.add(curr));
                } else {
                    convex.addToTail(vertices.add(curr));
                }
            } else {
                hash.add(vertices.add(curr));
            }
            prev = curr;
        }
        triangleIndices.reserve(3 * (n - 2));
        let mut vertexCount = n;
        while vertexCount > 3 {
            let mut success = false;
            let mut ear: *mut TriangulationVertex = null_mut();
            let mut p0: *mut TriangulationVertex = null_mut();
            let mut p2: *mut TriangulationVertex = null_mut();
            let mut iter = convex.begin();
            while !iter.get().is_null() {
                ear = iter.get();
                crate::SkASSERT!(@source b"src/utils/SkPolyUtils.cpp\0".as_ptr(), 1724; @condition b"TriangulationVertex::VertexType::kReflex != earVertex->fVertexType\0".as_ptr(); (*ear).fVertexType!=VertexType::kReflex);
                p0 = vertices.add((*ear).fPrevIndex as usize);
                p2 = vertices.add((*ear).fNextIndex as usize);
                let failed = hash.checkTriangle(
                    (*p0).fPosition,
                    (*ear).fPosition,
                    (*p2).fPosition,
                    (*p0).fIndex,
                    (*p2).fIndex,
                );
                if failed {
                    iter.next();
                    continue;
                }
                success = true;
                break;
            }
            if !success {
                return false;
            }
            triangleIndices.extend_from_slice(&[
                indexMap[(*p0).fIndex as usize],
                indexMap[(*ear).fIndex as usize],
                indexMap[(*p2).fIndex as usize],
            ]);
            convex.remove(ear);
            vertexCount -= 1;
            (*p0).fNextIndex = (*ear).fNextIndex;
            reclassify_vertex(p0, polygon, winding, &mut hash, convex);
            (*p2).fPrevIndex = (*ear).fPrevIndex;
            reclassify_vertex(p2, polygon, winding, &mut hash, convex);
        }
        let mut iter = convex.begin();
        while !iter.get().is_null() {
            let vertex = &*iter.get();
            triangleIndices.push(indexMap[vertex.fIndex as usize]);
            iter.next();
        }
        true
    }
}
