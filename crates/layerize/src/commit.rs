//! Commit vocabulary shared by the main (Layerize) and impl (compositor)
//! halves: the values that cross ProxyMain::BeginMainFrame ->
//! ProxyImpl::NotifyReadyToCommitOnImpl and the protected-sequence contract
//! under which LayerTreeHostImpl::FinishCommit reads main-thread state.
//! Nothing here names Blink paint types.

// ui/gfx/color_space.h moved to the cc value root.
pub use compositor_values::color_space;
pub mod commit_state;
pub mod completion_event;
pub mod geometry;
pub mod layer_impl_factory;
pub mod layer_tree_debug_state;
pub mod layer_tree_settings;
pub mod invalidation_region;
pub mod draw_property_utils;
pub mod layer_value_backend;
pub mod layerize_error;
pub mod layerize_family;
pub mod main_callbacks;
pub mod main_effects;
pub mod main_mutator;
pub mod main_paint_config;
pub mod paint_holding;
pub mod property_tree;
pub mod property_tree_services;
pub mod property_types;
pub mod paint;
pub mod protected_sequence_synchronizer;
pub mod proxy;
pub mod push_properties;
// cc/base/rtree and the Skia region backend moved with cc/paint.
pub use cc_paint::{rtree, skia_region};
pub mod scroll_elasticity;
pub mod scroll_elasticity_utils;
pub mod scroll_node;
pub mod skia_filters;
pub mod skia_value_backend;
pub mod surface_range;
pub mod swap_promise;
pub mod ui_resource;
// cc/trees/presentation_time_callback_buffer.h callback types.
pub mod presentation_time_callback;
// gfx::Transform lives in the cc shared root (both halves name it).
pub use compositor_values::transform;
pub mod values;
pub mod view_transition_request;
pub mod viz_metadata;

// cc value modules translated in the shared model packages. Both halves name
// them through this vocabulary.
// Source: cc/base/{math_util,synced_property}.h; cc/trees/{clip_node,
//         effect_node,transform_node,property_ids,viewport_property_ids,
//         target_property,damage_reason,property_animation_state,
//         compositor_commit_data,scroll_source_type}.h;
//         cc/input/{scroll_snap_data,overscroll_behavior,
//         main_thread_scrolling_reason}.h.
pub use compositor_animation::property_animation_state;

/// KeyframeModel installs and removals the Page applies to its main
/// AnimationHost (Blink CompositorAnimation -> cc::Animation::
/// AddKeyframeModel on the main thread; cc/animation/keyframe_model.h,
/// keyframed_animation_curve.h). They reach the impl AnimationHost only
/// through PushPropertiesTo in the commit.
pub mod animation {
    pub use compositor_animation::compositor_mutation::PropertyAnimationMutation;
    pub use compositor_animation::keyframe_model::{
        CompositorAnimationCurve, Direction as AnimationDirection, FillMode as AnimationFillMode,
    };
    pub use compositor_animation::keyframed_animation_curve::{
        KeyframedFloatAnimationCurve, KeyframedTransformAnimationCurve,
        TimingFunction as AnimationTimingFunction,
    };
    pub use compositor_tree::target_property::Type as AnimationTargetProperty;
}
pub use compositor_tree::damage_reason;
pub use compositor_values::{
    browser_controls_params, compositor_commit_data, event_listener_properties, input_types,
    layer_selection_bound, main_thread_scrolling_reason, overscroll_behavior, page_scale_animation,
    scroll_snap_data, scroll_source_type,
};
pub use compositor_tree::{
    clip_node, commit_types, effect_node, math_util, region_backend, property_ids, synced_property, target_property,
    transform_node, viewport_property_ids,
};

pub use commit_state::{CommitStateTypes, CommitTypes, PropertyTreesFor};

// Shared cc/gfx values named by cc/paint on both threads.
// Source: cc/tiles/prioritized_tile.h (ScrollOffsetMap);
//         ui/gfx/display_color_spaces.h (ContentColorUsage).
pub use compositor_values::surface_id::ContentColorUsage;
pub use compositor_values::prioritized_tile::ScrollOffsetMap;

// Main cc::Layer owners and their type families. LayerTreeHostImpl::
// FinishCommit reads them inside the protected sequence
// (TreeSynchronizer::PushLayerProperties -> Layer::PushPropertiesTo), so the
// impl half names them here. Their construction and update stay main-side.
// Source: cc/layers/layer.h; cc/trees/tree_synchronizer.cc.
pub mod layers {
    pub use crate::cc::input::scrollbar;
    pub use crate::cc::layers::*;
}
pub mod layer_types {
    pub use crate::adapter::*;
}
