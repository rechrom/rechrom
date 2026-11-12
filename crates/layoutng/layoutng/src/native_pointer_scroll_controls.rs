//! Original Scrollbar and PaintLayerScrollableArea input owners.
//! Source scrollbar.cc:386-429; pointer_event_manager.cc:901-954;
//! paint_layer_scrollable_area.cc:900-933,2442-2515.
pub use crate::internal::layout_scrollable_area::NativeResizePlan;
pub use crate::internal::layout_scrollable_area::{
    NativeOverlayVisibilityUpdate, NativeScrollbarInteraction,
};
use crate::{
    internal::{
        layout_scrollable_area::{PaintLayerScrollableArea, ResizerHitTestType},
        scrollbar::Scrollbar,
    },
    layout_engine::LayoutEngine,
};
use foundation::{gfx, Persistent};
#[derive(Clone)]
pub struct NativePointerResizeArea {
    area: Persistent<PaintLayerScrollableArea>,
}
impl NativePointerResizeArea {
    pub fn Identity(&self) -> usize {
        self.area.Get() as usize
    }
    fn area(&self) -> &PaintLayerScrollableArea {
        unsafe { &*self.area.Get() }
    }
}
impl LayoutEngine {
    pub fn PointerScrollbarInteraction(
        &self,
        scrollbar: &Scrollbar,
        interaction: NativeScrollbarInteraction,
        is_popup: bool,
    ) -> Option<(NativePointerResizeArea, NativeOverlayVisibilityUpdate)> {
        let _scope = self.NativeScrollPassScope();
        let area = unsafe { scrollbar.ScrollableArea().as_mut()? };
        if area.HasBeenDisposed() {
            return None;
        }
        if matches!(
            interaction,
            NativeScrollbarInteraction::Entered | NativeScrollbarInteraction::Exited
        ) {
            area.NotifyMacScrollbarMouseInteraction(
                scrollbar as *const _ as *mut _,
                matches!(interaction, NativeScrollbarInteraction::Entered),
            );
        }
        let update = area.HandleScrollbarInteraction(interaction, is_popup);
        Some((
            NativePointerResizeArea {
                area: Persistent::from_ptr(area),
            },
            update,
        ))
    }
    pub fn PointerScrollbarFadeTimerFired(&self, area: &NativePointerResizeArea) {
        let _scope = self.NativeScrollPassScope();
        if !area.area().HasBeenDisposed() {
            unsafe { &mut *area.area.Get() }.FadeOverlayScrollbarsTimerFired();
        }
    }
    pub fn PublishPointerMacScrollbarAnimation(
        &self,
        scrollbar: &Scrollbar,
        thumb: f32,
        track: f32,
        width: i32,
        thumb_dirty: bool,
        track_dirty: bool,
        hidden: Option<bool>,
    ) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(area) = (unsafe { scrollbar.ScrollableArea().as_mut() }) else {
            return false;
        };
        area.PublishMacScrollbarAnimation(scrollbar.Orientation() == crate::internal::scrollbar_orientation::ScrollbarOrientation::kHorizontalScrollbar, thumb, track, width, thumb_dirty, track_dirty, hidden)
    }
    /// scrollbar.cc:226-231,845-855. Geometry is evaluated at the source
    /// ScrollAnimator's DesiredTargetOffset, preserving its native theme.
    pub fn PointerScrollbarTargetThumbGeometry(
        &self,
        scrollbar: &Scrollbar,
        target: gfx::Vector2dF,
    ) -> (i32, i32) {
        let _scope = self.NativeScrollPassScope();
        let state = scrollbar
            .InputScrollState()
            .expect("connected source scrollbar");
        let position = if state.horizontal {
            target.x() - state.minimum_offset.x()
        } else {
            target.y() - state.minimum_offset.y()
        };
        let geometry = scrollbar.ThemeGeometryAt(position);
        (
            geometry.track_position + geometry.thumb_position,
            geometry.thumb_length,
        )
    }
    pub fn PointerResizeAreaForNode(&self, node: u64) -> Option<NativePointerResizeArea> {
        let _scope = self.NativeScrollPassScope();
        let object = self.GetLayoutTree()?.GetLayoutObjectForNode(node)?;
        let layer = unsafe { object.EnclosingLayer().as_ref() }?;
        let area = unsafe { layer.GetLayoutObject().GetScrollableArea().as_ref() }?;
        (!area.HasBeenDisposed()).then(|| NativePointerResizeArea {
            area: Persistent::from_ptr(area as *const _ as *mut _),
        })
    }
    pub fn PointerResizeAreaInResizeMode(&self, area: &NativePointerResizeArea) -> bool {
        let _scope = self.NativeScrollPassScope();
        !area.area().HasBeenDisposed() && area.area().InResizeMode()
    }
    pub fn SetPointerResizeAreaInResizeMode(&self, area: &NativePointerResizeArea, value: bool) {
        let _scope = self.NativeScrollPassScope();
        if !area.area().HasBeenDisposed() {
            unsafe { &mut *area.area.Get() }.SetInResizeMode(value);
        }
    }
    pub fn PointerResizeAreaHasLayerAndLayoutBox(&self, area: &NativePointerResizeArea) -> bool {
        let _scope = self.NativeScrollPassScope();
        !area.area().HasBeenDisposed()
            && unsafe { area.area().GetLayoutBox().as_ref() }
                .is_some_and(|box_| !box_.Layer().is_null())
    }
    pub fn PointerResizeAreaContainsTouchPoint(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
    ) -> bool {
        let _scope = self.NativeScrollPassScope();
        !area.area().HasBeenDisposed()
            && area
                .area()
                .IsAbsolutePointInResizeControl(&point, ResizerHitTestType::kResizerForTouch)
    }
    pub fn InitializePointerResizeTransform(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
    ) -> gfx::Transform {
        let _scope = self.NativeScrollPassScope();
        area.area().InitializeResizeTransform(&point)
    }
    pub fn PlanPointerResize(
        &self,
        area: &NativePointerResizeArea,
        point_in_containing_frame: gfx::Point,
        transform: &gfx::Transform,
    ) -> Option<NativeResizePlan> {
        let _scope = self.NativeScrollPassScope();
        if area.area().HasBeenDisposed() {
            return None;
        }
        area.area()
            .PlanResizeInContainingFrame(point_in_containing_frame, transform)
    }
    pub fn PointerScrollbarPointFromRootFrame(
        &self,
        scrollbar: &Scrollbar,
        point_in_containing_frame: gfx::Point,
    ) -> gfx::Point {
        self.ScrollbarPointFromContainingFrame(scrollbar, point_in_containing_frame)
    }
}
