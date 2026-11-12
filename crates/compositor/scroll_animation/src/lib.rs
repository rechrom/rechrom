//! Shared source scroll-offset curves and neutral animation time values.
//! No DOM, Layout, Paint, Compositor, scheduler or Page engine dependency.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
pub mod cubic_bezier;
pub mod scroll_offset_animation_curve;
pub mod scroll_offset_animation_curve_factory;
pub mod time;
pub mod tween;
pub use scroll_snap::geometry::{PointF, Vector2dF};
pub use time::{TimeDelta, TimeTicks};

pub mod animation_id_provider;
pub mod compositor_mutation;

pub mod overlay_scrollbar_animator_mac;
