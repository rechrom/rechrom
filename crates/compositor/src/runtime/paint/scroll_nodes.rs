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
        scroll_snap_data::{ScrollSnapType, SnapAxis, SnapContainerData, SnapStrictness},
    },
    runtime::types::CpuSkiaBackend,
};
fn scroll_snap_type(value: paint::paint_property_tree::PaintScrollSnapType) -> ScrollSnapType {
    use paint::paint_property_tree::{PaintScrollSnapAxis as Axis, PaintScrollSnapStrictness as S};
    let axis = match value.axis {
        Axis::Both => SnapAxis::kBoth,
        Axis::X => SnapAxis::kX,
        Axis::Y => SnapAxis::kY,
        Axis::Block => SnapAxis::kBlock,
        Axis::Inline => SnapAxis::kInline,
        Axis::Pair => SnapAxis::kPair,
    };
    let strictness = match value.strictness {
        S::Proximity => SnapStrictness::kProximity,
        S::Mandatory => SnapStrictness::kMandatory,
    };
    ScrollSnapType::new(false, axis, strictness)
}
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
    fn IsRoot(&self) -> bool {
        self.node().parent.is_none()
    }
    fn Parent(&self) -> Option<Self> {
        self.node()
            .parent
            .as_ref()
            .map(|p| Self::from_arc(p.clone()))
    }
    fn ContainerRect(&self) -> Rect {
        rect(self.node().ContainerRect())
    }
    fn GetCompositorElementId(&self) -> ElementId {
        let id = self.node().GetCompositorElementId().GetInternalValue();
        if id == 0 {
            ElementId::default()
        } else {
            ElementId::new(id)
        }
    }
    fn ScrollingContentsCullRect(&self) -> Rect {
        rect(self.node().ScrollingContentsCullRect())
    }
    fn ContentsRect(&self) -> Rect {
        rect(self.node().ContentsRect())
    }
    fn UserScrollableHorizontal(&self) -> bool {
        self.node().UserScrollableHorizontal()
    }
    fn UserScrollableVertical(&self) -> bool {
        self.node().UserScrollableVertical()
    }
    fn PreventViewportScrollingFromInner(&self) -> bool {
        self.node().PreventViewportScrollingFromInner()
    }
    fn MaxScrollOffsetAffectedByPageScale(&self) -> bool {
        self.node().MaxScrollOffsetAffectedByPageScale()
    }
    fn OverscrollBehavior(&self) -> OverscrollBehavior {
        let convert = |v| match v {
            paint::paint_property_tree::LayoutOverscrollBehavior::kNone => Type::kNone,
            paint::paint_property_tree::LayoutOverscrollBehavior::kAuto => Type::kAuto,
            paint::paint_property_tree::LayoutOverscrollBehavior::kContain => Type::kContain,
            paint::paint_property_tree::LayoutOverscrollBehavior::kChain => Type::kChain,
        };
        let b = self.node().OverscrollBehavior();
        OverscrollBehavior::FromAxes(convert(b[0]), convert(b[1]))
    }
    fn GetSnapContainerData(&self) -> Option<SnapContainerData> {
        assert!(
            self.node().parent.is_none()
                || self.node().compositor.snap_container_data_absent
                || self.node().compositor.snap_container_data.is_some(),
            "PaintInput rejects a non-root scroll node without native snap metadata"
        );
        self.node()
            .compositor
            .snap_container_data
            .map(scroll_snap_type)
            .map(SnapContainerData::FromType)
    }
    fn PreventScrollAxisLocking(&self) -> bool {
        self.node().PreventScrollAxisLocking()
    }
    fn GetMainThreadRepaintReasons(&self) -> MainThreadRepaintReasons {
        let bits = self.node().GetMainThreadRepaintReasons();
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
        self.node().GetCompositedScrollingPreference()
            == paint_property_tree::PaintCompositedScrollingPreference::NotPreferred
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computed_scroll_snap_type_reaches_cc_container_data() {
        let data = paint::paint_property_tree::PaintScrollSnapType {
            axis: paint::paint_property_tree::PaintScrollSnapAxis::X,
            strictness: paint::paint_property_tree::PaintScrollSnapStrictness::Mandatory,
        };
        let container = SnapContainerData::FromType(scroll_snap_type(data));
        assert_eq!(
            container.scroll_snap_type(),
            ScrollSnapType::new(false, SnapAxis::kX, SnapStrictness::kMandatory)
        );
    }
}
