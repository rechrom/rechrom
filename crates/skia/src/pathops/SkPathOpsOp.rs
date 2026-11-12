// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkPathOpsOp.cpp. Native release PathOps walk.
use super::{
    SkAddIntersections::AddIntersectTs,
    SkOpCoincidence::SkOpCoincidence,
    SkOpContour::{SkOpContour, SkOpContourHead},
    SkOpEdgeBuilder::SkOpEdgeBuilder,
    SkOpGlobalState::SkOpGlobalState,
    SkOpSegment::SkOpSegment,
    SkOpSpan::SkOpSpanBase,
    SkPathOpsCommon::{AngleWinding, FindSortableTop, HandleCoincidence, SortContourList},
    SkPathOpsTypes::kEvenOdd_PathOpsMask,
    SkPathWriter::SkPathWriter,
};
use crate::{
    include::{
        core::SkPathTypes::SkPathFillType,
        pathops::SkPathOps::{Simplify, SkPathOp, SkPathOp::*},
    },
    src::core::{
        SkArenaAlloc::SkSTArenaAlloc,
        SkPath::{SkPath, Verb},
        SkRect::SkRect,
    },
};
use core::ptr::null_mut;

unsafe fn findChaseOp(
    chase: &mut Vec<*mut SkOpSpanBase>,
    startPtr: &mut *mut SkOpSpanBase,
    endPtr: &mut *mut SkOpSpanBase,
    result: &mut *mut SkOpSegment,
) -> bool {
    while let Some(span) = chase.pop() {
        *startPtr = (*(*(*span).ptT()).prev()).span();
        let mut segment = (**startPtr).segment();
        let mut done = true;
        *endPtr = null_mut();
        let last = (*segment).activeAngle(*startPtr, startPtr, endPtr, &mut done);
        if !last.is_null() {
            *startPtr = (*last).start();
            *endPtr = (*last).end();
            chase.push(span);
            *result = (*last).segment();
            return true;
        }
        if done {
            continue;
        }
        let mut winding = 0;
        let mut sortable = false;
        let mut angle = AngleWinding(*startPtr, *endPtr, &mut winding, &mut sortable);
        if angle.is_null() {
            *result = null_mut();
            return true;
        }
        if winding == i32::MIN {
            continue;
        }
        let mut sumMiWinding = 0;
        let mut sumSuWinding = 0;
        if sortable {
            segment = (*angle).segment();
            sumMiWinding = (*segment).updateWindingReverse(angle);
            if sumMiWinding == i32::MIN {
                *result = null_mut();
                return true;
            }
            sumSuWinding = (*segment).updateOppWindingReverse(angle);
            if sumSuWinding == i32::MIN {
                *result = null_mut();
                return true;
            }
            if (*segment).operand() {
                core::mem::swap(&mut sumMiWinding, &mut sumSuWinding);
            }
        }
        let mut first: *mut SkOpSegment = null_mut();
        let firstAngle = angle;
        loop {
            angle = (*angle).next();
            if angle == firstAngle {
                break;
            }
            segment = (*angle).segment();
            let start = (*angle).start();
            let end = (*angle).end();
            let mut maxWinding = 0;
            let mut sumWinding = 0;
            let mut oppMaxWinding = 0;
            let mut oppSumWinding = 0;
            if sortable {
                (*segment).setUpWindings(
                    start,
                    end,
                    &mut sumMiWinding,
                    &mut sumSuWinding,
                    &mut maxWinding,
                    &mut sumWinding,
                    &mut oppMaxWinding,
                    &mut oppSumWinding,
                );
            }
            if !(*segment).doneAngle(angle) {
                if first.is_null() && (sortable || (*(*start).starter(end)).windSum() != i32::MIN) {
                    first = segment;
                    *startPtr = start;
                    *endPtr = end;
                }
                if sortable
                    && !(*segment).markAngleOp(
                        maxWinding,
                        sumWinding,
                        oppMaxWinding,
                        oppSumWinding,
                        angle,
                        null_mut(),
                    )
                {
                    return false;
                }
            }
        }
        if !first.is_null() {
            chase.push(span);
            *result = first;
            return true;
        }
    }
    *result = null_mut();
    true
}

