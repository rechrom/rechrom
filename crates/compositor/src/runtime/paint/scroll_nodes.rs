//! Native ScrollPaintPropertyNode reads. ScrollRoot's absent snap payload is
//! its real source constructor state; non-root incomplete output is rejected
//! at PaintInput until the full Layout/Paint scroll payload is available.
use super::*;
use crate::{
    adapter::{
        geometry::{Rect, RectF},
        input_types::ElementId,
        property_tree_manager::PropertyTreeManagerScrollNode,
    },
    cc::input::{
        main_thread_scrolling_reason::{MainThreadRepaintReason, MainThreadRepaintReasons},
        overscroll_behavior::{OverscrollBehavior, Type},
        scroll_snap_data::SnapContainerData,
    },
    runtime::types::CpuSkiaBackend,
};
fn rect(p: paint::paint_engine::PaintRect) -> Rect {
    crate::adapter::geometry::ToEnclosingRect(RectF::new(
        p.x as f32,
        p.y as f32,
        p.width as f32,
        p.height as f32,
    ))
}
impl<R: CpuSkiaBackend> PropertyTreeManagerScrollNode for ScrollNode<R> {
    fn CcNodeId(&self, s: i32) -> i32 {
        self.node().lifecycle.cc_node_id(s)
    }
    fn SetCcNodeId(&self, s: i32, id: i32) {
        self.node().lifecycle.set_cc_node_id(s, id)
    }
    fn Parent(&self) -> Option<Self> {
        self.node()
            .parent
            .as_ref()
            .map(|p| Self::from_arc(p.clone()))
    }
    fn ContainerRect(&self) -> Rect {
        rect(self.node().container_rect)
    }
    fn GetCompositorElementId(&self) -> ElementId {
        let id = self
            .node()
            .compositor
            .compositor_element_id
            .GetInternalValue();
        if id == 0 {
            ElementId::default()
        } else {
            ElementId::new(id)
        }
    }
    fn ScrollingContentsCullRect(&self) -> Rect {
        rect(self.node().compositor.scrolling_contents_cull_rect)
    }
    fn ContentsRect(&self) -> Rect {
        rect(self.node().contents_rect)
    }
    fn UserScrollableHorizontal(&self) -> bool {
        self.node().user_scrollable_horizontal
    }
    fn UserScrollableVertical(&self) -> bool {
        self.node().user_scrollable_vertical
    }
    fn PreventViewportScrollingFromInner(&self) -> bool {
        self.node().compositor.prevent_viewport_scrolling_from_inner
    }
    fn MaxScrollOffsetAffectedByPageScale(&self) -> bool {
        self.node()
            .compositor
            .max_scroll_offset_affected_by_page_scale
    }
    fn OverscrollBehavior(&self) -> OverscrollBehavior {
        let convert = |v| match v {
            paint::paint_property_tree::NativeOverscrollBehavior::kNone => Type::kNone,
            paint::paint_property_tree::NativeOverscrollBehavior::kAuto => Type::kAuto,
            paint::paint_property_tree::NativeOverscrollBehavior::kContain => Type::kContain,
            paint::paint_property_tree::NativeOverscrollBehavior::kChain => Type::kChain,
        };
        let b = self.node().compositor.overscroll_behavior;
        OverscrollBehavior::FromAxes(convert(b[0]), convert(b[1]))
    }
    fn GetSnapContainerData(&self) -> Option<SnapContainerData> {
        assert!(
            self.node().parent.is_none() || self.node().compositor.snap_container_data_absent,
            "PaintInput rejects a non-root scroll node without native snap metadata"
        );
        None
    }
    fn PreventScrollAxisLocking(&self) -> bool {
        self.node().compositor.prevent_scroll_axis_locking
    }
    fn GetMainThreadRepaintReasons(&self) -> MainThreadRepaintReasons {
        let bits = self.node().compositor.main_thread_repaint_reasons;
        let mut result = MainThreadRepaintReasons::new();
        for (i, r) in [
            MainThreadRepaintReason::kHasBackgroundAttachmentFixedObjects,
            MainThreadRepaintReason::kNotOpaqueForTextAndLCDText,
            MainThreadRepaintReason::kPreferNonCompositedScrolling,
            MainThreadRepaintReason::kBackgroundNeedsRepaintOnScroll,
        ]
        .into_iter()
        .enumerate()
        {
            if bits & (1 << i) != 0 {
                result.Put(r);
            }
        }
        result
    }
    fn CompositedScrollingNotPreferred(&self) -> bool {
        self.node().compositor.composited_scrolling_preference
            == native::PaintCompositedScrollingPreference::NotPreferred
    }
}
