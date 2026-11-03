use layoutng_assembly::fragment_tree::RasterEffectOutset;
use layoutng_assembly::internal::layout_input_types::Color;
use paint_model::display_item_id::DisplayItemId;
use paint_model::paint_property_tree::PropertyTreeState;
use paint_model::PaintRect;

use crate::{
    PaintHitTestData, PaintLayerSelectionData, PaintRegionCaptureData, PaintTrackedElementRect,
    PaintTrackedElementRects, RegionCaptureDataToString, TrackedElementRectsToString,
};

/// Source: `paint_chunk.h:33`.
pub const K_MIN_BACKGROUND_COLOR_COVERAGE_RATIO: f32 = 0.5;

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
    // Source: paint_chunk.h:103-110.
    pub fn CanMatchOldChunk(&self) -> bool {
        self.is_cacheable && !self.client_is_just_created
    }

    // Source: paint_chunk.h:99-115.
    pub fn Matches(&self, old: &Self) -> bool {
        old.is_cacheable && self.CanMatchOldChunk() && self.id == old.id
    }

    pub fn size(&self) -> u32 {
        debug_assert!(self.end_index >= self.begin_index);
        self.end_index - self.begin_index
    }

    // Source: paint_chunk.h:133-135.
    pub fn DrawsContent(&self) -> bool {
        !self.effectively_invisible && !self.drawable_bounds.is_empty()
    }

    // Source: paint_chunk.h:119-124.
    pub fn EnsureHitTestData(&mut self) -> &mut PaintHitTestData {
        let metadata = self
            .compositor
            .get_or_insert_with(|| Box::new(PaintChunkCompositorData::new()));
        metadata
            .hit_test_data
            .get_or_insert_with(|| Box::new(PaintHitTestData::default()))
    }

    // Source: paint_chunk.h:126-131.
    pub fn EnsureLayerSelectionData(&mut self) -> &mut PaintLayerSelectionData {
        let metadata = self
            .compositor
            .get_or_insert_with(|| Box::new(PaintChunkCompositorData::new()));
        metadata
            .layer_selection_data
            .get_or_insert_with(|| Box::new(PaintLayerSelectionData::default()))
    }

    // Source: paint_chunk.cc:37-54.
    pub fn EqualsForUnderInvalidationChecking(&self, other: &Self) -> bool {
        let self_metadata = self.compositor.as_deref();
        let other_metadata = other.compositor.as_deref();
        self.size() == other.size()
            && self.id == other.id
            && self.properties == other.properties
            && self.bounds == other.bounds
            && self_metadata.and_then(|m| m.hit_test_data.as_deref())
                == other_metadata.and_then(|m| m.hit_test_data.as_deref())
            && self_metadata.and_then(|m| m.region_capture_data.as_deref())
                == other_metadata.and_then(|m| m.region_capture_data.as_deref())
            && self_metadata.and_then(|m| m.tracked_element_rects.as_deref())
                == other_metadata.and_then(|m| m.tracked_element_rects.as_deref())
            && self.drawable_bounds == other.drawable_bounds
            && self.raster_effect_outset == other.raster_effect_outset
            && self_metadata.map_or(PaintHitTestOpaqueness::Transparent, |m| {
                m.hit_test_opaqueness
            }) == other_metadata.map_or(PaintHitTestOpaqueness::Transparent, |m| {
                m.hit_test_opaqueness
            })
            && self.effectively_invisible == other.effectively_invisible
    }

    // Source: paint_chunk.cc:56-76.
    pub fn MemoryUsageInBytes(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();
        let Some(metadata) = self.compositor.as_deref() else {
            return total;
        };
        total += std::mem::size_of::<PaintChunkCompositorData>();
        if let Some(hit_test) = metadata.hit_test_data.as_deref() {
            total += std::mem::size_of::<PaintHitTestData>();
            total +=
                hit_test.touch_action_rects.capacity() * std::mem::size_of::<(PaintRect, u32)>();
            total += hit_test.wheel_event_rects.capacity() * std::mem::size_of::<PaintRect>();
            total += hit_test.xr_regions.capacity()
                * std::mem::size_of::<foundation::CompositorElementId>();
        }
        if let Some(regions) = metadata.region_capture_data.as_deref() {
            total += std::mem::size_of::<Vec<([u64; 2], PaintRect)>>()
                + regions.capacity() * std::mem::size_of::<([u64; 2], PaintRect)>();
        }
        if let Some(rects) = metadata.tracked_element_rects.as_deref() {
            total += std::mem::size_of::<Vec<PaintTrackedElementRect>>()
                + rects.capacity() * std::mem::size_of::<PaintTrackedElementRect>();
        }
        if metadata.layer_selection_data.is_some() {
            total += std::mem::size_of::<PaintLayerSelectionData>();
        }
        total
    }

    // Source: paint_chunk.cc:78-111.
    fn ToStringImpl(&self, id_string: &str, concise: bool) -> String {
        let mut output = format!(
            "PaintChunk({}-{} id={} cacheable={} bounds={:?} from_cache={}",
            self.begin_index,
            self.end_index,
            id_string,
            self.is_cacheable,
            self.bounds,
            self.is_moved_from_cached_subsequence
        );
        if !concise {
            let hit_test_opaqueness = self
                .compositor
                .as_deref()
                .map_or(PaintHitTestOpaqueness::Transparent, |metadata| {
                    metadata.hit_test_opaqueness
                });
            output.push_str(&format!(
                " props=({:?}) rect_known_to_be_opaque={:?} hit_test_opaqueness={:?} effectively_invisible={} drawscontent={}",
                self.properties,
                self.rect_known_to_be_opaque,
                hit_test_opaqueness,
                self.effectively_invisible,
                self.DrawsContent()
            ));
            if let Some(metadata) = self.compositor.as_deref() {
                if let Some(hit_test_data) = metadata.hit_test_data.as_deref() {
                    output.push_str(" hit_test_data=");
                    output.push_str(&hit_test_data.ToString());
                }
                if let Some(region_capture_data) = metadata.region_capture_data.as_deref() {
                    output.push_str(" region_capture_data=");
                    output.push_str(&RegionCaptureDataToString(region_capture_data));
                }
                if let Some(tracked_element_rects) = metadata.tracked_element_rects.as_deref() {
                    output.push_str(" tracked_element_rects=");
                    output.push_str(&TrackedElementRectsToString(tracked_element_rects));
                }
            }
        }
        output.push(')');
        output
    }

    pub fn ToString(&self, concise: bool) -> String {
        self.ToStringImpl(
            &format!(
                "{:#x}:{:?}:{}",
                self.id.client_id, self.id.r#type, self.id.fragment
            ),
            concise,
        )
    }

    // Source: paint_chunk.cc:113-116. PaintArtifact supplies the source
    // client-aware id string without creating a second PaintChunk formatter.
    pub fn ToStringWithId(&self, id_string: &str, concise: bool) -> String {
        self.ToStringImpl(id_string, concise)
    }
}

// Source: paint_chunk.cc:118-120.
impl std::fmt::Display for PaintChunk {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.ToString(false))
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
    pub region_capture_data: Option<Box<PaintRegionCaptureData>>,
    pub tracked_element_rects: Option<Box<PaintTrackedElementRects>>,
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

pub type PaintChunks = Vec<PaintChunk>;
