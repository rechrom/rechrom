// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkAddIntersections.cpp:311-595. Native segment pair admission.
use super::{
    SkIntersectionHelper::{SegmentType::*, SkIntersectionHelper},
    SkIntersections::SkIntersections,
    SkOpCoincidence::SkOpCoincidence,
    SkOpContour::SkOpContour,
    SkOpSpan::SkOpPtT,
    SkPathOpsBounds::SkPathOpsBounds,
    SkPathOpsConic::SkDConic,
    SkPathOpsCubic::SkDCubic,
    SkPathOpsLine::SkDLine,
    SkPathOpsQuad::SkDQuad,
    SkPathOpsTypes::AlmostLessUlps,
};
use core::ptr::null_mut;
unsafe fn line(h: &SkIntersectionHelper) -> SkDLine {
    let mut p = SkDLine::default();
    p.set(&*h.pts().cast());
    p
}
unsafe fn quad(h: &SkIntersectionHelper) -> SkDQuad {
    let mut p = SkDQuad::default();
    p.set(&*h.pts().cast());
    p
}
unsafe fn conic(h: &SkIntersectionHelper) -> SkDConic {
    let mut p = SkDConic::default();
    p.set(&*h.pts().cast(), h.weight());
    p
}
unsafe fn cubic(h: &SkIntersectionHelper) -> SkDCubic {
    let mut p = SkDCubic::default();
    p.set(&*h.pts().cast());
    p
}
// These dispatch directly to the inline SkIntersections source wrappers after
// their one-time SkPoint->double primitive conversion, with no geometry change.
unsafe fn horizontal(
    ts: &mut SkIntersections,
    p: &SkIntersectionHelper,
    h: &SkIntersectionHelper,
) -> usize {
    match p.segmentType() {
        kHorizontalLine_Segment | kVerticalLine_Segment | kLine_Segment => ts.horizontalLine(
            &line(p),
            h.left() as f64,
            h.right() as f64,
            h.y() as f64,
            h.xFlipped(),
        ),
        kQuad_Segment => ts.horizontalQuad(
            &quad(p),
            h.left() as f64,
            h.right() as f64,
            h.y() as f64,
            h.xFlipped(),
        ),
        kConic_Segment => ts.horizontalConic(
            &conic(p),
            h.left() as f64,
            h.right() as f64,
            h.y() as f64,
            h.xFlipped(),
        ),
        kCubic_Segment => ts.horizontalCubic(
            &cubic(p),
            h.left() as f64,
            h.right() as f64,
            h.y() as f64,
            h.xFlipped(),
        ),
    }
}
unsafe fn vertical(
    ts: &mut SkIntersections,
    p: &SkIntersectionHelper,
    h: &SkIntersectionHelper,
) -> usize {
    match p.segmentType() {
        kHorizontalLine_Segment | kVerticalLine_Segment | kLine_Segment => ts.verticalLine(
            &line(p),
            h.top() as f64,
            h.bottom() as f64,
            h.x() as f64,
            h.yFlipped(),
        ),
        kQuad_Segment => ts.verticalQuad(
            &quad(p),
            h.top() as f64,
            h.bottom() as f64,
            h.x() as f64,
            h.yFlipped(),
        ),
        kConic_Segment => ts.verticalConic(
            &conic(p),
            h.top() as f64,
            h.bottom() as f64,
            h.x() as f64,
            h.yFlipped(),
        ),
        kCubic_Segment => ts.verticalCubic(
            &cubic(p),
            h.top() as f64,
            h.bottom() as f64,
            h.x() as f64,
            h.yFlipped(),
        ),
    }
}
pub unsafe fn AddIntersectTs(
    test: *mut SkOpContour,
    next: *mut SkOpContour,
    coincidence: *mut SkOpCoincidence,
) -> bool {
    if test != next {
        if AlmostLessUlps((*test).bounds().fBottom, (*next).bounds().fTop) {
            return false;
        }
        if !SkPathOpsBounds::Intersects((*test).bounds(), (*next).bounds()) {
            return true;
        }
    }
    let mut wt = SkIntersectionHelper::default();
    wt.init(test);
    loop {
        let mut wn = SkIntersectionHelper::default();
        wn.init(next);
        if test != next || wn.startAfter(&wt) {
            loop {
                if SkPathOpsBounds::Intersects(wt.bounds(), wn.bounds()) {
                    let mut ts = SkIntersections::default();
                    let mut swap = false;
                    let pts = match wt.segmentType() {
                        kHorizontalLine_Segment => {
                            swap = true;
                            horizontal(&mut ts, &wn, &wt)
                        }
                        kVerticalLine_Segment => {
                            swap = true;
                            vertical(&mut ts, &wn, &wt)
                        }
                        kLine_Segment => match wn.segmentType() {
                            kHorizontalLine_Segment => horizontal(&mut ts, &wt, &wn),
                            kVerticalLine_Segment => vertical(&mut ts, &wt, &wn),
                            kLine_Segment => ts.intersectLineLine(&line(&wt), &line(&wn)),
                            kQuad_Segment => {
                                swap = true;
                                ts.intersectQuadLine(&quad(&wn), &line(&wt))
                            }
                            kConic_Segment => {
                                swap = true;
                                ts.intersectConicLine(&conic(&wn), &line(&wt))
                            }
                            kCubic_Segment => {
                                swap = true;
                                ts.intersectCubicLine(&cubic(&wn), &line(&wt))
                            }
                        },
                        kQuad_Segment => match wn.segmentType() {
                            kHorizontalLine_Segment => horizontal(&mut ts, &wt, &wn),
                            kVerticalLine_Segment => vertical(&mut ts, &wt, &wn),
                            kLine_Segment => ts.intersectQuadLine(&quad(&wt), &line(&wn)),
                            kQuad_Segment => ts.intersectQuadQuad(&quad(&wt), &quad(&wn)),
                            kConic_Segment => {
                                swap = true;
                                ts.intersectConicQuad(&conic(&wn), &quad(&wt))
                            }
                            kCubic_Segment => {
                                swap = true;
                                ts.intersectCubicQuad(&cubic(&wn), &quad(&wt))
                            }
                        },
                        kConic_Segment => match wn.segmentType() {
                            kHorizontalLine_Segment => horizontal(&mut ts, &wt, &wn),
                            kVerticalLine_Segment => vertical(&mut ts, &wt, &wn),
                            kLine_Segment => ts.intersectConicLine(&conic(&wt), &line(&wn)),
                            kQuad_Segment => ts.intersectConicQuad(&conic(&wt), &quad(&wn)),
                            kConic_Segment => ts.intersectConicConic(&conic(&wt), &conic(&wn)),
                            kCubic_Segment => {
                                swap = true;
                                ts.intersectCubicConic(&cubic(&wn), &conic(&wt))
                            }
                        },
                        kCubic_Segment => match wn.segmentType() {
                            kHorizontalLine_Segment => horizontal(&mut ts, &wt, &wn),
                            kVerticalLine_Segment => vertical(&mut ts, &wt, &wn),
                            kLine_Segment => ts.intersectCubicLine(&cubic(&wt), &line(&wn)),
                            kQuad_Segment => ts.intersectCubicQuad(&cubic(&wt), &quad(&wn)),
                            kConic_Segment => ts.intersectCubicConic(&cubic(&wt), &conic(&wn)),
                            kCubic_Segment => ts.intersectCubicCubic(&cubic(&wt), &cubic(&wn)),
                        },
                    };
                    let mut coinIndex = -1;
                    let mut coinPtT: [*mut SkOpPtT; 2] = [null_mut(); 2];
                    for pt in 0..pts {
                        let iPt = ts.pt(pt).asSkPoint();
                        let integral = iPt.fX == (iPt.fX as f64).floor() as f32
                            && iPt.fY == (iPt.fY as f64).floor() as f32;
                        let mut a = if integral {
                            (*wt.segment()).addTPoint(ts[swap as usize][pt], &iPt)
                        } else {
                            (*wt.segment()).addT(ts[swap as usize][pt])
                        };
                        let mut b = if integral {
                            (*wn.segment()).addTPoint(ts[(!swap) as usize][pt], &iPt)
                        } else {
                            (*wn.segment()).addT(ts[(!swap) as usize][pt])
                        };
                        if !(*a).contains(b) {
                            let prev = (*a).oppPrev(b);
                            if !prev.is_null() {
                                (*(*a).span()).mergeMatches((*b).span());
                                (*a).addOpp(b, prev);
                            }
                            if (*a).fPt != (*b).fPt {
                                (*(*a).span()).unaligned();
                                (*(*b).span()).unaligned();
                            }
                        }
                        if !ts.isCoincident(pt) {
                            continue;
                        }
                        if coinIndex < 0 {
                            coinPtT = [a, b];
                            coinIndex = pt as i32;
                            continue;
                        }
                        if (*coinPtT[0]).span() == (*a).span() {
                            coinIndex = -1;
                            continue;
                        }
                        if (*coinPtT[1]).span() == (*b).span() {
                            coinIndex = -1;
                            continue;
                        }
                        if swap {
                            coinPtT.swap(0, 1);
                            core::mem::swap(&mut a, &mut b);
                        }
                        if (*(*coinPtT[0]).span()).deleted() {
                            coinIndex = -1;
                            continue;
                        }
                        if (*(*a).span()).deleted() {
                            coinIndex = -1;
                            continue;
                        }
                        (*coincidence).add(coinPtT[0], a, coinPtT[1], b);
                        coinIndex = -1;
                    }
                }
                if !wn.advance() {
                    break;
                }
            }
        }
        if !wt.advance() {
            break;
        }
    }
    true
}
