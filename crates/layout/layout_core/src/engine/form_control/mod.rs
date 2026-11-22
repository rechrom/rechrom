//! Form-control family: platform input-control commands, password
//! caps-lock paint invalidation and WebVTT cue/media-control geometry.
//! `input_control_geometry.rs` (image input sizes on `LayoutNode`) is compiled
//! as a child of `layout_output`. Exposed by `layout::views::form_control`;
//! commands arrive as `LayoutMutation::FormControl`.

pub(crate) mod input_control_platform;
mod input_paint;
mod vtt_geometry;
