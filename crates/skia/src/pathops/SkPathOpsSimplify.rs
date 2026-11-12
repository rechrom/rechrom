// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsSimplify.cpp. Native winding and XOR walks.
use super::{
    SkAddIntersections::AddIntersectTs,
    SkOpCoincidence::SkOpCoincidence,
    SkOpContour::{SkOpContour, SkOpContourHead},
    SkOpEdgeBuilder::SkOpEdgeBuilder,
    SkOpGlobalState::SkOpGlobalState,
    SkOpSpan::SkOpSpanBase,
    SkPathOpsCommon::{FindChase, FindSortableTop, FindUndone, HandleCoincidence, SortContourList},
    SkPathOpsTypes::kWinding_PathOpsMask,
    SkPathWriter::SkPathWriter,
};
use crate::{
    include::core::SkPathTypes::{SkPathFillType, SkPathVerb},
    src::core::{
        SkArenaAlloc::SkSTArenaAlloc,
        SkPath::{Iter, SkPath},
        SkPoint::SkPoint,
    },
};
use core::ptr::null_mut;
unsafe fn bridgeWinding(list: *mut SkOpContourHead, writer: &mut SkPathWriter) -> bool {
    let mut unsortable = false;
    loop {
        let span = FindSortableTop(list);
        if span.is_null() {
            break;
        }
        let mut current = (*span).segment();
        let mut start = (*span).next();
        let mut end = span.cast::<SkOpSpanBase>();
        let mut chase = Vec::new();
        loop {
            if (*current).activeWinding(start, end) {
                loop {
                    if !unsortable && (*current).done() {
                        break;
                    }

                    let mut nextStart = start;
                    let mut nextEnd = end;
                    let next = (*current).findNextWinding(
                        &mut chase,
                        &mut nextStart,
                        &mut nextEnd,
                        &mut unsortable,
                    );
                    if next.is_null() {
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
            current = FindChase(&mut chase, &mut start, &mut end);
            if current.is_null() {
                break;
            }
        }
    }
    true
}
unsafe fn bridgeXor(list: *mut SkOpContourHead, writer: &mut SkPathWriter) -> bool {
    let mut unsortable = false;
    let mut safetyNet = 1_000_000;
    loop {
        let span = FindUndone(list);
        if span.is_null() {
            break;
        }
        let mut current = (*span).segment();
        let mut start = (*span).next();
        let mut end = span.cast::<SkOpSpanBase>();
        loop {
            safetyNet -= 1;
            if safetyNet < 0 {
                return false;
            }
            if !unsortable && (*current).done() {
                break;
            }

            let mut nextStart = start;
            let mut nextEnd = end;
            let next = (*current).findNextXor(&mut nextStart, &mut nextEnd, &mut unsortable);
            if next.is_null() {
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
        if !writer.isClosed() && !(*(*start).starter(end)).done() {
            return false;
        }
        writer.finishContour();
    }
    true
}
fn path_is_trivial(path: &SkPath) -> bool {
    let mut iter = Iter::new(path, true);
    let mut prevPt = SkPoint::default();
    let mut prevVec = SkPoint::default();
    fn add(curr: SkPoint, prev: &mut SkPoint, vec: &mut SkPoint) -> bool {
        if curr == *prev {
            return true;
        }
        let currVec = curr - *prev;
        if SkPoint::CrossProduct(*vec, currVec) != 0.0 {
            return false;
        }
        *vec = currVec;
        *prev = curr;
        true
    }
    while let Some(rec) = iter.next() {
        let p = rec.fPoints;
        match rec.fVerb {
            SkPathVerb::kMove => {
                prevPt = p[0];
                prevVec = SkPoint::default();
            }
            SkPathVerb::kCubic => {
                if !add(p[3], &mut prevPt, &mut prevVec)
                    || !add(p[2], &mut prevPt, &mut prevVec)
                    || !add(p[1], &mut prevPt, &mut prevVec)
                    || !add(p[0], &mut prevPt, &mut prevVec)
                {
                    return false;
                }
            }
            SkPathVerb::kQuad | SkPathVerb::kConic => {
                if !add(p[2], &mut prevPt, &mut prevVec)
                    || !add(p[1], &mut prevPt, &mut prevVec)
                    || !add(p[0], &mut prevPt, &mut prevVec)
                {
                    return false;
                }
            }
            SkPathVerb::kLine => {
                if !add(p[1], &mut prevPt, &mut prevVec) || !add(p[0], &mut prevPt, &mut prevVec) {
                    return false;
                }
            }
            SkPathVerb::kClose => {}
        }
    }
    true
}
pub fn SimplifyDebug(path: &SkPath) -> Option<SkPath> {
    let fill = if path.isInverseFillType() {
        SkPathFillType::kInverseEvenOdd
    } else {
        SkPathFillType::kEvenOdd
    };
    if path.isConvex() {
        return Some(
            if path_is_trivial(path) {
                SkPath::default()
            } else {
                path.clone()
            }
            .makeFillType(fill),
        );
    }
    let mut allocator = core::pin::pin!(SkSTArenaAlloc::<4096>::new(0));
    let mut contour = SkOpContour::new();
    let mut list = (&mut contour as *mut SkOpContour).cast::<SkOpContourHead>();
    unsafe {
        let mut state = SkOpGlobalState::new(list, allocator.as_mut().arena());
        let mut coin = SkOpCoincidence::new(&mut state);
        coin.bind();
        let mut builder = SkOpEdgeBuilder::new(path, list, &mut state);
        if !builder.finish() {
            return None;
        }
        if !SortContourList(&mut list, false, false) {
            return Some(SkPath::default().makeFillType(fill));
        }
        let mut current = list.cast::<SkOpContour>();
        loop {
            let mut next = current;
            loop {
                if !AddIntersectTs(current, next, &mut coin) {
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
        if !HandleCoincidence(list, &mut coin) {
            return None;
        }
        let mut writer = SkPathWriter::new(fill);
        if builder.xorMask() == kWinding_PathOpsMask {
            if !bridgeWinding(list, &mut writer) {
                return None;
            }
        } else if !bridgeXor(list, &mut writer) {
            return None;
        }
        writer.assemble();
        Some(writer.nativePath())
    }
}
