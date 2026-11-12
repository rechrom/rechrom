// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkPathOpsCommon.cpp. Release geometry/lifecycle only.
#![allow(non_snake_case)]
pub use super::SkPathOpsWinding::FindSortableTop;
use super::{
    SkOpAngle::SkOpAngle,
    SkOpCoincidence::SkOpCoincidence,
    SkOpContour::{SkOpContour, SkOpContourHead},
    SkOpSegment::SkOpSegment,
    SkOpSpan::{SkOpSpan, SkOpSpanBase},
};
use core::ptr::null_mut;

pub unsafe fn AngleWinding(
    start: *mut SkOpSpanBase,
    end: *mut SkOpSpanBase,
    windingPtr: &mut i32,
    sortablePtr: &mut bool,
) -> *mut SkOpAngle {
    let mut segment = (*start).segment();
    let mut angle = (*segment).spanToAngle(start, end);
    if angle.is_null() {
        *windingPtr = i32::MIN;
        return null_mut();
    }
    let mut firstAngle = angle;
    let mut looped = false;
    let mut unorderable = false;
    let mut winding = i32::MIN;
    let computeWinding;
    loop {
        angle = (*angle).next();
        if angle.is_null() {
            return null_mut();
        }
        unorderable |= (*angle).unorderable();
        if unorderable || (angle == firstAngle && looped) {
            computeWinding = true;
            break;
        }
        looped |= angle == firstAngle;
        segment = (*angle).segment();
        winding = (*segment).windSumAngle(angle);
        if winding != i32::MIN {
            computeWinding = false;
            break;
        }
    }
    if computeWinding {
        firstAngle = angle;
        winding = i32::MIN;
        loop {
            let startSpan = (*angle).start();
            let endSpan = (*angle).end();
            let lesser = (*startSpan).starter(endSpan);
            let mut testWinding = (*lesser).windSum();
            if testWinding == i32::MIN {
                testWinding = (*lesser).computeWindSum();
            }
            if testWinding != i32::MIN {
                winding = testWinding;
            }
            angle = (*angle).next();
            if angle == firstAngle {
                break;
            }
        }
    }
    *sortablePtr = !unorderable;
    *windingPtr = winding;
    angle
}

pub unsafe fn FindUndone(contourHead: *mut SkOpContourHead) -> *mut SkOpSpan {
    let mut contour = contourHead.cast::<SkOpContour>();
    loop {
        if !(*contour).done() {
            let result = (*contour).undoneSpan();
            if !result.is_null() {
                return result;
            }
        }
        contour = (*contour).next();
        if contour.is_null() {
            return null_mut();
        }
    }
}

pub unsafe fn FindChase(
    chase: &mut Vec<*mut SkOpSpanBase>,
    startPtr: &mut *mut SkOpSpanBase,
    endPtr: &mut *mut SkOpSpanBase,
) -> *mut SkOpSegment {
    while let Some(span) = chase.pop() {
        let mut segment = (*span).segment();
        *startPtr = (*(*(*span).ptT()).next()).span();
        let mut done = true;
        *endPtr = null_mut();
        let last = (*segment).activeAngle(*startPtr, startPtr, endPtr, &mut done);
        if !last.is_null() {
            *startPtr = (*last).start();
            *endPtr = (*last).end();
            chase.push(span);
            return (*last).segment();
        }
        if done {
            continue;
        }
        let mut winding = 0;
        let mut sortable = false;
        let mut angle = AngleWinding(*startPtr, *endPtr, &mut winding, &mut sortable);
        if angle.is_null() {
            return null_mut();
        }
        if winding == i32::MIN {
            continue;
        }
        let mut sumWinding = 0;
        if sortable {
            segment = (*angle).segment();
            sumWinding = (*segment).updateWindingReverse(angle);
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
            if sortable {
                (*segment).setUpWinding(start, end, &mut maxWinding, &mut sumWinding);
            }
            if !(*segment).doneAngle(angle) {
                if first.is_null() && (sortable || (*(*start).starter(end)).windSum() != i32::MIN) {
                    first = segment;
                    *startPtr = start;
                    *endPtr = end;
                }
                if sortable {
                    let success = (*segment).markAngle(maxWinding, sumWinding, angle, null_mut());
                }
            }
        }
        if !first.is_null() {
            chase.push(span);
            return first;
        }
    }
    null_mut()
}

