#![allow(non_camel_case_types)]

//! Source paint display-list ABI. The entry point and private painters are
//! connected after their owning source files have been translated.

use std::sync::Arc;

pub use layoutng_assembly::fragment_tree::RasterEffectOutset;
use layoutng_assembly::fragment_tree::{PaintGlyph, PaintResources};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, FontSmoothing, FontVariation, Offset, PaintPathCommand, Size,
    TextDecorationStyle, TransformMatrix, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    BackgroundRepeatRule, PaintBlendMode, PaintCornerRadii, PaintFilterOperation,
    PaintImageTileRule, PaintMaskComposite, PaintMaskMode, PaintShader, SvgStrokeLineCap,
    SvgStrokeLineJoin,
};

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
    pub resource_id: u64,
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
            resource_id: 0,
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
    pub text_blob_origin: Offset,
    pub horizontal: bool,
    pub at_start: bool,
    pub rtl: bool,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
    pub font_smoothing: FontSmoothing,
    pub writing_mode: WritingMode,
    pub glyphs: Vec<PaintGlyph>,
}

impl Default for DisplayItem {
    fn default() -> Self {
        Self {
            r#type: DisplayItemType::kDrawRect,
            phase: PaintPhase::kForeground,
            node_id: 0,
            fragment_instance_id: 0,
            rect: PaintRect::default(),
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
            text_blob_origin: Offset::default(),
            horizontal: true,
            at_start: false,
            rtl: false,
            synthetic_bold: false,
            synthetic_italic: false,
            font_smoothing: FontSmoothing::kAuto,
            writing_mode: WritingMode::kHorizontalTb,
            glyphs: Vec::new(),
        }
    }
}

// Public producer analysis also serves canonical PaintRecords downstream.
pub use crate::paint_context::CalculateRectKnownToBeOpaqueForRecord;

// Blink paint_chunk.h uses DisplayItem::Id for chunk identity.
pub use crate::display_item_id::DisplayItemId as PaintChunkId;

/// Core Blink PaintChunk fields. Indices address `display_items`, not replay ops.
#[derive(Clone, PartialEq)]
pub struct PaintChunk {
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

/// A semantic item containing a PaintRecord range in the flat replay ABI.
/// Save/restore and multiple draw ops inside a recorder count as one item.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedDisplayItem {
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretGeometry {
    pub node_id: u64,
    pub rect: PaintRect,
    pub visible: bool,
}

/// Drawing display items, each selecting its recorded drawing operations.
pub type DisplayItemList = Vec<RecordedDisplayItem>;
pub type PaintChunks = Vec<PaintChunk>;

// cpp: paint/paint_engine.h:227-239
#[derive(Clone, Default)]
pub struct PaintArtifact {
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
    pub caret: Option<CaretGeometry>,
}

impl PartialEq for PaintArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.caret == other.caret
            && (Arc::ptr_eq(&self.items, &other.items) || self.items == other.items)
            && self.display_items == other.display_items
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
        || a.fonts.len() != b.fonts.len()
        || a.images.len() != b.images.len()
    {
        return false;
    }
    let same_fonts = a
        .fonts
        .iter()
        .zip(&b.fonts)
        .all(|(left, right)| left.face_index == right.face_index && left.bytes == right.bytes);
    same_fonts
        && a.images.iter().zip(&b.images).all(|(left, right)| {
            left.id == right.id
                && left.width == right.width
                && left.height == right.height
                && left.resolution_scale == right.resolution_scale
                && left.rgba8 == right.rgba8
        })
}

// cpp: paint/paint_engine.cc:56-91
#[cfg(feature = "translation_in_progress")]
fn ValidateReplayResources(resources: Option<&PaintResources>) {
    let Some(resources) = resources else {
        return;
    };
    if !resources.device_pixel_ratio.is_finite() || resources.device_pixel_ratio <= 0.0 {
        panic!("PaintResources device pixel ratio must be finite and positive");
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
        let expected_bytes = if dimensions_fit {
            width * height * 4
        } else {
            0
        };
        if image.id == 0
            || !image_ids.insert(image.id)
            || image.width == 0
            || image.height == 0
            || !image.resolution_scale.is_finite()
            || image.resolution_scale <= 0.0
            || !dimensions_fit
            || image.rgba8.len() != expected_bytes
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
        caret,
        item_capacity,
        display_item_capacity,
        chunk_capacity,
    )
}

