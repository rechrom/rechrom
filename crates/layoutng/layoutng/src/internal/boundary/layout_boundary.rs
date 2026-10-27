#![allow(non_snake_case)]

use crate::layout_boundary_support::{
    LayoutBoundaryCache, LayoutBoundaryHashMap, LayoutBoundaryHashSet, LayoutBoundarySourceEntry,
    LayoutUpdateKind,
};
use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::{Arc, OnceLock};

use font_engine::fonts::canvas_rotation_in_vertical::IsCanvasRotationInVerticalUpright;
use font_engine::fonts::opentype::open_type_math_support::{MathConstants, OpenTypeMathSupport};
use font_engine::text::native::hyphenation::HyphenationBackend;
use font_engine::text::native::hyphenation_services::{
    NativeHyphenationResolver, NativeHyphenationResolverScope,
};
use font_engine::text::native::opentype_font::FontMetric;
use font_engine::text::native::phrase_break_services::{
    NativePhraseBreakResolver, NativePhraseBreakResolverScope,
};
use font_engine::{
    CanvasRotationInVertical, Font, FontBaseline, FontDescription, FontOrientation,
    FontSmoothingMode, HarfBuzzShaper, Hyphenation, LayoutLocale, ShapeResultView, SimpleFontData,
};
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::{
    gfx, AtomicString, DynamicTo, EBorderStyle, EBoxDecorationBreak, EDisplay, EOverscrollBehavior,
    ETextDecorationSkipInk, ETextDecorationStyle as NativeTextDecorationStyle, ETextTransform,
    LayoutUnit, Length, MakeGarbageCollected, Member, Persistent, PhysicalOffset, PhysicalRect,
    PhysicalSize, String as BlinkString, TextDecorationLine, TextEmphasisMark, TextOffsetMap, To,
    UnsupportedLayout, WritingMode as NativeWritingMode,
};
use layout_fragment::block_break_token::BlockBreakToken;
use layout_fragment::fragment_item::{FragmentItem, ItemType};
use layout_fragment::fragment_tree::{
    BackgroundPaintLocation, CanvasBackgroundPaintData, CollapsedTableEdge,
    CollapsedTablePaintData, CollapsedTableSection, ColumnRule, ColumnRulePaintData,
    FieldsetPaintData, FormControlPaintData, FragmentBoxSides, FragmentKind, FragmentPaintData,
    FragmentPaintRect, FrameSetPaintData, LayoutResultHandle, MathMLPaintData, MathMLPaintKind,
    PaintGlyph, PaintGlyphRun, PaintProperties, PaintResources, ReplacedContentPaintData,
    ScrollContainerPaintData, ScrollbarPaintAxis, ScrollbarPaintData, StickyAxisPaintData,
    StickyPositionPaintData, StitchedDecorationData, SvgTextPaintData, TablePaintColumn,
    TablePaintData, TextCombineEmphasisPaintData,
};
use layout_fragment::layout_result::LayoutResult;
use layout_fragment::logical_fragment::LogicalFragment;
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
use layout_fragment::physical_fragment::PhysicalFragment;
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesHorizontal, kPhysicalAxesVertical, PhysicalAxes, PhysicalAxis,
};
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::ToLogicalSize;
use layoutng_geometry::geometry::logical_size::ToPhysicalSize;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::appearance::AppearanceValue;
use layoutng_style::style::computed_style::{
    ComputedStyle as NativeComputedStyle, ComputedStyleBuilder,
};
use layoutng_style::style::computed_style_constants::{EVerticalAlign, PseudoId};
use layoutng_style::style::gap_data_list::GapDataListIterator;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;
use layoutng_style::text_transform_services::{
    NativeTextTransformResolver, NativeTextTransformResolverScope,
};

