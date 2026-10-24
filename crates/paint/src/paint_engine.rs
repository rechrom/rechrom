#![allow(non_camel_case_types)]

//! Source paint display-list ABI. The entry point and private painters are
//! connected after their owning source files have been translated.

use std::{collections::HashMap, sync::Arc};

pub use layoutng_assembly::fragment_tree::RasterEffectOutset;
use layoutng_assembly::fragment_tree::{PaintGlyph, PaintResources};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, FontSmoothing, FontVariation, Offset, PaintPathCommand, Size,
    TextDecorationStyle, TransformMatrix, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    BackgroundRepeatRule, InterpolationQuality, PaintBlendMode, PaintImageTileRule,
};

pub use layoutng_assembly::internal::paint_input::{
    PaintCornerRadii, PaintFilterOperation, PaintMaskComposite, PaintMaskMode, PaintShader,
    SvgPatternCommand, SvgStrokeLineCap, SvgStrokeLineJoin,
};

pub(crate) fn ResolveDocumentImageContainerKey(
    image: &layoutng_assembly::internal::layout_input::PaintImage,
    target_size: Size,
    effective_zoom: f64,
    resources: Option<&PaintResources>,
    embedding_color_scheme: Option<u8>,
) -> Option<image_resource::ContainerKey> {
    let image_resource::PaintImageContent::Document(content) = &image.content else {
        return None;
    };
    let zoom = effective_zoom.max(f64::MIN_POSITIVE);
    let width = (target_size.width / zoom).round().max(1.0) as u32;
    let height = (target_size.height / zoom).round().max(1.0) as u32;
    let mut key = image_resource::ContainerKey::new(
        width,
        height,
        zoom,
        resources.map_or(1.0, |resources| resources.device_pixel_ratio),
    );
    // StyleFetchedImage::GetImage / StyleEngine::ResolveColorSchemeForEmbedding:
    // normal uses the embedding document preference; an authored color-scheme
    // uses the computed used scheme. Resolve before recording resource identity,
    // so Page publishes exactly the same key consumed by raster.
    key.preferred_color_scheme = embedding_color_scheme
        .unwrap_or_else(|| resources.map_or(0, |resources| resources.preferred_color_scheme));
    key.preferred_color_scheme_is_explicit = true;
    if let Some(current) = &image.document_container_key {
        key.view_specification = current.view_specification.clone();
    } else if let Some(initial) = content.variants().first() {
        key.view_specification = initial.container_key.view_specification.clone();
    }
    Some(key)
}

// cpp: paint/paint_engine.h:15-31
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaintPhase {
    kBlockBackground,
    kSelfBlockBackgroundOnly,
    kDescendantBlockBackgroundsOnly,
    kForcedColorsModeBackplate,
    kFloat,
    kForeground,
    kOutline,
    kSelfOutlineOnly,
    kDescendantOutlinesOnly,
    kOverlayOverflowControls,
    kSelectionDragImage,
    kTextClip,
    kMask,
}

// cpp: paint/paint_engine.h:33-68
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DisplayItemType {
    kSave,
    kRestore,
    kConcat,
    kClipRect,
    kClipRoundedRect,
    kClipOutRoundedRect,
    kClipPath,
    kClipOutRect,
    kSaveLayer,
    kSaveLayerAlpha,
    kSaveLayerBlend,
    kSaveLayerFilter,
    /// Internal Mask effect: isolated source restored with SkBlendMode::kDstIn.
    kSaveLayerDstIn,
    kBeginMask,
    kEndMask,
    kDrawRect,
    /// Mask-image layers draw the mask source plane, not the masked contents.
    kDrawMask,
    kDrawRoundedRect,
    kDrawDoubleRoundedRect,
    kDrawEllipse,
    kStrokeEllipse,
    kDrawPath,
    kStrokePath,
    kDrawGradientRect,
    kDrawTiledGradient,
    kStrokeRect,
    kStrokeLine,
    kStrokeWavyLine,
    kDrawGlyphRun,
    kDrawImageRect,
    kDrawTiledImage,
    kDrawBoxShadow,
    kDrawScrollbarTrack,
    kDrawScrollbarThumb,
    kDrawScrollbarButton,
    kDrawScrollbarCorner,
}

// cpp: paint/paint_engine.h:70-76
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl PaintRect {
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    // gfx::ToEnclosingRect(): semantic DisplayItem rectangles are integer
    // bounds even though the replay ABI retains floating-point geometry.
    pub fn enclosing(self) -> Self {
        let x = self.x.floor();
        let y = self.y.floor();
        Self {
            x,
            y,
            width: if self.width <= 0.0 {
                0.0
            } else {
                (self.x + self.width).ceil() - x
            },
            height: if self.height <= 0.0 {
                0.0
            } else {
                (self.y + self.height).ceil() - y
            },
        }
    }

    // gfx::ToEnclosedRect: only integer cells wholly inside the source rect
    // can be proved opaque. Invalid geometry supplies no such proof.
    pub fn enclosed(self) -> Self {
        let right = self.x + self.width;
        let bottom = self.y + self.height;
        if self.is_empty()
            || ![self.x, self.y, right, bottom]
                .iter()
                .all(|v| v.is_finite())
        {
            return Self::default();
        }
        let x = self.x.ceil();
        let y = self.y.ceil();
        Self {
            x,
            y,
            width: (right.floor() - x).max(0.0),
            height: (bottom.floor() - y).max(0.0),
        }
    }

    pub fn intersection(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        Self {
            x,
            y,
            width: (right - x).max(0.0),
            height: (bottom - y).max(0.0),
        }
    }

    // ui/gfx/geometry/rect.cc:386-417. A bounding union could include a hole.
    // Only either input or an expanded inclusive intersection is fully covered.
    pub fn maximum_covered_rect(a: Self, b: Self) -> Self {
        if a.is_empty() {
            return b;
        }
        if b.is_empty() {
            return a;
        }
        let area = |r: Self| r.width * r.height;
        let mut maximum = a;
        let mut maximum_area = area(a);
        if area(b) > maximum_area {
            maximum = b;
            maximum_area = area(b);
        }
        let x = a.x.max(b.x);
        let y = a.y.max(b.y);
        let right = (a.x + a.width).min(b.x + b.width);
        let bottom = (a.y + a.height).min(b.y + b.height);
        // InclusiveIntersect admits a shared edge, but excludes a shared point.
        if right >= x && bottom >= y && (right > x || bottom > y) {
            let top = a.y.min(b.y);
            let vertical = Self {
                x,
                y: top,
                width: right - x,
                height: (a.y + a.height).max(b.y + b.height) - top,
            };
            if area(vertical) > maximum_area {
                maximum = vertical;
                maximum_area = area(vertical);
            }
            let left = a.x.min(b.x);
            let horizontal = Self {
                x: left,
                y,
                width: (a.x + a.width).max(b.x + b.width) - left,
                height: bottom - y,
            };
            if area(horizontal) > maximum_area {
                maximum = horizontal;
            }
        }
        maximum
    }

    // gfx::Rect::Union ignores empty rects, unlike an origin-based min/max.
    pub fn union(&mut self, other: Self) {
        if other.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = other;
            return;
        }
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);
        self.x = self.x.min(other.x);
        self.y = self.y.min(other.y);
        self.width = right - self.x;
        self.height = bottom - self.y;
    }
}

// cpp: paint/paint_engine.h:82-100
#[derive(Clone, PartialEq)]
pub struct DisplayMaskLayer {
    pub source_error: Option<String>,
    pub resource_id: u64,
    pub image_interpolation_quality: InterpolationQuality,
    pub document_image_container_key: Option<image_resource::ContainerKey>,
    pub paint_shader: Option<PaintShader>,
    pub clip_rect: PaintRect,
    pub clip_radii: PaintCornerRadii,
    pub source_rect: PaintRect,
    pub tile_rect: PaintRect,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub repeat_rule_x: BackgroundRepeatRule,
    pub repeat_rule_y: BackgroundRepeatRule,
    pub tile_scale: Offset,
    pub tile_spacing: Size,
    pub mode: PaintMaskMode,
    pub composite: PaintMaskComposite,
}