unsafe fn bridgeOp(
    contourList: *mut SkOpContourHead,
    op: SkPathOp,
    xorMask: i32,
    xorOpMask: i32,
    writer: &mut SkPathWriter,
) -> bool {
    let mut unsortable = false;
    let mut simple = false;
    loop {
        let span = FindSortableTop(contourList);
        if span.is_null() {
            break;
        }
        let mut current = (*span).segment();
        let mut start = (*span).next();
        let mut end = span.cast::<SkOpSpanBase>();
        let mut chase = Vec::new();
        loop {
            if (*current).activeOp(start, end, xorMask, xorOpMask, op) {
                loop {
                    if !unsortable && (*current).done() {
                        break;
                    }

                    let mut nextStart = start;
                    let mut nextEnd = end;
                    let lastSimple = simple;
                    let next = (*current).findNextOp(
                        &mut chase,
                        &mut nextStart,
                        &mut nextEnd,
                        &mut unsortable,
                        &mut simple,
                        op,
                        xorMask,
                        xorOpMask,
                    );
                    if next.is_null() {
                        if !unsortable
                            && writer.hasMove()
                            && (*current).verb() != Verb::kLine_Verb
                            && !writer.isClosed()
                        {
                            if !(*current).addCurveTo(start, end, writer) {
                                return false;
                            }
                        } else if lastSimple {
                            if !(*current).addCurveTo(start, end, writer) {
                                return false;
                            }
                        }
                        break;
                    }
                    if !(*current).addCurveTo(start, end, writer) {
                        return false;
                    }
                    current = next;
                    start = nextStart;
                    end = nextEnd;
                    if writer.isClosed() || (unsortable && (*(*start).starter(end)).done()) {
                        break;
                    }
                }
                if (*current).activeWinding(start, end) && !writer.isClosed() {
                    let spanStart = (*start).starter(end);
                    if !(*spanStart).done() {
                        if !(*current).addCurveTo(start, end, writer) {
                            return false;
                        }
                        (*current).markDone(spanStart);
                    }
                }
                writer.finishContour();
            } else {
                let mut last = null_mut();
                if !(*current).markAndChaseDone(start, end, &mut last) {
                    return false;
                }
                if !last.is_null() && !(*last).chased() {
                    (*last).setChased(true);
                    chase.push(last);
                }
            }
            if !findChaseOp(&mut chase, &mut start, &mut end, &mut current) {
                return false;
            }
            if current.is_null() {
                break;
            }
        }
    }
    true
}

// Source: SkPathOpsOp.cpp:220-237. Preserve the operation remapping and
// inverse output lookup order; inverse path semantics are not bounds culling.
const gOpInverse: [[[SkPathOp; 2]; 2]; 5] = [
    [
        [kDifference_SkPathOp, kIntersect_SkPathOp],
        [kUnion_SkPathOp, kReverseDifference_SkPathOp],
    ],
    [
        [kIntersect_SkPathOp, kDifference_SkPathOp],
        [kReverseDifference_SkPathOp, kUnion_SkPathOp],
    ],
    [
        [kUnion_SkPathOp, kReverseDifference_SkPathOp],
        [kDifference_SkPathOp, kIntersect_SkPathOp],
    ],
    [
        [kXOR_SkPathOp, kXOR_SkPathOp],
        [kXOR_SkPathOp, kXOR_SkPathOp],
    ],
    [
        [kReverseDifference_SkPathOp, kUnion_SkPathOp],
        [kIntersect_SkPathOp, kDifference_SkPathOp],
    ],
];
const gOutInverse: [[[bool; 2]; 2]; 5] = [
    [[false, false], [true, false]],
    [[false, false], [false, true]],
    [[false, true], [true, true]],
    [[false, true], [true, false]],
    [[false, true], [false, false]],
];

