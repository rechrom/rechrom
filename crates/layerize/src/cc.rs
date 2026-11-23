//! Main-thread cc translations (cc/layers Layer side, cc/input Scrollbar).
//! The impl half reaches them through `commit::layers`, because
//! TreeSynchronizer::PushLayerProperties reads these owners inside the
//! protected sequence of LayerTreeHostImpl::FinishCommit.

pub mod input {
    pub(crate) use compositor_input::hit_test_opaqueness;
    pub mod scrollbar;
}
pub mod layers {
    pub(crate) use compositor_input::scroll_hit_test_rect;
    pub(crate) use compositor_layerize::content_layer_client;
    pub mod layer;
    pub mod layer_list_iterator;
    pub mod painted_scrollbar_layer;
    pub mod picture_layer;
    pub mod recording_source;
    pub mod scrollbar_layer_base;
    pub mod solid_color_layer;
    pub mod solid_color_scrollbar_layer;
    pub mod surface_layer;
    pub mod touch_action_region;
}

// Module paths of the shared cc value roots under which the moved
// translations name them (the compositor keeps the same paths). Aliases only.
pub(crate) mod base {
    pub use crate::commit::{invalidation_region, math_util, protected_sequence_synchronizer};
    pub use compositor_tree::region;
}
pub(crate) mod debug {
    pub use crate::commit::layer_tree_debug_state;
}
pub(crate) mod trees {
    pub use crate::commit::{
        commit_state, damage_reason, draw_property_utils, effect_node, layer_tree_settings,
        property_ids, viewport_property_ids,
    };
}