impl Default for DisplayMaskLayer {
    fn default() -> Self {
        Self {
            source_error: None,
            resource_id: 0,
            image_interpolation_quality: InterpolationQuality::kInterpolationMedium,
            document_image_container_key: None,
            paint_shader: None,
            clip_rect: PaintRect::default(),
            clip_radii: PaintCornerRadii::default(),
            source_rect: PaintRect::default(),
            tile_rect: PaintRect::default(),
            repeat_x: false,
            repeat_y: false,
            repeat_rule_x: BackgroundRepeatRule::kNoRepeat,
            repeat_rule_y: BackgroundRepeatRule::kNoRepeat,
            tile_scale: Offset { x: 1.0, y: 1.0 },
            tile_spacing: Size::default(),
            mode: PaintMaskMode::kAlpha,
            composite: PaintMaskComposite::kAdd,
        }
    }
}

// cpp: paint/paint_engine.h:102-203
#[derive(Clone, PartialEq)]
pub struct DisplayItem {
    pub r#type: DisplayItemType,
    pub phase: PaintPhase,
    pub node_id: u64,
    pub fragment_instance_id: u64,
    pub rect: PaintRect,
    /// DrawTextBlobOp's retained blob bounds, independent of decoration ink.
    /// None keeps unsupported text geometry conservative.
    pub glyph_ink_rect: Option<PaintRect>,
    pub source_rect: PaintRect,
    pub inner_rect: PaintRect,
    pub tile_rect: PaintRect,
    pub color: Color,
    pub line_style: BorderLineStyle,
    pub decoration_style: TextDecorationStyle,
    pub stroke_width: f64,
    pub corner_radius: f64,
    pub inner_corner_radius: f64,
    pub corner_radii: PaintCornerRadii,
    pub inner_corner_radii: PaintCornerRadii,
    pub dash_intervals: Vec<f64>,
    pub path: Vec<PaintPathCommand>,
    pub even_odd: bool,
    pub inverse_winding: bool,
    pub svg_line_cap: SvgStrokeLineCap,
    pub svg_line_join: SvgStrokeLineJoin,
    pub dash_offset: f64,
    pub dash_fit_thickness: f64,
    pub miter_limit: f64,
    pub antialias: bool,
    pub non_scaling_stroke: bool,
    pub paint_shader: Option<PaintShader>,
    pub blend_mode: PaintBlendMode,
    pub svg_marker_resource_id: u64,
    pub filters: Vec<PaintFilterOperation>,
    pub mask_layers: Vec<DisplayMaskLayer>,
    pub round_cap: bool,
    pub resource_id: u64,
    pub image_interpolation_quality: InterpolationQuality,
    /// Effective orientation for this image use after CSS image-orientation.
    /// Source rectangles are expressed in the oriented image coordinate space.
    pub image_orientation: image_resource::ImageOrientation,
    pub document_image_container_key: Option<image_resource::ContainerKey>,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub background_repeat_x: BackgroundRepeatRule,
    pub background_repeat_y: BackgroundRepeatRule,
    pub tile_rule_x: PaintImageTileRule,
    pub tile_rule_y: PaintImageTileRule,
    pub tile_scale: Offset,
    pub tile_phase: Offset,
    pub tile_spacing: Size,
    pub shadow_offset: Offset,
    pub blur_radius: f64,
    pub spread: f64,
    pub inset: bool,
    pub is_shadow: bool,
    pub shadow_has_opaque_background: bool,
    pub is_text_decoration: bool,
    pub stroke_glyphs: bool,
    pub skip_ink: bool,
    pub opacity: f32,
    pub transform: TransformMatrix,
    pub font_face_index: u32,
    pub font_variations: Vec<FontVariation>,
    pub font_size: f64,
    pub baseline: f64,
    pub text_line_top: f64,
    pub text_line_height: f64,
    pub text_blob_origin: Offset,
    pub horizontal: bool,
    pub at_start: bool,
    pub rtl: bool,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
    pub font_smoothing: FontSmoothing,
    pub writing_mode: WritingMode,
    /// SkTextBlob-like immutable glyph payload. Cached DrawingDisplayItems
    /// share it across paint artifacts instead of cloning every glyph during
    /// PaintController commit.
    pub glyphs: Arc<Vec<PaintGlyph>>,
}

impl Default for DisplayItem {
    fn default() -> Self {
        Self {
            r#type: DisplayItemType::kDrawRect,
            phase: PaintPhase::kForeground,
            node_id: 0,
            fragment_instance_id: 0,
            rect: PaintRect::default(),
            glyph_ink_rect: None,
            source_rect: PaintRect::default(),
            inner_rect: PaintRect::default(),
            tile_rect: PaintRect::default(),
            color: Color::default(),
            line_style: BorderLineStyle::kNone,
            decoration_style: TextDecorationStyle::kSolid,
            stroke_width: 0.0,
            corner_radius: 0.0,
            inner_corner_radius: 0.0,
            corner_radii: PaintCornerRadii::default(),
            inner_corner_radii: PaintCornerRadii::default(),
            dash_intervals: Vec::new(),
            path: Vec::new(),
            even_odd: false,
            inverse_winding: false,
            svg_line_cap: SvgStrokeLineCap::kButt,
            svg_line_join: SvgStrokeLineJoin::kMiter,
            dash_offset: 0.0,
            dash_fit_thickness: 0.0,
            miter_limit: 4.0,
            antialias: true,
            non_scaling_stroke: false,
            paint_shader: None,
            blend_mode: PaintBlendMode::kNormal,
            svg_marker_resource_id: 0,
            filters: Vec::new(),
            mask_layers: Vec::new(),
            round_cap: false,
            resource_id: 0,
            image_interpolation_quality: InterpolationQuality::kInterpolationMedium,
            image_orientation: Default::default(),
            document_image_container_key: None,
            repeat_x: false,
            repeat_y: false,
            background_repeat_x: BackgroundRepeatRule::kNoRepeat,
            background_repeat_y: BackgroundRepeatRule::kNoRepeat,
            tile_rule_x: PaintImageTileRule::kStretch,
            tile_rule_y: PaintImageTileRule::kStretch,
            tile_scale: Offset { x: 1.0, y: 1.0 },
            tile_phase: Offset::default(),
            tile_spacing: Size::default(),
            shadow_offset: Offset::default(),
            blur_radius: 0.0,
            spread: 0.0,
            inset: false,
            is_shadow: false,
            shadow_has_opaque_background: false,
            is_text_decoration: false,
            stroke_glyphs: false,
            skip_ink: false,
            opacity: 1.0,
            transform: TransformMatrix::default(),
            font_face_index: 0,
            font_variations: Vec::new(),
            font_size: 16.0,
            baseline: 0.0,
            text_line_top: 0.0,
            text_line_height: 0.0,
            text_blob_origin: Offset::default(),
            horizontal: true,
            at_start: false,
            rtl: false,
            synthetic_bold: false,
            synthetic_italic: false,
            font_smoothing: FontSmoothing::kAuto,
            writing_mode: WritingMode::kHorizontalTb,
            glyphs: Arc::new(Vec::new()),
        }
    }
}

// Public producer analysis also serves canonical PaintRecords downstream.
pub use crate::paint_context::CalculateRectKnownToBeOpaqueForRecord;
pub use image_resource::{
    downcast_document_paint_record, ContainerKey as DocumentImageContainerKey, PaintImageContent,
    PaintImageId as NativePaintImageId, PaintImageMetadataUnavailable as ImageMetadataUnavailable,
};
pub use layoutng_assembly::internal::layout_input::PaintImage as NativePaintImage;

// Blink paint_chunk.h uses DisplayItem::Id for chunk identity.
pub use crate::display_item_id::DisplayItemId as PaintChunkId;

