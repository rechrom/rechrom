// Pin adaptation: concrete blitters inherit !Unpin from their actual embedded
// SkBlitterStorage. Scan entrypoints receive the original pinned destination;
// local polymorphic owners are pinned before dispatch and never moved afterward.
// get_unchecked_mut only projects existing fields; it never extracts/replaces
// pinned storage or transfers an installed arena/inline allocation.
// Drawing safety: unsafe render entrypoints require the SkBlitter primitive
// contract: live exclusive destination/context/arena leases, valid bounds and
// initialized terminating sparse runs. Clip wrappers preserve this caller lease;
// clipping alone does not establish ownership or backing-storage lifetime.
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::include::core::SkPathTypes::{SkPathFillType, SkPathFillType_IsEvenOdd};
use crate::include::private::SkFixed::SkFixedRoundToInt;
use crate::src::core::SkBlitter::SkBlitter;
use crate::src::core::SkEdge::{SkCubicEdge, SkEdge, SkQuadraticEdge, Type};
use crate::src::core::SkScanPriv::{
    backward_insert_edge_based_on_x, backward_insert_start, insert_edge_after, remove_edge,
};
// Rust replacement for the original virtual nextSegment dispatch. Repr(C)
// derived edges retain their base as first field; their initial fEdgeType is
// immutable after construction. Callers must keep that concrete allocation live.
unsafe fn next_segment(edge: *mut SkEdge) -> bool {
    match (*edge).fEdgeType {
        Type::kLine => (*edge).nextSegment(),
        Type::kQuad => (*(edge as *mut SkQuadraticEdge)).nextSegment(),
        Type::kCubic => (*(edge as *mut SkCubicEdge)).nextSegment(),
    }
}
// Source: src/core/SkScan_Path.cpp:40-41
pub const kEDGE_HEAD_Y: i32 = i32::MIN;
pub const kEDGE_TAIL_Y: i32 = i32::MAX;
// Source: src/core/SkScan_Path.cpp:44-54
unsafe fn validate_sort(mut edge: *mut SkEdge) {
    #[cfg(sk_debug)]
    {
        let mut y = kEDGE_HEAD_Y;
        while (*edge).fFirstY != i32::MAX {
            (*edge).validate();
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 49; @condition b"y <= edge->fFirstY\0".as_ptr(); y <= (*edge).fFirstY);
            y = (*edge).fFirstY;
            edge = (*edge).fNext;
        }
    }
}
// Source: src/core/SkScan_Path.cpp:59-88
/// Safety: all nodes and sentinels in the intrusive edge list must be live.
pub unsafe fn insert_new_edges(mut new_edge: *mut SkEdge, curr_y: i32) {
    if (*new_edge).fFirstY != curr_y {
        return;
    }
    let prev = (*new_edge).fPrev;
    if (*prev).fX <= (*new_edge).fX {
        return;
    }
    let mut start = backward_insert_start(prev, (*new_edge).fX);
    loop {
        let next = (*new_edge).fNext;
        let mut already_next = false;
        loop {
            if (*start).fNext == new_edge {
                already_next = true;
                break;
            }
            let after = (*start).fNext;
            if (*after).fX >= (*new_edge).fX {
                break;
            }
            start = after;
        }
        if !already_next {
            remove_edge(new_edge);
            insert_edge_after(new_edge, start);
        }
        start = new_edge;
        new_edge = next;
        if (*new_edge).fFirstY != curr_y {
            break;
        }
    }
}
// Source: src/core/SkScan_Path.cpp:91-101
unsafe fn validate_edges_for_y(mut edge: *mut SkEdge, curr_y: i32) {
    #[cfg(sk_debug)]
    {
        while (*edge).fFirstY <= curr_y {
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 93; @condition b"edge->fPrev && edge->fNext\0".as_ptr(); !(*edge).fPrev.is_null() && !(*edge).fNext.is_null());
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 94; @condition b"edge->fPrev->fNext == edge\0".as_ptr(); ((*(*edge).fPrev).fNext) == ( edge));
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 95; @condition b"edge->fNext->fPrev == edge\0".as_ptr(); ((*(*edge).fNext).fPrev) == ( edge));
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 96; @condition b"edge->fFirstY <= edge->fLastY\0".as_ptr(); (*edge).fFirstY <= (*edge).fLastY);
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 98; @condition b"edge->fPrev->fX <= edge->fX\0".as_ptr(); (*(*edge).fPrev).fX <= (*edge).fX);
            edge = (*edge).fNext;
        }
    }
}
// Source: src/core/SkScan_Path.cpp:111-113
pub type PrePostProc = unsafe fn(core::pin::Pin<&mut dyn SkBlitter>, i32, bool);
pub const PREPOST_START: bool = true;
pub const PREPOST_END: bool = false;
// Source: src/core/SkScan_Path.cpp:115-200
/// Safety: a sorted, correctly sentinel-terminated list must own every concrete
/// edge until this traversal returns. The tag must match each edge allocation.
pub unsafe fn walk_edges(
    prev_head: *mut SkEdge,
    fill_type: SkPathFillType,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    start_y: i32,
    stop_y: i32,
    proc: Option<PrePostProc>,
    right_clip: i32,
) {
    validate_sort((*prev_head).fNext);
    let mut curr_y = start_y;
    let winding_mask = if SkPathFillType_IsEvenOdd(fill_type) {
        1
    } else {
        -1
    };
    loop {
        let mut w = 0;
        let mut left = 0;
        let mut curr_e = (*prev_head).fNext;
        let mut prev_x = (*prev_head).fX;
        validate_edges_for_y(curr_e, curr_y);
        if let Some(proc) = proc {
            proc(blitter.as_mut(), curr_y, PREPOST_START);
        }
        while (*curr_e).fFirstY <= curr_y {
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 136; @condition b"currE->fLastY >= curr_y\0".as_ptr(); (*curr_e).fLastY >= curr_y);
            let x = SkFixedRoundToInt((*curr_e).fX);
            if w & winding_mask == 0 {
                left = x;
            }
            w += (*curr_e).fWinding as i8 as i32;
            if w & winding_mask == 0 {
                let width = x - left;
                crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 148; @condition b"width >= 0\0".as_ptr(); width >= 0);
                if width > 0 {
                    blitter.as_mut().blitH(left, curr_y, width);
                }
            }
            let next = (*curr_e).fNext;
            let active;
            let new_x;
            if (*curr_e).fLastY == curr_y {
                if (*curr_e).hasNextSegment() && next_segment(curr_e) {
                    crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 160; @condition b"currE->fFirstY == curr_y + 1\0".as_ptr(); ((*curr_e).fFirstY) == (curr_y + 1));
                    new_x = (*curr_e).fX;
                    active = true;
                } else {
                    remove_edge(curr_e);
                    new_x = 0;
                    active = false;
                }
            } else {
                crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 168; @condition b"currE->fLastY > curr_y\0".as_ptr(); (*curr_e).fLastY > curr_y);
                new_x = (*curr_e).fX.wrapping_add((*curr_e).fDxDy);
                (*curr_e).fX = new_x;
                active = true;
            }
            if active {
                if new_x < prev_x {
                    backward_insert_edge_based_on_x(curr_e);
                } else {
                    prev_x = new_x;
                }
            }
            curr_e = next;
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 179; @condition b"currE\0".as_ptr(); !curr_e.is_null());
        }
        if w & winding_mask != 0 {
            let width = right_clip - left;
            if width > 0 {
                blitter.as_mut().blitH(left, curr_y, width);
            }
        }
        if let Some(proc) = proc {
            proc(blitter.as_mut(), curr_y, PREPOST_END);
        }
        curr_y += 1;
        if curr_y >= stop_y {
            break;
        }
        insert_new_edges(curr_e, curr_y);
    }
}
// Source: src/core/SkScan_Path.cpp:203-216
/// Safety: edge must retain its correctly tagged concrete allocation.
pub unsafe fn update_edge(edge: *mut SkEdge, last_y: i32) -> bool {
    crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 204; @condition b"edge->fLastY >= last_y\0".as_ptr(); (*edge).fLastY >= last_y);
    if last_y != (*edge).fLastY {
        return true;
    }
    if !(*edge).hasNextSegment() {
        return false;
    }
    if next_segment(edge) {
        crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 212; @condition b"edge->fFirstY == last_y + 1\0".as_ptr(); ((*edge).fFirstY) == (last_y + 1));
        return true;
    }
    false
}
// Source: src/core/SkScan_Path.cpp:219-225
macro_rules! assert_return {
    ($line:expr; $native_condition:expr; $cond:expr) => {
        if !$cond {
            crate::SkDEBUGFAILF!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), $line;
                "assert(%s)", $native_condition);
            return;
        }
    };
}
// Source: src/core/SkScan_Path.cpp:228-306
/// Safety: the correctly tagged concrete edge allocations must stay live;
/// the path must satisfy the upstream single-Y-turn constraint.
pub unsafe fn walk_simple_edges(
    prev_head: *mut SkEdge,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    start_y: i32,
    stop_y: i32,
) {
    validate_sort((*prev_head).fNext);
    let mut left_e = (*prev_head).fNext;
    let mut right_e = (*left_e).fNext;
    let mut curr_e = (*right_e).fNext;
    let mut local_top = (*left_e).fFirstY.max((*right_e).fFirstY);
    assert_return!(238; b"local_top >= start_y\0".as_ptr(); local_top >= start_y);
    while local_top < stop_y {
        {
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 241; @condition b"leftE->fFirstY <= stop_y\0".as_ptr(); (*left_e).fFirstY<=stop_y);
            crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 242; @condition b"riteE->fFirstY <= stop_y\0".as_ptr(); (*right_e).fFirstY<=stop_y);
        };
        let mut local_bot = (*left_e).fLastY.min((*right_e).fLastY);
        local_bot = local_bot.min(stop_y - 1);
        assert_return!(246; b"local_top <= local_bot\0".as_ptr(); local_top <= local_bot);
        let mut left = (*left_e).fX;
        let d_left = (*left_e).fDxDy;
        let mut right = (*right_e).fX;
        let d_right = (*right_e).fDxDy;
        let mut count = local_bot - local_top;
        assert_return!(253; b"count >= 0\0".as_ptr(); count >= 0);
        if d_left == 0 && d_right == 0 {
            let mut l = SkFixedRoundToInt(left);
            let mut r = SkFixedRoundToInt(right);
            if l > r {
                std::mem::swap(&mut l, &mut r);
            }
            if l < r {
                count += 1;
                blitter.as_mut().blitRect(l, local_top, r - l, count);
            }
            local_top = local_bot + 1;
        } else {
            loop {
                let mut l = SkFixedRoundToInt(left);
                let mut r = SkFixedRoundToInt(right);
                if l > r {
                    std::mem::swap(&mut l, &mut r);
                }
                if l < r {
                    blitter.as_mut().blitH(l, local_top, r - l);
                }
                left = left.wrapping_add(d_left);
                right = right.wrapping_add(d_right);
                local_top += 1;
                count -= 1;
                if count < 0 {
                    break;
                }
            }
        }
        (*left_e).fX = left;
        (*right_e).fX = right;
        if !update_edge(left_e, local_bot) {
            if (*curr_e).fFirstY >= stop_y {
                return;
            }
            left_e = curr_e;
            curr_e = (*curr_e).fNext;
            assert_return!(295; b"leftE->fFirstY == local_top\0".as_ptr(); (*left_e).fFirstY == local_top);
        }
        if !update_edge(right_e, local_bot) {
            if (*curr_e).fFirstY >= stop_y {
                return;
            }
            right_e = curr_e;
            curr_e = (*curr_e).fNext;
            assert_return!(303; b"riteE->fFirstY == local_top\0".as_ptr(); (*right_e).fFirstY == local_top);
        }
    }
}

