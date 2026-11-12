// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: include/pathops/SkPathOps.h:20-69.
use crate::src::core::SkPath::SkPath;
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPathOp {
    kDifference_SkPathOp,
    kIntersect_SkPathOp,
    kUnion_SkPathOp,
    kXOR_SkPathOp,
    kReverseDifference_SkPathOp,
}
pub use SkPathOp::*;
pub fn Op(one: &SkPath, two: &SkPath, op: SkPathOp) -> Option<SkPath> {
    crate::src::pathops::SkPathOpsOp::OpDebug(one, two, op)
}
pub fn Simplify(path: &SkPath) -> Option<SkPath> {
    crate::src::pathops::SkPathOpsSimplify::SimplifyDebug(path)
}