/// Core Blink PaintChunk fields. Indices address `display_items`, not replay ops.
#[derive(Clone, PartialEq)]
pub struct PaintChunk {
    /// The native derived-data owner. None is an explicit producer capability
    /// gap, not an empty/default hit-test or text proof. Compositor rejects it.
    pub compositor: Option<Box<PaintChunkCompositorData>>,
    pub id: PaintChunkId,
    pub is_cacheable: bool,
    pub client_is_just_created: bool,
    pub begin_index: u32,
    pub end_index: u32,
    // paint_chunk.h: bounds in this chunk's property-tree coordinate space.
    // Ancestor transform/clip and filter output expansion happen downstream.
    pub bounds: PaintRect,
    pub drawable_bounds: PaintRect,
    // DrawingDisplayItem's bounded PaintRecord alpha proof, in the same flat
    // recorded coordinate basis as VisualRect/drawable_bounds. Ancestor clips,
    // transforms and effects are applied separately by property consumers.
    pub rect_known_to_be_opaque: PaintRect,
    // An adapter diagnostic for branches whose native ink-overflow lifecycle
    // is still pending. Such bounds must not be used to reject a paint region.
    pub bounds_are_complete: bool,
    pub raster_effect_outset: RasterEffectOutset,
    pub properties: crate::paint_property_tree::PropertyTreeState,
    pub is_moved_from_cached_subsequence: bool,
    pub effectively_invisible: bool,
}

impl Default for PaintChunk {
    fn default() -> Self {
        Self {
            compositor: None,
            id: PaintChunkId::default(),
            is_cacheable: false,
            client_is_just_created: false,
            begin_index: 0,
            end_index: 0,
            bounds: PaintRect::default(),
            drawable_bounds: PaintRect::default(),
            rect_known_to_be_opaque: PaintRect::default(),
            bounds_are_complete: true,
            raster_effect_outset: RasterEffectOutset::kNone,
            properties: crate::paint_property_tree::PropertyTreeState::default(),
            is_moved_from_cached_subsequence: false,
            effectively_invisible: false,
        }
    }
}

/// Source: paint_chunk.h:168-221; drawing_display_item.h:39-43.
/// These values are produced alongside the record, never reconstructed by a
/// downstream compositor from a flattened command list.
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
// Source: drawing_display_item.h:39-43.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintChunkBackgroundColorInfo {
    pub background_color: Color,
    pub background_area: f32,
    pub background_is_solid_color: bool,
}
impl PaintChunkCompositorData {
    /// Source: paint_chunk.h:45-61,168-221. Fresh chunks own these exact
    /// initial values; PaintChunker updates derived fields while recording.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintHitTestOpaqueness {
    Transparent,
    Mixed,
    Opaque,
}
/// Source: hit_test_data.h:24-62. Cross-links retain the actual Paint node.
#[derive(Clone, Debug, PartialEq)]
pub struct PaintHitTestData {
    pub touch_action_rects: Vec<(PaintRect, u32)>,
    pub wheel_event_rects: Vec<PaintRect>,
    pub xr_regions: Vec<foundation::CompositorElementId>,
    pub scroll_hit_test_rect: PaintRect,
    pub scroll_translation: Option<Arc<crate::paint_property_tree::TransformPaintPropertyNode>>,
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

impl PaintChunk {
    // Blink paint_chunk.h: Matches(), CanMatchOldChunk().
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordedDisplayItemKind {
    Drawing,
    Scrollbar,
}

/// Source-owned inputs retained by ScrollbarDisplayItem until PAC chooses
/// composited or non-composited painting. The cc layer is created only at the
/// compositor boundary; Paint retains geometry and identity, not a cc owner.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarDisplayItemData {
    pub element_id: foundation::CompositorElementId,
    pub track_rect: PaintRect,
    pub thumb_rect: PaintRect,
    pub thumb_color: Color,
    pub horizontal: bool,
    pub overlay: bool,
    pub left_side_vertical: bool,
    pub minimum_thumb_length: i32,
    pub needs_update_display: bool,
    pub hit_test_opaqueness: PaintHitTestOpaqueness,
}

/// A semantic item containing a PaintRecord range in the flat replay ABI.
/// Save/restore and multiple draw ops inside a recorder count as one item.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedDisplayItem {
    // Source: display_item.h:260-265; paint_controller.cc:531-545.
    pub is_cacheable: bool,
    pub is_tombstone: bool,
    pub paint_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    /// Revision of the LayoutNG fragment inputs that produced this record.
    /// A drawing cache hit requires the same nonzero certified revision.
    pub source_revision: u64,
    /// Immutable content versions of image resources sampled by this record.
    ///
    /// Blink normally carries an image client's invalidation through the next
    /// PaintController transaction. Rechrom also permits an older immutable
    /// PaintArtifact to be presented again, so a one-shot dirty bit is not a
    /// sufficient proof. Keeping the exact `(resource id, revision)` set on
    /// the record lets RasterInvalidator compare any two retained artifacts in
    /// either direction without retaining Paint or Page state.
    pub image_content_revisions: Vec<(image_resource::ImageId, u64)>,
    pub scrollbar_needs_update_display: Option<bool>,
    pub scrollbar: Option<ScrollbarDisplayItemData>,
    pub kind: RecordedDisplayItemKind,
    pub id: crate::display_item_id::DisplayItemId,
    pub visual_rect: PaintRect,
    pub visual_rect_is_accurate: bool,
    pub draws_content: bool,
    pub raster_effect_outset: RasterEffectOutset,
    pub record_begin: usize,
    pub record_end: usize,
    pub scroll_translation: Option<Arc<crate::paint_property_tree::TransformPaintPropertyNode>>,
}

pub use layoutng_assembly::caret::CaretPaintState as CaretPosition;
pub use layoutng_assembly::editing_state::EditingPaintState;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretGeometry {
    pub node_id: u64,
    pub rect: PaintRect,
    pub visible: bool,
}

/// Drawing display items, each selecting its recorded drawing operations.
pub type DisplayItemList = Vec<RecordedDisplayItem>;
pub type PaintChunks = Vec<PaintChunk>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImagePaintDependency {
    pub record_index: usize,
    pub visual_rect: PaintRect,
}

/// Built while recording paint. Resource-only updates use this retained index
/// instead of walking the outer Fragment tree to rediscover image clients.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageDependencyIndex {
    by_image: HashMap<image_resource::ImageId, Vec<ImagePaintDependency>>,
}

impl ImageDependencyIndex {
    /// Borrows the existing producer dependency identities. No image catalog
    /// or command list is copied for compositor input validation.
    pub fn image_ids(&self) -> impl Iterator<Item = image_resource::ImageId> + '_ {
        self.by_image.keys().copied()
    }
    pub fn dependencies(&self, image_id: image_resource::ImageId) -> &[ImagePaintDependency] {
        self.by_image.get(&image_id).map_or(&[], Vec::as_slice)
    }

    fn build(
        items: &[DisplayItem],
        records: &[RecordedDisplayItem],
        chunks: &[PaintChunk],
    ) -> Self {
        let mut by_image: HashMap<_, Vec<_>> = HashMap::new();
        for (record_index, record) in records.iter().enumerate() {
            let mut ids = std::collections::HashSet::new();
            for item in &items[record.record_begin..record.record_end] {
                if item.resource_id != 0 {
                    ids.insert(item.resource_id);
                }
                if let Some(shader) = &item.paint_shader {
                    if shader.resource_id != 0 {
                        ids.insert(shader.resource_id);
                    }
                }
                for mask in &item.mask_layers {
                    if mask.resource_id != 0 {
                        ids.insert(mask.resource_id);
                    }
                    if let Some(shader) = &mask.paint_shader {
                        if shader.resource_id != 0 {
                            ids.insert(shader.resource_id);
                        }
                    }
                }
            }
            if let Some(chunk) = chunks.iter().find(|chunk| {
                chunk.begin_index as usize <= record_index
                    && record_index < chunk.end_index as usize
            }) {
                let mut effect = Some(chunk.properties.effect.as_ref());
                while let Some(node) = effect {
                    for filter in [&node.filters, &node.backdrop_filters] {
                        for layer in filter.reflection_mask_layers() {
                            if let Some(image) = &layer.image {
                                if image.id != 0 {
                                    ids.insert(image.id);
                                }
                            }
                            if let Some(shader) = &layer.paint_shader {
                                if shader.resource_id != 0 {
                                    ids.insert(shader.resource_id);
                                }
                            }
                        }
                    }
                    effect = node.parent.as_deref();
                }
            }
            for image_id in ids {
                by_image
                    .entry(image_id)
                    .or_default()
                    .push(ImagePaintDependency {
                        record_index,
                        visual_rect: record.visual_rect,
                    });
            }
        }
        Self { by_image }
    }
}

