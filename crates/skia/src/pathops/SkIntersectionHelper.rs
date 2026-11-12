// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkIntersectionHelper.h. Borrowed native segment cursor.
use super::{SkOpContour::SkOpContour, SkOpSegment::SkOpSegment, SkPathOpsBounds::SkPathOpsBounds};
use crate::src::core::{SkPath::Verb, SkPoint::SkPoint};
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    kHorizontalLine_Segment = -1,
    kVerticalLine_Segment = 0,
    kLine_Segment = 1,
    kQuad_Segment = 2,
    kConic_Segment = 3,
    kCubic_Segment = 4,
}
pub use SegmentType::*;
pub struct SkIntersectionHelper {
    fSegment: *mut SkOpSegment,
}
impl Default for SkIntersectionHelper {
    fn default() -> Self {
        Self {
            fSegment: core::ptr::null_mut(),
        }
    }
}
impl SkIntersectionHelper {
    pub unsafe fn advance(&mut self) -> bool {
        self.fSegment = (*self.fSegment).next();
        !self.fSegment.is_null()
    }
    pub unsafe fn init(&mut self, c: *mut SkOpContour) {
        self.fSegment = (*c).first();
    }
    pub unsafe fn startAfter(&mut self, a: &Self) -> bool {
        self.fSegment = (*a.fSegment).next();
        !self.fSegment.is_null()
    }
    pub unsafe fn bounds(&self) -> &SkPathOpsBounds {
        (*self.fSegment).bounds()
    }
    pub unsafe fn contour(&self) -> *mut SkOpContour {
        (*self.fSegment).contour()
    }
    pub fn segment(&self) -> *mut SkOpSegment {
        self.fSegment
    }
    pub unsafe fn pts(&self) -> *const SkPoint {
        (*self.fSegment).pts()
    }
    pub unsafe fn segmentType(&self) -> SegmentType {
        match (*self.fSegment).verb() {
            Verb::kLine_Verb => {
                if (*self.fSegment).isHorizontal() {
                    kHorizontalLine_Segment
                } else if (*self.fSegment).isVertical() {
                    kVerticalLine_Segment
                } else {
                    kLine_Segment
                }
            }
            Verb::kQuad_Verb => kQuad_Segment,
            Verb::kConic_Verb => kConic_Segment,
            Verb::kCubic_Verb => kCubic_Segment,
            _ => unreachable!("source segment verb"),
        }
    }
    pub unsafe fn bottom(&self) -> f32 {
        self.bounds().fBottom
    }
    pub unsafe fn top(&self) -> f32 {
        self.bounds().fTop
    }
    pub unsafe fn left(&self) -> f32 {
        self.bounds().fLeft
    }
    pub unsafe fn right(&self) -> f32 {
        self.bounds().fRight
    }
    pub unsafe fn weight(&self) -> f32 {
        (*self.fSegment).weight()
    }
    pub unsafe fn x(&self) -> f32 {
        self.bounds().fLeft
    }
    pub unsafe fn y(&self) -> f32 {
        self.bounds().fTop
    }
    pub unsafe fn xFlipped(&self) -> bool {
        self.x() != (*self.pts()).fX
    }
    pub unsafe fn yFlipped(&self) -> bool {
        self.y() != (*self.pts()).fY
    }
}
