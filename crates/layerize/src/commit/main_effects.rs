//! Values the main half reports to its client from the commit protocol
//! (LayerTreeHostClient::UpdateCompositorScrollState and the
//! cc::ScrollCallbacks notifications PAC installs on the scroll tree).
//! Source: cc/trees/layer_tree_host_client.h; cc/trees/property_tree.h:583-598.
//! `CompositorScrollState` moved from compositor/src/engine.rs and
//! `PacScrollEffect` from compositor/src/runtime/pac_scroll.rs.
use super::{
    geometry::PointF, input_types::ElementId, scroll_snap_data::TargetSnapAreaElementIds,
    scroll_source_type::ScrollSourceType,
};

/// LayerTreeHostClient::UpdateCompositorScrollState, from the same commit
/// as the preceding scroll deltas. Element/source identities remain native.
#[derive(Clone, Debug)]
pub struct CompositorScrollState {
    pub is_scroll_active: bool,
    pub scroll_latched_element_id: ElementId,
    pub source_type: ScrollSourceType,
    pub done_containers: Vec<ElementId>,
}

#[derive(Clone, Debug)]
pub enum PacScrollEffect {
    DidUpdateCompositorScrollState(CompositorScrollState),
    DidCompositorScroll {
        sequence: u64,
        element: ElementId,
        offset: PointF,
        source: ScrollSourceType,
        snap: Option<TargetSnapAreaElementIds>,
    },
    DidChangeScrollbarsHidden {
        element: ElementId,
        hidden: bool,
    },
}