/// Concrete immutable record emitted by document_image and consumed only by
/// raster replay. It contains no retained DOM, layout engine or timeline.
#[derive(Clone)]
pub struct DocumentPaintArtifactRecord {
    pub artifact: Arc<PaintArtifact>,
    pub intrinsic_size: image_resource::IntrinsicSize,
    /// Coordinate space used by the nested recording. Responsive SVG may
    /// paint at a container size different from its intrinsic dimensions.
    pub record_size: image_resource::IntrinsicSize,
    pub container_key: image_resource::ContainerKey,
}

#[derive(Clone)]
pub enum PaintMutation {
    ImageChanged {
        image_id: image_resource::ImageId,
        revision: u64,
        frame: Arc<image_resource::DocumentImageFrame>,
    },
}

/// Resource-only changes applied to an already committed artifact. The Page
/// chooses which update entered the pipeline; PaintEngine owns immutable
/// artifact replacement and validation.
pub enum PaintResourceUpdate {
    Snapshot(Arc<PaintResources>),
    Mutations(Vec<PaintMutation>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintMutationResult {
    pub damaged_records: Vec<ImagePaintDependency>,
}

/// One completed PaintEngine publication. The artifact is immutable; the
/// client receipt is opaque and can only be consumed through Layout's commit
/// interface. Neither value contains a copied fragment tree.
pub struct PaintOutput {
    artifact: Arc<PaintArtifact>,
    layout_commit: layoutng_assembly::layout_engine::PaintClientCommit,
}

impl layoutng_assembly::layout_engine::PaintCommitOutput for PaintOutput {
    type Artifact = Arc<PaintArtifact>;

    fn IntoLayoutCommit(
        self,
    ) -> (
        Self::Artifact,
        layoutng_assembly::layout_engine::PaintClientCommit,
    ) {
        (self.artifact, self.layout_commit)
    }
}

// cpp: paint/paint_engine.h:227-239
#[derive(Clone, Default)]
pub struct PaintArtifact {
    /// Source viewport identities published by Paint. None is missing producer
    /// input; the compositor must not invent a visual/layout viewport mode.
    pub compositor_viewport: Option<PaintCompositorViewport>,
    /// Source: paint_artifact.cc:44-49. Captured from the original Paint client
    /// together with its debug name at recording time.
    pub client_owner_node_ids: std::collections::HashMap<usize, i32>,
    // Source: paint_artifact.h:62-68,81-87; paint_artifact.cc:29-41.
    pub client_debug_names: std::collections::HashMap<usize, std::sync::Arc<str>>,
    /// Immutable PaintRecord operations shared by published artifacts. Property
    /// updates retain this backing; an actual operation edit uses Arc::make_mut
    /// to preserve old frame snapshots (Chromium PaintRecord sharing intent).
    pub items: Arc<Vec<DisplayItem>>,
    pub display_items: DisplayItemList,
    pub chunks: PaintChunks,
    /// Property snapshots when the flat records were generated. Scroll-only
    /// updates change chunk properties without changing this coordinate basis.
    pub recorded_properties: Vec<crate::paint_property_tree::PropertyTreeState>,
    /// Identity of the immutable display-item recording and its recording
    /// coordinate basis. A retained property-only scroll keeps this value;
    /// recording or changing any draw operation assigns a new value.
    ///
    /// Zero is reserved for externally assembled/test artifacts, for which
    /// downstream caches must retain their complete value proof.
    pub recording_revision: u64,
    pub resources: Option<Arc<PaintResources>>,
    pub image_dependencies: Arc<ImageDependencyIndex>,
    pub caret: Option<CaretGeometry>,
}

/// Source: third_party/blink/renderer/core/frame/local_frame_view.cc:3480-3510.
/// The current LayoutView producer has no active VisualViewport. In this mode
/// the five VisualViewport compositor properties are absent; the actual layout
/// scroll translation still appears in the full native scroll-node list.
#[derive(Clone, Debug, PartialEq)]
pub enum PaintCompositorViewportMode {
    SingleLayoutViewport,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PaintCompositorViewport {
    pub mode: PaintCompositorViewportMode,
    pub viewport_clip: Arc<crate::paint_property_tree::ClipPaintPropertyNode>,
    pub layout_scroll_translation:
        Option<Arc<crate::paint_property_tree::TransformPaintPropertyNode>>,
    pub scroll_translation_nodes: Vec<Arc<crate::paint_property_tree::TransformPaintPropertyNode>>,
}

impl PartialEq for PaintArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.caret == other.caret
            && self.compositor_viewport == other.compositor_viewport
            && self.client_owner_node_ids == other.client_owner_node_ids
            && self.client_debug_names == other.client_debug_names
            && (Arc::ptr_eq(&self.items, &other.items) || self.items == other.items)
            && self.display_items == other.display_items
            && self.image_dependencies == other.image_dependencies
            && self.chunks == other.chunks
            && self
                .chunks
                .iter()
                .zip(&other.chunks)
                .all(|(a, b)| a.properties.same_values(&b.properties))
            && self
                .chunks
                .iter()
                .zip(&other.chunks)
                .enumerate()
                .all(|(index, (a, b))| {
                    self.recorded_properties
                        .get(index)
                        .unwrap_or(&a.properties)
                        .same_values(
                            other
                                .recorded_properties
                                .get(index)
                                .unwrap_or(&b.properties),
                        )
                })
            && match (&self.resources, &other.resources) {
                (None, None) => true,
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                _ => false,
            }
    }
}

impl PaintArtifact {
    /// Rebuild the per-record image-content proof from the canonical retained
    /// dependency index and resource catalog. Returns the records whose sampled
    /// content changed and clears cached-subsequence reuse for their chunks.
    fn RefreshImageContentRevisions(&mut self) -> Vec<usize> {
        let mut revisions = vec![Vec::new(); self.display_items.len()];
        if let Some(resources) = self.resources.as_deref() {
            for image in &resources.images {
                for dependency in self.image_dependencies.dependencies(image.id) {
                    if let Some(record) = revisions.get_mut(dependency.record_index) {
                        record.push((image.id, image.revision));
                    }
                }
            }
        }
        for record in &mut revisions {
            record.sort_unstable();
            record.dedup();
        }
        let changed: Vec<_> = self
            .display_items
            .iter()
            .zip(&revisions)
            .enumerate()
            .filter_map(|(index, (item, revision))| {
                (item.image_content_revisions != *revision).then_some(index)
            })
            .collect();
        for (item, revision) in self.display_items.iter_mut().zip(revisions) {
            item.image_content_revisions = revision;
        }
        for chunk in &mut self.chunks {
            if changed.iter().any(|index| {
                chunk.begin_index as usize <= *index && *index < chunk.end_index as usize
            }) {
                chunk.is_moved_from_cached_subsequence = false;
            }
        }
        changed
    }

