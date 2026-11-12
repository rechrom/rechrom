//! Compositor-owned animation identities and animation algorithms.
//!
//! This package is below the `compositor` facade. It must not depend on the
//! facade, Page, Paint, Viz, Renderer, or platform presentation.

#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementListType {
    ACTIVE,
    PENDING,
}

pub mod animation;
pub mod animation_events;
pub mod animation_timeline;
pub mod cubic_bezier;
pub mod gfx_keyframe_effect;
pub mod gfx_keyframe_model;
pub mod host_iteration;
pub mod keyframe_effect;
pub mod keyframe_model;
pub mod paint_worklet;
pub mod property_animation_state;
pub mod scroll_offset_animation_curve;
pub mod scroll_offset_animation_curve_factory;
pub mod scroll_offset_animations;
pub mod scroll_offset_animations_impl;
pub mod tween;

pub mod animation_id_provider {
    pub use scroll_animation::animation_id_provider::*;
}
