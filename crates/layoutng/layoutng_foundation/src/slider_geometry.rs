// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/html/forms/slider_thumb_element.cc:105-137.
// One point query over the canonical resident input/track/thumb LayoutBoxes.
// Values keep LayoutUnit precision; this packet retains no layout tree/model.
use crate::{LayoutUnit, PhysicalOffset, PhysicalSize};
#[derive(Clone, Copy)]
pub struct SliderTrackGeometry {
    pub point_in_track: PhysicalOffset,
    pub thumb_offset: PhysicalOffset,
    pub thumb_size: PhysicalSize,
    pub thumb_margin_top: LayoutUnit,
    pub thumb_margin_right: LayoutUnit,
    pub thumb_margin_bottom: LayoutUnit,
    pub thumb_margin_left: LayoutUnit,
    pub track_content_size: PhysicalSize,
    pub horizontal: bool,
    pub flipped_inlines: bool,
}
// Scalar facts read from the same resident host/container ComputedStyle owner.
#[derive(Clone, Copy)]
pub struct SliderStyleFacts {
    pub host_appearance: Option<crate::AppearanceValue>,
    pub host_has_rotate_or_rotate_z: bool,
    pub container_horizontal: Option<bool>,
}
