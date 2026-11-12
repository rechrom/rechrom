// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathWriter.h/.cpp. Deferred contours and native assembly.
use super::{SkOpSpan::SkOpPtT, SkPathOpsTypes::zero_or_one};
use crate::{
    include::core::SkPathTypes::SkPathFillType,
    src::core::{
        SkPath::{AddPathMode, SkPath},
        SkPathBuilder::{Reserve, SkPathBuilder},
        SkPathEnums::SkResolveConvexity,
        SkPathPriv::SkPathPriv,
        SkPoint::SkPoint,
        SkTSort::SkTQSort,
    },
};
use core::{borrow::Borrow, ptr::null};

pub struct SkPathWriter {
    fBuilder: SkPathBuilder,
    fCurrent: SkPathBuilder,
    fPartials: Vec<SkPathBuilder>,
    fEndPtTs: Vec<*const SkOpPtT>,
    fDefer: [*const SkOpPtT; 2],
    fFirstPtT: *const SkOpPtT,
}
impl SkPathWriter {
    pub fn new(ft: SkPathFillType) -> Self {
        Self {
            fBuilder: SkPathBuilder::with_fill_type(ft),
            fCurrent: SkPathBuilder::new(),
            fPartials: Vec::new(),
            fEndPtTs: Vec::new(),
            fDefer: [null(); 2],
            fFirstPtT: null(),
        }
    }
    pub fn nativePath(&mut self) -> SkPath {
        self.fBuilder.detach(None)
    }
    pub fn hasMove(&self) -> bool {
        self.fFirstPtT.is_null()
    }
    pub fn init(&mut self) {
        self.fCurrent.reset();
        self.fFirstPtT = null();
        self.fDefer = [null(); 2];
    }
    pub unsafe fn isClosed(&self) -> bool {
        self.matchedLast(self.fFirstPtT)
    }
    unsafe fn close(&mut self) {
        if self.fCurrent.isEmpty() {
            return;
        }

        self.fCurrent.close();
        if let Some(raw) = SkPathPriv::RawBuilder(&self.fCurrent, SkResolveConvexity::kNo) {
            self.fBuilder.addRaw(&raw, Reserve::kExact);
        }
        self.init();
    }
    pub unsafe fn conicTo(&mut self, pt1: impl Borrow<SkPoint>, pt2: *const SkOpPtT, weight: f32) {
        let p = self.update(pt2);
        self.fCurrent.conicTo(*pt1.borrow(), p, weight);
    }
    pub unsafe fn cubicTo(
        &mut self,
        pt1: impl Borrow<SkPoint>,
        pt2: impl Borrow<SkPoint>,
        pt3: *const SkOpPtT,
    ) {
        let p = self.update(pt3);
        self.fCurrent.cubicTo(*pt1.borrow(), *pt2.borrow(), p);
    }
    pub unsafe fn quadTo(&mut self, pt1: impl Borrow<SkPoint>, pt2: *const SkOpPtT) {
        let p = self.update(pt2);
        self.fCurrent.quadTo(*pt1.borrow(), p);
    }
    pub unsafe fn deferredLine(&mut self, pt: *const SkOpPtT) -> bool {
        if self.fDefer[0] == pt {
            return true;
        }
        if (*pt).contains(self.fDefer[0]) {
            return true;
        }
        if self.matchedLast(pt) {
            return false;
        }
        if !self.fDefer[1].is_null() && self.changedSlopes(pt) {
            self.lineTo();
            self.fDefer[0] = self.fDefer[1];
        }
        self.fDefer[1] = pt;
        true
    }
    pub unsafe fn deferredMove(&mut self, pt: *const SkOpPtT) {
        if self.fDefer[1].is_null() {
            self.fFirstPtT = pt;
            self.fDefer[0] = pt;
            return;
        }

        if !self.matchedLast(pt) {
            self.finishContour();
            self.fFirstPtT = pt;
            self.fDefer[0] = pt;
        }
    }
    pub unsafe fn finishContour(&mut self) {
        if !self.matchedLast(self.fDefer[0]) {
            if self.fDefer[1].is_null() {
                return;
            }
            self.lineTo();
        }
        if self.fCurrent.isEmpty() {
            return;
        }
        if self.isClosed() {
            self.close();
        } else {
            self.fEndPtTs.push(self.fFirstPtT);
            self.fEndPtTs.push(self.fDefer[1]);
            self.fPartials.push(self.fCurrent.clone());
            self.init();
        }
    }
    unsafe fn lineTo(&mut self) {
        if self.fCurrent.isEmpty() {
            self.moveTo();
        }
        self.fCurrent.lineTo((*self.fDefer[1]).fPt);
    }
    unsafe fn moveTo(&mut self) {
        self.fCurrent.moveTo((*self.fFirstPtT).fPt);
    }
    unsafe fn matchedLast(&self, test: *const SkOpPtT) -> bool {
        if test == self.fDefer[1] {
            return true;
        }
        if test.is_null() || self.fDefer[1].is_null() {
            return false;
        }
        (*test).contains(self.fDefer[1])
    }
    unsafe fn update(&mut self, pt: *const SkOpPtT) -> SkPoint {
        if self.fDefer[1].is_null() {
            self.moveTo();
        } else if !self.matchedLast(self.fDefer[0]) {
            self.lineTo();
        }
        let mut result = (*pt).fPt;
        if !self.fFirstPtT.is_null()
            && result != (*self.fFirstPtT).fPt
            && (*self.fFirstPtT).contains(pt)
        {
            result = (*self.fFirstPtT).fPt;
        }
        self.fDefer = [pt; 2];
        result
    }
    unsafe fn changedSlopes(&self, pt: *const SkOpPtT) -> bool {
        if self.matchedLast(self.fDefer[0]) {
            return false;
        }
        let defer = (*self.fDefer[1]).fPt - (*self.fDefer[0]).fPt;
        let line = (*pt).fPt - (*self.fDefer[1]).fPt;
        defer.fX * line.fY != defer.fY * line.fX
    }
    unsafe fn someAssemblyRequired(&mut self) -> bool {
        self.finishContour();
        !self.fEndPtTs.is_empty()
    }
    pub unsafe fn assemble(&mut self) {
        if !self.someAssemblyRequired() {
            return;
        }
        let endCount = self.fEndPtTs.len();

        const MAX_PARTIAL_CONTOURS: usize = 10000;
        if endCount > MAX_PARTIAL_CONTOURS * 2 {
            return;
        }
        for pIndex in 0..endCount {
            let mut opPtT = self.fEndPtTs[pIndex] as *mut SkOpPtT;
            let mut partWriter = Self::new(SkPathFillType::kDefault);
            loop {
                if !zero_or_one((*opPtT).fT) {
                    break;
                }
                let base = (*opPtT).span();
                let mut start = if (*opPtT).fT != 0.0 {
                    (*base).prev().cast()
                } else {
                    (*(*base).upCast()).next()
                };
                let mut step = if (*opPtT).fT != 0.0 { 1 } else { -1 };
                let segment = (*base).segment();
                let next = (*segment).isSimple(&mut start, &mut step);
                if next.is_null() {
                    break;
                }
                let end = if (*start).t() != 0.0 {
                    (*start).prev().cast()
                } else {
                    (*(*start).upCast()).next()
                };
                if (*(*start).starter(end)).alreadyAdded() {
                    break;
                }
                (*next).addCurveTo(start, end, &mut partWriter);
                opPtT = (*end).ptT();
                self.fEndPtTs[pIndex] = opPtT;
            }
            partWriter.finishContour();
            if partWriter.fPartials.is_empty() {
                continue;
            }
            let partial = &mut self.fPartials[pIndex >> 1];
            let part = partWriter.fPartials[0].snapshot(None);
            if pIndex & 1 != 0 {
                partial.addPathUntransformed(&part, AddPathMode::kExtend_AddPathMode);
            } else {
                let mut reverse = SkPathBuilder::new();
                SkPathPriv::ReverseAddPath(&mut reverse, &part);
                reverse
                    .addPathUntransformed(&partial.detach(None), AddPathMode::kExtend_AddPathMode);
                *partial = reverse;
            }
        }
        let linkCount = endCount / 2;
        let mut sLink = vec![i32::MAX; linkCount];
        let mut eLink = vec![i32::MAX; linkCount];
        let entries = endCount * (endCount - 1) / 2;
        let mut distances = Vec::with_capacity(entries);
        let mut sortedDist = Vec::with_capacity(entries);
        let mut distLookup = Vec::with_capacity(entries);
        let mut rRow = 0;
        for r in 0..endCount - 1 {
            let o = self.fEndPtTs[r];
            for i in r + 1..endCount {
                let p = self.fEndPtTs[i];
                // Native subtracts SkScalar before assignment to double.
                let dx = ((*p).fPt.fX - (*o).fPt.fX) as f64;
                let dy = ((*p).fPt.fY - (*o).fPt.fY) as f64;
                distLookup.push(rRow + i);
                distances.push(dx * dx + dy * dy);
                sortedDist.push(sortedDist.len());
            }
            rRow += endCount;
        }

        SkTQSort(&mut sortedDist, &|a, b| distances[*a] < distances[*b]);
        let mut remaining = linkCount;
        for r in 0..entries {
            let pair = distLookup[sortedDist[r]];
            let row = pair / endCount;
            let col = pair - row * endCount;
            let one = row >> 1;
            let endOne = row & 1 != 0;
            if (if endOne { eLink[one] } else { sLink[one] }) != i32::MAX {
                continue;
            }
            let two = col >> 1;
            let endTwo = col & 1 != 0;
            if (if endTwo { eLink[two] } else { sLink[two] }) != i32::MAX {
                continue;
            }

            let flip = endOne == endTwo;
            let valOne = if flip { !(two as i32) } else { two as i32 };
            let valTwo = if flip { !(one as i32) } else { one as i32 };
            if endOne {
                eLink[one] = valOne;
            } else {
                sLink[one] = valOne;
            }
            if endTwo {
                eLink[two] = valTwo;
            } else {
                sLink[two] = valTwo;
            }
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }

        let mut r = 0;
        loop {
            let mut forward = true;
            let mut first = true;
            let sIndex = sLink[r];

            sLink[r] = i32::MAX;
            let mut eIndex = if sIndex < 0 {
                let i = (!sIndex) as usize;
                let e = sLink[i];
                sLink[i] = i32::MAX;
                e
            } else {
                let i = sIndex as usize;
                let e = eLink[i];
                eLink[i] = i32::MAX;
                e
            };

            loop {
                let contour = self.fPartials[r].snapshot(None);
                if !first {
                    let Some(_prior) = self.fBuilder.getLastPt() else {
                        return;
                    };
                    if !forward && contour.points().is_empty() {
                        return;
                    }
                }
                if forward {
                    self.fBuilder.addPathUntransformed(
                        &contour,
                        if first {
                            AddPathMode::kAppend_AddPathMode
                        } else {
                            AddPathMode::kExtend_AddPathMode
                        },
                    );
                } else {
                    SkPathPriv::ReversePathTo(&mut self.fBuilder, &contour);
                }
                first = false;
                if sIndex
                    == if ((r as i32) != eIndex) ^ forward {
                        eIndex
                    } else {
                        !eIndex
                    }
                {
                    self.fBuilder.close();
                    break;
                }
                if forward {
                    eIndex = eLink[r];

                    eLink[r] = i32::MAX;
                    if eIndex >= 0 {
                        sLink[eIndex as usize] = i32::MAX;
                    } else {
                        eLink[(!eIndex) as usize] = i32::MAX;
                    }
                } else {
                    eIndex = sLink[r];

                    sLink[r] = i32::MAX;
                    if eIndex >= 0 {
                        eLink[eIndex as usize] = i32::MAX;
                    } else {
                        sLink[(!eIndex) as usize] = i32::MAX;
                    }
                }
                if eIndex < 0 {
                    forward = !forward;
                    r = (!eIndex) as usize;
                } else {
                    r = eIndex as usize;
                }
            }
            r = 0;
            while r < linkCount && sLink[r] == i32::MAX {
                r += 1;
            }
            if r == linkCount {
                break;
            }
        }
    }
}
