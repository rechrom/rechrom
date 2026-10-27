//! Native scrollbar thumb geometry projection.
//!
//! This calculation is shared by horizontal and vertical scrollbar payloads
//! and keeps its own fragment-qualified sampling symbol.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:1195-1233
#[inline(never)]
pub(super) fn SetScrollbarThumb(
    axis: &mut ScrollbarPaintAxis,
    visible_length: f64,
    content_length: f64,
    current_offset: f64,
    minimum_offset: f64,
    maximum_offset: f64,
    has_buttons: bool,
    minimum_thumb_length: f64,
) {
    let track_length = if axis.horizontal {
        axis.track_size.width
    } else {
        axis.track_size.height
    };
    let cross_size = if axis.horizontal {
        axis.track_size.height
    } else {
        axis.track_size.width
    };
    axis.has_buttons = has_buttons && track_length >= 2.0 * cross_size + minimum_thumb_length;
    let button_extent = if axis.has_buttons {
        2.0 * cross_size
    } else {
        0.0
    };
    let available = (track_length - button_extent).max(0.0);
    let ratio = if content_length > 0.0 {
        (visible_length / content_length).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let thumb_length = available.min(minimum_thumb_length.max(available * ratio));
    let range = maximum_offset - minimum_offset;
    let position = if range > 0.0 {
        ((current_offset - minimum_offset) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let start = button_extent / 2.0 + (available - thumb_length) * position;
    axis.thumb_offset = axis.track_offset;
    axis.thumb_size = axis.track_size;
    if axis.horizontal {
        axis.thumb_offset.x += start;
        axis.thumb_size.width = thumb_length;
    } else {
        axis.thumb_offset.y += start;
        axis.thumb_size.height = thumb_length;
    }
}