    /// Compose a font-only overlay without mutating either source artifact.
    /// Images need separate resource-ID remapping and remain unsupported here.
    pub fn WithFontOnlyOverlay(content: Option<&Self>, popup: &Self) -> std::io::Result<Self> {
        use crate::paint_engine::{DisplayItem, DisplayItemType};
        let mut list = content.cloned().unwrap_or_default();
        let mut resources = list.resources.as_deref().cloned().unwrap_or_default();
        let font_offset = resources.fonts.len() as u32;
        if let Some(popup_resources) = &popup.resources {
            if !popup_resources.images.is_empty() {
                return Err(std::io::Error::other(
                    "overlay images require resource identity remapping",
                ));
            }
            resources
                .fonts
                .extend(popup_resources.fonts.iter().cloned());
        }
        list.resources = Some(std::sync::Arc::new(resources));
        list.recording_revision = 0;
        list.recorded_properties = list
            .chunks
            .iter()
            .enumerate()
            .map(|(index, chunk)| {
                list.recorded_properties
                    .get(index)
                    .unwrap_or(&chunk.properties)
                    .clone()
            })
            .collect();
        let display_item_offset = u32::try_from(list.display_items.len())
            .map_err(|_| std::io::Error::other("too many content display items"))?;
        std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
            r#type: DisplayItemType::kSave,
            ..Default::default()
        });
        // The wrapper save occupies one replay-op slot, but no semantic item slot.
        let record_offset = list.items.len();
        std::sync::Arc::make_mut(&mut list.items).extend(popup.items.iter().cloned().map(
            |mut item| {
                item.font_face_index += font_offset;
                item
            },
        ));
        list.display_items
            .extend(popup.display_items.iter().cloned().map(|mut item| {
                item.record_begin += record_offset;
                item.record_end += record_offset;
                item
            }));
        for (index, popup_chunk) in popup.chunks.iter().enumerate() {
            list.recorded_properties.push(
                popup
                    .recorded_properties
                    .get(index)
                    .unwrap_or(&popup_chunk.properties)
                    .clone(),
            );
            let mut chunk = popup_chunk.clone();
            chunk.begin_index = chunk
                .begin_index
                .checked_add(display_item_offset)
                .ok_or_else(|| std::io::Error::other("too many popup display items"))?;
            chunk.end_index = chunk
                .end_index
                .checked_add(display_item_offset)
                .ok_or_else(|| std::io::Error::other("too many popup display items"))?;
            // Each Page has its own native clients. Clone retains property-node
            // Arcs and IDs; neither DOM IDs nor dump-local property IDs replace them.
            list.chunks.push(chunk);
        }
        std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
            r#type: DisplayItemType::kRestore,
            ..Default::default()
        });
        Ok(list)
    }
    pub fn DocumentImageContainerRequirements(
        &self,
    ) -> Vec<(image_resource::ImageId, image_resource::ContainerKey)> {
        fn pattern_requirements(
            shader: &PaintShader,
            output: &mut Vec<(image_resource::ImageId, image_resource::ContainerKey)>,
        ) {
            let Some(pattern) = shader.pattern.as_ref() else {
                return;
            };
            for command in &pattern.commands {
                match command {
                    layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawImage(
                        image,
                    ) => {
                        if let Some(key) = &image.document_container_key {
                            let pair = (image.resource_id, key.clone());
                            if image.resource_id != 0 && !output.contains(&pair) {
                                output.push(pair);
                            }
                        }
                    }
                    layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawShape(
                        shape,
                    )
                    | layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawGlyph(
                        shape,
                    ) => {
                        if let Some(shader) = &shape.fill_server {
                            pattern_requirements(shader, output);
                        }
                        if let Some(shader) = &shape.stroke_server {
                            pattern_requirements(shader, output);
                        }
                    }
                    layoutng_assembly::internal::paint_input::SvgPatternCommand::SaveLayerMask(
                        mask,
                    ) => {
                        let nested = PaintShader {
                            pattern: Some(Arc::new(
                                layoutng_assembly::internal::paint_input::SvgPatternData {
                                    commands: mask.commands.clone(),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        };
                        pattern_requirements(&nested, output);
                    }
                    _ => {}
                }
            }
        }
        let mut output = Vec::new();
        for item in self.items.iter() {
            if let Some(key) = &item.document_image_container_key {
                let pair = (item.resource_id, key.clone());
                if item.resource_id != 0 && !output.contains(&pair) {
                    output.push(pair);
                }
            }
            if let Some(shader) = &item.paint_shader {
                pattern_requirements(shader, &mut output);
            }
            for mask in &item.mask_layers {
                if let Some(key) = &mask.document_image_container_key {
                    let pair = (mask.resource_id, key.clone());
                    if mask.resource_id != 0 && !output.contains(&pair) {
                        output.push(pair);
                    }
                }
                if let Some(shader) = &mask.paint_shader {
                    pattern_requirements(shader, &mut output);
                }
            }
        }
        // Reflection masks live on effect nodes rather than display items.
        // They use the same per-container SVG records and must be fulfilled
        // before the second recording snapshots their resource payloads.
        let mut visited = std::collections::HashSet::new();
        for chunk in &self.chunks {
            let mut effect = Some(chunk.properties.effect.as_ref());
            while let Some(node) = effect {
                if !visited.insert(node as *const _) {
                    break;
                }
                for filter in [&node.filters, &node.backdrop_filters] {
                    for layer in filter.reflection_mask_layers() {
                        if let (Some(image), Some(key)) =
                            (&layer.image, &layer.document_image_container_key)
                        {
                            let pair = (image.id, key.clone());
                            if image.id != 0 && !output.contains(&pair) {
                                output.push(pair);
                            }
                        }
                        if let Some(shader) = &layer.paint_shader {
                            pattern_requirements(shader, &mut output);
                        }
                    }
                }
                effect = node.parent.as_deref();
            }
        }
        output
    }

    /// Whether this immutable artifact carries the exact native document
    /// record named by every display item. A newer Page catalog cannot certify
    /// an older artifact snapshot.
    pub fn HasExactDocumentImageContainerRecords(&self) -> bool {
        let requirements = self.DocumentImageContainerRequirements();
        let Some(resources) = self.resources.as_ref() else {
            return requirements.is_empty();
        };
        requirements.iter().all(|(resource_id, container)| {
            resources
                .images
                .iter()
                .find(|image| image.id == *resource_id)
                .is_some_and(|image| match &image.content {
                    image_resource::PaintImageContent::Document(content) => content
                        .exact_record_for(container)
                        .and_then(|record| {
                            downcast_document_paint_record::<DocumentPaintArtifactRecord>(record)
                        })
                        .is_some(),
                    image_resource::PaintImageContent::Bitmap(_) => false,
                })
        })
    }
}

#[cfg(test)]
#[test]
fn document_image_container_proof_uses_the_artifacts_own_catalog() {
    let mut requested = image_resource::ContainerKey::new(17, 17, 1.0, 2.0);
    requested.preferred_color_scheme_is_explicit = true;
    let initial = image_resource::ContainerKey::new(24, 24, 1.0, 2.0);
    let record = |key: &image_resource::ContainerKey| {
        Arc::new(DocumentPaintArtifactRecord {
            artifact: Arc::new(PaintArtifact::default()),
            intrinsic_size: image_resource::IntrinsicSize {
                width: key.width,
                height: key.height,
            },
            record_size: image_resource::IntrinsicSize {
                width: key.width,
                height: key.height,
            },
            container_key: key.clone(),
        }) as Arc<dyn image_resource::DocumentPaintRecord>
    };
    let mut artifact = PaintArtifact {
        items: vec![DisplayItem {
            r#type: DisplayItemType::kDrawImageRect,
            resource_id: 7,
            document_image_container_key: Some(requested.clone()),
            ..Default::default()
        }]
        .into(),
        resources: Some(Arc::new(PaintResources {
            images: vec![NativePaintImage {
                id: 7,
                document_container_key: Some(initial.clone()),
                content: PaintImageContent::Document(Arc::new(
                    image_resource::DocumentPaintContent::new(initial.clone(), record(&initial)),
                )),
                ..Default::default()
            }],
            ..Default::default()
        })),
        ..Default::default()
    };
    assert!(!artifact.HasExactDocumentImageContainerRecords());

    let resources = Arc::make_mut(artifact.resources.as_mut().unwrap());
    resources.images[0].content = PaintImageContent::Document(Arc::new(
        image_resource::DocumentPaintContent::new(requested.clone(), record(&requested)),
    ));
    assert!(artifact.HasExactDocumentImageContainerRecords());
}

#[cfg(test)]
#[test]
fn artifact_property_updates_share_ops_and_real_op_edits_isolate_old_frames() {
    let old = Arc::new(PaintArtifact {
        items: vec![DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect: PaintRect {
                x: 3.0,
                y: 4.0,
                width: 20.0,
                height: 10.0,
            },
            ..Default::default()
        }]
        .into(),
        chunks: vec![PaintChunk::default()],
        ..Default::default()
    });
    let mut scrolling = old.clone();
    let updated = Arc::make_mut(&mut scrolling);
    // This ownership test models a new property snapshot only; it invokes no
    // native identity, layout or recording algorithm.
    Arc::make_mut(&mut updated.chunks[0].properties.transform)
        .matrix
        .values[13] = -12.0;
    assert!(
        Arc::ptr_eq(&old.items, &scrolling.items),
        "property changes share the immutable PaintRecord operations"
    );
    assert_eq!(old.chunks[0].properties.transform.matrix.values[13], 0.0);
    assert_eq!(
        scrolling.chunks[0].properties.transform.matrix.values[13],
        -12.0
    );
    Arc::make_mut(&mut Arc::make_mut(&mut scrolling).items)[0]
        .rect
        .x = 9.0;
    assert!(!Arc::ptr_eq(&old.items, &scrolling.items));
    assert_eq!(
        old.items[0].rect.x, 3.0,
        "a real operation edit leaves the old frame immutable"
    );
    assert_eq!(scrolling.items[0].rect.x, 9.0);
    assert_eq!(old.items.len(), scrolling.items.len());
}

#[cfg(all(test, feature = "translation_in_progress"))]
#[test]
fn document_image_mutation_retains_recording_and_targets_its_client() {
    use image_resource::{ContainerKey, DocumentImageFrame, IntrinsicSize, PaintImageContent};
    let draw = DisplayItem {
        r#type: DisplayItemType::kDrawImageRect,
        resource_id: 7,
        rect: PaintRect {
            x: 32.0,
            y: 48.0,
            width: 80.0,
            height: 40.0,
        },
        ..Default::default()
    };
    let display = RecordedDisplayItem {
        is_cacheable: true,
        is_tombstone: false,
        paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
        source_revision: 1,
        image_content_revisions: Vec::new(),
        scrollbar_needs_update_display: None,
        scrollbar: None,
        kind: RecordedDisplayItemKind::Drawing,
        id: Default::default(),
        visual_rect: draw.rect,
        visual_rect_is_accurate: true,
        draws_content: true,
        raster_effect_outset: RasterEffectOutset::kNone,
        record_begin: 0,
        record_end: 1,
        scroll_translation: None,
    };
    let items: Arc<Vec<_>> = Arc::new(vec![draw]);
    let dependencies = Arc::new(ImageDependencyIndex::build(
        &items,
        std::slice::from_ref(&display),
        &[],
    ));
    let old_record: Arc<dyn image_resource::DocumentPaintRecord> = Arc::new(1u8);
    let resources = Arc::new(PaintResources {
        images: vec![layoutng_assembly::internal::layout_input::PaintImage {
            id: 7,
            revision: 1,
            width: 80,
            height: 40,
            content: PaintImageContent::Document(Arc::new(
                image_resource::DocumentPaintContent::new(
                    ContainerKey::new(80, 40, 1.0, 1.0),
                    old_record,
                ),
            )),
            ..Default::default()
        }],
        ..Default::default()
    });
    let old = Arc::new(PaintArtifact {
        items: items.clone(),
        display_items: vec![display],
        resources: Some(resources),
        image_dependencies: dependencies,
        recording_revision: 99,
        ..Default::default()
    });
    let mut engine = PaintEngine::new();
    engine.paint_result = Some(old.clone());
    let next_record: Arc<dyn image_resource::DocumentPaintRecord> = Arc::new(2u8);
    let frame = Arc::new(DocumentImageFrame {
        animation_info: image_resource::PaintImageAnimationInfo::confirmed_static(),
        completion_state: image_resource::PaintImageCompletionState::Done,
        resource_id: 7,
        revision: 2,
        intrinsic_size: IntrinsicSize {
            width: 80,
            height: 40,
        },
        container_key: ContainerKey::new(80, 40, 1.0, 1.0),
        record: next_record,
    });
    let result = engine
        .ApplyMutation(PaintMutation::ImageChanged {
            image_id: 7,
            revision: 2,
            frame: frame.clone(),
        })
        .unwrap();
    let next = engine.GetPaintResult().unwrap();
    assert_eq!(result.damaged_records.len(), 1);
    assert_eq!(
        result.damaged_records[0].visual_rect,
        PaintRect {
            x: 32.0,
            y: 48.0,
            width: 80.0,
            height: 40.0
        }
    );
    assert!(Arc::ptr_eq(&old.items, &next.items));
    assert_eq!(old.recording_revision, next.recording_revision);
    assert_eq!(old.resources.as_ref().unwrap().images[0].revision, 1);
    assert_eq!(next.resources.as_ref().unwrap().images[0].revision, 2);
    let subsumed = engine
        .ApplyMutation(PaintMutation::ImageChanged {
            image_id: 7,
            revision: 2,
            frame,
        })
        .expect("the committed immutable revision subsumes its queued invalidation");
    assert!(subsumed.damaged_records.is_empty());
}

#[cfg(feature = "translation_in_progress")]
use std::cell::RefCell;
#[cfg(feature = "translation_in_progress")]
use std::collections::HashSet;

#[cfg(feature = "translation_in_progress")]
use layoutng_assembly::fragment_tree::FragmentNode;

#[cfg(feature = "translation_in_progress")]
use crate::paint_context::PaintContext;
#[cfg(feature = "translation_in_progress")]
use crate::paint_layer_painter::PaintLayerPainter;
#[cfg(feature = "translation_in_progress")]
use crate::pre_paint_tree_walk::PrePaintTreeWalk;

// cpp: paint/paint_engine.h:78-80
// cpp: paint/paint_engine.cc:15-29
#[cfg(feature = "translation_in_progress")]
pub fn FragmentClientRects(root: &FragmentNode, node_id: u64) -> Vec<PaintRect> {
    FragmentClientRectsByNode(root)
        .remove(&node_id)
        .unwrap_or_default()
}

// Build once for one immutable layout snapshot, then query by node id.
#[cfg(feature = "translation_in_progress")]
pub fn FragmentClientRectsByNode(
    root: &FragmentNode,
) -> std::collections::HashMap<u64, Vec<PaintRect>> {
    crate::client_rects::CollectFragmentClientRects(root)
}

// cpp: paint/paint_engine.cc:33-54
#[cfg(feature = "translation_in_progress")]
fn SameReplayResources(a: &PaintResources, b: &PaintResources) -> bool {
    if a.device_pixel_ratio != b.device_pixel_ratio
        || a.preferred_color_scheme != b.preferred_color_scheme
        || a.layout_zoom_factor != b.layout_zoom_factor
        || a.viewport != b.viewport
        || a.fonts.len() != b.fonts.len()
        || a.images.len() != b.images.len()
    {
        return false;
    }
    let same_fonts = a.fonts.ptr_eq(&b.fonts)
        || a.fonts
            .iter()
            .zip(&b.fonts)
            .all(|(left, right)| left.face_index == right.face_index && left.bytes == right.bytes);
    same_fonts && a.images == b.images
}

// cpp: paint/paint_engine.cc:56-91
#[cfg(feature = "translation_in_progress")]
fn ValidateReplayResources(resources: Option<&PaintResources>) {
    let Some(resources) = resources else {
        return;
    };
    if !resources.device_pixel_ratio.is_finite()
        || resources.device_pixel_ratio <= 0.0
        || !resources.layout_zoom_factor.is_finite()
        || resources.layout_zoom_factor <= 0.0
        || !resources.ArtifactToDeviceScale().is_finite()
    {
        panic!("PaintResources device and layout scales must be finite and positive");
    }
    if let Some(viewport) = resources.viewport {
        if viewport.size.width < 0 || viewport.size.height < 0 {
            panic!("PaintResources viewport size must be nonnegative");
        }
    }
    for font in &resources.fonts {
        if font.bytes.is_empty() {
            panic!("PaintResources font bytes are empty");
        }
    }
    let mut image_ids = HashSet::new();
    for image in &resources.images {
        let width = image.width as usize;
        let height = image.height as usize;
        let dimensions_fit = width != 0 && height != 0 && width <= usize::MAX / 4 / height;
        let bitmap_valid = image
            .BitmapPixels()
            .is_none_or(|pixels| dimensions_fit && pixels.len() == width * height * 4);
        if image.id == 0
            || !image_ids.insert(image.id)
            || image.width == 0
            || image.height == 0
            || !image.resolution_scale.is_finite()
            || image.resolution_scale <= 0.0
            || !dimensions_fit
            || !bitmap_valid
        {
            panic!("PaintResources images must have unique ids and valid RGBA data");
        }
    }
}

// cpp: paint/paint_engine.cc:93-112
#[cfg(feature = "translation_in_progress")]
fn ValidateResourceCatalog(
    fragment: &FragmentNode,
    catalog: Option<&PaintResources>,
    validated_catalogs: &mut HashSet<*const PaintResources>,
) {
    if let Some(candidate) = fragment.paint.resources.as_deref() {
        let pointer = candidate as *const PaintResources;
        if !validated_catalogs.contains(&pointer) {
            if catalog.is_none_or(|root| !SameReplayResources(candidate, root)) {
                panic!("all FragmentNode paint data must use the root replay resources");
            }
            validated_catalogs.insert(pointer);
        }
    }
    for child in &fragment.children {
        ValidateResourceCatalog(child, catalog, validated_catalogs);
    }
}

// cpp: paint/paint_engine.h:241-243
// cpp: paint/paint_engine.cc:124-143
#[cfg(feature = "translation_in_progress")]
pub fn Paint(fragments: &FragmentNode) -> PaintArtifact {
    PaintWithCaret(fragments, None)
}

#[cfg(feature = "translation_in_progress")]
#[allow(non_snake_case)]
pub fn PaintWithCaret(fragments: &FragmentNode, caret: Option<CaretPosition>) -> PaintArtifact {
    PaintWithCaretAndCapacity(fragments, caret, 0, 0, 0)
}

/// Previous output lengths are allocation hints only; every item is regenerated.
#[cfg(feature = "translation_in_progress")]
#[allow(non_snake_case)]
pub fn PaintWithCaretAndCapacity(
    fragments: &FragmentNode,
    caret: Option<CaretPosition>,
    item_capacity: usize,
    display_item_capacity: usize,
    chunk_capacity: usize,
) -> PaintArtifact {
    RecordPaint(
        &mut PrePaintTreeWalk::default(),
        fragments,
        EditingPaintState {
            caret,
            document_selection: None,
        },
        item_capacity,
        display_item_capacity,
        chunk_capacity,
        None,
    )
}

#[cfg(feature = "translation_in_progress")]
fn RecordPaint(
    pre_paint: &mut PrePaintTreeWalk,
    fragments: &FragmentNode,
    editing: EditingPaintState,
    item_capacity: usize,
    display_item_capacity: usize,
    chunk_capacity: usize,
    previous: Option<Arc<PaintArtifact>>,
) -> PaintArtifact {
    let mut trace = browser_tracing::span("paint", "Paint.RecordPaint");
    ValidateReplayResources(fragments.paint.resources.as_deref());
    let mut validated_catalogs = HashSet::new();
    if let Some(catalog) = fragments.paint.resources.as_deref() {
        validated_catalogs.insert(catalog as *const PaintResources);
    }
    ValidateResourceCatalog(
        fragments,
        fragments.paint.resources.as_deref(),
        &mut validated_catalogs,
    );
    let paint_tree = {
        let mut trace = browser_tracing::span("prepaint", "PrePaintTreeWalk.Walk");
        let tree = pre_paint.Walk(fragments);
        let stats = pre_paint.stats;
        trace.set("computed_nodes", stats.computed_nodes as f64);
        trace.set("reused_nodes", stats.reused_nodes as f64);
        trace.set("skipped_subtrees", stats.skipped_subtrees as f64);
        trace.set("post_walk", stats.post_walk as u8 as f64);
        tree
    };
    let mut output = PaintArtifact::default();
    output.compositor_viewport = pre_paint.ExportCompositorViewport(fragments);
    Arc::make_mut(&mut output.items).reserve(item_capacity);
    output.display_items.reserve(display_item_capacity);
    output.chunks.reserve(chunk_capacity);
    output.resources = fragments.paint.resources.clone();
    // PrePaint's resident nodes deliberately omit FragmentNode.children.
    // Resolve against the complete layout snapshot before painting that tree.
    fn caret_owner(node: &FragmentNode, id: u64) -> Option<&FragmentNode> {
        if node.node_id == id && node.paint.text_control_caret_metrics.is_some() {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| caret_owner(child, id))
    }
    let caret_local_rect = editing.caret.and_then(|state| {
        layoutng_assembly::caret::geometry::TextControlCaretRect(
            caret_owner(fragments, state.node_id)?,
            state.offset,
            state.empty,
        )
    });
    let mut paint_context = PaintContext::WithEditingAndCache(&mut output, editing, previous);
    paint_context.caret_local_rect = caret_local_rect;
    let context = RefCell::new(paint_context);
    context
        .borrow_mut()
        .SetNextPropertyId(pre_paint.next_property_id);
    {
        let _trace = browser_tracing::span("paint", "PaintLayerPainter.Paint");
        PaintLayerPainter::new(&paint_tree, &context, Vec::new()).Paint();
    }
    context.borrow_mut().Finish();
    trace.set(
        "cached_display_items",
        context.borrow().cached_display_items as f64,
    );
    drop(context);
    output.recorded_properties = output
        .chunks
        .iter()
        .map(|chunk| chunk.properties.clone())
        .collect();
    output.image_dependencies = Arc::new(ImageDependencyIndex::build(
        &output.items,
        &output.display_items,
        &output.chunks,
    ));
    output.RefreshImageContentRevisions();
    output.recording_revision = foundation::NewUniqueObjectId();
    if pre_paint.stats.post_walk || pre_paint.retained_scroll_state.is_none() {
        pre_paint.retained_scroll_state =
            crate::scroll_paint_update::RetainedScrollPaintState::capture(
                &paint_tree,
                &pre_paint.property_node_store,
                &output,
            );
    }
    pre_paint.RetainTree(paint_tree);
    trace.set("items", output.items.len() as f64);
    trace.set("display_items", output.display_items.len() as f64);
    trace.set("chunks", output.chunks.len() as f64);
    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
        let stats = pre_paint.stats;
        eprintln!(
            "prepaint-update computed_nodes={} reused_nodes={} skipped_subtrees={} post_walk={}",
            stats.computed_nodes, stats.reused_nodes, stats.skipped_subtrees, stats.post_walk
        );
    }
    output
}

