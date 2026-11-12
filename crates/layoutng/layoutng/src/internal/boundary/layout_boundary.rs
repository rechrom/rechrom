#![allow(non_snake_case)]

use crate::layout_boundary_support::{
    FragmentIdentityKey, LayoutBoundaryCache, LayoutBoundaryHashMap, LayoutBoundaryHashSet,
    LayoutBoundarySourceEntry, LayoutBoundaryStyleSourceEntry, LayoutUpdateKind,
};
use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::Arc;

use font_engine::fonts::canvas_rotation_in_vertical::IsCanvasRotationInVerticalUpright;
use font_engine::fonts::opentype::open_type_math_support::{MathConstants, OpenTypeMathSupport};
use font_engine::text::implementation::hyphenation::HyphenationBackend;
use font_engine::text::implementation::hyphenation_services::{
    HyphenationResolver, HyphenationResolverScope,
};
use font_engine::text::implementation::opentype_font::FontMetric;
use font_engine::text::implementation::phrase_break_services::{
    PhraseBreakResolver, PhraseBreakResolverScope,
};
use font_engine::{
    CanvasRotationInVertical, Font, FontBaseline, FontDescription, FontOrientation,
    FontSmoothingMode, HarfBuzzShaper, Hyphenation, LayoutLocale, ShapeResultView, SimpleFontData,
};
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::{
    gfx, AtomicString, DynamicTo, EBorderStyle, EBoxDecorationBreak, EDisplay, EOverscrollBehavior,
    ETextDecorationSkipInk, ETextDecorationStyle as StyleTextDecorationStyle, ETextTransform,
    LayoutUnit, Length, MakeGarbageCollected, Member, Persistent, PhysicalOffset, PhysicalRect,
    PhysicalSize, String as BlinkString, TextDecorationLine, TextEmphasisMark, TextOffsetMap, To,
    UnsupportedLayout, WritingMode as StyleWritingMode,
};
use layout_fragment::block_break_token::BlockBreakToken;
use layout_fragment::fragment_item::{FragmentItem, ItemType};
use layout_fragment::fragment_tree::{
    BackgroundPaintLocation, CanvasBackgroundPaintData, CollapsedTableEdge,
    CollapsedTablePaintData, CollapsedTableSection, ColumnRule, ColumnRulePaintData,
    FieldsetPaintData, FormControlPaintData, FragmentBoxSides, FragmentKind, FragmentPaintData,
    FragmentPaintRect, FrameSetPaintData, GapRule, GapRulePaintData, GapRuleSide,
    LayoutResultHandle, MathMLPaintData, MathMLPaintKind, PaintGlyph, PaintGlyphRun,
    PaintProperties, PaintPropertiesHandle, PaintResources, PrePaintInvalidation,
    ReplacedContentPaintData, ScrollContainerPaintData, ScrollSnapAxisPaintData,
    ScrollSnapStrictnessPaintData, ScrollSnapTypePaintData, ScrollbarPaintAxis, ScrollbarPaintData,
    SelfOutlinePaintData, StickyAxisPaintData, StickyPositionPaintData, StitchedDecorationData,
    SvgTextPaintData, TablePaintColumn, TablePaintData, TextCombineEmphasisPaintData,
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
use layoutng_style::style::applied_text_decoration::AppliedTextDecoration;
use layoutng_style::style::computed_style::{
    ComputedStyle as StyleComputedStyle, ComputedStyleBuilder,
};
use layoutng_style::style::computed_style_constants::{EFillBox, EVerticalAlign, PseudoId};
use layoutng_style::style::gap_data_list::GapDataListIterator;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;
use layoutng_style::text_transform_services::{TextTransformResolver, TextTransformResolverScope};

use crate::internal::boundary::input::{CurrentFontResolver, FontResolverScope};
use crate::internal::constraint_space::ConstraintSpace as LayoutConstraintSpace;
use crate::internal::form_control_types::{AutofillState, FormControlType};
use crate::internal::fragmentation_utils::{
    FindPreviousBreakToken, IsBreakInside, OffsetInStitchedFragments,
};
use crate::internal::gap::gap_geometry::ContainerType;
use crate::internal::layout_block::LayoutBlock;
use crate::internal::layout_block_flow::LayoutBlockFlow;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_box_model_object::LayoutBoxModelObject;
use crate::internal::layout_font_resolver::{FontRequest, FontResolver};
use crate::internal::layout_inline::LayoutInline;
use crate::internal::layout_input::{
    BorderLineStyle, BoxDecorationBreak, ComputedStyle, ConstraintSpace, Display, Edges,
    ExtendedStyle, FontSmoothing, FontVariation, HyphenationProvider, NodeConstructionData,
    NodeKind, Offset, PaintImage, PaintStyle, PhraseBreakProvider, Position, Size,
    TextDecorationPaint, TextDecorationStyle, TextDirection, TextTransform, TextTransformProvider,
    TransformMatrix, WritingMode,
};
use crate::internal::layout_invalidation_reason::{kFontsChanged, kStyleChange};
use crate::internal::layout_node_metadata::{Element, Node, NodeMetadataRelations};
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
use crate::internal::shapes::shape_image_services::{ShapeImageResolver, ShapeImageResolverScope};
use crate::internal::shapes::shape_outside_info::ShapeOutsideInfo;
use crate::internal::table_borders::EdgeSide;
use crate::layout_boundary_support::LayoutBoundaryEnvironment;

// Paint export retains an optical axis selected implicitly by the source face.
const OPTICAL_SIZE_TAG: u32 = u32::from_be_bytes(*b"opsz");

#[path = "../../../../layout_reuse/src/layout_environment_cache.rs"]
mod layout_reuse_layout_environment_cache;
use layout_reuse_layout_environment_cache::FontBinding;
pub(crate) use layout_reuse_layout_environment_cache::ReusableLayoutEnvironment;
#[path = "../../../../layout_algo_float/src/shape_image_environment.rs"]
mod layout_algo_float_shape_image_environment;
use layout_algo_float_shape_image_environment::InputShapeImageResolver;
#[path = "../../../../layout_algo_inline/src/text_input_environment.rs"]
mod layout_algo_inline_text_input_environment;
use layout_algo_inline_text_input_environment::{
    InputHyphenationResolver, InputPhraseBreakResolver, InputTextTransformResolver,
};
#[path = "layout_environment.rs"]
mod layout_boundary_layout_environment;
pub(crate) use layout_boundary_layout_environment::LayoutEnvironment;

#[path = "../../../../layout_reuse/src/font_resolver.rs"]
mod layout_reuse_font_resolver;
pub(crate) use layout_reuse_font_resolver::ConstructionFontResolver;
use layout_reuse_font_resolver::EngineFontResolver;
#[path = "../../../../layout_invalidation/src/layout_object_factory.rs"]
mod layout_invalidation_layout_object_factory;
pub(crate) use layout_invalidation_layout_object_factory::CreateLayoutObject;
#[path = "../../../../layout_fragment/src/fragment_export_support.rs"]
mod layout_fragment_fragment_export_support;
pub(crate) use layout_fragment_fragment_export_support::ExportAppliedTextDecoration;
use layout_fragment_fragment_export_support::{
    DebugNameCache, ExportAppliedTextDecorations, ExportSelectionPaintStyle,
    ExportSelectionTextDecoration, ExportedSelectionStyleCache, ExportedStyleCache, InputId,
    Number, PublicWritingMode, SourceEntry, SourceMap, StyleSourceEntry, StyleSourceMap,
};
#[path = "../../../../layout_fragment/src/fragment_property_export.rs"]
mod layout_fragment_fragment_property_export;
pub(crate) use layout_fragment_fragment_property_export::ExportSelectionPrePaintInvalidation;
use layout_fragment_fragment_property_export::{
    ExportPaintProperties, ExportPrePaintInvalidation, ExportProperties, PropertyFragmentUniqueId,
};
#[path = "../../../../layout_fragment/src/text_paint_export.rs"]
mod layout_fragment_text_paint_export;
pub(crate) use layout_fragment_text_paint_export::{BuildPaintGlyphRuns, ExportTextPaintBaseline};
use layout_fragment_text_paint_export::{
    ExportListMarkerSymbol, ExportShapeResult, ExportTextCombineEmphasis,
    ExportTextCombineInlineOffset,
};
#[path = "../../../../layout_fragment/src/fragment_geometry_export.rs"]
mod layout_fragment_fragment_geometry_export;
use layout_fragment_fragment_geometry_export::{
    ExportBoxPaintGeometry, ExportContentSize, ExportReplacedContentPaintData,
    ExportSelfOutlineGeometry, ExportSvgTextPaintData,
};
#[path = "../../../../layout_fragment/src/scrollbar_paint_export.rs"]
mod layout_fragment_scrollbar_paint_export;
use layout_fragment_scrollbar_paint_export::ExportScrollbarPaintData;
#[path = "../../../../layout_fragment/src/specialized_paint_export.rs"]
mod layout_fragment_specialized_paint_export;
use layout_fragment_specialized_paint_export::{
    ExportColumnRulePaintData, ExportFieldsetPaintData, ExportFrameSetPaintData,
    ExportGapRulePaintData, ExportMathMLPaintData, ExportOverflowClipMargin, ExportTablePaintData,
};
#[path = "../../../../layout_fragment/src/collapsed_table_paint_export.rs"]
mod layout_fragment_collapsed_table_paint_export;
use layout_fragment_collapsed_table_paint_export::ExportCollapsedTablePaintData;
#[path = "../../../../layout_fragment/src/fragment_export_state.rs"]
mod layout_fragment_fragment_export_state;
use layout_fragment_fragment_export_state::{FragmentOwners, PopulateSVGChildPaintData};
#[path = "../../../../layout_fragment/src/ink_overflow_export.rs"]
mod layout_fragment_ink_overflow_export;
use layout_fragment_ink_overflow_export::{
    ExportBoxInkOverflow, ExportInkRect, ExportInlineInkOverflow,
};
#[path = "../../../../layout_fragment/src/nine_piece_image_export.rs"]
mod layout_fragment_nine_piece_image_export;
use layout_fragment_nine_piece_image_export::ExportNinePieceImages;
#[path = "../../../../layout_fragment/src/fragment_paint_population.rs"]
mod layout_fragment_fragment_paint_population;
use layout_fragment_fragment_paint_population::PopulatePhysicalFragmentPaintData;
#[path = "../../../../layout_fragment/src/global_paint_export.rs"]
mod layout_fragment_global_paint_export;
use layout_fragment_global_paint_export::{
    ExportCanvasBackground, ExportGlobalMainThreadScrollFacts,
};
#[path = "../../../../layout_fragment/src/inline_fragment_paint_policy.rs"]
mod layout_fragment_inline_fragment_paint_policy;
pub(crate) use layout_fragment_inline_fragment_paint_policy::{
    ComputedStyleRequiresPaintBackedInlineFragment, RequiresPaintBackedInlineFragment,
};
#[path = "../../../../layout_fragment/src/scrollbar_thumb_geometry.rs"]
mod layout_fragment_scrollbar_thumb_geometry;
use layout_fragment_scrollbar_thumb_geometry::SetScrollbarThumb;
#[path = "../../../../layout_fragment/src/fragment_relationship_export.rs"]
mod layout_fragment_fragment_relationship_export;
use layout_fragment_fragment_relationship_export::{
    PopulateInlineStitchedDecorations, ResolveLogicalParentOccurrences,
};
#[path = "../../../../layout_fragment/src/sticky_paint_export.rs"]
mod layout_fragment_sticky_paint_export;
use layout_fragment_sticky_paint_export::ExportStickyPaintProperties;
pub(crate) fn UpdateStickyOffsets(
    objects: impl IntoIterator<Item = *mut LayoutObject>,
    reuse_constraints: bool,
) {
    crate::layout_fragment_sticky_offsets::update_sticky_offsets(objects, reuse_constraints)
}

// Scroll-only paint-property update, corresponding to Blink's scroll translation
// and sticky property update. The caller has verified unchanged LayoutResult;
// styles, fragments, clips and resources keep their existing owned snapshot.
#[path = "../../../../layout_fragment/src/scroll_paint_property_refresh.rs"]
mod layout_fragment_scroll_paint_property_refresh;

pub fn RefreshScrollPaintProperties(
    root: &mut LayoutObject,
    fragments: crate::layout_engine::PaintFragmentCursor<'_>,
    reuse_sticky_constraints: bool,
) {
    layout_fragment_scroll_paint_property_refresh::refresh_scroll_paint_properties(
        root,
        fragments,
        reuse_sticky_constraints,
    )
}

#[path = "../../../../layout_fragment/src/scroll_paint_property_index.rs"]
mod layout_fragment_scroll_paint_property_index;
pub(crate) use layout_fragment_scroll_paint_property_index::ScrollPaintPropertyIndex;

#[path = "../../../../layout_invalidation/src/prepare_tree.rs"]
mod layout_invalidation_prepare_tree;

#[path = "../../../../layout_algo_inline/src/initial_letter_style.rs"]
mod layout_algo_inline_initial_letter_style;
use layout_algo_inline_initial_letter_style::InitialLetterTextStyle;
// cpp: layoutng/internal/boundary/layout_boundary.h:34-40
// cpp: layoutng/internal/boundary/layout_boundary.cc:2755-2820
pub fn PrepareLayoutObjectTree(
    root: &mut LayoutObject,
    space: &ConstraintSpace,
    layout_space: &LayoutConstraintSpace,
    reuse_prepared_fonts: bool,
    relations_dirty: bool,
    shape_image_inputs_dirty: bool,
    document_image_container_dirty: bool,
    targeted_font_objects: Option<&[*mut LayoutObject]>,
) {
    layout_invalidation_prepare_tree::prepare_layout_object_tree(
        root,
        space,
        layout_space,
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
    layout_space: &LayoutConstraintSpace,
) {
    PrepareLayoutObjectTree(root, space, layout_space, false, true, true, true, None)
}

#[path = "../../../../layout_fragment/src/layout_result_publish.rs"]
mod layout_fragment_layout_result_publish;
#[path = "../../../../layout_fragment/src/retained_paint_refresh.rs"]
mod layout_fragment_retained_paint_refresh;

/// Refresh local paint inputs from existing source objects, retaining layout
/// geometry, shaping, physical fragment identities and resource ownership.
/// Special geometry clients retain ordinary export until its proof is available.
pub(crate) fn RefreshRetainedPaintInputs(
    root: &mut LayoutObject,
    fragments: crate::layout_engine::PaintFragmentCursor<'_>,
    changed: &HashSet<u64>,
    inline_ink_changes: &HashSet<u64>,
    should_apply_root_background_behavior: bool,
) -> bool {
    layout_fragment_retained_paint_refresh::refresh_retained_paint_inputs(
        root,
        fragments,
        changed,
        inline_ink_changes,
        should_apply_root_background_behavior,
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
    layout_fragment_layout_result_publish::populate_layout_result_paint_data(
        root,
        result,
        space,
        LayoutUpdateKind::Paint,
        cache,
    )
}

#[path = "../../../../layout_fragment/src/paint_cache_refresh.rs"]
mod layout_fragment_paint_cache_refresh;

pub(crate) fn RefreshCompatibleNodeInputCache(
    cache: &mut LayoutBoundaryCache,
    update: &NodeInputUpdate,
) {
    layout_fragment_paint_cache_refresh::refresh_compatible_node_input_cache(cache, update)
}

pub(crate) fn RefreshCompatiblePaintCache(cache: &mut LayoutBoundaryCache) {
    layout_fragment_paint_cache_refresh::refresh_compatible_paint_cache(cache)
}
/// Rebuild the stable document-order indices which Chromium gets directly
/// from its resident LayoutObject graph. After a topology mutation this
/// restores the document-wide boundary indices before a later scroll cull
/// update needs them.
pub(crate) fn EnsureLayoutBoundaryObjectIndex(
    root: &mut LayoutObject,
    cache: &mut LayoutBoundaryCache,
) {
    crate::layout_fragment_object_index::ensure_layout_boundary_object_index(root, cache)
}