use crate::internal::boundary::native_input::{CurrentNativeFontResolver, NativeFontResolverScope};
use crate::internal::constraint_space::ConstraintSpace as NativeConstraintSpace;
use crate::internal::form_control_types::AutofillState;
use crate::internal::fragmentation_utils::{
    FindPreviousBreakToken, IsBreakInside, OffsetInStitchedFragments,
};
use crate::internal::gap::gap_geometry::ContainerType;
use crate::internal::layout_block_flow::LayoutBlockFlow;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_box_model_object::LayoutBoxModelObject;
use crate::internal::layout_font_resolver::{NativeFontRequest, NativeFontResolver};
use crate::internal::layout_inline::LayoutInline;
use crate::internal::layout_input::{
    BorderLineStyle, BoxDecorationBreak, ComputedStyle, ConstraintSpace, Display, Edges,
    ExtendedStyle, FontSmoothing, FontVariation, HyphenationProvider, NativeNodeConstructionData,
    NodeKind, Offset, PaintImage, PaintStyle, PhraseBreakProvider, Position, Size,
    TextDecorationPaint, TextDecorationStyle, TextDirection, TextTransform, TextTransformProvider,
    TransformMatrix, WritingMode,
};
use crate::internal::layout_invalidation_reason::{kFontsChanged, kStyleChange};
use crate::internal::layout_node_metadata::{Element, NativeNodeMetadataRelations, Node};
use crate::internal::layout_object::ApplyStyleChanges;
use crate::internal::layout_object::LayoutObject;
use crate::internal::layout_object_builder::NodeInputUpdate;
use crate::internal::layout_object_factory_set::{LayoutObjectFactory, LayoutObjectFactorySet};
use crate::internal::layout_pass_scope::LayoutPassScope;
use crate::internal::layout_replaced::LayoutReplaced;
use crate::internal::layout_text::LayoutText;
use crate::internal::layout_text_combine::LayoutTextCombine;
use crate::internal::layout_view::LayoutView;
use crate::internal::paint_input::PaintBlendMode;
use crate::internal::relative_utils::ComputeRelativeOffset;
use crate::internal::scroll_types::IncludeScrollbarsInRect;
use crate::internal::scrollbar_theme_metrics::ScrollbarThemeThickness;
use crate::internal::shapes::shape_image_services::{
    NativeShapeImageResolver, NativeShapeImageResolverScope,
};
use crate::internal::shapes::shape_outside_info::ShapeOutsideInfo;
use crate::internal::table_borders::EdgeSide;
use crate::layout_boundary_support::LayoutBoundaryEnvironment;

// Paint export retains an optical axis selected implicitly by the native face.
const OPTICAL_SIZE_TAG: u32 = u32::from_be_bytes(*b"opsz");

#[path = "../../../../layout_reuse/src/native_layout_environment_cache.rs"]
mod layout_reuse_native_layout_environment_cache;
use layout_reuse_native_layout_environment_cache::FontBinding;
pub(crate) use layout_reuse_native_layout_environment_cache::ReusableLayoutEnvironment;
#[path = "../../../../layout_algo_float/src/native_shape_image_environment.rs"]
mod layout_algo_float_native_shape_image_environment;
use layout_algo_float_native_shape_image_environment::InputShapeImageResolver;
#[path = "../../../../layout_algo_inline/src/native_text_input_environment.rs"]
mod layout_algo_inline_native_text_input_environment;
use layout_algo_inline_native_text_input_environment::{
    InputHyphenationResolver, InputPhraseBreakResolver, InputTextTransformResolver,
};
#[path = "native_layout_environment.rs"]
mod layout_boundary_native_layout_environment;
pub(crate) use layout_boundary_native_layout_environment::NativeLayoutEnvironment;

