use crate::include::core::SkRefCnt::sk_sp;
use crate::src::core::SkContourMeasure::{SkContourMeasure, SkContourMeasureIter};
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkPoint::SkPoint;
// Source: include/core/SkPathMeasure.h:54-58
pub type MatrixFlags = u32;
pub const kGetPosition_MatrixFlag: MatrixFlags = 0x01;
pub const kGetTangent_MatrixFlag: MatrixFlags = 0x02;
pub const kGetPosAndTan_MatrixFlag: MatrixFlags = kGetPosition_MatrixFlag | kGetTangent_MatrixFlag;
// Source: include/core/SkPathMeasure.h:89-89
impl SkPathMeasure {
    pub fn currentMeasure(&self) -> Option<&SkContourMeasure> {
        self.fContour.get()
    }
}
// Source: include/core/SkPathMeasure.h:92-93
// Native members destruct in reverse declaration order: retained contour before its iterator.
pub struct SkPathMeasure {
    fContour: sk_sp<SkContourMeasure>,
    fIter: SkContourMeasureIter,
}
// Source: src/core/SkPathMeasure.cpp:22-22
impl Default for SkPathMeasure {
    fn default() -> Self {
        Self {
            fIter: SkContourMeasureIter::default(),
            fContour: sk_sp::default(),
        }
    }
}
impl SkPathMeasure {
    // Source: src/core/SkPathMeasure.cpp:24-28
    pub fn new(path: &SkPath, forceClosed: bool, resScale: f32) -> Self {
        let mut fIter = SkContourMeasureIter::new(path, forceClosed, resScale);
        let fContour = fIter.next();
        Self { fIter, fContour }
    }
    // Source: src/core/SkPathMeasure.cpp:32-35
    pub fn setPath(&mut self, path: Option<&SkPath>, forceClosed: bool) {
        match path {
            Some(path) => self.fIter.reset(path, forceClosed, 1.0),
            None => self.fIter.reset(&SkPath::default(), forceClosed, 1.0),
        }
        self.fContour = self.fIter.next();
    }
    // Source: src/core/SkPathMeasure.cpp:37-39
    pub fn getLength(&self) -> f32 {
        if let Some(contour) = self.fContour.get() {
            contour.length()
        } else {
            0.0
        }
    }
    // Source: src/core/SkPathMeasure.cpp:41-43
    pub fn getPosTan(
        &self,
        distance: f32,
        position: Option<&mut SkPoint>,
        tangent: Option<&mut SkPoint>,
    ) -> bool {
        self.fContour
            .get()
            .is_some_and(|c| c.getPosTan(distance, position, tangent))
    }
    // Source: src/core/SkPathMeasure.cpp:45-47
    pub fn getMatrix(
        &self,
        distance: f32,
        matrix: Option<&mut SkMatrix>,
        flags: MatrixFlags,
    ) -> bool {
        self.fContour
            .get()
            .is_some_and(|c| c.getMatrix(distance, matrix, flags))
    }
    // Source: src/core/SkPathMeasure.cpp:49-52
    pub fn getSegment(
        &self,
        startD: f32,
        stopD: f32,
        dst: &mut SkPathBuilder,
        startWithMoveTo: bool,
    ) -> bool {
        self.fContour
            .get()
            .is_some_and(|c| c.getSegment(startD, stopD, dst, startWithMoveTo))
    }
    // Source: src/core/SkPathMeasure.cpp:54-56
    pub fn isClosed(&self) -> bool {
        self.fContour.get().is_some_and(|c| c.isClosed())
    }
    // Source: src/core/SkPathMeasure.cpp:58-61
    pub fn nextContour(&mut self) -> bool {
        self.fContour = self.fIter.next();
        self.fContour.is_some()
    }
}
// Source: src/core/SkPathMeasure.cpp:69-74
impl crate::src::core::SkPathMeasurePriv::SkPathMeasurePriv {
    pub fn CountSegments(meas: &SkPathMeasure) -> usize {
        if let Some(cntr) = meas.currentMeasure() {
            return cntr.fSegments.size() as usize;
        }
        0
    }
}