pub fn OpDebug(one: &SkPath, two: &SkPath, op: SkPathOp) -> Option<SkPath> {
    let a = one.isInverseFillType() as usize;
    let b = two.isInverseFillType() as usize;
    let mut op = gOpInverse[op as usize][a][b];
    let inverseFill = gOutInverse[op as usize][a][b];
    let fillType = if inverseFill {
        SkPathFillType::kInverseEvenOdd
    } else {
        SkPathFillType::kEvenOdd
    };
    let mut rect1 = SkRect::default();
    let mut rect2 = SkRect::default();
    if op == kIntersect_SkPathOp
        && one.isRect(Some(&mut rect1), None, None)
        && two.isRect(Some(&mut rect2), None, None)
    {
        let result = if rect1.intersect(&rect2) {
            SkPath::RectDefault(
                &rect1,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                0,
            )
        } else {
            SkPath::default()
        };
        return Some(result.makeFillType(fillType));
    }
    if one.isEmpty() || two.isEmpty() {
        let mut work = match op {
            kIntersect_SkPathOp => SkPath::default(),
            kUnion_SkPathOp | kXOR_SkPathOp => {
                if one.isEmpty() {
                    two.clone()
                } else {
                    one.clone()
                }
            }
            kDifference_SkPathOp => {
                if !one.isEmpty() {
                    one.clone()
                } else {
                    SkPath::default()
                }
            }
            kReverseDifference_SkPathOp => {
                if !two.isEmpty() {
                    two.clone()
                } else {
                    SkPath::default()
                }
            }
        };
        if inverseFill != work.isInverseFillType() {
            work.toggleInverseFillType();
        }
        return Simplify(&work);
    }
    let mut allocator = core::pin::pin!(SkSTArenaAlloc::<4096>::new(0));
    let mut contour = SkOpContour::new();
    let mut contourList = (&mut contour as *mut SkOpContour).cast::<SkOpContourHead>();
    // All raw graph pointers borrow this invocation's pinned arena and local
    // owners. The writer freezes its own immutable path before those drop.
    unsafe {
        let mut globalState = SkOpGlobalState::new(contourList, allocator.as_mut().arena());
        let mut coincidence = SkOpCoincidence::new(&mut globalState);
        coincidence.bind();
        let (minuend, subtrahend) = if op == kReverseDifference_SkPathOp {
            op = kDifference_SkPathOp;
            (two, one)
        } else {
            (one, two)
        };
        let mut builder = SkOpEdgeBuilder::new(minuend, contourList, &mut globalState);
        if builder.unparseable() {
            return None;
        }
        let xorMask = builder.xorMask() as i32;
        builder.addOperand(subtrahend);
        if !builder.finish() {
            return None;
        }
        let xorOpMask = builder.xorMask() as i32;
        if !SortContourList(
            &mut contourList,
            xorMask == kEvenOdd_PathOpsMask as i32,
            xorOpMask == kEvenOdd_PathOpsMask as i32,
        ) {
            return Some(SkPath::default().makeFillType(fillType));
        }
        let mut current = contourList.cast::<SkOpContour>();
        loop {
            let mut next = current;
            loop {
                if !AddIntersectTs(current, next, &mut coincidence) {
                    break;
                }
                next = (*next).next();
                if next.is_null() {
                    break;
                }
            }
            current = (*current).next();
            if current.is_null() {
                break;
            }
        }
        if !HandleCoincidence(contourList, &mut coincidence) {
            return None;
        }
        let mut wrapper = SkPathWriter::new(fillType);
        if !bridgeOp(contourList, op, xorMask, xorOpMask, &mut wrapper) {
            return None;
        }
        wrapper.assemble();
        Some(wrapper.nativePath())
    }
}
