// Copyright 2013 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkOpContour.h and SkOpContour.cpp.
#![allow(non_snake_case)]
use super::{
    SkOpSegment::SkOpSegment,
    SkOpSpan::SkOpSpan,
    SkPathOpsBounds::SkPathOpsBounds,
    SkPathOpsTypes::{SkOpGlobalState, SkPathOpsVerbToPoints},
    SkPathWriter::SkPathWriter,
};
use crate::src::core::{SkPath::Verb, SkPoint::SkPoint};
use core::ptr::{copy_nonoverlapping, null_mut};

#[repr(C)]
pub struct SkOpContour {
    pub(super) fState: *mut SkOpGlobalState,
    pub(super) fHead: SkOpSegment,
    pub(super) fTail: *mut SkOpSegment,
    pub(super) fNext: *mut Self,
    pub(super) fBounds: SkPathOpsBounds,
    pub(super) fCcw: i32,
    pub(super) fCount: i32,
    pub(super) fFirstSorted: i32,
    pub(super) fDone: bool,
    pub(super) fOperand: bool,
    pub(super) fReverse: bool,
    pub(super) fXor: bool,
    pub(super) fOppXor: bool,
}
// The native derived head has no additional storage or identity. Its additional
// list operations are methods on this same owner in Rust.
pub type SkOpContourHead = SkOpContour;
impl Default for SkOpContour {
    fn default() -> Self {
        Self::new()
    }
}
impl SkOpContour {
    pub fn new() -> Self {
        Self {
            fState: null_mut(),
            fHead: SkOpSegment::default(),
            fTail: null_mut(),
            fNext: null_mut(),
            fBounds: SkPathOpsBounds::default(),
            fCcw: 0,
            fCount: 0,
            fFirstSorted: -1,
            fDone: false,
            fOperand: false,
            fReverse: false,
            fXor: false,
            fOppXor: false,
        }
    }
    pub fn lessThan(&self, other: &Self) -> bool {
        if self.fBounds.fTop == other.fBounds.fTop {
            self.fBounds.fLeft < other.fBounds.fLeft
        } else {
            self.fBounds.fTop < other.fBounds.fTop
        }
    }
    pub unsafe fn appendSegment(&mut self) -> *mut SkOpSegment {
        let result = if self.fCount != 0 {
            (*(*self.globalState()).allocator()).make(SkOpSegment::default())
        } else {
            core::ptr::addr_of_mut!(self.fHead)
        };
        self.fCount += 1;
        (*result).setPrev(self.fTail);
        if !self.fTail.is_null() {
            (*self.fTail).setNext(result);
        }
        self.fTail = result;
        result
    }
    pub unsafe fn addConic(&mut self, pts: *mut SkPoint, weight: f32) {
        let segment = self.appendSegment();
        (*segment).addConic(pts, weight, self);
    }
    pub unsafe fn addCubic(&mut self, pts: *mut SkPoint) {
        let segment = self.appendSegment();
        (*segment).addCubic(pts, self);
    }
    pub unsafe fn addLine(&mut self, pts: *mut SkPoint) -> *mut SkOpSegment {
        let segment = self.appendSegment();
        (*segment).addLine(pts, self)
    }
    pub unsafe fn addQuad(&mut self, pts: *mut SkPoint) {
        let segment = self.appendSegment();
        (*segment).addQuad(pts, self);
    }
    pub fn bounds(&self) -> &SkPathOpsBounds {
        &self.fBounds
    }
    pub unsafe fn calcAngles(&mut self) {
        let mut segment = self.first();
        loop {
            (*segment).calcAngles();
            segment = (*segment).next();
            if segment.is_null() {
                break;
            }
        }
    }
    pub unsafe fn complete(&mut self) {
        self.setBounds();
    }
    pub fn count(&self) -> i32 {
        self.fCount
    }
    pub fn globalState(&self) -> *mut SkOpGlobalState {
        self.fState
    }
    pub fn done(&self) -> bool {
        self.fDone
    }
    pub unsafe fn end(&self) -> &SkPoint {
        &*(*self.fTail)
            .pts()
            .add(SkPathOpsVerbToPoints((*self.fTail).verb()) as usize)
    }
    pub fn first(&self) -> *mut SkOpSegment {
        core::ptr::addr_of!(self.fHead).cast_mut()
    }
    pub fn init(&mut self, state: *mut SkOpGlobalState, operand: bool, isXor: bool) {
        self.fState = state;
        self.fOperand = operand;
        self.fXor = isXor;
    }
    pub fn isCcw(&self) -> i32 {
        self.fCcw
    }
    pub fn isXor(&self) -> bool {
        self.fXor
    }
    pub unsafe fn joinSegments(&mut self) {
        let head = self.first();
        let mut segment = head;
        loop {
            let next = (*segment).next();
            (*segment).joinEnds(if next.is_null() { head } else { next });
            segment = next;
            if segment.is_null() {
                break;
            }
        }
    }
    pub unsafe fn markAllDone(&mut self) {
        let mut segment = self.first();
        loop {
            (*segment).markAllDone();
            segment = (*segment).next();
            if segment.is_null() {
                break;
            }
        }
    }
    pub unsafe fn missingCoincidence(&mut self) -> bool {
        let mut segment = self.first();
        let mut result = false;
        loop {
            if (*segment).missingCoincidence() {
                result = true;
            }
            segment = (*segment).next();
            if segment.is_null() {
                break;
            }
        }
        result
    }
    pub unsafe fn moveMultiples(&mut self) -> bool {
        let mut segment = self.first();
        loop {
            if !(*segment).moveMultiples() {
                return false;
            }
            segment = (*segment).next();
            if segment.is_null() {
                return true;
            }
        }
    }
    pub unsafe fn moveNearby(&mut self) -> bool {
        let mut segment = self.first();
        loop {
            if !(*segment).moveNearby() {
                return false;
            }
            segment = (*segment).next();
            if segment.is_null() {
                return true;
            }
        }
    }
    pub fn next(&self) -> *mut Self {
        self.fNext
    }
    pub fn operand(&self) -> bool {
        self.fOperand
    }
    pub fn oppXor(&self) -> bool {
        self.fOppXor
    }
    pub fn reset(&mut self) {
        self.fTail = null_mut();
        self.fNext = null_mut();
        self.fCount = 0;
        self.fDone = false;
        self.fFirstSorted = -1;
    }
    pub unsafe fn resetReverse(&mut self) {
        let mut next = self as *mut Self;
        loop {
            if (*next).count() != 0 {
                (*next).fCcw = -1;
                (*next).fReverse = false;
            }
            next = (*next).next();
            if next.is_null() {
                break;
            }
        }
    }
    pub fn reversed(&self) -> bool {
        self.fReverse
    }
    pub unsafe fn setBounds(&mut self) {
        let mut segment = self.first();
        self.fBounds = *(*segment).bounds();
        loop {
            segment = (*segment).next();
            if segment.is_null() {
                break;
            }
            self.fBounds.addBounds((*segment).bounds());
        }
    }
    pub fn setCcw(&mut self, ccw: i32) {
        self.fCcw = ccw;
    }
    pub fn setGlobalState(&mut self, state: *mut SkOpGlobalState) {
        self.fState = state;
    }
    pub fn setNext(&mut self, contour: *mut Self) {
        self.fNext = contour;
    }
    pub fn setOperand(&mut self, operand: bool) {
        self.fOperand = operand;
    }
    pub fn setOppXor(&mut self, value: bool) {
        self.fOppXor = value;
    }
    pub fn setReverse(&mut self) {
        self.fReverse = true;
    }
    pub fn setXor(&mut self, value: bool) {
        self.fXor = value;
    }
    pub unsafe fn sortAngles(&mut self) -> bool {
        let mut segment = self.first();
        loop {
            if !(*segment).sortAngles() {
                return false;
            }
            segment = (*segment).next();
            if segment.is_null() {
                return true;
            }
        }
    }
    pub unsafe fn start(&self) -> &SkPoint {
        &*self.fHead.pts()
    }
    pub unsafe fn toPartialBackward(&self, path: *mut SkPathWriter) {
        let mut segment = self.fTail;
        loop {
            let ok = (*segment).addCurveTo((*segment).tail(), (*segment).head().cast(), path);

            segment = (*segment).prev();
            if segment.is_null() {
                break;
            }
        }
    }
    pub unsafe fn toPartialForward(&self, path: *mut SkPathWriter) {
        let mut segment = self.first();
        loop {
            let ok = (*segment).addCurveTo((*segment).head().cast(), (*segment).tail(), path);

            segment = (*segment).next();
            if segment.is_null() {
                break;
            }
        }
    }
    pub unsafe fn toPath(&self, path: *mut SkPathWriter) {
        if self.count() == 0 {
            return;
        }
        self.toPartialForward(path);
        (*path).finishContour();
        (*path).assemble();
    }
    pub unsafe fn toReversePath(&self, path: *mut SkPathWriter) {
        self.toPartialBackward(path);
        (*path).finishContour();
        (*path).assemble();
    }
    pub unsafe fn undoneSpan(&mut self) -> *mut SkOpSpan {
        let mut test = self.first();
        loop {
            if !(*test).done() {
                return (*test).undoneSpan();
            }
            test = (*test).next();
            if test.is_null() {
                break;
            }
        }
        self.fDone = true;
        null_mut()
    }
    pub unsafe fn appendContour(&mut self) -> *mut Self {
        let contour = (*(*self.globalState()).allocator()).make(Self::new());
        (*contour).setNext(null_mut());
        let mut prev = self as *mut Self;
        loop {
            let next = (*prev).next();
            if next.is_null() {
                break;
            }
            prev = next;
        }
        (*prev).setNext(contour);
        contour
    }
    pub unsafe fn joinAllSegments(&mut self) {
        let mut next = self as *mut Self;
        loop {
            if (*next).count() != 0 {
                (*next).joinSegments();
            }
            next = (*next).next();
            if next.is_null() {
                break;
            }
        }
    }
    pub unsafe fn remove(&mut self, contour: *mut Self) {
        if core::ptr::eq(contour, self) {
            return;
        }

        let mut prev = self as *mut Self;
        loop {
            let next = (*prev).next();
            if next == contour {
                break;
            }

            prev = next;
        }
        (*prev).setNext(null_mut());
    }
}

