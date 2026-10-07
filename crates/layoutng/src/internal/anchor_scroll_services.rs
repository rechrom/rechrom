#![allow(non_snake_case)]

use foundation::{gfx, CompositorElementId};

use super::layout_box::LayoutBox;
use super::layout_input::OverscrollType;

// cpp: layoutng/internal/anchor_scroll_services.h:10-15
#[derive(Clone, Copy, Debug)]
pub struct AnchorScrollContainerState {
    pub id: CompositorElementId,
    pub offset: gfx::Vector2dF,
    pub origin: gfx::Vector2d,
    pub is_layout_view: bool,
}

// cpp: layoutng/internal/anchor_scroll_services.h:18-18
// cpp: layoutng/internal/anchor_position_scroll_data.cc:19-25
pub fn ReadAnchorScrollContainerState(box_: &LayoutBox) -> AnchorScrollContainerState {
    let area = box_.GetScrollableArea();
    assert!(!area.is_null());
    let area = unsafe { &*area };
    AnchorScrollContainerState {
        id: area.GetScrollElementId(),
        offset: area.GetScrollOffset(),
        origin: area.ScrollOrigin().OffsetFromOrigin(),
        is_layout_view: unsafe { &*area.GetLayoutBox() }.IsLayoutView(),
    }
}

// cpp: layoutng/internal/anchor_scroll_services.h:19-19
// cpp: layoutng/internal/anchor_position_scroll_data.cc:27-30
pub fn AnchorViewportUsesTransformOverscroll(box_: &LayoutBox) -> bool {
    box_.ViewportGeometryForLayout().overscroll_type == OverscrollType::kTransform
}
