use std::sync::Arc;

use layoutng_assembly::fragment_tree::RasterEffectOutset;
use layoutng_assembly::internal::layout_input_types::Color;
use paint_model::display_item_id::DisplayItemId;
use paint_model::paint_property_tree::{PropertyTreeState, TransformPaintPropertyNode};
use paint_model::PaintRect;

/// Blink `PaintChunk` fields. Indices address semantic display items, not
/// flattened replay operations.
#[derive(Clone, PartialEq)]
pub struct PaintChunk {
    pub compositor: Option<Box<PaintChunkCompositorData>>,
    pub id: DisplayItemId,
    pub is_cacheable: bool,
    pub client_is_just_created: bool,
    pub begin_index: u32,
    pub end_index: u32,
    pub bounds: PaintRect,
    pub drawable_bounds: PaintRect,
    pub rect_known_to_be_opaque: PaintRect,
    pub bounds_are_complete: bool,
    pub raster_effect_outset: RasterEffectOutset,
    pub properties: PropertyTreeState,
    pub is_moved_from_cached_subsequence: bool,
    pub effectively_invisible: bool,
}

impl Default for PaintChunk {
    fn default() -> Self {
        Self {
            compositor: None,
            id: DisplayItemId::default(),
            is_cacheable: false,
            client_is_just_created: false,
            begin_index: 0,
            end_index: 0,
            bounds: PaintRect::default(),
            drawable_bounds: PaintRect::default(),
            rect_known_to_be_opaque: PaintRect::default(),
            bounds_are_complete: true,
            raster_effect_outset: RasterEffectOutset::kNone,
            properties: PropertyTreeState::default(),
            is_moved_from_cached_subsequence: false,
            effectively_invisible: false,
        }
    }
}

impl PaintChunk {
    pub fn can_match_old_chunk(&self) -> bool {
        self.is_cacheable && !self.client_is_just_created
    }

    pub fn matches(&self, old: &Self) -> bool {
        old.is_cacheable && self.can_match_old_chunk() && self.id == old.id
    }

    pub fn size(&self) -> u32 {
        self.end_index - self.begin_index
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintChunkCompositorData {
    pub background_color: Color,
    pub background_area: f32,
    pub background_is_solid_color: bool,
    pub has_text: bool,
    pub text_known_to_be_on_opaque_background: bool,
    pub hit_test_opaqueness: PaintHitTestOpaqueness,
    pub hit_test_data: Option<Box<PaintHitTestData>>,
    pub region_capture_data: Option<Box<Vec<([u64; 2], PaintRect)>>>,
    pub tracked_element_rects: Option<Box<Vec<PaintTrackedElementRect>>>,
    pub layer_selection_data: Option<Box<PaintLayerSelectionData>>,
}

impl PaintChunkCompositorData {
    pub fn new() -> Self {
        Self {
            background_color: Color::default(),
            background_area: 0.,
            background_is_solid_color: false,
            has_text: false,
            text_known_to_be_on_opaque_background: true,
            hit_test_opaqueness: PaintHitTestOpaqueness::Transparent,
            hit_test_data: None,
            region_capture_data: None,
            tracked_element_rects: None,
            layer_selection_data: None,
        }
    }
}

impl Default for PaintChunkCompositorData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintChunkBackgroundColorInfo {
    pub background_color: Color,
    pub background_area: f32,
    pub background_is_solid_color: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintHitTestOpaqueness {
    Transparent,
    Mixed,
    Opaque,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintHitTestData {
    pub touch_action_rects: Vec<(PaintRect, u32)>,
    pub wheel_event_rects: Vec<PaintRect>,
    pub xr_regions: Vec<foundation::CompositorElementId>,
    pub scroll_hit_test_rect: PaintRect,
    pub scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
    pub scrolling_contents_cull_rect: PaintRect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintTrackedElementRect {
    pub feature: i32,
    pub rect: PaintRect,
    pub id: [u64; 2],
    pub should_add_to_compositor_frame_metadata: bool,
    pub should_exclude_fixed_and_sticky_occlusions: bool,
    pub frame_token: Option<PaintFrameToken>,
    pub parent_frame_token: Option<PaintFrameToken>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintFrameToken {
    Local([u64; 2]),
    Remote([u64; 2]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintSelectionBoundType {
    Left,
    Right,
    Center,
    Empty,
    Hidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintSelectionBound {
    pub bound_type: PaintSelectionBoundType,
    pub edge_start: [i32; 2],
    pub edge_end: [i32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintLayerSelectionData {
    pub start: Option<PaintSelectionBound>,
    pub end: Option<PaintSelectionBound>,
    pub any_selection_was_painted: bool,
}

pub type PaintChunks = Vec<PaintChunk>;