pub unsafe fn SortContourList(
    contourList: &mut *mut SkOpContourHead,
    evenOdd: bool,
    oppEvenOdd: bool,
) -> bool {
    let mut list = Vec::<*mut SkOpContour>::new();
    let mut contour = (*contourList).cast::<SkOpContour>();
    loop {
        if (*contour).count() != 0 {
            (*contour).setOppXor(if (*contour).operand() {
                evenOdd
            } else {
                oppEvenOdd
            });
            list.push(contour);
        }
        contour = (*contour).next();
        if contour.is_null() {
            break;
        }
    }
    if list.is_empty() {
        return false;
    }
    if list.len() > 1 {
        crate::src::core::SkTSort::SkTQSort(&mut list, &|a, b| (**a).lessThan(&**b));
    }
    contour = list[0];
    let head = contour.cast::<SkOpContourHead>();
    (*(*contour).globalState()).setContourHead(head);
    *contourList = head;
    for &next in &list[1..] {
        (*contour).setNext(next);
        contour = next;
    }
    (*contour).setNext(null_mut());
    true
}

unsafe fn calc_angles(contourList: *mut SkOpContourHead) {
    let mut contour = contourList.cast::<SkOpContour>();
    loop {
        (*contour).calcAngles();
        contour = (*contour).next();
        if contour.is_null() {
            break;
        }
    }
}
unsafe fn missing_coincidence(contourList: *mut SkOpContourHead) -> bool {
    let mut contour = contourList.cast::<SkOpContour>();
    let mut result = false;
    loop {
        result |= (*contour).missingCoincidence();
        contour = (*contour).next();
        if contour.is_null() {
            break;
        }
    }
    result
}
unsafe fn move_multiples(contourList: *mut SkOpContourHead) -> bool {
    let mut contour = contourList.cast::<SkOpContour>();
    loop {
        if !(*contour).moveMultiples() {
            return false;
        }
        contour = (*contour).next();
        if contour.is_null() {
            return true;
        }
    }
}
unsafe fn move_nearby(contourList: *mut SkOpContourHead) -> bool {
    let mut contour = contourList.cast::<SkOpContour>();
    loop {
        if !(*contour).moveNearby() {
            return false;
        }
        contour = (*contour).next();
        if contour.is_null() {
            return true;
        }
    }
}
unsafe fn sort_angles(contourList: *mut SkOpContourHead) -> bool {
    let mut contour = contourList.cast::<SkOpContour>();
    loop {
        if !(*contour).sortAngles() {
            return false;
        }
        contour = (*contour).next();
        if contour.is_null() {
            return true;
        }
    }
}

pub unsafe fn HandleCoincidence(
    contourList: *mut SkOpContourHead,
    coincidence: *mut SkOpCoincidence,
) -> bool {
    let globalState = (*contourList.cast::<SkOpContour>()).globalState();
    if !(*coincidence).addExpanded() {
        return false;
    }
    if !move_multiples(contourList) {
        return false;
    }
    if !move_nearby(contourList) {
        return false;
    }
    (*coincidence).correctEnds();
    if !(*coincidence).addEndMovedSpans() {
        return false;
    }
    const SAFETY_COUNT: i32 = 3;
    let mut safetyHatch = SAFETY_COUNT;
    loop {
        let mut added = false;
        if !(*coincidence).addMissing(&mut added) {
            return false;
        }
        if !added {
            break;
        }
        safetyHatch -= 1;
        if safetyHatch == 0 {
            return false;
        }
        move_nearby(contourList);
    }
    if (*coincidence).expand() {
        let mut added = false;
        if !(*coincidence).addMissing(&mut added) {
            return false;
        }
        if !(*coincidence).addExpanded() {
            return false;
        }
        if !move_multiples(contourList) {
            return false;
        }
        move_nearby(contourList);
    }
    if !(*coincidence).addExpanded() {
        return false;
    }
    (*coincidence).mark();
    if missing_coincidence(contourList) {
        (*coincidence).expand();
        if !(*coincidence).addExpanded() {
            return false;
        }
        if !(*coincidence).mark() {
            return false;
        }
    } else {
        (*coincidence).expand();
    }
    (*coincidence).expand();
    let mut overlaps = SkOpCoincidence::new(globalState);
    overlaps.bind();
    safetyHatch = SAFETY_COUNT;
    loop {
        let pairs = if overlaps.isEmpty() {
            coincidence
        } else {
            &mut overlaps
        };
        if !(*pairs).apply() {
            return false;
        }
        if !SkOpCoincidence::FindOverlaps(pairs, core::ptr::addr_of_mut!(overlaps)) {
            return false;
        }
        safetyHatch -= 1;
        if safetyHatch == 0 {
            return false;
        }
        if overlaps.isEmpty() {
            break;
        }
    }
    calc_angles(contourList);
    sort_angles(contourList)
}
