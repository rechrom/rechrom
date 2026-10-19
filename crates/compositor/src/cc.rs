//! Module assembly for source-relative Chromium translations.
//!
//! A same-stem Chromium `.h` and `.cc` pair maps to one Rust `.rs` file
//! under `cc/<directory>/<stem>.rs`.

pub mod animation {
    pub mod animation {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/animation.rs"
        ));
    }
    pub mod keyframe_effect {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/keyframe_effect.rs"
        ));
    }
    pub mod keyframe_model {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/keyframe_model.rs"
        ));
    }
    pub mod animation_id_provider {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/animation_id_provider.rs"
        ));
    }
    pub mod animation_events {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/animation_events.rs"
        ));
    }
    pub mod animation_timeline {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/animation_timeline.rs"
        ));
    }
    pub mod scroll_offset_animation_curve {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/scroll_offset_animation_curve.rs"
        ));
    }
    pub mod scroll_offset_animation_curve_factory {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/scroll_offset_animation_curve_factory.rs"
        ));
    }
    pub mod scroll_offset_animations_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/animation/scroll_offset_animations_impl.rs"
        ));
    }
}
pub mod paint {
    pub mod display_item_list {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/paint/display_item_list.rs"
        ));
    }
}
pub mod benchmarks {
    pub mod micro_benchmark_controller_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/benchmarks/micro_benchmark_controller_impl.rs"
        ));
    }
}
pub mod debug {
    pub mod debug_colors {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/debug/debug_colors.rs"
        ));
    }
    pub mod layer_tree_debug_state {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/debug/layer_tree_debug_state.rs"
        ));
    }
}
pub mod metrics {
    pub mod compositor_timing_history {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/metrics/compositor_timing_history.rs"
        ));
    }
    pub mod submit_info {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/metrics/submit_info.rs"
        ));
    }
}
pub mod resources {
    pub mod resource_pool {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/resources/resource_pool.rs"
        ));
    }
    pub mod memory_history {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/resources/memory_history.rs"
        ));
    }
}
pub mod raster {
    pub mod task_graph_work_queue {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/raster/task_graph_work_queue.rs"
        ));
    }
    pub mod synchronous_task_graph_runner {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/raster/synchronous_task_graph_runner.rs"
        ));
    }
    pub mod task_graph_runner {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/raster/task_graph_runner.rs"
        ));
    }
    pub mod tile_task {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/raster/tile_task.rs"
        ));
    }
    pub mod task {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/raster/task.rs"
        ));
    }
    pub mod task_category {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/raster/task_category.rs"
        ));
    }
}
pub mod tiles {
    pub mod picture_layer_tiling {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/picture_layer_tiling.rs"
        ));
    }
    pub mod picture_layer_tiling_set {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/picture_layer_tiling_set.rs"
        ));
    }
    pub mod tiling_set_coverage_iterator {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tiling_set_coverage_iterator.rs"
        ));
    }
    pub mod eviction_tile_priority_queue {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/eviction_tile_priority_queue.rs"
        ));
    }
    pub mod raster_tile_priority_queue_all {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/raster_tile_priority_queue_all.rs"
        ));
    }
    pub mod tiling_set_eviction_queue {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tiling_set_eviction_queue.rs"
        ));
    }
    pub mod tiling_set_raster_queue_all {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tiling_set_raster_queue_all.rs"
        ));
    }
    pub mod tiling_set_raster_queue_required {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tiling_set_raster_queue_required.rs"
        ));
    }
    pub mod raster_tile_priority_queue_required {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/raster_tile_priority_queue_required.rs"
        ));
    }
    pub mod raster_tile_priority_queue {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/raster_tile_priority_queue.rs"
        ));
    }
    pub mod tile_task_manager {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tile_task_manager.rs"
        ));
    }
    pub mod prioritized_tile {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/prioritized_tile.rs"
        ));
    }
    pub mod tile {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cc/tiles/tile.rs"));
    }
    pub mod tile_draw_info {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tile_draw_info.rs"
        ));
    }
    pub mod tiles_with_resource_iterator {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tiles_with_resource_iterator.rs"
        ));
    }
    pub mod tile_manager {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tile_manager.rs"
        ));
    }
    pub mod tile_manager_client {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tile_manager_client.rs"
        ));
    }
    pub mod tiling_coverage_iterator {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tiling_coverage_iterator.rs"
        ));
    }
    pub mod tiling_internal {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tiling_internal.rs"
        ));
    }
    pub mod mipmap_util {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/mipmap_util.rs"
        ));
    }
    pub mod tile_index {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tile_index.rs"
        ));
    }
    pub mod tile_priority {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tile_priority.rs"
        ));
    }
    pub mod tile_manager_settings {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/tiles/tile_manager_settings.rs"
        ));
    }
}
pub mod base {
    pub mod rolling_time_delta_history {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/rolling_time_delta_history.rs"
        ));
    }
    pub mod rtree {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cc/base/rtree.rs"));
    }
    pub mod container_util {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/container_util.rs"
        ));
    }
    pub mod base_export {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/base_export.rs"
        ));
    }
    pub mod delayed_unique_notifier {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/delayed_unique_notifier.rs"
        ));
    }
    pub mod invalidation_region {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/invalidation_region.rs"
        ));
    }
    pub mod tiling_data {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/tiling_data.rs"
        ));
    }
    pub mod index_rect {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/index_rect.rs"
        ));
    }
    pub mod math_util {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/math_util.rs"
        ));
    }
    pub mod region {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/region.rs"
        ));
    }
    pub mod reverse_spiral_iterator {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/reverse_spiral_iterator.rs"
        ));
    }
    pub mod spiral_iterator {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/spiral_iterator.rs"
        ));
    }
    pub mod simple_enclosed_region {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/simple_enclosed_region.rs"
        ));
    }
    pub mod synced_property {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/synced_property.rs"
        ));
    }
    pub mod unique_notifier {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/unique_notifier.rs"
        ));
    }
    pub mod protected_sequence_synchronizer {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/base/protected_sequence_synchronizer.rs"
        ));
    }
}
pub mod input {
    pub mod hit_test_opaqueness {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/hit_test_opaqueness.rs"
        ));
    }

    pub mod actively_scrolling_type {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/actively_scrolling_type.rs"
        ));
    }
    pub mod browser_controls_state {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/browser_controls_state.rs"
        ));
    }
    pub mod browser_controls_offset_tag_modifications {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/browser_controls_offset_tag_modifications.rs"
        ));
    }
    pub mod browser_controls_offset_tags {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/browser_controls_offset_tags.rs"
        ));
    }
    pub mod browser_controls_offset_manager {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/browser_controls_offset_manager.rs"
        ));
    }
    pub mod browser_controls_offset_manager_client {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/browser_controls_offset_manager_client.rs"
        ));
    }
    pub mod compositor_input_interfaces {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/compositor_input_interfaces.rs"
        ));
    }
    pub mod event_listener_properties {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/event_listener_properties.rs"
        ));
    }
    pub mod input_handler {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/input_handler.rs"
        ));
    }
    pub mod layer_selection_bound {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/layer_selection_bound.rs"
        ));
    }
    pub mod scroll_elasticity_helper {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scroll_elasticity_helper.rs"
        ));
    }
    pub mod main_thread_scrolling_reason {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/main_thread_scrolling_reason.rs"
        ));
    }
    pub mod overscroll_behavior {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/overscroll_behavior.rs"
        ));
    }
    pub mod page_scale_animation {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/page_scale_animation.rs"
        ));
    }
    pub mod scroll_state_data {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scroll_state_data.rs"
        ));
    }
    pub mod scroll_state {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scroll_state.rs"
        ));
    }
    pub mod scroll_utils {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scroll_utils.rs"
        ));
    }
    pub mod scroll_velocity_tracker {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scroll_velocity_tracker.rs"
        ));
    }
    pub mod scrollbar {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scrollbar.rs"
        ));
    }
    pub mod scrollbar_controller {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scrollbar_controller.rs"
        ));
    }
    pub mod scrollbar_animation_controller {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scrollbar_animation_controller.rs"
        ));
    }
    pub mod single_scrollbar_animation_controller_thinning {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/single_scrollbar_animation_controller_thinning.rs"
        ));
    }
    pub mod scroll_snap_data {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/scroll_snap_data.rs"
        ));
    }
    pub mod snap_selection_strategy {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/snap_selection_strategy.rs"
        ));
    }
    pub mod touch_action {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/input/touch_action.rs"
        ));
    }
}
pub mod layers {
    pub mod render_surface_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/render_surface_impl.rs"
        ));
    }
    pub mod layer {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/layer.rs"
        ));
    }
    pub mod layer_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/layer_impl.rs"
        ));
    }
    pub mod layer_list_iterator {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/layer_list_iterator.rs"
        ));
    }
    pub mod picture_layer {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/picture_layer.rs"
        ));
    }
    pub mod solid_color_layer {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/solid_color_layer.rs"
        ));
    }
    pub mod solid_color_layer_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/solid_color_layer_impl.rs"
        ));
    }
    pub mod scrollbar_layer_base {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/scrollbar_layer_base.rs"
        ));
    }
    pub mod scrollbar_layer_impl_base {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/scrollbar_layer_impl_base.rs"
        ));
    }
    pub mod solid_color_scrollbar_layer {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/solid_color_scrollbar_layer.rs"
        ));
    }
    pub mod solid_color_scrollbar_layer_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/solid_color_scrollbar_layer_impl.rs"
        ));
    }
    pub mod picture_layer_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/picture_layer_impl.rs"
        ));
    }
    pub mod tile_based_layer_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/tile_based_layer_impl.rs"
        ));
    }
    pub mod recording_source {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/recording_source.rs"
        ));
    }
    pub mod layer_collections {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/layer_collections.rs"
        ));
    }
    pub mod scroll_hit_test_rect {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/scroll_hit_test_rect.rs"
        ));
    }
    pub mod touch_action_region {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/touch_action_region.rs"
        ));
    }
    pub mod draw_properties {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/draw_properties.rs"
        ));
    }
    pub mod content_layer_client {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/content_layer_client.rs"
        ));
    }
    pub mod append_quads_context {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/append_quads_context.rs"
        ));
    }
    pub mod append_quads_data {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/append_quads_data.rs"
        ));
    }
    pub mod draw_mode {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/draw_mode.rs"
        ));
    }
    pub mod effect_tree_layer_list_iterator {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/effect_tree_layer_list_iterator.rs"
        ));
    }
    pub mod viewport {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/viewport.rs"
        ));
    }
    pub mod tile_size_calculator {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/layers/tile_size_calculator.rs"
        ));
    }
}
pub mod scheduler {
    pub mod commit_earlyout_reason {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/commit_earlyout_reason.rs"
        ));
    }
    pub mod draw_result {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/draw_result.rs"
        ));
    }
    pub mod scheduler_settings {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/scheduler_settings.rs"
        ));
    }
    pub mod begin_frame_tracker {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/begin_frame_tracker.rs"
        ));
    }
    pub mod scheduler_state_machine {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/scheduler_state_machine.rs"
        ));
    }
    pub mod scheduler {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/scheduler.rs"
        ));
    }
    pub mod headless_scheduler_state_machine {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/headless_scheduler_state_machine.rs"
        ));
    }
    pub mod slim_scheduler_state_machine {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/slim_scheduler_state_machine.rs"
        ));
    }
    pub mod video_frame_controller {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/video_frame_controller.rs"
        ));
    }
    pub mod webview_scheduler_state_machine {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/scheduler/webview_scheduler_state_machine.rs"
        ));
    }
}
pub mod trees {
    pub mod animated_paint_worklet_tracker {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/animated_paint_worklet_tracker.rs"
        ));
    }
    pub mod begin_main_frame_trace_id {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/begin_main_frame_trace_id.rs"
        ));
    }
    pub mod browser_controls_params {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/browser_controls_params.rs"
        ));
    }
    pub mod clip_node {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/clip_node.rs"
        ));
    }
    pub mod compositor_commit_data {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/compositor_commit_data.rs"
        ));
    }
    pub mod commit_state {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/commit_state.rs"
        ));
    }
    pub mod debug_rect_history {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/debug_rect_history.rs"
        ));
    }
    pub mod layer_tree_settings {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/layer_tree_settings.rs"
        ));
    }
    pub mod image_animation_controller {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/image_animation_controller.rs"
        ));
    }
    pub mod layer_tree_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/layer_tree_impl.rs"
        ));
    }
    pub mod client_layer_tree_host_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/client_layer_tree_host_impl.rs"
        ));
    }
    pub mod layer_tree_host_impl {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/layer_tree_host_impl.rs"
        ));
    }
    pub mod damage_tracker {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/damage_tracker.rs"
        ));
    }
    pub mod draw_property_utils {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/draw_property_utils.rs"
        ));
    }
    pub mod property_animation_state {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/property_animation_state.rs"
        ));
    }
    pub mod presentation_time_callback_buffer {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/presentation_time_callback_buffer.rs"
        ));
    }
    pub mod property_tree {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/property_tree.rs"
        ));
    }
    pub mod property_ids {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/property_ids.rs"
        ));
    }
    pub mod damage_reason {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/damage_reason.rs"
        ));
    }
    pub mod effect_node {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/effect_node.rs"
        ));
    }
    pub mod frame_data {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/frame_data.rs"
        ));
    }
    pub mod managed_memory_policy {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/managed_memory_policy.rs"
        ));
    }
    pub mod mobile_optimized_viewport_util {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/mobile_optimized_viewport_util.rs"
        ));
    }
    pub mod occlusion {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/occlusion.rs"
        ));
    }
    pub mod occlusion_tracker {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/occlusion_tracker.rs"
        ));
    }
    pub mod raster_capabilities {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/raster_capabilities.rs"
        ));
    }
    pub mod render_frame_metadata {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/render_frame_metadata.rs"
        ));
    }
    pub mod scroll_source_type {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/scroll_source_type.rs"
        ));
    }
    pub mod sticky_position_constraint {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/sticky_position_constraint.rs"
        ));
    }
    pub mod swap_promise {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/swap_promise.rs"
        ));
    }
    pub mod scroll_node {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/scroll_node.rs"
        ));
    }
    pub mod scroll_elasticity_utils {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/scroll_elasticity_utils.rs"
        ));
    }
    pub mod target_property {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/target_property.rs"
        ));
    }
    pub mod throttle_decider {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/throttle_decider.rs"
        ));
    }
    pub mod tree_synchronizer {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/tree_synchronizer.rs"
        ));
    }
    pub mod unbounded_frame_sink_handler {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/unbounded_frame_sink_handler.rs"
        ));
    }
    pub mod transform_node {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/transform_node.rs"
        ));
    }
    pub mod viewport_property_ids {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/viewport_property_ids.rs"
        ));
    }
    pub mod proxy_common {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cc/trees/proxy_common.rs"
        ));
    }
}