#[cfg(feature = "translation_in_progress")]
fn RecordPaint(
    pre_paint: &mut PrePaintTreeWalk,
    fragments: &FragmentNode,
    caret: Option<CaretPosition>,
    item_capacity: usize,
    display_item_capacity: usize,
    chunk_capacity: usize,
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
    let caret_local_rect = caret.and_then(|state| {
        layoutng_assembly::caret::geometry::TextControlCaretRect(
            caret_owner(fragments, state.node_id)?,
            state.offset,
            state.empty,
        )
    });
    let mut paint_context = PaintContext::WithCaret(&mut output, caret);
    paint_context.caret_local_rect = caret_local_rect;
    let context = RefCell::new(paint_context);
    context
        .borrow_mut()
        .SetNextPropertyId(pre_paint.next_property_id);
    {
        let _trace = browser_tracing::span("paint", "PaintLayerPainter.Paint");
        PaintLayerPainter::new(&paint_tree, &context, Vec::new()).Paint();
    }
    drop(context);
    output.recorded_properties = output
        .chunks
        .iter()
        .map(|chunk| chunk.properties.clone())
        .collect();
    output.recording_revision = foundation::NewUniqueObjectId();
    if pre_paint.scroll_recording_basis.is_some()
        || pre_paint.stats.post_walk
        || pre_paint.retained_scroll_state.is_none()
    {
        pre_paint.retained_scroll_state =
            crate::scroll_paint_update::RetainedScrollPaintState::capture(
                &paint_tree,
                &pre_paint.property_node_store,
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
}

#[cfg(feature = "translation_in_progress")]
impl PaintEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update resident properties and record a new artifact. Publish only once
    /// this pass completes; a failed paint cannot expose the previous result
    /// as if it represented the new inputs.
    pub fn Paint(&mut self, layout_result: &FragmentNode, caret: Option<CaretPosition>) {
        let mut trace = browser_tracing::span("paint", "PaintEngine.Paint");
        let capacities = self.paint_result.take().map_or((0, 0, 0), |old| {
            (old.items.len(), old.display_items.len(), old.chunks.len())
        });
        if caret.is_some() {
            self.pre_paint.ResetScrollRecordingBasis();
        }
        self.pre_paint.PrepareScrollRecordingBasis(layout_result);
        self.pre_paint.SetRetainedNativeOwnerLifetimes(false);
        let mut result = RecordPaint(
            &mut self.pre_paint,
            layout_result,
            caret,
            capacities.0,
            capacities.1,
            capacities.2,
        );
        if self.pre_paint.scroll_recording_basis.is_some() {
            // RecordPaint captured the drawing-coordinate view. Publish the
            // actual current scroll graph only after the same geometry/cull
            // proof as retained records. New records need no old-client match.
            self.pre_paint.scroll_recording_basis =
                Some(self.pre_paint.property_node_store.scroll_offsets());
            if let Err(reason) = crate::scroll_paint_update::TryUpdateScrollPropertiesWithPrePaint(
                layout_result,
                &mut result,
                &mut self.pre_paint,
                crate::scroll_paint_update::ScrollRecordingUpdate::Fresh,
            ) {
                trace.set("recording_basis_rebase", 1.0);
                // New sticky geometry, unsupported properties or exhausted
                // cull coverage require a real new recording coordinate basis.
                self.pre_paint.ResetScrollRecordingBasis();
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    eprintln!("paint-recording-basis-rebase reason={reason:?}");
                }
                result = RecordPaint(
                    &mut self.pre_paint,
                    layout_result,
                    caret,
                    capacities.0,
                    capacities.1,
                    capacities.2,
                );
            }
        }
        self.paint_result = Some(Arc::new(result));
    }

    pub fn GetPrePaintUpdateStats(&self) -> crate::pre_paint_tree_walk::PrePaintUpdateStats {
        self.pre_paint.stats
    }

    pub fn GetPaintResult(&self) -> Option<&Arc<PaintArtifact>> {
        self.paint_result.as_ref()
    }

    /// Complete native client validation before publishing this artifact to
    /// the renderer. Existing external snapshots remain immutable.
    pub fn CommitPaintResult(&mut self, commit: impl FnOnce(&mut PaintArtifact)) {
        let _trace = browser_tracing::span("paint", "PaintEngine.CommitPaintResult");
        let artifact = Arc::make_mut(
            self.paint_result
                .as_mut()
                .expect("Paint must complete before commit"),
        );
        commit(artifact);
        self.pre_paint.CommitClients(
            artifact
                .display_items
                .iter()
                .map(|item| item.id.client_id)
                .chain(artifact.chunks.iter().map(|chunk| chunk.id.client_id)),
        );
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
            self.pre_paint.ResetScrollRecordingBasis();
            return Err(crate::scroll_paint_update::ScrollPaintInvalidation::MissingCoverage);
        };
        trace.set("artifact_strong_count", Arc::strong_count(artifact) as f64);
        // Native scroll refresh versions the changed nodes and ancestor paths.
        // Keep the resident tree as the old geometry/context baseline: the
        // next ordinary Walk must compare it, not force unrelated subtrees.
        let basis = self
            .pre_paint
            .scroll_recording_basis
            .is_none()
            .then(|| self.pre_paint.property_node_store.scroll_offsets());
        let result = crate::scroll_paint_update::TryUpdateScrollPropertiesWithPrePaint(
            fragments,
            Arc::make_mut(artifact),
            &mut self.pre_paint,
            crate::scroll_paint_update::ScrollRecordingUpdate::Retained,
        );
        trace.set("records_reused", result.is_ok() as u8 as f64);
        if let Ok(update) = &result {
            trace.set("updated_chunks", update.updated_chunks as f64);
            trace.set(
                "updated_scrollbar_records",
                update.updated_scrollbar_records as f64,
            );
        }
        if result.is_ok() {
            if let Some(basis) = basis {
                self.pre_paint.scroll_recording_basis = Some(basis);
            }
        } else {
            self.pre_paint.ResetScrollRecordingBasis();
        }
        result
    }
}