pub struct SkOpContourBuilder {
    pub(super) fContour: *mut SkOpContour,
    pub(super) fLastLine: [SkPoint; 2],
    pub(super) fLastIsLine: bool,
}
impl SkOpContourBuilder {
    pub fn new(contour: *mut SkOpContour) -> Self {
        Self {
            fContour: contour,
            fLastLine: [SkPoint::default(); 2],
            fLastIsLine: false,
        }
    }
    pub unsafe fn addConic(&mut self, pts: *mut SkPoint, weight: f32) {
        self.flush();
        (*self.fContour).addConic(pts, weight);
    }
    pub unsafe fn addCubic(&mut self, pts: *mut SkPoint) {
        self.flush();
        (*self.fContour).addCubic(pts);
    }
    pub unsafe fn addQuad(&mut self, pts: *mut SkPoint) {
        self.flush();
        (*self.fContour).addQuad(pts);
    }
    pub unsafe fn addCurve(&mut self, verb: Verb, pts: *const SkPoint, weight: f32) {
        if verb == Verb::kLine_Verb {
            self.addLine(pts);
            return;
        }
        let allocator = (*(*self.fContour).globalState()).allocator();
        match verb {
            Verb::kQuad_Verb => {
                let storage = (*allocator).makeArrayDefault::<SkPoint>(3);
                copy_nonoverlapping(pts, storage, 3);
                self.addQuad(storage);
            }
            Verb::kConic_Verb => {
                let storage = (*allocator).makeArrayDefault::<SkPoint>(3);
                copy_nonoverlapping(pts, storage, 3);
                self.addConic(storage, weight);
            }
            Verb::kCubic_Verb => {
                let storage = (*allocator).makeArrayDefault::<SkPoint>(4);
                copy_nonoverlapping(pts, storage, 4);
                self.addCubic(storage);
            }
            _ => (),
        }
    }
    pub unsafe fn addLine(&mut self, pts: *const SkPoint) {
        if self.fLastIsLine {
            if self.fLastLine[0] == *pts.add(1) && self.fLastLine[1] == *pts {
                self.fLastIsLine = false;
                return;
            } else {
                self.flush();
            }
        }
        copy_nonoverlapping(pts, self.fLastLine.as_mut_ptr(), 2);
        self.fLastIsLine = true;
    }
    pub unsafe fn flush(&mut self) {
        if !self.fLastIsLine {
            return;
        }
        let allocator = (*(*self.fContour).globalState()).allocator();
        let storage = (*allocator).makeArrayDefault::<SkPoint>(2);
        copy_nonoverlapping(self.fLastLine.as_ptr(), storage, 2);
        (*self.fContour).addLine(storage);
        self.fLastIsLine = false;
    }
    pub fn contour(&self) -> *mut SkOpContour {
        self.fContour
    }
    pub unsafe fn setContour(&mut self, contour: *mut SkOpContour) {
        self.flush();
        self.fContour = contour;
    }
}
