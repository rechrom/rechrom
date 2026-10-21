#![allow(non_snake_case)]

use std::ffi::c_char;

use foundation::{MakeGarbageCollected, Member, Visitor};

use super::layout_invalidation_reason;
use super::layout_scrollable_area::PaintLayerScrollableArea;

// cpp: layoutng/internal/text_overflow_post_layout_snapshot.h:21-38
pub struct TextOverflowPostLayoutSnapshot {
    scroller_: Member<PaintLayerScrollableArea>,
    is_scrolled_: bool,
}

impl TextOverflowPostLayoutSnapshot {
    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.h:24-24
    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.cc:19-23
    pub fn new(scroller: &mut PaintLayerScrollableArea) -> *mut Self {
        let snapshot = MakeGarbageCollected(Self {
            scroller_: Member::from_ptr(scroller),
            is_scrolled_: false,
        });
        scroller.RegisterTextOverflowPostLayoutSnapshot(snapshot);
        snapshot
    }

    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.h:26-26
    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.cc:25-35
    pub fn UpdateSnapshot(&mut self) -> bool {
        let scroller = unsafe { &*self.scroller_.Get() };
        let box_ = scroller.GetLayoutBox();
        if !box_.is_null() {
            let is_scrolled = self.ComputeIsScrolled();
            if self.is_scrolled_ != is_scrolled {
                self.is_scrolled_ = is_scrolled;
                let reason = unsafe { &layout_invalidation_reason::kUnknown as *const c_char };
                unsafe { &mut *box_ }.SetNeedsLayout(reason);
                return true;
            }
        }
        false
    }

    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.h:27-27
    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.cc:45-47
    pub fn ShouldScheduleNextService(&self) -> bool {
        false
    }

    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.h:29-29
    pub fn IsScrolled(&self) -> bool {
        self.is_scrolled_
    }

    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.cc:37-43
    fn ComputeIsScrolled(&self) -> bool {
        let scroller = unsafe { &*self.scroller_.Get() };
        let box_ = scroller.GetLayoutBox();
        if !box_.is_null() {
            let offset = scroller.GetScrollOffset();
            return if unsafe { &*box_ }.IsHorizontalWritingMode() {
                offset.x() != 0.0
            } else {
                offset.y() != 0.0
            };
        }
        false
    }

    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.h:31-31
    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.cc:49-51
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.scroller_);
    }
}

#[allow(non_snake_case)]
impl PaintLayerScrollableArea {
    // cpp: layoutng/internal/layout_scrollable_area.h:135-137
    // cpp: layoutng/internal/text_overflow_post_layout_snapshot.cc:14-17
    pub fn UpdateTextOverflowSnapshot(&mut self) -> bool {
        let snapshot = self.GetTextOverflowPostLayoutSnapshot();
        !snapshot.is_null() && unsafe { &mut *snapshot }.UpdateSnapshot()
    }
}