/// Resident paint facade. PrePaint's property ownership directory and the
/// committed artifact have independent lifetimes, like ObjectPaintProperties
/// and PaintControllerPersistentData in Blink. Completed nodes retain shallow
/// source data and geometry; no layout-root Rc or borrowed snapshot is retained.
#[cfg(feature = "translation_in_progress")]
#[derive(Default)]
pub struct PaintEngine {
    pre_paint: PrePaintTreeWalk,
    paint_result: Option<Arc<PaintArtifact>>,
    animation_targeted_nodes: Vec<u64>,
}

#[cfg(feature = "translation_in_progress")]
impl PaintEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ApplyAnimationMutation(&mut self, mutation: animation::PaintMutation) {
        match mutation {
            animation::PaintMutation::SetTargetedProperties { node_ids } => {
                self.animation_targeted_nodes = node_ids;
            }
            animation::PaintMutation::Commit => self.animation_targeted_nodes.clear(),
        }
    }

    pub fn AnimationTargetedNodes(&self) -> &[u64] {
        &self.animation_targeted_nodes
    }

    /// Update resident properties and record a new artifact. Publish only once
    /// this pass completes; a failed paint cannot expose the previous result
    /// as if it represented the new inputs.
    pub fn Paint(&mut self, layout_result: &FragmentNode, editing: EditingPaintState) {
        let _trace = browser_tracing::span("paint", "PaintEngine.Paint");
        let previous = self.paint_result.clone();
        let capacities = previous.as_ref().map_or((0, 0, 0), |old| {
            (old.items.len(), old.display_items.len(), old.chunks.len())
        });
        self.pre_paint.SetRetainedNativeOwnerLifetimes(false);
        let result = RecordPaint(
            &mut self.pre_paint,
            layout_result,
            editing,
            capacities.0,
            capacities.1,
            capacities.2,
            previous,
        );
        self.paint_result = Some(Arc::new(result));
    }

    pub fn GetPrePaintUpdateStats(&self) -> crate::pre_paint_tree_walk::PrePaintUpdateStats {
        self.pre_paint.stats
    }

    pub fn GetPaintResult(&self) -> Option<&Arc<PaintArtifact>> {
        self.paint_result.as_ref()
    }

    /// Publish a new immutable resource catalog while retaining the recorded
    /// display items and paint properties. This is the document-image analogue
    /// of ImageResourceContent invalidation: stable resource ids let the
    /// compositor damage only records that sample changed pixels.
    pub fn UpdatePaintResources(&mut self, resources: Arc<PaintResources>) -> bool {
        ValidateReplayResources(Some(&resources));
        let Some(artifact) = self.paint_result.as_mut() else {
            return false;
        };
        if artifact
            .resources
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &resources))
        {
            return true;
        }
        let artifact = Arc::make_mut(artifact);
        artifact.resources = Some(resources);
        artifact.RefreshImageContentRevisions();
        true
    }

    /// Apply a content-only image mutation to the committed artifact. Stable
    /// display items remain shared; the retained dependency index identifies
    /// exactly which paint clients LayerTile must invalidate.
    pub fn ApplyMutation(&mut self, mutation: PaintMutation) -> Option<PaintMutationResult> {
        let PaintMutation::ImageChanged {
            image_id,
            revision,
            frame,
        } = mutation;
        if frame.resource_id != image_id || frame.revision != revision {
            return None;
        }
        let artifact = self.paint_result.as_mut()?;
        let old_resources = artifact.resources.as_ref()?.as_ref();
        let old = old_resources
            .images
            .iter()
            .find(|image| image.id == image_id)?;
        if old.width != frame.intrinsic_size.width || old.height != frame.intrinsic_size.height {
            return None;
        }
        // A full Paint can commit the same image frame before Page drains a
        // queued client-level invalidation. Chromium treats that invalidation
        // as subsumed by the newer paint controller commit. It is not a bad
        // resource transition and must not terminate the page. Preserve the
        // immutable-revision contract by accepting an equal revision only
        // when it names the exact record already published.
        if revision <= old.revision {
            if revision == old.revision
                && !matches!(&old.content,
                    image_resource::PaintImageContent::Document(content)
                    if content.record_for(&frame.container_key)
                        .is_some_and(|record| Arc::ptr_eq(record, &frame.record)))
            {
                return None;
            }
            return Some(PaintMutationResult::default());
        }
        let damaged_records = artifact.image_dependencies.dependencies(image_id).to_vec();
        let mut resources = old_resources.clone();
        let image = resources
            .images
            .iter_mut()
            .find(|image| image.id == image_id)?;
        image.revision = revision;
        image.UpdateDocumentFrameMetadata(&frame);
        image.document_container_key = Some(frame.container_key.clone());
        let image_resource::PaintImageContent::Document(content) = &image.content else {
            return None;
        };
        image.content = image_resource::PaintImageContent::Document(Arc::new(
            content.with_variant(frame.container_key.clone(), frame.record.clone()),
        ));
        let artifact = Arc::make_mut(artifact);
        artifact.resources = Some(Arc::new(resources));
        artifact.RefreshImageContentRevisions();
        Some(PaintMutationResult { damaged_records })
    }

    /// Reconcile resource-only changes with the artifact currently published
    /// by Page. This keeps Arc COW/adoption and resource revision validation
    /// inside PaintEngine while older compositor snapshots remain immutable.
    pub fn PublishResourceUpdate(
        &mut self,
        artifact: Arc<PaintArtifact>,
        update: PaintResourceUpdate,
    ) -> Option<Arc<PaintArtifact>> {
        self.paint_result = Some(artifact);
        match update {
            PaintResourceUpdate::Snapshot(resources) => {
                self.UpdatePaintResources(resources).then_some(())?;
            }
            PaintResourceUpdate::Mutations(mutations) => {
                for mutation in mutations {
                    self.ApplyMutation(mutation)?;
                }
            }
        }
        self.paint_result.clone()
    }

    /// Complete Paint-owned client validation and publish an immutable output.
    /// Layout consumes the opaque client receipt at its own boundary; Page
    /// never receives mutable access to the artifact or the client set.
    pub fn CommitPaintResult(&mut self) -> PaintOutput {
        let _trace = browser_tracing::span("paint", "PaintEngine.CommitPaintResult");
        let artifact = Arc::make_mut(
            self.paint_result
                .as_mut()
                .expect("Paint must complete before commit"),
        );
        let clients = artifact
            .display_items
            .iter()
            .map(|item| item.id.client_id)
            .chain(artifact.chunks.iter().map(|chunk| chunk.id.client_id))
            .collect::<HashSet<_>>();
        // cpp: platform/graphics/paint/paint_controller.cc:141-143
        for chunk in &mut artifact.chunks {
            chunk.client_is_just_created = false;
        }
        self.pre_paint.CommitClients(clients.iter().copied());
        PaintOutput {
            artifact: self
                .paint_result
                .as_ref()
                .expect("committed Paint result")
                .clone(),
            layout_commit: layoutng_assembly::layout_engine::PaintClientCommit::FromClientSet(
                clients,
            ),
        }
    }

    /// Move Page's current frame reference back before updating the artifact.
    /// This prevents Engine/Page sharing alone from causing a deep copy.
    #[doc(hidden)]
    pub fn AdoptPaintResult(&mut self, artifact: Arc<PaintArtifact>) {
        self.paint_result = Some(artifact);
    }

    /// Requires the native retained-geometry proof from Page, just as the
    /// existing scroll update path. A rejection leaves the artifact unchanged.
    #[doc(hidden)]
    pub fn TryUpdateScrollProperties(
        &mut self,
        fragments: &FragmentNode,
    ) -> Result<
        crate::scroll_paint_update::ScrollPropertyUpdate,
        crate::scroll_paint_update::ScrollPaintInvalidation,
    > {
        let mut trace = browser_tracing::span("paint", "PaintEngine.TryUpdateScrollProperties");
        trace.set("records_reused", 0.0);
        let Some(artifact) = self.paint_result.as_mut() else {
            return Err(crate::scroll_paint_update::ScrollPaintInvalidation::MissingCoverage);
        };
        trace.set("artifact_strong_count", Arc::strong_count(artifact) as f64);
        // Native scroll refresh versions the changed nodes and ancestor paths.
        // Keep the resident tree as the old geometry/context baseline: the
        // next ordinary Walk must compare it, not force unrelated subtrees.
        let result = crate::scroll_paint_update::TryUpdateScrollPropertiesWithPrePaint(
            fragments,
            Arc::make_mut(artifact),
            &mut self.pre_paint,
        );
        trace.set("records_reused", result.is_ok() as u8 as f64);
        if let Ok(update) = &result {
            trace.set("updated_chunks", update.updated_chunks as f64);
            trace.set(
                "updated_scrollbar_records",
                update.updated_scrollbar_records as f64,
            );
        }
        result
    }
}