#[path = "../../../../layout_reuse/src/native_font_resolver.rs"]
mod layout_reuse_native_font_resolver;
pub(crate) use layout_reuse_native_font_resolver::ConstructionFontResolver;
use layout_reuse_native_font_resolver::EngineFontResolver;
#[path = "../../../../layout_invalidation/src/native_layout_object_factory.rs"]
mod layout_invalidation_native_layout_object_factory;
pub(crate) use layout_invalidation_native_layout_object_factory::CreateLayoutObject;
#[path = "../../../../layout_fragment/src/native_fragment_export_support.rs"]
mod layout_fragment_native_fragment_export_support;
use layout_fragment_native_fragment_export_support::{
    DebugNameCache, ExportAppliedTextDecorations, ExportedStyleCache, InputId, Number,
    PublicWritingMode, SourceEntry, SourceMap, StyleSourceMap, TraceInlineExport,
};
#[path = "../../../../layout_fragment/src/native_fragment_property_export.rs"]
mod layout_fragment_native_fragment_property_export;
use layout_fragment_native_fragment_property_export::{ExportPaintProperties, ExportProperties};
#[path = "../../../../layout_fragment/src/native_text_paint_export.rs"]
mod layout_fragment_native_text_paint_export;
pub(crate) use layout_fragment_native_text_paint_export::{
    BuildPaintGlyphRuns, NativeTextPaintBaseline,
};
use layout_fragment_native_text_paint_export::{
    ExportListMarkerSymbol, ExportShapeResult, ExportTextCombineEmphasis,
    ExportTextCombineInlineOffset,
};
#[path = "../../../../layout_fragment/src/native_fragment_geometry_export.rs"]
mod layout_fragment_native_fragment_geometry_export;
use layout_fragment_native_fragment_geometry_export::{
    ExportBoxPaintGeometry, ExportContentSize, ExportSvgTextPaintData,
};
#[path = "../../../../layout_fragment/src/native_scrollbar_paint_export.rs"]
mod layout_fragment_native_scrollbar_paint_export;
use layout_fragment_native_scrollbar_paint_export::ExportScrollbarPaintData;
#[path = "../../../../layout_fragment/src/native_specialized_paint_export.rs"]
mod layout_fragment_native_specialized_paint_export;
use layout_fragment_native_specialized_paint_export::{
    ExportColumnRulePaintData, ExportFieldsetPaintData, ExportFrameSetPaintData,
    ExportMathMLPaintData, ExportOverflowClipMargin, ExportTablePaintData,
};
#[path = "../../../../layout_fragment/src/native_collapsed_table_paint_export.rs"]
mod layout_fragment_native_collapsed_table_paint_export;
use layout_fragment_native_collapsed_table_paint_export::ExportCollapsedTablePaintData;
#[path = "../../../../layout_fragment/src/native_fragment_export_state.rs"]
mod layout_fragment_native_fragment_export_state;
use layout_fragment_native_fragment_export_state::{
    AddPhysicalOffsets, BuildPaintCullRect, FragmentOwners, MarkUncullablePaintAncestors,
    PaintCullRect, PopulateSVGChildPaintData, ShouldCullPhysicalFragment, SubtractScrollOffset,
};
#[path = "../../../../layout_fragment/src/native_ink_overflow_export.rs"]
mod layout_fragment_native_ink_overflow_export;
use layout_fragment_native_ink_overflow_export::{
    ExportBoxInkOverflow, ExportInlineInkOverflow, ExportNativeInkRect,
};
#[path = "../../../../layout_fragment/src/native_nine_piece_image_export.rs"]
mod layout_fragment_native_nine_piece_image_export;
use layout_fragment_native_nine_piece_image_export::ExportNativeNinePieceImages;
#[path = "../../../../layout_fragment/src/native_fragment_paint_population.rs"]
mod layout_fragment_native_fragment_paint_population;
use layout_fragment_native_fragment_paint_population::PopulatePhysicalFragmentPaintData;
#[path = "../../../../layout_fragment/src/native_global_paint_export.rs"]
mod layout_fragment_native_global_paint_export;
use layout_fragment_native_global_paint_export::{
    ExportCanvasBackgroundNative, ExportGlobalMainThreadScrollFactsNative,
};
#[path = "../../../../layout_fragment/src/native_inline_fragment_policy.rs"]
mod layout_fragment_native_inline_fragment_policy;
pub(crate) use layout_fragment_native_inline_fragment_policy::{
    NativeStyleRequiresPaintBackedInlineFragment, RequiresPaintBackedInlineFragment,
};
#[path = "../../../../layout_fragment/src/native_scrollbar_thumb_geometry.rs"]
mod layout_fragment_native_scrollbar_thumb_geometry;
use layout_fragment_native_scrollbar_thumb_geometry::SetScrollbarThumb;
#[path = "../../../../layout_fragment/src/native_fragment_relationship_export.rs"]
mod layout_fragment_native_fragment_relationship_export;
use layout_fragment_native_fragment_relationship_export::{
    PopulateNativeInlineStitchedDecorations, ResolveNativeLogicalParentOccurrences,
};
#[path = "../../../../layout_fragment/src/native_sticky_paint_export.rs"]
mod layout_fragment_native_sticky_paint_export;
use layout_fragment_native_sticky_paint_export::ExportStickyPaintProperties;
pub(crate) fn UpdateStickyOffsets(
    objects: impl IntoIterator<Item = *mut LayoutObject>,
    reuse_constraints: bool,
) {
    crate::layout_fragment_native_sticky::update_sticky_offsets(objects, reuse_constraints)
}

// Scroll-only paint-property update, corresponding to Blink's scroll translation
// and sticky property update. The caller has verified unchanged LayoutResult;
// styles, fragments, clips and resources keep their existing owned snapshot.
#[path = "../../../../layout_fragment/src/native_scroll_refresh.rs"]
mod layout_fragment_native_scroll_refresh;

