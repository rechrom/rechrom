// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsTypes.h:34-201, SkPathOpsTypes.cpp:197-227.
use super::{
    SkOpCoincidence::SkOpCoincidence, SkOpContour::SkOpContourHead, SkPathOpsTypes::SkOpPhase,
};
use crate::src::core::SkArenaAlloc::SkArenaAlloc;
use core::ptr::null_mut;

pub struct SkOpGlobalState {
    fAllocator: *mut SkArenaAlloc,
    fCoincidence: *mut SkOpCoincidence,
    fContourHead: *mut SkOpContourHead,
    fNested: i32,
    fAllocatedOpSpan: bool,
    fWindingFailed: bool,
    fPhase: SkOpPhase,
}
impl SkOpGlobalState {
    pub const kMaxWindingTries: i32 = 10;
    pub fn new(head: *mut SkOpContourHead, allocator: *mut SkArenaAlloc) -> Self {
        Self {
            fAllocator: allocator,
            fCoincidence: null_mut(),
            fContourHead: head,
            fNested: 0,
            fAllocatedOpSpan: false,
            fWindingFailed: false,
            fPhase: SkOpPhase::kIntersecting,
        }
    }
    pub fn allocatedOpSpan(&self) -> bool {
        self.fAllocatedOpSpan
    }
    pub fn allocator(&self) -> *mut SkArenaAlloc {
        self.fAllocator
    }
    pub fn bumpNested(&mut self) {
        self.fNested += 1;
    }
    pub fn clearNested(&mut self) {
        self.fNested = 0;
    }
    pub fn nested(&self) -> i32 {
        self.fNested
    }
    pub fn coincidence(&self) -> *mut SkOpCoincidence {
        self.fCoincidence
    }
    pub fn contourHead(&self) -> *mut SkOpContourHead {
        self.fContourHead
    }
    pub fn phase(&self) -> SkOpPhase {
        self.fPhase
    }
    pub fn resetAllocatedOpSpan(&mut self) {
        self.fAllocatedOpSpan = false;
    }
    pub fn setAllocatedOpSpan(&mut self) {
        self.fAllocatedOpSpan = true;
    }
    pub fn setCoincidence(&mut self, c: *mut SkOpCoincidence) {
        self.fCoincidence = c;
    }
    pub fn setContourHead(&mut self, c: *mut SkOpContourHead) {
        self.fContourHead = c;
    }
    pub fn setPhase(&mut self, p: SkOpPhase) {
        if p == SkOpPhase::kNoChange {
            return;
        }

        self.fPhase = p;
    }
    pub fn setWindingFailed(&mut self) {
        self.fWindingFailed = true;
    }
    pub fn windingFailed(&self) -> bool {
        self.fWindingFailed
    }
}
