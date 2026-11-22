//! Resident ScrollableArea traversal for wheel may-begin handling.
//!
//! This is the package-boundary form of Blink's `ScrollableAreaTraversal`.
//! The cursor roots each original `PaintLayerScrollableArea`; it does not
//! materialize a scroll tree or copy scroll geometry.

use crate::{
    internal::{layout_box::LayoutBox, layout_scrollable_area::PaintLayerScrollableArea},
    layout_engine::LayoutEngine,
};
use foundation::Persistent;

/// Opaque ownership of one resident Blink `ScrollableArea` identity.
pub struct NativeWheelScrollableArea {
    area: Persistent<PaintLayerScrollableArea>,
}

impl Clone for NativeWheelScrollableArea {
    fn clone(&self) -> Self {
        Self {
            area: Persistent::from_ptr(self.area.Get()),
        }
    }
}

impl NativeWheelScrollableArea {
    fn FromPointer(area: *mut PaintLayerScrollableArea) -> Option<Self> {
        (!area.is_null()).then(|| Self {
            area: Persistent::from_ptr(area),
        })
    }

    fn Area(&self) -> &PaintLayerScrollableArea {
        unsafe { &*self.area.Get() }
    }

    pub fn Identity(&self) -> usize {
        self.area.Get() as usize
    }

    pub fn GetScrollElementId(&self) -> foundation::CompositorElementId {
        self.Area().GetScrollElementId()
    }

    pub fn ScrollOffset(&self) -> (f64, f64) {
        let offset = self.Area().GetScrollOffset();
        (f64::from(offset.x()), f64::from(offset.y()))
    }

    pub fn MinimumScrollOffset(&self) -> (f64, f64) {
        let offset = self.Area().MinimumScrollOffset();
        (f64::from(offset.x()), f64::from(offset.y()))
    }

    pub fn MaximumScrollOffset(&self) -> (f64, f64) {
        let offset = self.Area().MaximumScrollOffset();
        (f64::from(offset.x()), f64::from(offset.y()))
    }

    pub fn FadeInScrollbarIfExists(&self, horizontal: bool, vertical: bool) -> bool {
        self.Area().FadeInScrollbarIfExists(horizontal, vertical)
    }

    pub fn FadeOutScrollbarIfNeeded(&self) {
        self.Area().FadeOutScrollbarIfNeeded();
    }

    fn Next(&self) -> Option<Self> {
        let current_box = unsafe { self.Area().GetLayoutBox().as_ref() }?;
        let next_box = GetEnclosingLayoutBoxCrossingDocumentBoundary(current_box);
        NativeWheelScrollableArea::FromPointer(GetNearestScrollableArea(next_box))
    }
}

/// Nearest-to-farthest cursor over the original resident scroll owners.
pub struct NativeWheelScrollableAreaTraversal {
    current: Option<NativeWheelScrollableArea>,
}

impl NativeWheelScrollableAreaTraversal {
    pub(crate) fn ForObject(object: &crate::internal::layout_object::LayoutObject) -> Self {
        let diagnose = std::env::var_os("RECHROM_DIAG_INPUT_HIT_TEST").is_some();
        if diagnose {
            let node = unsafe { object.GetNode().as_ref() }.map(|node| node.InputId());
            eprintln!(
                "interaction-scroll-traversal target_object={:p} target_node={node:?} target_name={}",
                object,
                object.DebugName().Utf8(),
            );
        }
        let Some(layout_box) = (unsafe { object.EnclosingBox().as_ref() }) else {
            if diagnose {
                eprintln!("interaction-scroll-traversal enclosing_box=none");
            }
            return Self::Empty();
        };
        if diagnose {
            let node = unsafe { layout_box.GetNode().as_ref() }.map(|node| node.InputId());
            eprintln!(
                "interaction-scroll-traversal enclosing_box={:p} node={node:?} name={}",
                layout_box,
                layout_box.DebugName().Utf8(),
            );
        }
        Self {
            current: NativeWheelScrollableArea::FromPointer(GetNearestScrollableArea(
                layout_box as *const LayoutBox as *mut LayoutBox,
            )),
        }
    }
    fn Empty() -> Self {
        Self { current: None }
    }
}

impl Iterator for NativeWheelScrollableAreaTraversal {
    type Item = NativeWheelScrollableArea;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        self.current = current.Next();
        Some(current)
    }
}

// Source: core/scroll/scrollable_area.cc:1489-1498. A LayoutEngine owns one
// local-frame tree and intentionally has no Frame::OwnerLayoutObject edge.
// Therefore LayoutView ends this local traversal; Page may compose an outer
// frame traversal when it owns that typed host edge.
fn GetEnclosingLayoutBoxCrossingDocumentBoundary(layout_box: &LayoutBox) -> *mut LayoutBox {
    if layout_box.IsLayoutView() {
        std::ptr::null_mut()
    } else {
        layout_box.ContainingBlock().cast()
    }
}

// Source: core/scroll/scrollable_area.cc:1500-1522.
fn GetNearestScrollableArea(mut next_box: *mut LayoutBox) -> *mut PaintLayerScrollableArea {
    let diagnose = std::env::var_os("RECHROM_DIAG_INPUT_HIT_TEST").is_some();
    while let Some(box_) = unsafe { next_box.as_ref() } {
        let area = box_.GetScrollableArea();
        if diagnose {
            let node = unsafe { box_.GetNode().as_ref() }.map(|node| node.InputId());
            let area_element = unsafe { area.as_ref() }.map(|area| area.GetScrollElementId());
            eprintln!(
                "interaction-scroll-traversal box={:p} node={node:?} name={} view={} global_root={} scroll_container={} overscroll_container={} area={:p} area_element={area_element:?}",
                box_,
                box_.DebugName().Utf8(),
                box_.IsLayoutView(),
                box_.IsGlobalRootScroller(),
                box_.IsScrollContainer(),
                box_.IsOverscrollContainer(),
                area,
            );
        }
        let is_source_scrollable = box_.IsGlobalRootScroller()
            || (box_.IsScrollContainer() && {
                assert!(
                    !area.is_null(),
                    "source scroll container owns a PaintLayerScrollableArea"
                );
                let area = unsafe { &*area };
                area.ScrollsOverflow() || !area.CanPropagateScroll()
            })
            || box_.IsOverscrollContainer();
        if is_source_scrollable {
            assert!(
                !area.is_null(),
                "source scroll/overscroll owner has a PaintLayerScrollableArea"
            );
            if diagnose {
                eprintln!(
                    "interaction-scroll-traversal selected_area={:p} selected_element={:?}",
                    area,
                    unsafe { &*area }.GetScrollElementId(),
                );
            }
            return area;
        }
        next_box = GetEnclosingLayoutBoxCrossingDocumentBoundary(box_);
    }
    // Chromium returns Page::GetVisualViewport here. Rechrom has no resident
    // VisualViewport ScrollableArea in LayoutNG, so ending the cursor is the
    // explicit unsupported branch; do not substitute the LayoutView area.
    std::ptr::null_mut()
}

impl LayoutEngine {
    /// Source: core/scroll/scrollable_area.cc:1526-1536.
    pub fn WheelScrollableAreaTraversal(
        &self,
        target_node_id: u64,
    ) -> NativeWheelScrollableAreaTraversal {
        let Some(object) = self
            .GetLayoutTree()
            .and_then(|tree| tree.OutputNode(target_node_id))
        else {
            return NativeWheelScrollableAreaTraversal::Empty();
        };
        NativeWheelScrollableAreaTraversal::ForObject(object)
    }
}
