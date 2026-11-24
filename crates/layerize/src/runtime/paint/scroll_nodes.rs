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
    runtime::types::PaintSkiaBackend,
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
impl<R: PaintSkiaBackend> PropertyTreeManagerScrollNode for ScrollNode<R> {
    fn CcNodeId(&self, s: i32) -> i32 {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.cc_node_id(a, s))
    }
    fn SetCcNodeId(&self, s: i32, id: i32) {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.set_cc_node_id(a, s, id))
    }
    fn IsRoot(&self) -> bool {
        read(|a| self.node().Parent(a).is_none())
    }
    fn Parent(&self) -> Option<Self> {
        self.parent_handle()
    }
    fn ContainerRect(&self) -> Rect {
        rect(read(|a| self.node().ContainerRect(a)))
    }
    fn GetCompositorElementId(&self) -> ElementId {
        let id = read(|a| self.node().GetCompositorElementId(a)).GetInternalValue();
        if id == 0 {
            ElementId::default()
        } else {
            ElementId::new(id)
        }
    }
    fn ScrollingContentsCullRect(&self) -> Rect {
        rect(read(|a| self.node().ScrollingContentsCullRect(a)))
    }
    fn ContentsRect(&self) -> Rect {
        rect(read(|a| self.node().ContentsRect(a)))
    }
    fn UserScrollableHorizontal(&self) -> bool {
        read(|a| self.node().UserScrollableHorizontal(a))
    }
    fn UserScrollableVertical(&self) -> bool {
        read(|a| self.node().UserScrollableVertical(a))
    }
    fn PreventViewportScrollingFromInner(&self) -> bool {
        read(|a| self.node().PreventViewportScrollingFromInner(a))
    }
    fn MaxScrollOffsetAffectedByPageScale(&self) -> bool {
        read(|a| self.node().MaxScrollOffsetAffectedByPageScale(a))
    }
    fn OverscrollBehavior(&self) -> OverscrollBehavior {
        let convert = |v| match v {
            paint::paint_property_tree::LayoutOverscrollBehavior::kNone => Type::kNone,
            paint::paint_property_tree::LayoutOverscrollBehavior::kAuto => Type::kAuto,
            paint::paint_property_tree::LayoutOverscrollBehavior::kContain => Type::kContain,
            paint::paint_property_tree::LayoutOverscrollBehavior::kChain => Type::kChain,
        };
        let b = read(|a| self.node().OverscrollBehavior(a));
        OverscrollBehavior::FromAxes(convert(b[0]), convert(b[1]))
    }
    fn GetSnapContainerData(&self) -> Option<SnapContainerData> {
        let (is_root, absent, snap) = read(|a| {
            let state = self.node().state(a);
            (
                state.parent.is_none(),
                state.compositor.snap_container_data_absent,
                state.compositor.snap_container_data,
            )
        });
        assert!(
            is_root || absent || snap.is_some(),
            "PaintInput rejects a non-root scroll node without native snap metadata"
        );
        snap.map(scroll_snap_type).map(SnapContainerData::FromType)
    }
    fn PreventScrollAxisLocking(&self) -> bool {
        read(|a| self.node().PreventScrollAxisLocking(a))
    }
    fn GetMainThreadRepaintReasons(&self) -> MainThreadRepaintReasons {
        let bits = read(|a| self.node().GetMainThreadRepaintReasons(a));
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
        read(|a| self.node().GetCompositedScrollingPreference(a))
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