// Source: src/core/SkScan_Path.cpp:315-359
struct InverseBlitter<'a> {
    fBlitter: core::pin::Pin<&'a mut dyn SkBlitter>,
    fFirstX: i32,
    fLastX: i32,
    fPrevX: i32,
    storage: crate::src::core::SkBlitter::SkBlitterStorage,
}
impl<'a> InverseBlitter<'a> {
    fn new(
        mut blitter: core::pin::Pin<&'a mut dyn SkBlitter>,
        clip: &crate::src::core::SkRect::SkIRect,
    ) -> Self {
        Self {
            fBlitter: blitter,
            fFirstX: clip.fLeft,
            fLastX: clip.fRight,
            fPrevX: 0,
            storage: crate::src::core::SkBlitter::SkBlitterStorage::new(),
        }
    }
    unsafe fn prepost(&mut self, y: i32, isStart: bool) {
        if isStart {
            self.fPrevX = self.fFirstX;
        } else {
            let invWidth = self.fLastX - self.fPrevX;
            if invWidth > 0 {
                self.fBlitter.as_mut().blitH(self.fPrevX, y, invWidth);
            }
        }
    }
}
unsafe impl SkBlitter for InverseBlitter<'_> {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.storage.allocBlitMemory(sz)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        let invWidth = x - this.fPrevX;
        if invWidth > 0 {
            this.fBlitter.as_mut().blitH(this.fPrevX, y, invWidth);
        }
        this.fPrevX = x + width;
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        _: i32,
        _: i32,
        _: &mut [u8],
        _: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkDEBUGFAIL!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 344; b"blitAntiH unexpected\0".as_ptr());
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: i32, _: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkDEBUGFAIL!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 347; b"blitV unexpected\0".as_ptr());
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: i32, _: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkDEBUGFAIL!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 350; b"blitRect unexpected\0".as_ptr());
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        _: &crate::src::core::SkMask::SkMask,
        _: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkDEBUGFAIL!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 353; b"blitMask unexpected\0".as_ptr());
    }
}
// Source: src/core/SkScan_Path.cpp:361-363
// Private callback: sk_fill_path only installs this with the matching concrete
// InverseBlitter, retaining the native downcast invariant at that single callsite.
unsafe fn PrePostInverseBlitterProc(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    y: i32,
    isStart: bool,
) {
    unsafe {
        (*(blitter.get_unchecked_mut() as *mut dyn SkBlitter as *mut InverseBlitter<'_>))
            .prepost(y, isStart);
    }
}

// Source: src/core/SkScan_Path.cpp:371-377
unsafe fn compare_edges(a: *const SkEdge, b: *const SkEdge) -> bool {
    if (*a).fFirstY != (*b).fFirstY {
        return (*a).fFirstY < (*b).fFirstY;
    }
    (*a).fX < (*b).fX
}
// Source: src/core/SkScan_Path.cpp:379-390
unsafe fn sort_edges(list: &mut [*mut SkEdge], last: &mut *mut SkEdge) -> *mut SkEdge {
    crate::src::core::SkTSort::SkTQSort(list, &|a, b| compare_edges(*a, *b));
    for i in 1..list.len() {
        (*list[i - 1]).fNext = list[i];
        (*list[i]).fPrev = list[i - 1];
    }
    *last = list[list.len() - 1];
    list[0]
}
// Source: src/core/SkScan_Path.cpp:392-464
pub unsafe fn sk_fill_path<const EVEN: bool>(
    raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
    clipRect: &crate::src::core::SkRect::SkIRect,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    mut start_y: i32,
    mut stop_y: i32,
    pathContainedInClip: bool,
) {
    let mut builder = crate::src::core::SkEdgeBuilder::SkBasicEdgeBuilder::new();
    let count = builder.buildEdges::<EVEN>(
        raw,
        if pathContainedInClip {
            None
        } else {
            Some(clipRect)
        },
    );
    if count == 0 {
        if raw.isInverseFillType() {
            let mut rect = *clipRect;
            if rect.fTop < start_y {
                rect.fTop = start_y;
            }
            if rect.fBottom > stop_y {
                rect.fBottom = stop_y;
            }
            if !rect.is_empty() {
                blitter
                    .as_mut()
                    .blitRect(rect.fLeft, rect.fTop, rect.width(), rect.height());
            }
        }
        return;
    }
    let mut headEdge = SkEdge::new();
    let mut tailEdge = SkEdge::new();
    let mut last = core::ptr::null_mut();
    // Builder owns every concrete edge through traversal; no list allocation or
    // sentinel moves occur after linking their raw pointers.
    unsafe {
        let edge = sort_edges(builder.edgeList(), &mut last);
        headEdge.fPrev = core::ptr::null_mut();
        headEdge.fNext = edge;
        headEdge.fFirstY = kEDGE_HEAD_Y;
        headEdge.fX = i32::MIN;
        (*edge).fPrev = &mut headEdge;
        tailEdge.fPrev = last;
        tailEdge.fNext = core::ptr::null_mut();
        tailEdge.fFirstY = kEDGE_TAIL_Y;
        (*last).fNext = &mut tailEdge;
        if !pathContainedInClip && start_y < clipRect.fTop {
            start_y = clipRect.fTop;
        }
        if !pathContainedInClip && stop_y > clipRect.fBottom {
            stop_y = clipRect.fBottom;
        }
        if raw.isInverseFillType() {
            let mut ib = Box::pin(InverseBlitter::new(blitter.as_mut(), clipRect));
            walk_edges(
                &mut headEdge,
                raw.fillType(),
                ib.as_mut(),
                start_y,
                stop_y,
                Some(PrePostInverseBlitterProc),
                clipRect.fRight,
            );
        } else if raw.isKnownToBeConvex() && count >= 2 {
            walk_simple_edges(&mut headEdge, blitter.as_mut(), start_y, stop_y);
        } else {
            walk_edges(
                &mut headEdge,
                raw.fillType(),
                blitter.as_mut(),
                start_y,
                stop_y,
                None,
                clipRect.fRight,
            );
        }
    }
}
// Source: src/core/SkScan_Path.cpp:466-477
pub unsafe fn sk_blit_above(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ir: &crate::src::core::SkRect::SkIRect,
    clip: &crate::src::core::SkRegion::SkRegion,
) {
    let cr = clip.get_bounds();
    let tmp = crate::src::core::SkRect::SkIRect {
        fLeft: cr.fLeft,
        fRight: cr.fRight,
        fTop: cr.fTop,
        fBottom: ir.fTop,
    };
    if !tmp.is_empty() {
        blitter.as_mut().blitRectRegion(&tmp, clip);
    }
}
// Source: src/core/SkScan_Path.cpp:479-490
pub unsafe fn sk_blit_below(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ir: &crate::src::core::SkRect::SkIRect,
    clip: &crate::src::core::SkRegion::SkRegion,
) {
    let cr = clip.get_bounds();
    let tmp = crate::src::core::SkRect::SkIRect {
        fLeft: cr.fLeft,
        fRight: cr.fRight,
        fTop: ir.fBottom,
        fBottom: cr.fBottom,
    };
    if !tmp.is_empty() {
        blitter.as_mut().blitRectRegion(&tmp, clip);
    }
}
impl<'a> crate::src::core::SkScanPriv::SkScanClipper<'a> {
    // Source: src/core/SkScan_Path.cpp:499-536
    pub fn new(
        mut blitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
        clip: Option<&'a crate::src::core::SkRegion::SkRegion>,
        ir: &crate::src::core::SkRect::SkIRect,
        skipRejectTest: bool,
        irPreClipped: bool,
    ) -> Self {
        use crate::src::core::SkBlitter::{SkRectClipBlitter, SkRgnClipBlitter};
        use crate::src::core::SkScanPriv::ScanBlitter;
        let mut clip_rect = None;
        if let Some(clip) = clip {
            let rect = *clip.get_bounds();
            clip_rect = Some(rect);
            if !skipRejectTest
                && !(rect.fLeft < ir.fRight
                    && ir.fLeft < rect.fRight
                    && rect.fTop < ir.fBottom
                    && ir.fTop < rect.fBottom)
            {
                return crate::src::core::SkScanPriv::SkScanClipper {
                    blitter: ScanBlitter::None,
                    clip_rect,
                };
            }
            if clip.is_rect() {
                if !irPreClipped && rect.contains_rect(ir) {
                    clip_rect = None;
                    #[cfg(sk_debug)]
                    return Self {
                        blitter: ScanBlitter::Check(
                            crate::src::core::SkBlitter::SkRectClipCheckBlitter::new(blitter, rect),
                        ),
                        clip_rect,
                    };
                } else if irPreClipped || rect.fLeft > ir.fLeft || rect.fRight < ir.fRight {
                    return crate::src::core::SkScanPriv::SkScanClipper {
                        blitter: ScanBlitter::Rect(SkRectClipBlitter::new(blitter, rect)),
                        clip_rect,
                    };
                } else {
                    #[cfg(sk_debug)]
                    return Self {
                        blitter: ScanBlitter::Check(
                            crate::src::core::SkBlitter::SkRectClipCheckBlitter::new(blitter, rect),
                        ),
                        clip_rect,
                    };
                }
            } else {
                return crate::src::core::SkScanPriv::SkScanClipper {
                    blitter: ScanBlitter::Region(SkRgnClipBlitter::new(blitter, clip)),
                    clip_rect,
                };
            }
        }
        crate::src::core::SkScanPriv::SkScanClipper {
            blitter: ScanBlitter::Direct(blitter),
            clip_rect,
        }
    }
}
// Source: src/core/SkScan_Path.cpp:540-552
fn clip_to_limit(
    orig: &crate::src::core::SkRegion::SkRegion,
    reduced: &mut crate::src::core::SkRegion::SkRegion,
) -> bool {
    let limit = 32767 >> 1;
    let limitR = crate::src::core::SkRect::SkIRect {
        fLeft: -limit,
        fTop: -limit,
        fRight: limit,
        fBottom: limit,
    };
    if limitR.contains_rect(orig.get_bounds()) {
        return false;
    }
    reduced.op_region_rect(orig, limitR, crate::src::core::SkRegion::Op::Intersect);
    true
}
// Source: src/core/SkScan_Path.cpp:562-562
const kConservativeRoundBias: f64 = 0.5 + 1.5 / crate::src::core::SkFDot6::SK_FDot6One as f64;
// Source: src/core/SkScan_Path.cpp:570-574
fn round_down_to_int(x: f32) -> i32 {
    let mut xx = x as f64;
    xx -= kConservativeRoundBias;
    crate::src::core::SkFloatingPoint::sk_double_saturate2int(xx.ceil())
}
// Source: src/core/SkScan_Path.cpp:581-585
fn round_up_to_int(x: f32) -> i32 {
    let mut xx = x as f64;
    xx += kConservativeRoundBias;
    crate::src::core::SkFloatingPoint::sk_double_saturate2int(xx.floor())
}
// Source: src/core/SkScan_Path.cpp:595-602
fn conservative_round_to_int(
    src: &crate::src::core::SkRect::SkRect,
) -> crate::src::core::SkRect::SkIRect {
    crate::src::core::SkRect::SkIRect {
        fLeft: round_down_to_int(src.fLeft),
        fTop: round_down_to_int(src.fTop),
        fRight: round_up_to_int(src.fRight),
        fBottom: round_up_to_int(src.fBottom),
    }
}

impl crate::src::core::SkScan::SkScan {
    /// Portable build configuration without SK_RASTERIZE_EVEN_ROUNDING.
    pub unsafe fn FillPath(
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        origClip: &crate::src::core::SkRegion::SkRegion,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        Self::FillPathWithRounding::<false>(raw, origClip, blitter.as_mut());
    }
    // Source: src/core/SkScan_Path.cpp:604-658
    pub unsafe fn FillPathWithRounding<const EVEN: bool>(
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        origClip: &crate::src::core::SkRegion::SkRegion,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        use crate::src::core::SkRectPriv::SkRectPriv;
        use crate::src::core::SkRegion::SkRegion;
        use crate::src::core::SkScanPriv::SkScanClipper;
        if origClip.is_empty() {
            return;
        }
        let mut finiteClip = SkRegion::new();
        let clipPtr = if clip_to_limit(origClip, &mut finiteClip) {
            if finiteClip.is_empty() {
                return;
            }
            &finiteClip
        } else {
            origClip
        };
        let mut bounds = raw.bounds();
        let mut irPreClipped = false;
        if !SkRectPriv::MakeLargeS32().contains(&bounds) {
            if !bounds.intersect(&SkRectPriv::MakeLargeS32()) {
                bounds.setEmpty();
            }
            irPreClipped = true;
        }
        let ir = conservative_round_to_int(&bounds);
        if ir.is_empty() {
            if raw.isInverseFillType() {
                blitter.as_mut().blitRegion(clipPtr);
            }
            return;
        }
        let mut clipper = Box::pin(SkScanClipper::new(
            blitter.as_mut(),
            Some(clipPtr),
            &ir,
            raw.isInverseFillType(),
            irPreClipped,
        ));
        let pathContainedInClip = clipper.getClipRect().is_none();
        crate::SkASSERT!(@source b"src/core/SkScan_Path.cpp\0".as_ptr(), 648; @condition b"clipper.getClipRect() == nullptr || *clipper.getClipRect() == clipPtr->getBounds()\0".as_ptr(); clipper.getClipRect().is_none() || clipper.getClipRect() == Some(clipPtr.get_bounds()));
        if let Some(mut blitter) = clipper.as_mut().getBlitter() {
            if raw.isInverseFillType() {
                sk_blit_above(blitter.as_mut(), &ir, clipPtr);
            }
            sk_fill_path::<EVEN>(
                raw,
                clipPtr.get_bounds(),
                blitter.as_mut(),
                ir.fTop,
                ir.fBottom,
                pathContainedInClip,
            );
            if raw.isInverseFillType() {
                sk_blit_below(blitter.as_mut(), &ir, clipPtr);
            }
        }
    }
    // Source: src/core/SkScan_Path.cpp:660-663
    pub fn PathRequiresTiling(bounds: &crate::src::core::SkRect::SkIRect) -> bool {
        let mut out = crate::src::core::SkRegion::SkRegion::new();
        clip_to_limit(
            &crate::src::core::SkRegion::SkRegion::from_rect(*bounds),
            &mut out,
        )
    }
}
// Source: src/core/SkScan_Path.cpp:667-683
fn build_tri_edges<const EVEN: bool>(
    edge: &mut [SkEdge; 3],
    pts: &[crate::src::core::SkPoint::SkPoint; 3],
    clip: Option<&crate::src::core::SkRect::SkIRect>,
    list: &mut [*mut SkEdge; 3],
) -> usize {
    let mut count = 0;
    if edge[count].setLine::<EVEN>(&pts[0], &pts[1], clip) {
        list[count] = &mut edge[count];
        count += 1;
    }
    if edge[count].setLine::<EVEN>(&pts[1], &pts[2], clip) {
        list[count] = &mut edge[count];
        count += 1;
    }
    if edge[count].setLine::<EVEN>(&pts[2], &pts[0], clip) {
        list[count] = &mut edge[count];
        count += 1;
    }
    count
}
// Source: src/core/SkScan_Path.cpp:686-724
unsafe fn sk_fill_triangle<const EVEN: bool>(
    pts: &[crate::src::core::SkPoint::SkPoint; 3],
    clip: Option<&crate::src::core::SkRect::SkIRect>,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ir: &crate::src::core::SkRect::SkIRect,
) {
    let mut edges = [SkEdge::new(), SkEdge::new(), SkEdge::new()];
    let mut list = [core::ptr::null_mut(); 3];
    let count = build_tri_edges::<EVEN>(&mut edges, pts, clip, &mut list);
    if count < 2 {
        return;
    }
    let mut head = SkEdge::new();
    let mut tail = SkEdge::new();
    let mut last = core::ptr::null_mut();
    // These stack arrays/sentinels stay at their addresses until traversal returns.
    let first = sort_edges(&mut list[..count], &mut last);
    head.fPrev = core::ptr::null_mut();
    head.fNext = first;
    head.fFirstY = kEDGE_HEAD_Y;
    head.fX = i32::MIN;
    (*first).fPrev = &mut head;
    tail.fPrev = last;
    tail.fNext = core::ptr::null_mut();
    tail.fFirstY = kEDGE_TAIL_Y;
    (*last).fNext = &mut tail;
    let mut stop_y = ir.fBottom;
    if let Some(clip) = clip {
        if stop_y > clip.fBottom {
            stop_y = clip.fBottom;
        }
    }
    let mut start_y = ir.fTop;
    if let Some(clip) = clip {
        if start_y < clip.fTop {
            start_y = clip.fTop;
        }
    }
    walk_simple_edges(&mut head, blitter.as_mut(), start_y, stop_y);
}
/// Safety: destination/context leases satisfy the SkBlitter primitive contract.
pub(crate) unsafe fn FillTriangleRasterClip(
    pts: &[crate::src::core::SkPoint::SkPoint; 3],
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    FillTriangleWithRounding::<false>(pts, clip, blitter.as_mut());
}
// Source: src/core/SkScan_Path.cpp:726-767
unsafe fn FillTriangleWithRounding<const EVEN: bool>(
    pts: &[crate::src::core::SkPoint::SkPoint; 3],
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    use crate::src::core::SkRect::{SkIRect, SkRect};
    if clip.is_empty() {
        return;
    }
    let Some(bounds) = SkRect::Bounds(pts) else {
        return;
    };
    let limit = (i16::MAX >> 1) as f32;
    if !SkRect::MakeLTRB(-limit, -limit, limit, limit).contains(&bounds) {
        let triangle = crate::src::core::SkPathRawShapes::Triangle::new(pts, &bounds);
        crate::src::core::SkScan::SkScan::FillPathRasterClip(
            &triangle.raw(),
            clip,
            blitter.as_mut(),
        );
        return;
    }
    let ir = conservative_round_to_int(&bounds);
    if ir.is_empty() || !SkIRect::intersects(&ir, clip.get_bounds()) {
        return;
    }
    let mut wrapper = Box::pin(
        crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(clip, blitter.as_mut()),
    );
    let (region, mut blitter) = wrapper.as_mut().parts();
    let mut clipper = Box::pin(crate::src::core::SkScanPriv::SkScanClipper::new(
        blitter.as_mut(),
        Some(region),
        &ir,
        false,
        false,
    ));
    let rect = clipper.getClipRect().copied();
    if let Some(mut blitter) = clipper.as_mut().getBlitter() {
        sk_fill_triangle::<EVEN>(pts, rect.as_ref(), blitter.as_mut(), &ir);
    }
}
impl crate::src::core::SkScan::SkScan {
    // Source: src/core/SkScan_Path.cpp:769-785
    // Rust names distinguish the original Region/RasterClip overloads.
    pub unsafe fn FillPathRasterClip(
        raw: &crate::src::core::SkPathRaw::SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
    ) {
        if clip.is_empty() {
            return;
        }
        if clip.is_bw() {
            Self::FillPath(raw, clip.bw_rgn(), blitter.as_mut());
        } else {
            let mut tmp = crate::src::core::SkRegion::SkRegion::new();
            let mut aa_blitter =
                core::pin::pin!(crate::src::core::SkAAClip::SkAAClipBlitter::new());
            tmp.set_rect(*clip.get_bounds());
            aa_blitter.as_mut().init(blitter.as_mut(), clip.aa_rgn());
            Self::FillPath(raw, &tmp, aa_blitter.as_mut());
        }
    }
}
