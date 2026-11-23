//! Commit vocabulary shared by the main (Layerize) and impl (compositor)
//! halves: the values that cross ProxyMain::BeginMainFrame ->
//! ProxyImpl::NotifyReadyToCommitOnImpl and the protected-sequence contract
//! under which LayerTreeHostImpl::FinishCommit reads main-thread state.
//! Nothing here names Blink paint types.

pub mod commit_state;
pub mod geometry;
pub mod layer_impl_factory;
pub mod layer_tree_debug_state;
pub mod layer_value_backend;
pub mod property_tree;
pub mod property_tree_services;
pub mod property_types;
pub mod protected_sequence_synchronizer;
pub mod push_properties;
pub mod scroll_elasticity;
pub mod scroll_elasticity_utils;
pub mod scroll_node;
pub mod skia_filters;
pub mod values;

// cc value modules translated in the shared model packages. Both halves name
// them through this vocabulary.
// Source: cc/base/{math_util,synced_property}.h; cc/trees/{clip_node,
//         effect_node,transform_node,property_ids,viewport_property_ids,
//         target_property,damage_reason,property_animation_state,
//         compositor_commit_data,scroll_source_type}.h;
//         cc/input/{scroll_snap_data,overscroll_behavior,
//         main_thread_scrolling_reason}.h.
pub use compositor_animation::property_animation_state;
pub use compositor_draw::damage_reason;
pub use compositor_input::{
    browser_controls_params, compositor_commit_data, event_listener_properties, input_types,
    layer_selection_bound, main_thread_scrolling_reason, overscroll_behavior, page_scale_animation,
    scroll_snap_data, scroll_source_type,
};
pub use compositor_tree::{
    clip_node, commit_types, effect_node, math_util, region_backend, property_ids, synced_property, target_property,
    transform_node, viewport_property_ids,
};

pub use commit_state::{CommitStateTypes, CommitTypes, PropertyTreesFor};