pub fn RefreshScrollPaintProperties(
    root: &mut LayoutObject,
    fragments: crate::layout_engine::NativePaintFragmentCursor<'_>,
    reuse_sticky_constraints: bool,
) {
    layout_fragment_native_scroll_refresh::refresh_scroll_paint_properties(
        root,
        fragments,
        reuse_sticky_constraints,
    )
}

#[path = "../../../../layout_fragment/src/native_scroll_paint_property_index.rs"]
mod layout_fragment_native_scroll_paint_property_index;
pub(crate) use layout_fragment_native_scroll_paint_property_index::ScrollPaintPropertyIndex;

#[path = "../../../../layout_invalidation/src/native_prepare_tree.rs"]
mod layout_invalidation_native_prepare_tree;

#[path = "../../../../layout_algo_inline/src/native_initial_letter_style.rs"]
mod layout_algo_inline_native_initial_letter_style;
use layout_algo_inline_native_initial_letter_style::InitialLetterTextStyle;
// cpp: layoutng/internal/boundary/layout_boundary.h:34-40
// cpp: layoutng/internal/boundary/layout_boundary.cc:2755-2820
pub fn PrepareLayoutObjectTree(
    root: &mut LayoutObject,
    space: &ConstraintSpace,
    native_space: &NativeConstraintSpace,
    reuse_prepared_fonts: bool,
    relations_dirty: bool,
    shape_image_inputs_dirty: bool,
    document_image_container_dirty: bool,
    targeted_font_objects: Option<&[*mut LayoutObject]>,
) {
    layout_invalidation_native_prepare_tree::prepare_layout_object_tree(
        root,
        space,
        native_space,
        reuse_prepared_fonts,
        relations_dirty,
        shape_image_inputs_dirty,
        document_image_container_dirty,
        targeted_font_objects,
    )
}

pub fn PrepareLayoutObjectTreeDefault(
    root: &mut LayoutObject,
    space: &ConstraintSpace,
    native_space: &NativeConstraintSpace,
) {
    PrepareLayoutObjectTree(root, space, native_space, false, true, true, true, None)
}

#[path = "../../../../layout_fragment/src/native_layout_result_publish.rs"]
mod layout_fragment_native_layout_result_publish;
#[path = "../../../../layout_fragment/src/native_retained_paint_refresh.rs"]
mod layout_fragment_native_retained_paint_refresh;

/// Refresh local paint inputs from existing native objects, retaining layout
/// geometry, shaping, physical fragment identities and resource ownership.
/// Special geometry clients retain ordinary export until its proof is available.
pub(crate) fn RefreshRetainedPaintInputs(
    root: &mut LayoutObject,
    fragments: crate::layout_engine::NativePaintFragmentCursor<'_>,
    changed: &HashSet<u64>,
    inline_ink_changes: &HashSet<u64>,
) -> bool {
    layout_fragment_native_retained_paint_refresh::refresh_retained_paint_inputs(
        root,
        fragments,
        changed,
        inline_ink_changes,
    )
}
// cpp: layoutng/internal/boundary/layout_boundary.h:41-43
// cpp: layoutng/internal/boundary/layout_boundary.cc:2822-2858
pub fn PublishLayoutResult(
    root: &mut LayoutObject,
    result: &LayoutResult,
    space: &ConstraintSpace,
    cache: &mut LayoutBoundaryCache,
) -> LayoutResultHandle {
    layout_fragment_native_layout_result_publish::populate_layout_result_paint_data(
        root,
        result,
        space,
        LayoutUpdateKind::Paint,
        cache,
    )
}

#[path = "../../../../layout_fragment/src/native_paint_cache_refresh.rs"]
mod layout_fragment_native_paint_cache_refresh;

pub(crate) fn RefreshCompatibleNodeInputCache(
    cache: &mut LayoutBoundaryCache,
    update: &NodeInputUpdate,
) {
    layout_fragment_native_paint_cache_refresh::refresh_compatible_node_input_cache(cache, update)
}

pub(crate) fn RefreshCompatiblePaintCache(cache: &mut LayoutBoundaryCache) {
    layout_fragment_native_paint_cache_refresh::refresh_compatible_paint_cache(cache)
}
/// Rebuild the stable document-order indices which Chromium gets directly
/// from its resident LayoutObject graph. After a topology mutation this
/// restores the document-wide boundary indices before a later scroll cull
/// update needs them.
pub(crate) fn EnsureLayoutBoundaryObjectIndex(
    root: &mut LayoutObject,
    cache: &mut LayoutBoundaryCache,
) {
    crate::layout_fragment_native::ensure_layout_boundary_object_index(root, cache)
}
