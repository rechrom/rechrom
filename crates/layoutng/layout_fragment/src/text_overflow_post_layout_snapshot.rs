#![allow(non_snake_case)]

use std::ffi::c_char;

use foundation::{MakeGarbageCollected, Member, Visitor};

use super::super::layout_invalidation_reason;
use super::super::layout_scrollable_area::PaintLayerScrollableArea;

pub struct TextOverflowPostLayoutSnapshot {
    scroller_: Member<PaintLayerScrollableArea>,
    is_scrolled_: bool,
}

impl TextOverflowPostLayoutSnapshot {
    pub fn new(scroller: &mut PaintLayerScrollableArea) -> *mut Self {
        let snapshot = MakeGarbageCollected(Self {
            scroller_: Member::from_ptr(scroller),
            is_scrolled_: false,
        });
        scroller.RegisterTextOverflowPostLayoutSnapshot(snapshot);
        snapshot
    }

    #[inline(never)]
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

    pub fn ShouldScheduleNextService(&self) -> bool {
        false
    }

    pub fn IsScrolled(&self) -> bool {
        self.is_scrolled_
    }

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

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.scroller_);
    }
}

#[allow(non_snake_case)]
impl PaintLayerScrollableArea {
    pub fn UpdateTextOverflowSnapshot(&mut self) -> bool {
        let snapshot = self.GetTextOverflowPostLayoutSnapshot();
        !snapshot.is_null() && unsafe { &mut *snapshot }.UpdateSnapshot()
    }
}
