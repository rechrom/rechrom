#![allow(non_snake_case)]

use foundation::{DynamicTo, MakeGarbageCollected, Member, RuntimeEnabledFeatures, Visitor};

use super::super::layout_box::LayoutBox;
use super::super::layout_invalidation_reason;
use super::super::layout_object::LayoutObject;
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
                unsafe { &mut *box_ }.SetNeedsLayout(layout_invalidation_reason::UnknownReason());
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

/// Run the TextOverflowPostLayoutSnapshot part of LocalFrame's
/// "snapshot post-layout state steps" over the resident source tree.
///
/// Chromium keeps a weak client set on LocalFrame.  This extracted package
/// has no frame owner, so the clean LayoutObject tree is the authoritative
/// client registry: only scrollable areas with an installed snapshot are
/// visited.  Take the child pointer before servicing the object because an
/// invalid snapshot marks its LayoutBox (and ancestors) for the repeat pass.
pub(crate) fn UpdateTextOverflowPostLayoutSnapshots(root: &mut LayoutObject) -> bool {
    if !RuntimeEnabledFeatures::DisableEllipsisWhenScrolledEnabled() {
        return false;
    }
    let mut invalid = false;
    let mut pending = vec![root as *mut LayoutObject];
    while let Some(object) = pending.pop() {
        let object_ref = unsafe { &*object };
        let mut child = object_ref.SlowFirstChild();
        while !child.is_null() {
            pending.push(child);
            child = unsafe { &*child }.NextSibling();
        }

        let box_ = DynamicTo::<LayoutBox>(object);
        if box_.is_null() {
            continue;
        }
        let area = unsafe { &*box_ }.GetScrollableArea();
        if !area.is_null() {
            invalid |= unsafe { &mut *area }.UpdateTextOverflowSnapshot();
        }
    }
    invalid
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{gfx, LayoutHeapScope};
    use layoutng_style::style::scroll_enums::mojom::blink::ScrollType;

    #[test]
    fn snapshot_changes_only_when_inline_scroll_state_toggles() {
        let _heap = LayoutHeapScope::new();
        let mut box_ = LayoutBox::new(std::ptr::null_mut());
        box_.SetIsHorizontalWritingMode(true);
        let mut area = PaintLayerScrollableArea::new(&mut box_);
        let snapshot = TextOverflowPostLayoutSnapshot::new(&mut area);

        area.UpdateScrollOffset(gfx::Vector2dF::new(0.0, 8.0), ScrollType::kClamping);
        assert!(!unsafe { &mut *snapshot }.UpdateSnapshot());
        assert!(!unsafe { &*snapshot }.IsScrolled());

        area.UpdateScrollOffset(gfx::Vector2dF::new(3.0, 8.0), ScrollType::kClamping);
        assert!(unsafe { &mut *snapshot }.UpdateSnapshot());
        assert!(unsafe { &*snapshot }.IsScrolled());

        area.UpdateScrollOffset(gfx::Vector2dF::new(4.0, 0.0), ScrollType::kClamping);
        assert!(!unsafe { &mut *snapshot }.UpdateSnapshot());

        box_.SetIsHorizontalWritingMode(false);
        assert!(unsafe { &mut *snapshot }.UpdateSnapshot());
        assert!(!unsafe { &*snapshot }.IsScrolled());
        area.UpdateScrollOffset(gfx::Vector2dF::new(4.0, 2.0), ScrollType::kClamping);
        assert!(unsafe { &mut *snapshot }.UpdateSnapshot());
        assert!(unsafe { &*snapshot }.IsScrolled());
    }
}
