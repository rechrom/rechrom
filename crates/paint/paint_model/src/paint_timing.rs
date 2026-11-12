//! Neutral Paint observation output. Candidate selection, navigation ownership
//! and presentation queues belong to their timing owners above this boundary.
use crate::PaintRect;
#[derive(Clone, Debug, PartialEq)]
pub struct TextPaintObservation {
    pub node_id: u64,
    pub layout_object_identity: u64,
    pub frame_visual_rect: PaintRect,
    pub local_root_visual_rect: PaintRect,
    pub color_is_fully_transparent: bool,
    pub has_text_shadow: bool,
    pub has_text_stroke: bool,
    pub ignore_depth: i32,
    pub document_element_is_invisible: bool,
}
