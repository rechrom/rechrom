//! Draw-property calculation, damage, occlusion, render surfaces, and quads.
//!
//! This package builds compositor draw decisions but owns no Viz surface or
//! renderer target.

#![allow(non_camel_case_types, non_upper_case_globals)]

pub mod append_quads_context;
pub mod append_quads_data;
pub mod damage_context;
pub mod damage_reason;
pub mod damage_tracker;
pub mod draw_mode;
pub mod draw_properties;
pub mod draw_property_pipeline;
pub mod draw_property_policy;
pub mod draw_property_walk;
pub mod frame_damage;
pub mod initial_render_surface_list;
pub mod metadata_geometry;
pub mod occlusion;
pub mod occlusion_context;
pub mod occlusion_tracker;
pub mod occlusion_walk;
pub mod prepare_to_draw;
pub mod render_pass_build;
pub mod render_pass_pipeline;
pub mod render_pass_pruning;
pub mod render_pass_quad_walk;
pub mod render_surface_content;
pub mod render_surface_policy;
pub mod render_surface_pruning;
pub mod simple_enclosed_region;
pub mod subnormal_float;
pub mod throttle;
pub mod tree_draw_update;

pub type ViewTransitionToken = [u64; 2];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color4f {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawResult {
    kInvalidResult = 0,
    kSuccess = 1,
    kAbortedCheckerboardAnimations = 2,
    kAbortedMissingHighResContent = 3,
    kAbortedCantDraw = 4,
    kAbortedDrainingPipeline = 5,
}

impl DrawResult {
    pub const kMaxValue: Self = Self::kAbortedDrainingPipeline;
}
