use std::sync::{Arc, OnceLock};

pub use crate::boundary_types::{
    BackgroundPaintLocation, CollapsedTableEdge, CollapsedTablePaintData, CollapsedTableSection,
    ColumnRule, ColumnRulePaintData, FieldsetPaintData, FormControlPaintData, FragmentBoxSides,
    FragmentKind, FragmentPaintRect, FrameSetPaintData, GapRule, GapRulePaintData, GapRuleSide,
    MathMLPaintData, MathMLPaintKind,
    PaintGlyph, PaintGlyphRun, PaintResources, PrePaintInvalidation, PreviousBoxGeometry,
    ReplacedContentPaintData, ScrollContainerPaintData, ScrollSnapAxisPaintData,
    ScrollSnapStrictnessPaintData, ScrollSnapTypePaintData, ScrollbarPaintAxis, ScrollbarPaintData,
    SelfOutlinePaintData, StickyAxisPaintData, StickyPositionPaintData, StitchedDecorationData,
    SvgTextPaintData, SvgViewportGeometry, TextCombineEmphasisPaintData, TextCombinePaintData,
};
pub use crate::layout_result_handle::LayoutResultHandle;

pub mod paint_resource_usage;

pub use foundation::graphics_types::graphics::paint::display_item_client_types::RasterEffectOutset;
use foundation::{LayoutOffset as Offset, LayoutSize as Size};
use crate::internal::layout_box_model_object::LayoutBoxModelObject;
use crate::internal::layout_input::{
    BorderLineStyle, BoxDecorationBreak, Display, Edges, FloatSide, FocusedImageMapArea,
    ListStyleType, NodeKind, Overflow, OverscrollBehavior, PaintStyle, Position, SvgShapeData,
    SvgViewBoxData, TextDecorationPaint, TextDirection, TransformMatrix, WritingMode,
};
use crate::internal::layout_object::LayoutObject;
use crate::internal::paint_layer::PaintLayer;
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::PaintInvalidationReason;
use crate::internal::layout_text::LayoutText;

/// A DisplayItemClient's PaintController-visible state, read from the live
/// source client (or from a detached payload's stored state).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintClientState {
    pub invalidation_reason: Option<PaintInvalidationReason>,
    pub is_cacheable: bool,
    pub is_just_created: bool,
}

/// The overflow-control DisplayItemClients of a box (scroll corner,
/// horizontal and vertical scrollbar), each with its live state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollbarClients {
    pub corner: Option<(u64, PaintClientState)>,
    pub horizontal: Option<(u64, PaintClientState)>,
    pub vertical: Option<(u64, PaintClientState)>,
}
use crate::internal::selection_state::SelectionState;

/// Opaque reference to a resident source LayoutObject that PrePaint retains
/// only until the end of its current walk: PaintInvalidator's
/// `pending_delayed_paint_invalidations_` (core/paint/paint_invalidator.h).
/// The LayoutObject tree keeps the object live for the synchronous walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrePaintSourceObject(*mut LayoutObject);

impl PrePaintSourceObject {
    /// PaintInvalidator::ProcessPendingDelayedPaintInvalidations
    /// (core/paint/paint_invalidator.cc:291-294).
    pub fn SetShouldDelayFullPaintInvalidation(self) {
        unsafe { &mut *self.0 }
            .GetMutableForPainting()
            .SetShouldDelayFullPaintInvalidation();
    }
}

thread_local! {
    static RESIDENT_PAINT_SOURCE_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Paint's tree-walk scope over one leased resident Layout result.
///
/// Chromium's painters read `PhysicalFragment::layout_object_` and the
/// object's ComputedStyle directly while LocalFrameView::PaintTree walks the
/// PaintLayer tree (core/frame/local_frame_view.cc:3183-3226): no layout
/// object is destroyed during the walk, and every fragment it reaches is the
/// live tree's. While this scope is open, PaintProperties reads its bound
/// source LayoutObject without the per-call heap membership query; the
/// query remains for payloads read outside a walk. Sequence-confined: the
/// scope is neither Send nor Sync.
pub struct ResidentPaintSourceScope {
    _sequence: std::marker::PhantomData<*const ()>,
}

impl ResidentPaintSourceScope {
    pub fn new() -> Self {
        RESIDENT_PAINT_SOURCE_DEPTH.with(|depth| depth.set(depth.get() + 1));
        Self {
            _sequence: std::marker::PhantomData,
        }
    }

    /// Whether a paint tree walk is open on this sequence.
    #[inline]
    pub(crate) fn IsActive() -> bool {
        RESIDENT_PAINT_SOURCE_DEPTH.with(|depth| depth.get() != 0)
    }
}

impl Drop for ResidentPaintSourceScope {
    fn drop(&mut self) {
        RESIDENT_PAINT_SOURCE_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

// The C++ definitions are public API; all crate::* names above retain their
// owner in the later layoutng crate. The Cargo cycle needs an interface split.

// cpp: layoutng_fragment_tree/fragment_tree.h:25
// Defaulted C++ equality maps to the derived fieldwise comparison above.

// cpp: layoutng_fragment_tree/fragment_tree.h:131
// Defaulted C++ equality maps to the derived fieldwise comparison above.

/// Assembly specialization of fragment-owned table geometry with LayoutNG's
/// immutable paint-style snapshot. Existing consumers keep the original API.
pub type TablePaintColumn = crate::boundary_types::TablePaintColumn<PaintStyle>;
pub type TablePaintData = crate::boundary_types::TablePaintData<PaintStyle>;

// The source fragment is the complete paint input. Shared payloads use Arc
// because C++ shared_ptr<const T> is shared immutable ownership.
// cpp: layoutng_fragment_tree/fragment_tree.h:32-73
// cpp: layoutng_fragment_tree/fragment_tree.h:82-114
// cpp: layoutng_fragment_tree/fragment_tree.h:119-120
// cpp: layoutng_fragment_tree/fragment_tree.h:281-301
/// CSS canvas background selected by Layout from the document element or its
/// first body element.  It is owned by the root fragment because Paint must
/// cover the viewport and must not repaint the source element's own box.
///
/// This is the package-boundary equivalent of Blink's propagated LayoutView
/// background.  Keeping the selected immutable style here avoids asking Paint
/// to rediscover DOM roles or containment rules.
pub type CanvasBackgroundPaintData =
    crate::boundary_types::CanvasBackgroundPaintData<PaintStyle>;
/// The same selection read live from the resident root/body fragments
/// (PaintFragmentCursor::ViewBackground / CanvasBackground); the style is the
/// selected fragment's own PaintStyle projection, borrowed, not copied.
pub type CanvasBackgroundPaintView<'a> =
    crate::boundary_types::CanvasBackgroundPaintData<&'a PaintStyle>;

/// BoxBackgroundPaintContext state for a page border box painting the
/// document canvas. The positioning size belongs to the LayoutView root while
/// the offset selects this page inside the stitched paginated document.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PageBackgroundPaintData {
    pub positioning_size: Size,
    pub positioning_area_offset: Offset,
    pub border: Edges,
    pub padding: Edges,
    pub has_background_fixed_to_viewport: bool,
}

/// BoxBackgroundPaintContext state for the `@page` background painted by a
/// page-container fragment. The paint destination is the whole page
/// container, while image origin/border/padding geometry belongs to its page
/// border-box child.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PageContainerBackgroundPaintData {
    pub positioning_size: Size,
    pub root_element_paint_offset: Offset,
    pub positioning_box_offset: Offset,
    pub border: Edges,
    pub padding: Edges,
    pub has_background_fixed_to_viewport: bool,
}

/// Assembly specializations preserve the historical non-generic API while
/// fragment-local storage and sharing remain owned by `boundary_types`.
pub type PaintPropertiesHandle = crate::boundary_types::PaintPropertiesHandle<PaintProperties>;
pub type FragmentPaintData = crate::boundary_types::FragmentPaintData<PaintProperties>;

#[derive(Clone, PartialEq)]
pub struct PaintProperties {
    // Non-owning reference to the canonical LayoutObject. Resident fragment
    // leases keep the source layout result alive through PrePaint and Paint.
    // Production getters below read object facts through this identity rather
    // than copying them into the fragment boundary.
    source_object_address: usize,
    // Source: Chromium painters and PaintPropertyTreeBuilder read these
    // facts from LayoutObject::StyleRef() (e.g. paint_property_tree_builder.cc
    // OverflowClip/Scroll nodes, box_painter_base.cc backgrounds). True when
    // `style` is that live ComputedStyle; the stored copies are fixture-only.
    live_style_facts: bool,
    /// Chromium LayoutObject dirty bits consumed by Chromium's
    /// PrePaintTreeWalk. They are lifecycle facts, not drawing inputs: the
    /// layout boundary snapshots them before Paint and clears them only after
    /// the corresponding artifact commits.
    pub property_fragment_unique_id: u64,
    /// Chromium PhysicalBoxFragment::IsFirstForNode() and the canonical
    /// CanPaintMultipleFragments() result. Paint consumes these directly
    /// instead of duplicating LayoutObject-type fragmentation policy.
    pub is_first_for_node: bool,
    pub is_layout_view: bool,
    /// Exact LayoutObject/Document facts consumed by
    /// CullRectUpdater::ShouldUseInfiniteCullRect. PrePaint must not infer
    /// these from style or fragment geometry.
    pub is_printing: bool,
    /// Host Settings::GetShouldPrintBackgrounds() snapshot. This is distinct
    /// from printing itself because print-economy color rewriting is disabled
    /// when the caller explicitly requests backgrounds.
    pub should_print_backgrounds: bool,
    pub canvas_draw_element_enabled: bool,
    pub is_in_canvas_subtree: bool,
    pub is_overscroll_area_parent: bool,
    /// Source: PaintLayerPainter::Paint() early lifecycle admission.
    pub needs_layout: bool,
    pub child_layout_blocked_by_display_lock: bool,
    pub is_fragment_less_box: bool,
    /// LayoutObject::AdditionalCompositingReasons(), kept as the source bitset
    /// rather than reconstructed from style or scrolling geometry.
    pub additional_compositing_reasons: foundation::CompositingReasons,
    /// Canonical LayoutObject type facts used by
    /// FragmentPaintPropertyTreeBuilder::NeedsPaintOffsetTranslation. Paint
    /// must not reconstruct these from CSS display values: anonymous and SVG
    /// layout objects do not have a one-to-one display/type mapping.
    pub is_box_model_object: bool,
    pub is_layout_box: bool,
    /// PhysicalBoxFragment::kPageContainer. Chromium routes this anonymous
    /// fragment through ViewPainter rather than ordinary box decoration.
    /// The page box facts below are read live (ResidentBoxGeometry); these
    /// stored values serve only detached fixtures without a resident owner.
    pub is_page_container: bool,
    /// PhysicalBoxFragment::kPageBorderBox. This is fragment identity, not a
    /// LayoutObject/style proxy, and selects ScopedPaintState's document-space
    /// LocalBorderBoxProperties path while printing.
    pub is_page_border_box: bool,
    /// PhysicalBoxFragment::IsPaginatedRoot(). Chromium deliberately bypasses
    /// ink-overflow culling for this box so the document background reaches
    /// every printed page.
    pub is_paginated_root: bool,
    pub is_layout_inline: bool,
    pub should_clip_overflow_along_either_axis: bool,
    pub should_clip_overflow_along_both_axes: bool,
    pub is_layout_block_flow: bool,
    /// PhysicalBoxFragment::IsInlineFormattingContext(). Forced-colors
    /// backplates belong to the block that owns FragmentItems, not to an
    /// arbitrary ancestor reconstructed from the flattened paint tree.
    pub is_inline_formatting_context: bool,
    pub in_forced_colors_mode: bool,
    /// Fixture-only: production reads
    /// PaintProperties::RequiresMainThreadScrollingForBackgroundAttachmentFixed
    /// live from the LayoutView (local_frame_view.cc:1504-1527).
    pub requires_main_thread_scrolling_for_background_attachment_fixed: Option<bool>,
    /// PaintController snapshots the client's actual reason when appending an
    /// item (paint_controller.cc:531-545). None is an unexported external input.
    pub display_item_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub paint_layer_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    // Detached fixtures only: resident fragments and items read ink overflow
    // live (ResidentBoxGeometry::ViewInkOverflows). None means unset.
    pub self_ink_overflow: Option<FragmentPaintRect>,
    pub contents_ink_overflow: Option<FragmentPaintRect>,
    pub ink_overflow: Option<FragmentPaintRect>,
    /// LayoutBoxModelObject::ApplyFiltersToRect(InkOverflowRect()). This is the
    /// exact local-space admission rect used by BoxFragmentPainter::ShouldPaint.
    pub filtered_ink_overflow: Option<FragmentPaintRect>,
    /// CSSMaskPainter::MaskBoundingBox for ordinary CSS boxes and
    /// SVGForeignObject, in LayoutObject-local coordinates before the current
    /// FragmentData paint offset is applied. This is deliberately broader
    /// than an individual mask layer's clip box: Chromium uses LayersClipMax
    /// and includes mask-box-image outsets when building MaskClip.
    pub css_mask_bounding_box: Option<FragmentPaintRect>,
    // BoxPainter::VisualRect uses LayoutBox::SelfVisualOverflowRect(), which
    // is a stitched LayoutBox rect rather than a physical fragment ink rect.
    pub box_self_visual_overflow: Option<FragmentPaintRect>,
    /// Exact FragmentPainter outline geometry. Some(empty) is authoritative:
    /// Chromium returns without recording an outline for that fragment.
    pub self_outline: Option<SelfOutlinePaintData>,
    pub previous_box_geometry: Option<PreviousBoxGeometry>,
    pub box_fragment_is_inline_box: bool,
    // Chromium DisplayItemClient identity and lifecycle, independently of DOM
    // node ids and the source occurrence's fragment_instance_id.
    // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
    pub display_item_client_id: u64,
    // A distinct persistent source PaintLayer DisplayItemClient, when present.
    // cpp: core/paint/paint_layer.h:168-171
    /// Chromium layer ownership was queried, including the authoritative no-layer result.
    /// False is reserved for standalone fragments without a source Layout owner.
    pub paint_layer_state_is_authoritative: bool,
    pub paint_layer_client_id: u64,
    pub paint_layer_is_self_painting: bool,
    /// Exact inputs to PaintLayerPainter's PaintResult/cull admission. These
    /// are source LayoutObject/PaintLayer facts, not reconstructable from a
    /// physical fragment or projected CSS style.
    pub paint_layer_is_fragmented: bool,
    /// Source: PaintLayerPainter::FirstFragmentVisualRect(). This is the
    /// filtered, paint-offset, enclosing-pixel rect consumed by subsequence
    /// admission; Paint must not rebuild it from a fragment border box.
    pub paint_layer_first_fragment_visual_rect: Option<FragmentPaintRect>,
    /// LayoutBox::ContentsVisualOverflowRect moved by the first fragment's
    /// paint offset. Paint applies ReplacedContentTransform in its resident
    /// property tree when the object is an SVG replaced root.
    pub paint_layer_contents_visual_rect: Option<FragmentPaintRect>,
    /// PaintLayer::FilterReferenceBox(), in the layer's stitched local
    /// coordinate space. Chromium derives this from LocalBoundingBoxIncluding-
    /// SelfPaintingDescendants for pixel-moving filters; Paint must not
    /// substitute the current physical fragment's border box.
    pub paint_layer_filter_reference_box: Option<FragmentPaintRect>,
    /// PaintLayer::FilterViewport(), in the same stitched local coordinate
    /// system. `None` is meaningful for LayoutSVGForeignObject and must not be
    /// replaced by either the filter reference box or a physical fragment.
    pub paint_layer_filter_viewport: Option<Size>,
    /// LayoutObject::ChildPaintBlockedByDisplayLock(). This is an object fact,
    /// not a PaintLayer fact: LayoutImage consumes it even without a layer.
    pub child_paint_blocked_by_display_lock: bool,
    /// Canonical LayoutObject/PaintLayer stacking metadata. Paint must not
    /// reconstruct these from the projected style because source adjustment
    /// includes flex/grid items, containment and other grouping properties.
    pub paint_layer_is_stacking_context: bool,
    /// LayoutObject::IsStacked(). Chromium uses this independently of the
    /// effective z-index to separate normal-flow PaintLayers from the
    /// negative/non-negative stacking lists.
    pub paint_layer_is_stacked: bool,
    pub paint_layer_stacking_level: i32,
    /// UniqueId of the flex/grid ordering ancestor's parent. None means the
    /// PaintLayer is not a child/grandchild covered by OrderLessThan().
    pub paint_layer_order_parent_unique_id: Option<u64>,
    pub paint_layer_order: i32,
    pub display_item_client_is_cacheable: bool,
    pub display_item_client_is_just_created: std::cell::Cell<bool>,
    pub selection_client_id: u64,
    pub selection_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub selection_client_is_cacheable: bool,
    // The active ScopedDisplayItemFragment value: inline FragmentId(), layer
    // fragment index, or the containing fragmentainer's break-token identity.
    // cpp: core/layout/inline/fragment_item.h:172-188
    // cpp: core/paint/box_fragment_painter.cc:394-403
    pub display_item_fragment: u32,
    pub has_source: bool,
    /// LayoutObject::VisibleToHitTesting(), captured before the immutable
    /// Layout/Paint boundary. This includes visibility and used pointer-events.
    pub visible_to_hit_testing: bool,
    /// LayoutObject::EffectiveAllowedTouchAction(), including the blocking
    /// touch-handler override applied by LayoutObject.
    pub effective_allowed_touch_action: foundation::TouchAction,
    /// LayoutObject::InsideBlockingWheelEventHandler(), consumed by ordinary
    /// hit-test chunk metadata independently from scroll hit-test records.
    pub inside_blocking_wheel_event_handler: bool,
    /// Element::GetRegionCaptureCropId(), exported as the exact token words
    /// consumed by PaintController's region-capture chunk metadata.
    pub region_capture_crop_id: Option<[u64; 2]>,
    pub tracked_element_sub_rects: Vec<crate::internal::layout_input::TrackedElementSubRect>,
    /// LayoutObject::IsSVGChild(). SVG hit-test opacity cannot be reconstructed
    /// from the reduced NodeKind set because every SVG child is mixed.
    pub is_svg_child: bool,
    pub logical_tree_order: u64,
    pub establishes_paint_state: bool,
    pub hidden: bool,
    /// FragmentItem::IsLineBreak(), used by BuildBackplate's paragraph split.
    /// False for physical fragments and non-text items.
    pub is_line_break: bool,
    /// Chromium LayoutSVGHiddenContainer semantics: retain SVG layout and
    /// resource descendants, but omit this subtree from ordinary paint/hits.
    pub svg_hidden_container: bool,
    /// LayoutSVGContainer::SVGDescendantMayHaveTransformRelatedOperations().
    /// SVGContainerPainter disables subtree culling when this source aggregate
    /// is set; Paint must not rediscover it from exported descendants.
    pub svg_descendant_may_have_transform_related_operations: bool,
    /// SVGViewportContainerElement::HasEmptyViewBox(). Keep this authored
    /// element fact distinct from `SvgViewBox()`, which is later reused for
    /// the nearest resolved viewport dimensions of every SVG child.
    pub svg_view_box_is_empty: bool,
    pub painted_atomically: bool,
    pub source_kind: NodeKind,
    /// Chromium LayoutImage::IsLayoutImage() identity. `NodeKind::kReplaced`
    /// also covers canvas, video, embedded content and frames, so Paint must
    /// consume the assembled virtual result instead of DOM/resource guesses.
    /// cpp: core/layout/layout_image.h:135-138
    /// cpp: core/paint/image_painter.cc:117-136,184-195
    pub replaced_is_image: bool,
    /// Immutable software frame currently published by HTMLVideoElement.
    /// This is deliberately distinct from LayoutImage's resource identity:
    /// VideoPainter consumes it only for the source software-frame path while
    /// accelerated video retains kForeignLayerVideo ownership.
    /// cpp: core/paint/video_painter.cc:99-113
    pub video_frame_resource_id: Option<image_resource::ImageId>,
    /// ImageElementBase::decoding_mode_ projected from the associated HTML
    /// image. Generated/non-HTML LayoutImages retain kUnspecified.
    pub image_decoding_mode: image_resource::ImageDecodingMode,
    /// ImageElementBase::last_painted_image_id_. Paint mutates this only after
    /// cache reuse has failed, matching GetDecodingModeForPainting().
    pub image_last_painted_id: std::cell::Cell<image_resource::PaintImageId>,
    /// LayoutImage::last_paint_rect_. ImagePainter mutates this retained Paint
    /// lifecycle value immediately before consulting the Drawing cache. It is
    /// deliberately excluded from immutable pre-paint input equivalence.
    /// cpp: core/layout/layout_image.h:202-203
    /// cpp: core/layout/layout_image.cc:411-421
    pub image_last_paint_rect: std::cell::Cell<Option<FragmentPaintRect>>,
    pub effective_zoom: f32,
    pub style: PaintStyle,
    /// HighlightStyleUtils::HighlightPaintingStyle for ::selection. This is a
    /// distinct immutable text-paint input: selection backgrounds, glyphs,
    /// shadows and decorations must not be reconstructed from hard-coded
    /// colors in Paint. Read through SelectionPaintStyle(), which resolves it
    /// from the live source; stored only for detached fixtures.
    pub selection_style: PaintStyle,
    /// LayoutReplaced::GetSelectionState and LocalSelectionVisualRect.  The
    /// latter may aggregate multiple inline fragment items, so Paint must not
    /// reconstruct it from the physical box fragment.
    /// cpp: core/layout/layout_replaced.cc:599-617
    pub replaced_selection_state: SelectionState,
    pub replaced_local_selection_visual_rect: Option<FragmentPaintRect>,
    /// TextPainter::SvgTextPaintState::TextDecorationObject() style.  SVG text
    /// decorations use the fill/stroke of the first ancestor that declares the
    /// decoration, which can differ from the leaf text's own style. Read
    /// through SvgTextDecorationStyle(); stored only for detached fixtures.
    pub svg_text_decoration_style: Option<PaintStyle>,
    /// True for an element whose CSS background is painted by the root canvas
    /// painter. Borders, shadows and outlines remain ordinary box decoration.
    /// Read live by BackgroundTransfersToView(); stored only for fixtures.
    pub background_transfers_to_view: bool,
    /// LayoutBox::BackgroundIsKnownToBeObscured(). Layout computes this after
    /// foreground opacity is known; Paint uses the retained decision instead
    /// of re-running replaced-content coverage analysis.
    pub background_is_known_to_be_obscured: bool,
    /// Fixture-only LayoutView::StyleRef() for a page border box. Production
    /// Paint reads it live through PaintFragmentCursor::DocumentElementBackground().
    pub page_document_background_style: Option<PaintStyle>,
    /// Fixture-only page geometry. Production reads it live through
    /// ResidentBoxGeometry::Page*BackgroundGeometry() from the page fragment
    /// and its LayoutView.
    pub page_document_background_geometry: Option<PageBackgroundPaintData>,
    pub page_container_background_geometry: Option<PageContainerBackgroundPaintData>,
    /// Detached-fixture canvas background. Production Paint reads the live
    /// selection through PaintFragmentCursor::CanvasBackground().
    pub canvas_background: Option<CanvasBackgroundPaintData>,
    /// Fixture-only LayoutView background invalidation selection. Production
    /// PrePaint reads PaintFragmentCursor::ViewBackground() live and compares
    /// it with the root box's saved previous geometry.
    pub view_background_invalidation: Option<CanvasBackgroundPaintData>,
    pub background_paint_location: BackgroundPaintLocation,
    /// Number of source FillLayers, including image:none slots omitted from
    /// the drawing projection. BoxPaintInvalidator treats multiple layers as
    /// size-dependent before checking whether the first image can render.
    pub background_fill_layer_count: usize,
    pub has_background: bool,
    pub background_any_local_attachment_image: bool,
    pub background_any_default_attachment_image: bool,
    pub background_any_layer_uses_content_box: bool,
    /// LayoutBox::ScrollableOverflowRect(), retained so PrePaint can compare
    /// the current positioning area with the previous committed fragment.
    pub scrollable_overflow: Option<FragmentPaintRect>,
    /// LayoutView::BackgroundRect(), the positioning area for fixed canvas
    /// backgrounds. Present only on the LayoutView.
    pub view_background_rect: Option<FragmentPaintRect>,
    pub margin: Edges,
    pub border: Edges,
    pub border_styles: [BorderLineStyle; 4],
    pub padding: Edges,
    pub display: Display,
    pub position: Position,
    // A fixed-position LayoutBox whose containing block is the LayoutView.
    // PrePaint switches this box above the document scroll translation.
    pub fixed_to_view: bool,
    /// LayoutBox::NeedsAnchorPositionScrollAdjustment(), consumed by
    /// NeedsPaintOffsetTranslation without reconstructing anchor state.
    pub needs_anchor_position_scroll_adjustment: bool,
    pub floating: FloatSide,
    pub sticky_offset: Offset,
    pub sticky_constraint: Option<StickyPositionPaintData>,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub overscroll_behavior_x: OverscrollBehavior,
    pub overscroll_behavior_y: OverscrollBehavior,
    /// Source: core/paint/paint_property_tree_builder.cc:3634-3635.
    pub prevent_scroll_axis_locking: bool,
    pub scroll_offset: Offset,
    /// PhysicalBoxFragment::OverflowClipRect with overlay scrollbar space
    /// excluded for hit testing. None means this fragment does not clip
    /// overflow along either axis.
    pub hit_test_overflow_clip: Option<FragmentPaintRect>,
    /// PhysicalBoxFragment::PixelSnappedScrolledContentOffset(). Paint and
    /// Interaction consume the source snap instead of re-rounding scroll_offset.
    pub pixel_snapped_scrolled_content_offset: Offset,
    pub scroll_size: Size,
    // Chromium scrollable area, including viewport-propagated overflow. CSS
    // input overflow alone does not identify the LayoutView's scrolling owner.
    pub scroll_container: Option<ScrollContainerPaintData>,
    pub writing_mode: WritingMode,
    pub direction: TextDirection,
    /// FragmentItem::GetTextFitScale(). SVG text keeps 1 because its separate
    /// SVG transform/scaled-font path owns that scaling.
    pub text_fit_scale: f64,
    pub text_combine: Option<TextCombinePaintData>,
    /// LayoutTextCombine::AdjustTextLeftForPaint, quantized as LayoutUnit.
    pub text_combine_inline_offset: f64,
    pub replaced_content: Option<ReplacedContentPaintData>,
    pub glyph_runs: Arc<Vec<PaintGlyphRun>>,
    pub list_marker_symbol: Option<ListStyleType>,
    pub list_marker_inside: bool,
    pub mathml: Option<Arc<MathMLPaintData>>,
    pub fieldset: Option<Arc<FieldsetPaintData>>,
    /// Final physical gap-rule segments in Chromium's rule-overlap order.
    pub gap_rules: Option<Arc<GapRulePaintData>>,
    pub scrollbars: Option<Arc<ScrollbarPaintData>>,
    pub form_control: Option<Arc<FormControlPaintData>>,
    pub svg_shape: Option<Arc<SvgShapeData>>,
    pub svg_viewport: Option<SvgViewportGeometry>,
}

impl Default for PaintProperties {
    fn default() -> Self {
        static EMPTY_GLYPH_RUNS: OnceLock<Arc<Vec<PaintGlyphRun>>> = OnceLock::new();
        Self {
            source_object_address: 0,
            live_style_facts: false,
            property_fragment_unique_id: 0,
            is_first_for_node: true,
            is_layout_view: false,
            is_printing: false,
            should_print_backgrounds: false,
            canvas_draw_element_enabled: false,
            is_in_canvas_subtree: false,
            is_overscroll_area_parent: false,
            needs_layout: false,
            child_layout_blocked_by_display_lock: false,
            is_fragment_less_box: false,
            additional_compositing_reasons: Default::default(),
            is_box_model_object: false,
            is_layout_box: false,
            is_page_container: false,
            is_page_border_box: false,
            is_paginated_root: false,
            is_layout_inline: false,
            should_clip_overflow_along_either_axis: false,
            should_clip_overflow_along_both_axes: false,
            is_layout_block_flow: false,
            is_inline_formatting_context: false,
            in_forced_colors_mode: false,
            requires_main_thread_scrolling_for_background_attachment_fixed: None,
            display_item_client_invalidation_reason: None,
            paint_layer_client_invalidation_reason: None,
            self_ink_overflow: None,
            contents_ink_overflow: None,
            ink_overflow: None,
            filtered_ink_overflow: None,
            css_mask_bounding_box: None,
            box_self_visual_overflow: None,
            self_outline: None,
            previous_box_geometry: None,
            box_fragment_is_inline_box: false,
            display_item_client_id: 0,
            paint_layer_state_is_authoritative: false,
            paint_layer_client_id: 0,
            paint_layer_is_self_painting: false,
            paint_layer_is_fragmented: false,
            paint_layer_first_fragment_visual_rect: None,
            paint_layer_contents_visual_rect: None,
            paint_layer_filter_reference_box: None,
            paint_layer_filter_viewport: None,
            child_paint_blocked_by_display_lock: false,
            paint_layer_is_stacking_context: false,
            paint_layer_is_stacked: false,
            paint_layer_stacking_level: 0,
            paint_layer_order_parent_unique_id: None,
            paint_layer_order: 0,
            display_item_client_is_cacheable: false,
            display_item_client_is_just_created: std::cell::Cell::new(false),
            selection_client_id: 0,
            selection_client_invalidation_reason: None,
            selection_client_is_cacheable: false,
            display_item_fragment: 0,
            has_source: false,
            visible_to_hit_testing: true,
            effective_allowed_touch_action: foundation::TouchAction::kAuto,
            inside_blocking_wheel_event_handler: false,
            region_capture_crop_id: None,
            tracked_element_sub_rects: Vec::new(),
            is_svg_child: false,
            logical_tree_order: 0,
            establishes_paint_state: false,
            hidden: false,
            is_line_break: false,
            svg_hidden_container: false,
            svg_descendant_may_have_transform_related_operations: false,
            svg_view_box_is_empty: false,
            painted_atomically: false,
            source_kind: NodeKind::kBox,
            replaced_is_image: false,
            video_frame_resource_id: None,
            image_decoding_mode: Default::default(),
            image_last_painted_id: std::cell::Cell::new(image_resource::PaintImageId::INVALID),
            image_last_paint_rect: std::cell::Cell::new(None),
            effective_zoom: 1.0,
            style: PaintStyle::default(),
            selection_style: PaintStyle::default(),
            replaced_selection_state: SelectionState::kNone,
            replaced_local_selection_visual_rect: None,
            svg_text_decoration_style: None,
            background_transfers_to_view: false,
            background_is_known_to_be_obscured: false,
            page_document_background_style: None,
            page_document_background_geometry: None,
            page_container_background_geometry: None,
            canvas_background: None,
            view_background_invalidation: None,
            background_paint_location: BackgroundPaintLocation::BorderBox,
            background_fill_layer_count: 0,
            has_background: false,
            background_any_local_attachment_image: false,
            background_any_default_attachment_image: false,
            background_any_layer_uses_content_box: false,
            scrollable_overflow: None,
            view_background_rect: None,
            margin: Edges::default(),
            border: Edges::default(),
            border_styles: [BorderLineStyle::kNone; 4],
            padding: Edges::default(),
            display: Display::kBlock,
            position: Position::kStatic,
            fixed_to_view: false,
            needs_anchor_position_scroll_adjustment: false,
            floating: FloatSide::kNone,
            sticky_offset: Offset::default(),
            sticky_constraint: None,
            overflow_x: Overflow::kVisible,
            overflow_y: Overflow::kVisible,
            overscroll_behavior_x: Default::default(),
            overscroll_behavior_y: Default::default(),
            prevent_scroll_axis_locking: false,
            scroll_offset: Offset::default(),
            hit_test_overflow_clip: None,
            pixel_snapped_scrolled_content_offset: Offset::default(),
            scroll_size: Size::default(),
            scroll_container: None,
            writing_mode: WritingMode::kHorizontalTb,
            direction: TextDirection::kLtr,
            text_fit_scale: 1.0,
            text_combine: None,
            text_combine_inline_offset: 0.0,
            replaced_content: None,
            glyph_runs: Arc::clone(EMPTY_GLYPH_RUNS.get_or_init(|| Arc::new(Vec::new()))),
            list_marker_symbol: None,
            list_marker_inside: false,
            mathml: None,
            fieldset: None,
            gap_rules: None,
            scrollbars: None,
            form_control: None,
            svg_shape: None,
            svg_viewport: None,
        }
    }
}

#[allow(non_snake_case)]
/// Box geometry read from the slot's resident fragment, as Chromium's box
/// painters read PhysicalBoxFragment::Borders(), Padding() and
/// SidesToInclude() from the fragment being painted. Slots without a resident
/// box fragment (text and line items, hand-built fixtures) keep the values
/// exported from style.
pub trait ResidentBoxGeometry {
    fn ResidentBoxFragment(&self) -> Option<&crate::physical_box_fragment::PhysicalBoxFragment>;
    /// The resident text or generated-text item and its FragmentItems.
    fn ResidentTextItem(
        &self,
    ) -> Option<(&crate::fragment_item::FragmentItem, &crate::fragment_items::FragmentItems)>;
    /// Whether the resident owner is an inline item.
    fn ResidentOwnerIsInline(&self) -> bool;
    /// Borrowed-view size: PhysicalFragment::Size() of the resident physical
    /// owner, or FragmentItem::RectInContainerFragment() size of the resident
    /// inline item (what population stores in `size`). Detached fixtures
    /// without an owner keep the stored value.
    fn ViewSize(&self) -> Size;
    /// Borrowed-view content size: the box fragment's border box minus
    /// borders and padding (clamped at zero), else the view size.
    fn ViewContentSize(&self) -> Size;
    /// Borrowed-view source Node id: the resident owner's LayoutObject
    /// Node (0 without one, and for an inline line item).
    fn ViewNodeId(&self) -> u64;
    /// Borrowed-view fragment kind from the resident owner's type.
    fn ViewKind(&self) -> FragmentKind;
    /// Borrowed-view ink overflow (self, contents, ink, filtered), read
    /// from the resident fragment or item. Only detached fixtures read the
    /// stored values.
    fn ViewInkOverflows(&self) -> [Option<FragmentPaintRect>; 4];
    /// One entry of `ViewInkOverflows()` (0 self, 1 contents, 2 ink,
    /// 3 filtered ink), computed alone: painters read one rect per call, as
    /// Chromium's read PhysicalBoxFragment::InkOverflowRect() and friends
    /// directly (core/paint/box_fragment_painter.cc:2122-2133).
    fn ViewInkOverflowAt(&self, which: usize) -> Option<FragmentPaintRect> {
        self.ViewInkOverflows()[which]
    }
    /// A resident text item's shaped glyph ink in item-local physical
    /// coordinates (the glyph stage of InkOverflow::ComputeTextInkOverflow,
    /// ink_overflow.cc:439-489). `None` for other owners.
    fn ViewTextGlyphInk(&self) -> Option<FragmentPaintRect>;
    /// Box-fragment geometry facts (inline box, hit-test overflow clip,
    /// scrolled content offset, self visual overflow) from the resident box.
    fn ViewBoxGeometryFacts(&self) -> (bool, Option<FragmentPaintRect>, Offset, Option<FragmentPaintRect>);
    /// Text-item facts (ellipsis, text-fit scale, list-marker symbol and
    /// inside flag, text-combine inline offset) from the resident item.
    fn ViewTextItemFacts(&self) -> (bool, f64, Option<ListStyleType>, bool, f64);
    fn ViewIsLineBreak(&self) -> bool;
    /// The active ScopedDisplayItemFragment value; an inline item's
    /// FragmentItem::FragmentId().
    fn ViewDisplayItemFragment(&self) -> u32;
    /// The ScopedDisplayItemFragment this fragment opens for itself and its
    /// paint descendants, or `None` when it paints in its paint parent's
    /// scope. A fragmentainer box opens its break token's SequenceNumber()+1
    /// (0 without a token; core/paint/box_fragment_painter.cc:394-403), a
    /// layer box its index in LayoutBox::PhysicalFragments()
    /// (core/paint/paint_layer_painter.cc:776-778), an inline item its
    /// FragmentItem::FragmentId() (core/paint/inline_box_fragment_painter.cc:63-64).
    /// Detached fixtures without an owner keep the stored value.
    fn ViewOwnDisplayItemFragment(&self) -> Option<u32>;
    /// A text item's containing line top relative to the item and the line
    /// height, read from the resident FragmentItems through its containing
    /// line (InlineCursor::MoveToContainingLine; core/paint/
    /// text_fragment_painter.cc:69-103). `None` for other owners.
    fn ViewTextLineGeometry(&self) -> Option<(f64, f64)>;
    fn ViewTextLineTopOffset(&self) -> Option<f64> {
        self.ViewTextLineGeometry().map(|(top, _)| top)
    }
    fn ViewTextLineHeight(&self) -> Option<f64> {
        self.ViewTextLineGeometry().map(|(_, height)| height)
    }
    /// The `::first-line` decoration rect of a resident line item whose
    /// first-line style differs from its normal style (LineBoxFragmentPainter).
    /// `None` means the line owns no line-box decoration item.
    fn ViewFirstLineDecorationRect(&self) -> Option<FragmentPaintRect>;
    /// Whether the fragment is hidden for paint; an inline item's
    /// FragmentItem::IsHiddenForPaint() or a disabled SVG viewport.
    fn ViewHidden(&self) -> bool;
    /// FrameSet / fieldset paint data computed from the resident box
    /// fragment (its FrameSetLayoutData, legend cutout).
    fn ViewFrameSet(&self) -> Option<Arc<FrameSetPaintData>>;
    fn ViewFieldset(&self) -> Option<Arc<FieldsetPaintData>>;
    /// Column rules / gap rules computed from the resident box's gap geometry.
    fn ViewColumnRules(&self) -> Option<Arc<ColumnRulePaintData>>;
    fn ViewGapRules(&self) -> Option<Arc<GapRulePaintData>>;
    /// MathML paint data and first baseline from the resident box fragment.
    fn ViewMathML(&self) -> Option<Arc<MathMLPaintData>>;
    fn ViewFirstBaseline(&self) -> Option<f64>;
    /// SVG text paint data computed from the resident inline item.
    fn ViewSvgText(&self) -> Option<Arc<SvgTextPaintData>>;
    /// The replaced content rect of the resident box's LayoutReplaced.
    fn ViewReplacedContent(&self) -> Option<ReplacedContentPaintData>;
    /// Scroll container inputs from the resident box's LayoutBox and
    /// PaintLayerScrollableArea.
    fn ViewScrollContainer(&self) -> Option<ScrollContainerPaintData>;
    /// `ViewScrollContainer().is_some()` without materializing the inputs.
    fn ViewHasScrollContainer(&self) -> bool;
    /// `ViewScrollContainer()`'s `scroll_origin` alone.
    fn ViewScrollOrigin(&self) -> Option<Offset>;
    /// `ViewScrollbars()`'s `should_paint_as_overlay` alone (`None` when the
    /// box owns no overflow controls).
    fn ViewScrollbarsPaintAsOverlay(&self) -> Option<bool>;
    /// `ViewScrollContainer()`'s scrolling-background client id and state
    /// alone, without materializing the scroll geometry.
    fn ViewScrollingBackgroundClient(&self) -> Option<(u64, PaintClientState)>;
    /// `ViewScrollbars()`'s corner and scrollbar client ids and states alone,
    /// without positioning or materializing the overflow controls.
    fn ViewScrollbarClients(&self) -> Option<ScrollbarClients>;
    /// Scroll size, CSSOM client size and ScrollsOverflow of the resident box.
    fn ViewScrollGeometry(&self) -> (Size, Option<Size>, bool);
    /// `ViewScrollGeometry()`'s ScrollsOverflow alone
    /// (LayoutBox::ScrollsOverflow), without the scroll and client sizes.
    fn ViewScrollsOverflow(&self) -> bool {
        self.ViewScrollGeometry().2
    }
    /// Overflow-control paint data from the resident box's scrollable area.
    fn ViewScrollbars(&self) -> Option<Arc<ScrollbarPaintData>>;
    /// The resident box's self outline rects (AddSelfOutlineRects).
    fn ViewSelfOutline(&self) -> Option<SelfOutlinePaintData>;
    /// overflow-clip-margin outsets of the resident box.
    fn ViewOverflowClipMarginOutsets(&self) -> Option<Edges>;
    /// Table grid, column rect and column geometry of the resident table box.
    fn ViewTable(&self) -> Option<Arc<TablePaintData>>;
    /// FragmentData::UniqueId of the owner's property fragment (the box
    /// fragment's FragmentData, else the object's first fragment).
    fn ViewPropertyFragmentUniqueId(&self) -> u64;
    /// Chromium identity of this fragment occurrence, read live from the
    /// resident owner (see `PhysicalFragmentInstanceId` /
    /// `InlineItemInstanceId`). A detached fixture without a resident owner
    /// keeps its recorded `fragment_instance_id`.
    fn ViewFragmentInstanceId(&self) -> u64;
    /// The fragmentainer identity this fragment opens for its descendants (a
    /// column or page area: `FragmentainerInstanceId`), else `None`; every
    /// other fragment is inside the fragmentainer its paint-tree walk
    /// carries, as PrePaintTreeWalk carries `fragmentainer_idx`. A detached
    /// fixture returns its recorded nonzero `fragmentainer_instance_id`.
    fn ViewOwnFragmentainerInstanceId(&self) -> Option<u64>;
    /// Whether this occurrence applies the box fragment's own paint state
    /// (its FragmentData properties, decorations and layer): every box
    /// fragment and every kBox inline item with a box fragment. Boxes reached
    /// through an SVG root's children are applied once, under the root's
    /// first fragment, which only the traversal knows.
    fn ViewEstablishesPaintState(&self) -> bool;
    /// A text item's offsets in its text content (FragmentItem::StartOffset /
    /// EndOffset).
    fn ViewTextStart(&self) -> Option<u32>;
    fn ViewTextEnd(&self) -> Option<u32>;
    /// Whether a text item's painted text has a non-whitespace character
    /// (text paint timing ignores whitespace-only text).
    fn ViewTextPaintTimingNonWhitespace(&self) -> bool;
    /// Stitched decoration geometry: computed from the resident block box
    /// across its LayoutBox fragments; an inline box computes its strip from
    /// the live inline cursor (InlineBoxFragmentPainter).
    fn ViewStitchedDecoration(&self) -> Option<StitchedDecorationData>;
    /// LayoutImage::MutableForPainting::UpdatePaintedRect on the live
    /// LayoutImage; true when the painted rect changed.
    fn UpdateImagePaintedRect(&self, rect: FragmentPaintRect) -> bool;
    /// Store the painted image id on the image element
    /// (ImageElementBase::last_painted_image_id_) and return the previous one.
    fn ReplaceImageLastPaintedId(
        &self,
        id: image_resource::PaintImageId,
    ) -> image_resource::PaintImageId;
    /// A text item's point-test bounds relative to the item.
    fn ViewTextHitTestBounds(&self) -> Option<FragmentPaintRect>;
    /// Collapsed-border geometry of the resident table box.
    fn ViewCollapsedTable(&self) -> Option<Arc<CollapsedTablePaintData>>;
    /// PhysicalFragment::IsPaintedAtomically of the resident box.
    fn ViewPaintedAtomically(&self) -> bool;
    /// PhysicalFragment::HasCollapsedBorders of the resident box.
    fn ViewHasCollapsedBorders(&self) -> bool;
    /// PhysicalBoxFragment::IsInlineFormattingContext of the resident box.
    fn ViewIsInlineFormattingContext(&self) -> bool;
    /// CanPaintMultipleFragments for the resident box.
    fn ViewCanPaintMultipleFragments(&self) -> bool;
    /// The table cell's column index (PhysicalBoxFragment::TableCellColumnIndex).
    fn ViewTableCellColumn(&self) -> Option<usize>;
    /// The resident box's computed box-decoration-break.
    fn ViewBoxDecorationBreak(&self) -> BoxDecorationBreak;
    fn ViewIsInlineBox(&self) -> bool {
        self.ViewBoxGeometryFacts().0
    }
    fn ViewHitTestOverflowClip(&self) -> Option<FragmentPaintRect> {
        self.ViewBoxGeometryFacts().1
    }
    fn ViewScrolledContentOffset(&self) -> Offset {
        self.ViewBoxGeometryFacts().2
    }
    fn ViewBoxSelfVisualOverflow(&self) -> Option<FragmentPaintRect> {
        self.ViewBoxGeometryFacts().3
    }
    fn ViewSelfInkOverflow(&self) -> Option<FragmentPaintRect> {
        self.ViewInkOverflowAt(0)
    }
    fn ViewContentsInkOverflow(&self) -> Option<FragmentPaintRect> {
        self.ViewInkOverflowAt(1)
    }
    fn ViewInkOverflow(&self) -> Option<FragmentPaintRect> {
        self.ViewInkOverflowAt(2)
    }
    fn ViewFilteredInkOverflow(&self) -> Option<FragmentPaintRect> {
        self.ViewInkOverflowAt(3)
    }
    fn Borders(&self) -> Edges;
    fn Padding(&self) -> Edges;
    fn BorderSides(&self) -> crate::boundary_types::FragmentBoxSides;
    /// The border-image of a box fragment, resolved from its live style,
    /// size, borders and the document's image resources
    /// (NinePieceImagePainter reads the fragment being painted and its
    /// Document's ImageResourceContent).
    fn BorderImage(
        &self,
        resources: Option<&PaintResources>,
    ) -> Option<crate::internal::paint_input::NinePieceImagePaint>;
    /// The -webkit-mask-box-image of a box fragment (see BorderImage).
    fn MaskBoxImage(
        &self,
        resources: Option<&PaintResources>,
    ) -> Option<crate::internal::paint_input::NinePieceImagePaint>;
    /// The resident fragment's own ComputedStyle as a projection-free
    /// PaintStyle view: PhysicalFragment::Style() or FragmentItem::Style(),
    /// both LayoutObject::EffectiveStyle(StyleVariant()), so a fragment that
    /// UsesFirstLineStyle reads the first-line style. Chromium's NodeAtPoint
    /// reads this style (core/paint/box_fragment_painter.cc:2261-2468,
    /// core/layout/hit_test_request.h VisibleToHitTestRequest). Detached
    /// fixtures keep the stored projection.
    fn ViewStyle(&self) -> PaintStyle;
    /// BoxFragmentPainter::NodeAtPoint's repeated-fragment gate
    /// (core/paint/box_fragment_painter.cc:2261-2271):
    /// `IsFirstForNode() || CanPaintMultipleFragments()` of the resident box.
    fn ViewCanPaintFragmentOccurrence(&self) -> bool;
    /// Whether the resident box's LayoutObject is a LayoutReplaced
    /// (ViewReplacedContent().is_some() without building its paint data).
    fn ViewIsReplaced(&self) -> bool;
    /// PhysicalFragment::GetBoxType() == kPageContainer
    /// (core/layout/physical_fragment.h). ViewPainter paints the `@page`
    /// background for this anonymous fragment.
    fn IsPageContainer(&self) -> bool;
    /// PhysicalFragment::GetBoxType() == kPageBorderBox.
    fn IsPageBorderBox(&self) -> bool;
    /// PhysicalBoxFragment::IsPaginatedRoot().
    fn IsPaginatedRoot(&self) -> bool;
    /// BoxBackgroundPaintContext geometry of a page container: the page
    /// border-box link offset and its borders/padding are read from the
    /// resident fragment (view_painter.cc:397-402, GetPageBorderBoxLink);
    /// the LayoutView-global positioning size, root paint offset and fixed
    /// attachment fact are read live through the fragment's View().
    fn PageContainerBackgroundGeometry(&self) -> Option<PageContainerBackgroundPaintData>;
    /// BoxBackgroundPaintContext geometry of a page border box. The page
    /// area's OffsetFromRootFragmentationContext() is read from the resident
    /// fragment; the LayoutView-global facts are read live (see above).
    fn PageDocumentBackgroundGeometry(&self) -> Option<PageBackgroundPaintData>;
}

/// The LayoutView-global facts of a paginated page background (positioning
/// size, borders/padding, root paint offset, fixed attachment;
/// view_painter.cc:397-402), read live through the page fragment's
/// LayoutObject View() rather than published into each page slot.
fn PageViewBackgroundFacts(
    fragment: &crate::physical_box_fragment::PhysicalBoxFragment,
) -> Option<(PageBackgroundPaintData, Offset)> {
    let object = unsafe { fragment.GetLayoutObject().as_ref() }?;
    crate::internal::boundary::layout_boundary::PageBackgroundViewFacts(object)
}

// Chromium: core/paint/box_fragment_painter.cc:102-127 and
// core/layout/inline/fragment_item.h IsHiddenForPaint. A self-painting atomic
// inline is entered through its own physical fragment, but its hidden-for-paint
// state (line truncation) is recorded on the FragmentItem that places it.
fn IsHiddenForPaintIncludingInlineItem(fragment: &crate::physical_fragment::PhysicalFragment) -> bool {
    if fragment.IsHiddenForPaint() {
        return true;
    }
    if !fragment.IsAtomicInline() || !fragment.HasSelfPaintingLayer() {
        return false;
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() || !unsafe { &*object }.IsInLayoutNGInlineFormattingContext() {
        return false;
    }
    let mut cursor = crate::inline_cursor::InlineCursor::default();
    cursor.MoveToLayoutObject(unsafe { &*object });
    cursor.IsNotNull() && cursor.Current().IsHiddenForPaint()
}

/// ExportNinePieceImageForFragment for a box fragment with a live style.
/// Text, line and source-less fragments never paint nine-piece images.
fn LiveNinePieceImage(
    fragment: &FragmentPaintData,
    resources: Option<&PaintResources>,
    image: impl FnOnce(&layout_style::style::computed_style::ComputedStyle)
        -> &layout_style::style::nine_piece_image::NinePieceImage,
) -> Option<Option<crate::internal::paint_input::NinePieceImagePaint>> {
    let style = fragment.paint.style.Source()?;
    if fragment.ViewKind() != FragmentKind::kBox || !fragment.paint.HasSource() {
        return Some(None);
    }
    let source = image(style);
    let resource_id = unsafe { source.GetImage().as_ref() }.and_then(
        crate::internal::boundary::paint_style_export::FetchedNinePieceImageResourceId,
    );
    let resource = resource_id.and_then(|id| resources?.images.iter().find(|image| image.id == id));
    Some(
        crate::internal::boundary::paint_style_export::ExportNinePieceImageForFragment(
            style,
            source,
            fragment.ViewSize(),
            fragment.Borders(),
            resource,
        ),
    )
}

// Live fragment identities.
//
// Chromium keys retained per-fragment paint state by identities that survive
// relayout without a per-publication census:
//   * a box fragment by its FragmentData (FragmentData::UniqueId,
//     core/paint/fragment_data.h:123-128);
//   * an inline item by its DisplayItemClient and FragmentItem::FragmentId(),
//     the pair DisplayItem::Id uses (platform/graphics/paint/display_item.h,
//     DisplayItem::Id{client_id, type, fragment}); FragmentItems::
//     FinalizeAfterLayout numbers every item of one LayoutObject across the
//     whole inline formatting context, and line items of the container from
//     kInitialLineFragmentId (fragment_items.cc:167-253);
//   * a fragmentainer by its break token SequenceNumber, as
//     BoxFragmentPainter scopes its display items
//     (box_fragment_painter.cc:398-403).
// FragmentData ids and DisplayItemClient ids are NewUniqueObjectId counters
// (high bit clear); derived ids set bit 63 so the spaces never meet.
//
// Three occurrences have no unique Chromium pair, because Chromium never
// needs one for them (it keys paint state by FragmentData and display items
// additionally by type), and are disambiguated here:
//   * a line-clamp ellipsis item owned by the container itself, which
//     FinalizeAfterLayout skips (fragment_items.cc:181-185) and so leaves at
//     FragmentId 0, and any item of an IFC that FinalizeAfterLayout has not
//     numbered (every item still at 0): an item at FragmentId 0 that is not
//     its object's first item (LayoutObject::FirstInlineFragmentItemIndex,
//     set by the same FinalizeAfterLayout) mixes in its item index in the
//     whole IFC, which is unique and stays stable while earlier items keep
//     their count;
//   * the last fragmentainer of a fragmentation context (no break token, so
//     no SequenceNumber) and a fragmentainer without an owner client: several
//     of them can share an owner (one per nested outer fragment, page areas),
//     so they key by their immutable PhysicalBoxFragment, like a line box;
//   * a line box fragment, or a fragment without a LayoutObject or
//     FragmentData: no Chromium paint identity, keyed by its immutable
//     PhysicalFragment so its slot is retained exactly while LayoutNG reuses
//     that fragment.
const DERIVED_INSTANCE_ID_BIT: u64 = 1 << 63;
const FRAGMENTAINER_INSTANCE_ID_BIT: u64 = 1 << 62;
const HASHED_INSTANCE_ID_BIT: u64 = 1 << 61;
const PACKED_CLIENT_LIMIT: u64 = 1 << 29;
/// Salts that separate the hashed sub-spaces from the (client, fragment) one.
const ITEM_INDEX_SALT: u64 = 0x1_0000_0000;
const OWNER_ADDRESS_SALT: u64 = 0x2_0000_0000;

/// splitmix64 finalizer over a pair, tagged into the hashed derived space.
fn HashedInstanceId(kind: u64, a: u64, b: u64) -> u64 {
    let mut z = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    DERIVED_INSTANCE_ID_BIT | kind | HASHED_INSTANCE_ID_BIT | (z & (HASHED_INSTANCE_ID_BIT - 1))
}

/// Pack (client, fragment) exactly while the client id fits 29 bits; beyond
/// that (after half a billion process-wide object ids) mix it into the
/// separately tagged hashed space.
fn DerivedFragmentInstanceId(client: u64, fragmentainer: bool, fragment: u32) -> u64 {
    let kind = if fragmentainer {
        FRAGMENTAINER_INSTANCE_ID_BIT
    } else {
        0
    };
    if client < PACKED_CLIENT_LIMIT {
        return DERIVED_INSTANCE_ID_BIT | kind | (client << 32) | u64::from(fragment);
    }
    HashedInstanceId(kind, client, u64::from(fragment))
}

/// Whether a live fragment identity is a box fragment's FragmentData::UniqueId
/// (rather than a derived inline-item, fragmentainer or fragment-keyed id).
pub fn IsFragmentDataInstanceId(id: u64) -> bool {
    id != 0 && id & DERIVED_INSTANCE_ID_BIT == 0
}

/// Identity of an occurrence without a Chromium pair, keyed by its immutable
/// resident owner's address.
fn OwnerAddressInstanceId(kind: u64, address: usize) -> u64 {
    HashedInstanceId(kind, address as u64, OWNER_ADDRESS_SALT)
}

fn ClientInstanceId(object: *const LayoutObject) -> Option<u64> {
    (!object.is_null()).then(|| unsafe { &*(object as *const DisplayItemClient) }.Id() as u64)
}

/// Identity of a fragmentainer box (column or page area): its owner's
/// DisplayItemClient and the display-item fragment BoxFragmentPainter
/// scopes it with (break token SequenceNumber + 1). The last fragmentainer
/// (no break token) and an owner-less one key by their fragment instead.
/// Descendants are inside this fragmentainer only through the tree walk
/// (Chromium's PrePaintTreeWalk `fragmentainer_idx`); a walk propagates it.
pub fn FragmentainerInstanceId(
    fragment: &crate::physical_box_fragment::PhysicalBoxFragment,
) -> u64 {
    debug_assert!(fragment.IsFragmentainerBox());
    // A fragmentainer is not a CSS box, so GetLayoutObject() hides the
    // fragmentation-context owner it was laid out for.
    let client = ClientInstanceId(fragment.layout_object_ptr());
    let token = fragment.GetBreakToken();
    match client {
        Some(client) if !token.is_null() => {
            DerivedFragmentInstanceId(client, true, unsafe { &*token }.SequenceNumber() + 1)
        }
        _ => OwnerAddressInstanceId(
            FRAGMENTAINER_INSTANCE_ID_BIT,
            fragment as *const crate::physical_box_fragment::PhysicalBoxFragment as usize,
        ),
    }
}

/// Identity of a resident physical fragment: FragmentData::UniqueId of a CSS
/// box (its own FragmentData, else its object's first fragment), the
/// fragmentainer identity of a column/page area, and the fragment's own
/// address for a line box or a fragment with no FragmentData. Never 0.
pub fn PhysicalFragmentInstanceId(fragment: &crate::physical_fragment::PhysicalFragment) -> u64 {
    if fragment.IsFragmentainerBox() {
        return FragmentainerInstanceId(unsafe {
            &*(fragment as *const crate::physical_fragment::PhysicalFragment
                as *const crate::physical_box_fragment::PhysicalBoxFragment)
        });
    }
    let object = fragment.GetLayoutObject();
    let id = if object.is_null() || fragment.IsLineBox() {
        0
    } else {
        let fragment_data = if fragment.IsBox() {
            fragment.GetFragmentData().cast_mut()
        } else {
            std::ptr::null_mut()
        };
        crate::internal::boundary::layout_boundary::PropertyFragmentUniqueId(object, fragment_data)
    };
    if id != 0 {
        return id;
    }
    OwnerAddressInstanceId(
        0,
        fragment as *const crate::physical_fragment::PhysicalFragment as usize,
    )
}

/// Identity of the resident inline item `items.Items()[index]`:
/// (DisplayItemClient id, FragmentItem::FragmentId()). Stable across relayout
/// while the object keeps the same fragment sequence number, independent of
/// the item's text offsets or container FragmentData. An item at FragmentId
/// 0 that is not its object's first item (container-owned line-clamp
/// ellipsis, unfinalized IFC) mixes in its IFC item index instead. Never 0.
pub fn InlineItemInstanceId(
    items: &crate::fragment_items::FragmentItems,
    index: usize,
) -> u64 {
    let Some(item) = items.Items().get(index) else {
        return OwnerAddressInstanceId(0, items as *const _ as usize ^ index);
    };
    let object = item.GetLayoutObject();
    let Some(client) = ClientInstanceId(object) else {
        return OwnerAddressInstanceId(0, item as *const _ as usize);
    };
    let fragment = item.UncheckedFragmentId();
    // FinalizeAfterLayout numbers lines from kInitialLineFragmentId and
    // records the 1-based IFC index of the item it numbers 0
    // (fragment_items.cc:167-225).
    let ifc_index = items.SizeOfEarlierFragments() as usize + index;
    let numbered = if item.Type() == crate::fragment_item::ItemType::kLine {
        fragment >= crate::fragment_item::FragmentItem::kInitialLineFragmentId
    } else {
        fragment != 0
            || (!item.IsEllipsis()
                && unsafe { &*object }.FirstInlineFragmentItemIndex() == ifc_index + 1)
    };
    if numbered {
        return DerivedFragmentInstanceId(client, false, fragment);
    }
    HashedInstanceId(0, client, ITEM_INDEX_SALT | ifc_index as u64)
}

impl ResidentBoxGeometry for FragmentPaintData {
    fn ViewEstablishesPaintState(&self) -> bool {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items.Items().get(index).is_some_and(|item| {
                    item.Type() == crate::fragment_item::ItemType::kBox
                        && !item.BoxFragment().is_null()
                })
            }
            ResidentFragmentOwner::Physical(address) => {
                let fragment =
                    unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) };
                if !fragment.IsBox() {
                    return false;
                }
                let object = unsafe { fragment.GetLayoutObject().as_ref() };
                if object.is_some_and(|object| object.IsSVGForeignObject() || object.IsSVGText()) {
                    self.paint.establishes_paint_state
                } else {
                    true
                }
            }
            ResidentFragmentOwner::None => self.paint.establishes_paint_state,
        }
    }

    fn ResidentTextItem(
        &self,
    ) -> Option<(&crate::fragment_item::FragmentItem, &crate::fragment_items::FragmentItems)> {
        let crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } =
            self.resident_owner
        else {
            return None;
        };
        let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
        let item = items.Items().get(index)?;
        matches!(
            item.Type(),
            crate::fragment_item::ItemType::kText | crate::fragment_item::ItemType::kGeneratedText
        )
        .then_some((item, items))
    }

    fn ResidentOwnerIsInline(&self) -> bool {
        matches!(
            self.resident_owner,
            crate::boundary_types::ResidentFragmentOwner::InlineItem { .. }
        )
    }

    fn ViewTextStart(&self) -> Option<u32> {
        match self.ResidentTextItem() {
            Some((item, _)) => Some(item.StartOffset()),
            None if self.ResidentOwnerIsInline() => None,
            None => self.text_start,
        }
    }

    fn ViewTextEnd(&self) -> Option<u32> {
        match self.ResidentTextItem() {
            Some((item, _)) => Some(item.EndOffset()),
            None if self.ResidentOwnerIsInline() => None,
            None => self.text_end,
        }
    }

    fn ViewTextPaintTimingNonWhitespace(&self) -> bool {
        match self.ResidentTextItem() {
            Some((item, items)) => {
                let text = item.TextPaintInfo(items);
                !text.text.SubstringContainsOnlyWhitespaceOrEmpty(text.from, text.to)
            }
            None if self.ResidentOwnerIsInline() => false,
            None => self.text_paint_timing_non_whitespace,
        }
    }

    fn ViewStitchedDecoration(&self) -> Option<StitchedDecorationData> {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.ResidentBoxFragment() {
            Some(box_fragment) if !box_fragment.IsInlineBox() => {
                crate::internal::boundary::layout_boundary::ExportStitchedDecorationData(
                    box_fragment,
                )
            }
            // An inline box computes its strip from its own resident item
            // when Paint reads it (InlineBoxFragmentPainterBase::
            // ComputeFragmentOffsetOnLine, core/paint/
            // inline_box_fragment_painter.cc:277-311).
            Some(_) => match self.resident_owner {
                ResidentFragmentOwner::InlineItem { items, index } => {
                    let items =
                        unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                    items.Items().get(index).and_then(|item| {
                        crate::internal::boundary::layout_boundary::ExportInlineStitchedDecorationData(
                            item,
                        )
                    })
                }
                _ => None,
            },
            None => None,
        }
    }

    fn UpdateImagePaintedRect(&self, rect: FragmentPaintRect) -> bool {
        if let Some(object) = self.paint.SourceObject().filter(|object| object.IsLayoutImage()) {
            let image = unsafe {
                &*(object as *const LayoutObject
                    as *const crate::internal::layout_replaced::LayoutInputReplaced)
            };
            let unit = foundation::LayoutUnit::from_f64;
            return image.UpdatePaintedRect(foundation::PhysicalRect::new(
                foundation::PhysicalOffset::new(unit(rect.offset.x), unit(rect.offset.y)),
                foundation::PhysicalSize::new(unit(rect.size.width), unit(rect.size.height)),
            ));
        }
        // Detached fixtures keep the value on the payload.
        let changed = self.paint.image_last_paint_rect.get() != Some(rect);
        self.paint.image_last_paint_rect.set(Some(rect));
        changed
    }

    fn ReplaceImageLastPaintedId(
        &self,
        id: image_resource::PaintImageId,
    ) -> image_resource::PaintImageId {
        let node = self.paint.SourceObject().map(LayoutObject::GetNode);
        if let Some(node) = node.filter(|node| !node.is_null()) {
            let node = unsafe { &*node };
            if node.IsElementNode() {
                let element = unsafe {
                    &*(node as *const crate::internal::layout_node_metadata::Node
                        as *const crate::internal::layout_node_metadata::Element)
                };
                return element.ReplaceLastPaintedImageId(id);
            }
        }
        self.paint.image_last_painted_id.replace(id)
    }

    fn ViewPropertyFragmentUniqueId(&self) -> u64 {
        use crate::boundary_types::ResidentFragmentOwner;
        let object = self.paint.SourceObjectPointer();
        if object.is_null() || matches!(self.resident_owner, ResidentFragmentOwner::None) {
            return self.paint.property_fragment_unique_id;
        }
        let fragment_data = match self.resident_owner {
            ResidentFragmentOwner::Physical(address) => {
                let fragment =
                    unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) };
                if fragment.IsBox() {
                    fragment.GetFragmentData().cast_mut()
                } else {
                    std::ptr::null_mut()
                }
            }
            _ => std::ptr::null_mut(),
        };
        crate::internal::boundary::layout_boundary::PropertyFragmentUniqueId(object, fragment_data)
    }

    fn ViewFragmentInstanceId(&self) -> u64 {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::None => self.fragment_instance_id,
            ResidentFragmentOwner::Physical(address) => PhysicalFragmentInstanceId(unsafe {
                &*(address as *const crate::physical_fragment::PhysicalFragment)
            }),
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                InlineItemInstanceId(items, index)
            }
        }
    }

    fn ViewOwnFragmentainerInstanceId(&self) -> Option<u64> {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            // Detached fixtures record the enclosing fragmentainer directly.
            ResidentFragmentOwner::None => {
                (self.fragmentainer_instance_id != 0).then_some(self.fragmentainer_instance_id)
            }
            ResidentFragmentOwner::Physical(address) => {
                let fragment =
                    unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) };
                fragment.IsFragmentainerBox().then(|| {
                    FragmentainerInstanceId(unsafe {
                        &*(fragment as *const crate::physical_fragment::PhysicalFragment
                            as *const crate::physical_box_fragment::PhysicalBoxFragment)
                    })
                })
            }
            ResidentFragmentOwner::InlineItem { .. } => None,
        }
    }

    fn ViewTextHitTestBounds(&self) -> Option<FragmentPaintRect> {
        match self.resident_owner {
            crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items
                    .Items()
                    .get(index)
                    .and_then(crate::internal::boundary::layout_boundary::LiveTextHitTestBounds)
            }
            _ => None,
        }
    }

    fn ViewCollapsedTable(&self) -> Option<Arc<CollapsedTablePaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::ExportCollapsedTablePaintData(
                    box_fragment,
                )
            }
            None => None,
        }
    }

    fn ViewPaintedAtomically(&self) -> bool {
        self.ResidentBoxFragment()
            .map_or(self.paint.painted_atomically, |box_fragment| {
                box_fragment.IsPaintedAtomically()
            })
    }

    fn ViewHasCollapsedBorders(&self) -> bool {
        self.ResidentBoxFragment()
            .map_or(false, |box_fragment| {
                box_fragment.HasCollapsedBorders()
            })
    }

    fn ViewIsInlineFormattingContext(&self) -> bool {
        self.ResidentBoxFragment()
            .map_or(self.paint.is_inline_formatting_context, |box_fragment| {
                box_fragment.IsInlineFormattingContext()
            })
    }

    fn ViewCanPaintMultipleFragments(&self) -> bool {
        self.ResidentBoxFragment()
            .map_or(true, |box_fragment| {
                crate::internal::fragmentation_utils::CanPaintMultipleFragmentsForFragment(
                    box_fragment,
                )
            })
    }

    fn ViewTableCellColumn(&self) -> Option<usize> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => box_fragment
                .IsTableCell()
                .then(|| box_fragment.TableCellColumnIndex() as usize),
            None => None,
        }
    }

    fn ViewBoxDecorationBreak(&self) -> BoxDecorationBreak {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveBoxDecorationBreak(box_fragment)
            }
            None => BoxDecorationBreak::kSlice,
        }
    }

    fn ViewTable(&self) -> Option<Arc<TablePaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::ExportTablePaintData(box_fragment)
            }
            None => None,
        }
    }

    fn ViewOverflowClipMarginOutsets(&self) -> Option<Edges> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::ExportOverflowClipMargin(box_fragment)
            }
            None => None,
        }
    }

    fn ViewSelfOutline(&self) -> Option<SelfOutlinePaintData> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => crate::internal::boundary::layout_boundary::ExportSelfOutlineGeometry(
                box_fragment,
                box_fragment.Style(),
            ),
            None => self.paint.self_outline.clone(),
        }
    }

    fn ViewScrollbars(&self) -> Option<Arc<ScrollbarPaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::ExportScrollbarPaintData(box_fragment)
            }
            None => self.paint.scrollbars.clone(),
        }
    }

    fn ViewScrollGeometry(&self) -> (Size, Option<Size>, bool) {
        let layout_box = self.ResidentBoxFragment().and_then(|box_fragment| unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(
                box_fragment.GetLayoutObject(),
            )
            .as_ref()
        });
        match layout_box {
            Some(layout_box) => {
                crate::internal::boundary::layout_boundary::LiveScrollGeometry(layout_box)
            }
            None => (self.paint.scroll_size, None, false),
        }
    }

    fn ViewScrollsOverflow(&self) -> bool {
        self.ResidentBoxFragment()
            .and_then(|box_fragment| unsafe {
                foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(
                    box_fragment.GetLayoutObject(),
                )
                .as_ref()
            })
            .is_some_and(|layout_box| layout_box.ScrollsOverflow())
    }

    fn ViewScrollContainer(&self) -> Option<ScrollContainerPaintData> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveScrollContainer(box_fragment)
            }
            None => self.paint.scroll_container.clone(),
        }
    }

    fn ViewHasScrollContainer(&self) -> bool {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveScrollContainerBox(box_fragment)
                    .is_some()
            }
            None => self.paint.scroll_container.is_some(),
        }
    }

    fn ViewScrollOrigin(&self) -> Option<Offset> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveScrollOrigin(box_fragment)
            }
            None => self
                .paint
                .scroll_container
                .as_ref()
                .map(|scroll| scroll.scroll_origin),
        }
    }

    fn ViewScrollingBackgroundClient(&self) -> Option<(u64, PaintClientState)> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveScrollingBackgroundClient(
                    box_fragment,
                )
            }
            None => self.paint.scroll_container.as_ref().map(|scroll| {
                (
                    scroll.scrolling_background_client_id,
                    self.paint.ScrollClientState(
                        scroll.scrolling_background_client_id,
                        scroll.scrolling_background_client_invalidation_reason,
                        scroll.scrolling_background_client_is_cacheable,
                        scroll.scrolling_background_client_is_just_created.get(),
                    ),
                )
            }),
        }
    }

    fn ViewScrollbarClients(&self) -> Option<ScrollbarClients> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveScrollbarClients(box_fragment)
            }
            None => self.paint.scrollbars.as_ref().map(|bars| {
                let axis = |axis: &Option<ScrollbarPaintAxis>| {
                    axis.as_ref().map(|axis| {
                        (
                            axis.display_item_client_id,
                            self.paint.ScrollClientState(
                                axis.display_item_client_id,
                                axis.display_item_client_invalidation_reason,
                                axis.display_item_client_is_cacheable,
                                axis.display_item_client_is_just_created.get(),
                            ),
                        )
                    })
                };
                ScrollbarClients {
                    corner: (bars.corner_client_id != 0).then(|| {
                        (
                            bars.corner_client_id,
                            self.paint.ScrollClientState(
                                bars.corner_client_id,
                                bars.corner_client_invalidation_reason,
                                bars.corner_client_is_cacheable,
                                bars.corner_client_is_just_created.get(),
                            ),
                        )
                    }),
                    horizontal: axis(&bars.horizontal),
                    vertical: axis(&bars.vertical),
                }
            }),
        }
    }

    fn ViewScrollbarsPaintAsOverlay(&self) -> Option<bool> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveScrollbarsPaintAsOverlay(
                    box_fragment,
                )
            }
            None => self
                .paint
                .scrollbars
                .as_ref()
                .map(|scrollbars| scrollbars.should_paint_as_overlay),
        }
    }

    fn ViewReplacedContent(&self) -> Option<ReplacedContentPaintData> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                let replaced = foundation::DynamicTo::<crate::internal::layout_replaced::LayoutReplaced>(
                    box_fragment.GetLayoutObject(),
                );
                (!replaced.is_null()).then(|| {
                    crate::internal::boundary::layout_boundary::ExportReplacedContentPaintData(replaced)
                })
            }
            None => self.paint.replaced_content.clone(),
        }
    }

    fn ViewSvgText(&self) -> Option<Arc<SvgTextPaintData>> {
        match self.resident_owner {
            crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items.Items().get(index).and_then(|item| {
                    crate::internal::boundary::layout_boundary::ExportSvgTextPaintData(
                        item,
                        *item.RectInContainerFragment(),
                    )
                })
            }
            _ => None,
        }
    }

    fn ViewMathML(&self) -> Option<Arc<MathMLPaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::ExportMathMLPaintData(box_fragment, &self.paint)
            }
            None => self.paint.mathml.clone(),
        }
    }

    fn ViewFirstBaseline(&self) -> Option<f64> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveFirstBaseline(box_fragment)
            }
            None => None,
        }
    }

    fn ViewColumnRules(&self) -> Option<Arc<ColumnRulePaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => crate::internal::boundary::layout_boundary::ExportColumnRulePaintData(
                box_fragment,
                &self.paint,
            ),
            None => None,
        }
    }

    fn ViewGapRules(&self) -> Option<Arc<GapRulePaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => crate::internal::boundary::layout_boundary::ExportGapRulePaintData(
                box_fragment,
                &self.paint,
            ),
            None => self.paint.gap_rules.clone(),
        }
    }

    fn ViewFrameSet(&self) -> Option<Arc<FrameSetPaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => crate::internal::boundary::layout_boundary::ExportFrameSetPaintData(
                box_fragment,
                &self.paint,
            ),
            None => None,
        }
    }

    fn ViewFieldset(&self) -> Option<Arc<FieldsetPaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => crate::internal::boundary::layout_boundary::ExportFieldsetPaintData(
                box_fragment,
                &self.paint,
            ),
            None => self.paint.fieldset.clone(),
        }
    }

    fn ViewTextLineGeometry(&self) -> Option<(f64, f64)> {
        match self.resident_owner {
            crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                crate::internal::boundary::layout_boundary::LiveTextLineGeometry(items, index)
            }
            _ => None,
        }
    }

    fn ViewFirstLineDecorationRect(&self) -> Option<FragmentPaintRect> {
        match self.resident_owner {
            crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items.Items().get(index).and_then(
                    crate::internal::boundary::layout_boundary::LiveFirstLineDecorationRect,
                )
            }
            _ => None,
        }
    }

    fn ViewDisplayItemFragment(&self) -> u32 {
        match self.resident_owner {
            crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items
                    .Items()
                    .get(index)
                    .map_or(self.paint.display_item_fragment, |item| item.FragmentId())
            }
            _ => self.paint.display_item_fragment,
        }
    }

    fn ViewOwnDisplayItemFragment(&self) -> Option<u32> {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::None => Some(self.paint.display_item_fragment),
            ResidentFragmentOwner::InlineItem { .. } => Some(self.ViewDisplayItemFragment()),
            ResidentFragmentOwner::Physical(_) => {
                let box_fragment = self.ResidentBoxFragment()?;
                if box_fragment.IsFragmentainerBox() {
                    // cpp: core/paint/box_fragment_painter.cc:398-403,1211-1212
                    let token = box_fragment.GetBreakToken();
                    return Some(if token.is_null() {
                        0
                    } else {
                        unsafe { &*token }.SequenceNumber() + 1
                    });
                }
                // PaintLayerPainter scopes each physical fragment using its
                // index in the owning LayoutBox.
                // cpp: core/paint/paint_layer_painter.cc:776-778
                let object = unsafe { box_fragment.GetLayoutObject().as_ref() }?;
                if !object.HasLayer() {
                    return None;
                }
                let mut fragments =
                    crate::fragment_data_iterator::AccompaniedFragmentIterator::new(object);
                let mut index = 0;
                while let Some(physical) = fragments.GetPhysicalBoxFragment() {
                    if std::ptr::eq(physical, box_fragment) {
                        return Some(index);
                    }
                    index += 1;
                    fragments.Advance();
                }
                None
            }
        }
    }

    fn ViewHidden(&self) -> bool {
        match self.resident_owner {
            crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items.Items().get(index).map_or(self.paint.hidden, |item| {
                    item.IsHiddenForPaint()
                        || self.paint.SvgViewport().is_some_and(|viewport| viewport.disabled)
                })
            }
            crate::boundary_types::ResidentFragmentOwner::Physical(address) => {
                let fragment =
                    unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) };
                IsHiddenForPaintIncludingInlineItem(fragment)
                    || self.paint.SvgViewport().is_some_and(|viewport| viewport.disabled)
            }
            crate::boundary_types::ResidentFragmentOwner::None => self.paint.hidden,
        }
    }

    fn ViewTextItemFacts(&self) -> (bool, f64, Option<ListStyleType>, bool, f64) {
        let stored = (
            false,
            self.paint.text_fit_scale,
            self.paint.list_marker_symbol,
            self.paint.list_marker_inside,
            self.paint.text_combine_inline_offset,
        );
        match self.resident_owner {
            crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items
                    .Items()
                    .get(index)
                    .and_then(crate::internal::boundary::layout_boundary::LiveTextItemFacts)
                    .unwrap_or(stored)
            }
            _ => stored,
        }
    }

    fn ViewIsLineBreak(&self) -> bool {
        match self.resident_owner {
            crate::boundary_types::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items
                    .Items()
                    .get(index)
                    .map_or(self.paint.is_line_break, |item| item.IsText() && item.IsLineBreak())
            }
            _ => self.paint.is_line_break,
        }
    }

    fn ViewBoxGeometryFacts(&self) -> (bool, Option<FragmentPaintRect>, Offset, Option<FragmentPaintRect>) {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => crate::internal::boundary::layout_boundary::LiveBoxGeometryFacts(
                box_fragment,
                self.paint.ShouldClipOverflowAlongEitherAxis(),
            ),
            None => (
                self.paint.box_fragment_is_inline_box,
                self.paint.hit_test_overflow_clip,
                self.paint.pixel_snapped_scrolled_content_offset,
                self.paint.box_self_visual_overflow,
            ),
        }
    }

    // The single facts of ViewBoxGeometryFacts, read alone from the resident
    // box as Chromium's painters read PhysicalBoxFragment::IsInlineBox() and
    // friends, without computing the other three.
    fn ViewIsInlineBox(&self) -> bool {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => box_fragment.IsInlineBox(),
            None => self.paint.box_fragment_is_inline_box,
        }
    }

    fn ViewHitTestOverflowClip(&self) -> Option<FragmentPaintRect> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveBoxHitTestOverflowClip(
                    box_fragment,
                    self.paint.ShouldClipOverflowAlongEitherAxis(),
                )
            }
            None => self.paint.hit_test_overflow_clip,
        }
    }

    fn ViewScrolledContentOffset(&self) -> Offset {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveBoxScrolledContentOffset(
                    box_fragment,
                )
            }
            None => self.paint.pixel_snapped_scrolled_content_offset,
        }
    }

    fn ViewBoxSelfVisualOverflow(&self) -> Option<FragmentPaintRect> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveBoxSelfVisualOverflow(box_fragment)
            }
            None => self.paint.box_self_visual_overflow,
        }
    }

    fn ViewInkOverflows(&self) -> [Option<FragmentPaintRect>; 4] {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::None => [
                self.paint.self_ink_overflow,
                self.paint.contents_ink_overflow,
                self.paint.ink_overflow,
                self.paint.filtered_ink_overflow,
            ],
            // A physical line box owns no ink overflow; its line item does
            // (FragmentItem kLine, fragment_item.cc:1052-1076).
            ResidentFragmentOwner::Physical(_) => self
                .ResidentBoxFragment()
                .map_or([None; 4], crate::internal::boundary::layout_boundary::LiveBoxInkOverflow),
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items
                    .Items()
                    .get(index)
                    .map_or([None; 4], crate::internal::boundary::layout_boundary::LiveItemInkOverflow)
            }
        }
    }

    fn ViewInkOverflowAt(&self, which: usize) -> Option<FragmentPaintRect> {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::None => self.ViewInkOverflows()[which],
            ResidentFragmentOwner::Physical(_) => self.ResidentBoxFragment().and_then(|fragment| {
                crate::internal::boundary::layout_boundary::LiveBoxInkOverflowAt(fragment, which)
            }),
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items.Items().get(index).and_then(|item| {
                    crate::internal::boundary::layout_boundary::LiveItemInkOverflowAt(item, which)
                })
            }
        }
    }

    fn ViewTextGlyphInk(&self) -> Option<FragmentPaintRect> {
        let (item, items) = self.ResidentTextItem()?;
        crate::internal::boundary::layout_boundary::LiveTextGlyphInk(item, items)
    }

    fn ViewKind(&self) -> FragmentKind {
        use crate::fragment_item::ItemType;
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::None => self.kind,
            ResidentFragmentOwner::Physical(address) => {
                if unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) }
                    .IsBox()
                {
                    FragmentKind::kBox
                } else {
                    FragmentKind::kLine
                }
            }
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                match items.Items().get(index).map(|item| item.Type()) {
                    Some(ItemType::kText) => FragmentKind::kText,
                    Some(ItemType::kGeneratedText) => FragmentKind::kGeneratedText,
                    Some(ItemType::kLine) => FragmentKind::kLine,
                    Some(ItemType::kBox) => FragmentKind::kBox,
                    _ => self.kind,
                }
            }
        }
    }

    fn ViewNodeId(&self) -> u64 {
        use crate::boundary_types::ResidentFragmentOwner;
        let object = match self.resident_owner {
            ResidentFragmentOwner::None => return self.node_id,
            ResidentFragmentOwner::Physical(address) => {
                unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) }
                    .GetLayoutObject()
            }
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                let Some(item) = items.Items().get(index) else {
                    return self.node_id;
                };
                if item.Type() == crate::fragment_item::ItemType::kLine {
                    return 0;
                }
                item.GetLayoutObject()
            }
        };
        let Some(object) = (unsafe { object.as_ref() }) else {
            return 0;
        };
        unsafe { object.GetNode().as_ref() }.map_or(0, |node| node.InputId())
    }

    fn ViewContentSize(&self) -> Size {
        if matches!(self.resident_owner, crate::boundary_types::ResidentFragmentOwner::None) {
            return self.content_size;
        }
        match self.ResidentBoxFragment() {
            // Chromium's PhysicalBoxFragment::ContentRect() DCHECKs that
            // border and padding fit inside the fragment, which does not hold
            // for every table-part fragment (CSS padding may remain in the
            // fragment metadata although the table algorithm resolves a zero
            // row block-size). Use Blink's ShrinkLogicalSize non-negative
            // subtraction instead of calling ContentRect() outside its domain.
            Some(box_fragment) => {
                let insets = box_fragment.Borders() + box_fragment.Padding();
                let size = box_fragment.Size();
                Size {
                    width: (size.width - insets.HorizontalSum()).ClampNegativeToZero().ToDouble(),
                    height: (size.height - insets.VerticalSum()).ClampNegativeToZero().ToDouble(),
                }
            }
            None => self.ViewSize(),
        }
    }

    fn ViewSize(&self) -> Size {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::None => self.size,
            ResidentFragmentOwner::Physical(address) => {
                let size = unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) }
                    .Size();
                Size {
                    width: size.width.ToDouble(),
                    height: size.height.ToDouble(),
                }
            }
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                match items.Items().get(index) {
                    Some(item) => {
                        let size = item.RectInContainerFragment().size;
                        Size {
                            width: size.width.ToDouble(),
                            height: size.height.ToDouble(),
                        }
                    }
                    None => self.size,
                }
            }
        }
    }

    fn ViewStyle(&self) -> PaintStyle {
        use crate::boundary_types::ResidentFragmentOwner;
        let style = match self.resident_owner {
            ResidentFragmentOwner::None => None,
            ResidentFragmentOwner::Physical(address) => Some(
                unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) }
                    .Style(),
            ),
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items.Items().get(index).map(crate::fragment_item::FragmentItem::Style)
            }
        };
        style
            .and_then(PaintStyle::LiveView)
            .unwrap_or_else(|| self.paint.style.clone())
    }

    fn ViewCanPaintFragmentOccurrence(&self) -> bool {
        use crate::boundary_types::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::None => self.paint.CanPaintFragmentOccurrence(),
            // Only box fragments carry IsFirstForNode; line boxes and inline
            // items are not repeated-fragment painting roots.
            ResidentFragmentOwner::Physical(_) => {
                self.ResidentBoxFragment().is_none_or(|box_fragment| {
                    box_fragment.IsFirstForNode()
                        || crate::internal::fragmentation_utils::CanPaintMultipleFragmentsForFragment(
                            box_fragment,
                        )
                })
            }
            ResidentFragmentOwner::InlineItem { .. } => true,
        }
    }

    fn ViewIsReplaced(&self) -> bool {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => !foundation::DynamicTo::<
                crate::internal::layout_replaced::LayoutReplaced,
            >(box_fragment.GetLayoutObject())
            .is_null(),
            None => self.paint.replaced_content.is_some(),
        }
    }

    fn ResidentBoxFragment(&self) -> Option<&crate::physical_box_fragment::PhysicalBoxFragment> {
        use crate::physical_box_fragment::PhysicalBoxFragment;
        use crate::boundary_types::ResidentFragmentOwner;
        // The owner was bound by the publication that this slot belongs to;
        // the LayoutResult lease held by Paint keeps it resident.
        match self.resident_owner {
            ResidentFragmentOwner::None => None,
            ResidentFragmentOwner::Physical(address) => {
                let fragment =
                    unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) };
                fragment
                    .IsBox()
                    .then(|| unsafe { &*foundation::To::<PhysicalBoxFragment>(fragment as *const _) })
            }
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                unsafe { items.Items().get(index)?.BoxFragment().as_ref() }
            }
        }
    }

    fn Borders(&self) -> Edges {
        match self.ResidentBoxFragment() {
            Some(fragment) => {
                let borders = fragment.Borders();
                Edges {
                    top: borders.top.ToDouble(),
                    right: borders.right.ToDouble(),
                    bottom: borders.bottom.ToDouble(),
                    left: borders.left.ToDouble(),
                }
            }
            None => self.paint.border,
        }
    }

    fn Padding(&self) -> Edges {
        match self.ResidentBoxFragment() {
            Some(fragment) => {
                let padding = fragment.Padding();
                Edges {
                    top: padding.top.ToDouble(),
                    right: padding.right.ToDouble(),
                    bottom: padding.bottom.ToDouble(),
                    left: padding.left.ToDouble(),
                }
            }
            None => self.paint.padding,
        }
    }

    fn BorderImage(
        &self,
        resources: Option<&PaintResources>,
    ) -> Option<crate::internal::paint_input::NinePieceImagePaint> {
        LiveNinePieceImage(self, resources, |style| style.BorderImage())
            .unwrap_or_else(|| self.paint.style.border_image.clone())
    }

    fn MaskBoxImage(
        &self,
        resources: Option<&PaintResources>,
    ) -> Option<crate::internal::paint_input::NinePieceImagePaint> {
        LiveNinePieceImage(self, resources, |style| style.MaskBoxImage())
            .unwrap_or_else(|| self.paint.style.mask_box_image.clone())
    }

    fn BorderSides(&self) -> crate::boundary_types::FragmentBoxSides {
        match self.ResidentBoxFragment() {
            Some(fragment) => {
                let sides = fragment.SidesToInclude();
                crate::boundary_types::FragmentBoxSides {
                    top: sides.top,
                    right: sides.right,
                    bottom: sides.bottom,
                    left: sides.left,
                }
            }
            None => crate::boundary_types::FragmentBoxSides::default(),
        }
    }

    fn IsPageContainer(&self) -> bool {
        match self.ResidentBoxFragment() {
            Some(fragment) => {
                fragment.GetBoxType() == crate::physical_fragment::BoxType::kPageContainer
            }
            None => self.paint.is_page_container,
        }
    }

    fn IsPageBorderBox(&self) -> bool {
        match self.ResidentBoxFragment() {
            Some(fragment) => {
                fragment.GetBoxType() == crate::physical_fragment::BoxType::kPageBorderBox
            }
            None => self.paint.is_page_border_box,
        }
    }

    fn IsPaginatedRoot(&self) -> bool {
        match self.ResidentBoxFragment() {
            Some(fragment) => fragment.IsPaginatedRoot(),
            None => self.paint.is_paginated_root,
        }
    }

    fn PageContainerBackgroundGeometry(&self) -> Option<PageContainerBackgroundPaintData> {
        use crate::physical_box_fragment::PhysicalBoxFragment;
        let Some(fragment) = self.ResidentBoxFragment() else {
            return self.paint.page_container_background_geometry;
        };
        if fragment.GetBoxType() != crate::physical_fragment::BoxType::kPageContainer {
            return None;
        }
        // Source: pagination_utils.cc GetPageBorderBoxLink; view_painter.cc
        // subtracts this link offset from the background image origin.
        let page_border_link = fragment
            .Children()
            .iter()
            .find(|child| child.GetBoxType() == crate::physical_fragment::BoxType::kPageBorderBox)
            .expect("page container must contain its page border box");
        let page_border = page_border_link.get();
        assert!(!page_border.is_null());
        let page_border = unsafe { &*page_border.cast::<PhysicalBoxFragment>() };
        let border = page_border.Borders();
        let padding = page_border.Padding();
        Some(PageContainerBackgroundPaintData {
            positioning_box_offset: Offset {
                x: page_border_link.offset.left.ToDouble(),
                y: page_border_link.offset.top.ToDouble(),
            },
            border: Edges {
                top: border.top.ToDouble(),
                right: border.right.ToDouble(),
                bottom: border.bottom.ToDouble(),
                left: border.left.ToDouble(),
            },
            padding: Edges {
                top: padding.top.ToDouble(),
                right: padding.right.ToDouble(),
                bottom: padding.bottom.ToDouble(),
                left: padding.left.ToDouble(),
            },
            ..PageViewBackgroundFacts(fragment)
                .map(|(document, root_element_paint_offset)| PageContainerBackgroundPaintData {
                    positioning_size: document.positioning_size,
                    root_element_paint_offset,
                    has_background_fixed_to_viewport: document
                        .has_background_fixed_to_viewport,
                    ..Default::default()
                })
                .unwrap_or_default()
        })
    }

    fn PageDocumentBackgroundGeometry(&self) -> Option<PageBackgroundPaintData> {
        use crate::physical_box_fragment::PhysicalBoxFragment;
        let Some(fragment) = self.ResidentBoxFragment() else {
            return self.paint.page_document_background_geometry;
        };
        if fragment.GetBoxType() != crate::physical_fragment::BoxType::kPageBorderBox {
            return None;
        }
        // Source: pagination_utils.cc GetPageArea and
        // PhysicalBoxFragment::OffsetFromRootFragmentationContext().
        let page_area = fragment
            .Children()
            .first()
            .expect("page border box must contain its page area")
            .get();
        assert!(!page_area.is_null());
        let page_area = unsafe { &*page_area.cast::<PhysicalBoxFragment>() };
        debug_assert_eq!(
            page_area.GetBoxType(),
            crate::physical_fragment::BoxType::kPageArea
        );
        let offset = page_area.OffsetFromRootFragmentationContext();
        Some(PageBackgroundPaintData {
            positioning_area_offset: Offset {
                x: offset.left.ToDouble(),
                y: offset.top.ToDouble(),
            },
            ..PageViewBackgroundFacts(fragment)
                .map(|(document, _)| document)
                .unwrap_or_default()
        })
    }
}

/// The ::selection painting style, resolved on first use. Text painting
/// borrows it up front but reads it only while painting selected content.
pub struct LazySelectionPaintStyle<'a> {
    paint: &'a PaintProperties,
    style: std::cell::OnceCell<std::borrow::Cow<'a, PaintStyle>>,
}

impl std::ops::Deref for LazySelectionPaintStyle<'_> {
    type Target = PaintStyle;
    fn deref(&self) -> &PaintStyle {
        self.style.get_or_init(|| self.paint.SelectionPaintStyle())
    }
}

/// See PaintProperties::LivePaintLayerFacts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LivePaintLayerFacts {
    pub authoritative: bool,
    pub is_self_painting: bool,
    pub invisible_for_position_visibility: bool,
    pub has_ancestor_invisible_for_position_visibility: bool,
    pub has_visible_content: bool,
    pub has_self_painting_layer_descendant: bool,
    pub is_under_svg_hidden_container: bool,
    pub supports_subsequence_caching: bool,
    pub self_or_descendant_needs_repaint: bool,
    pub is_fragmented: bool,
    pub known_to_clip_subtree_to_padding_box: bool,
    pub child_paint_blocked_by_display_lock: bool,
    pub is_z_order_list_visible: bool,
}

impl PaintProperties {
    /// Source: core/highlight/highlight_style_utils.cc HighlightPaintingStyle.
    /// Chromium resolves the ::selection painting style from the live
    /// ComputedStyle while painting selected text. Projections without a live
    /// source (detached fixtures) keep the stored export.
    pub fn SelectionPaintStyle(&self) -> std::borrow::Cow<'_, PaintStyle> {
        let Some(source) = self.style.Source() else {
            return std::borrow::Cow::Borrowed(&self.selection_style);
        };
        let mut style =
            crate::internal::boundary::layout_boundary::ExportSelectionPaintStyle(source);
        // PrePaint applies the inherited auto-dark pause to the originating
        // style; the selection layer follows the same node state.
        let paused = self.style.auto_dark_mode_paused();
        if style.auto_dark_mode_paused != paused {
            style.auto_dark_mode_paused = paused;
        }
        std::borrow::Cow::Owned(style)
    }

    pub fn LazySelectionPaintStyle(&self) -> LazySelectionPaintStyle<'_> {
        LazySelectionPaintStyle {
            paint: self,
            style: std::cell::OnceCell::new(),
        }
    }

    /// Source: highlight_style_utils.cc:385-402, resolved like
    /// SelectionPaintStyle.
    pub fn SelectionTextDecoration(&self) -> Option<TextDecorationPaint> {
        self.style
            .Source()
            .and_then(crate::internal::boundary::layout_boundary::ExportSelectionTextDecoration)
    }

    /// Source: core/paint/text_painter.cc:743-787. The style of the first
    /// ancestor declaring the decoration (its ::first-line style for
    /// first-line fragments), resolved from the live LayoutObject tree while
    /// painting. Projections without a live source keep the stored value.
    pub fn SvgTextDecorationStyle(&self) -> Option<std::borrow::Cow<'_, PaintStyle>> {
        let Some(object) = self.SourceObject() else {
            return self
                .svg_text_decoration_style
                .as_ref()
                .map(std::borrow::Cow::Borrowed);
        };
        let mut style = crate::internal::boundary::layout_boundary::LiveSvgTextDecorationStyle(
            object,
            self.FactStyle().unwrap_or_else(|| object.StyleRef()),
        )?;
        // PrePaint applies the inherited auto-dark pause to this node's
        // originating style; the decorating style follows the same node state.
        style.auto_dark_mode_paused = self.style.auto_dark_mode_paused();
        Some(std::borrow::Cow::Owned(style))
    }
}

impl PaintProperties {
    pub fn LayoutObjectIdentity(
        &self,
    ) -> Option<crate::layout_engine::PaintLayoutObjectIdentity<'_>> {
        self.SourceObject()
            .map(|object| crate::layout_engine::PaintLayoutObjectIdentity { object })
    }

    pub(crate) fn BindSourceObject(&mut self, object: *const LayoutObject) {
        self.source_object_address = object as usize;
    }

    /// Borrow the resident Chromium LayoutObject that owns this fragment's
    /// paint inputs. The LayoutResult and its heap scope keep the object live;
    /// this does not create a second owner or an exported paint-data copy.
    ///
    /// Inside a `ResidentPaintSourceScope` (the paint tree walk) every bound
    /// source belongs to the leased resident result, so the address is read
    /// directly, as Chromium's painters read PhysicalFragment::layout_object_
    /// (core/layout/physical_fragment.h). Outside it the heap membership
    /// query still guards retained payloads.
    #[inline]
    pub(crate) fn SourceObjectPointer(&self) -> *mut LayoutObject {
        let object = self.source_object_address as *mut LayoutObject;
        if object.is_null() {
            return std::ptr::null_mut();
        }
        if ResidentPaintSourceScope::IsActive() {
            debug_assert!(foundation::IsManagedLayoutAddress(object));
            return object;
        }
        if foundation::IsManagedLayoutAddress(object) {
            object
        } else {
            std::ptr::null_mut()
        }
    }

    fn SourceObject(&self) -> Option<&LayoutObject> {
        unsafe { self.SourceObjectPointer().as_ref() }
    }

    /// The bound source LayoutObject's current PrePaint dirty bits, read live
    /// (LayoutObject::NeedsPaintPropertyUpdate, ShouldCheckForPaintInvalidation,
    /// their descendant/subtree variants and the PrePaint subtree-walk
    /// reasons; core/layout/layout_object.h), the same facts PrePaintTreeWalk
    /// admits nodes by (core/paint/pre_paint_tree_walk.cc:418-436). `None`
    /// for hand-built projections without a live source; callers then use
    /// the slot's `pre_paint_invalidation` snapshot.
    ///
    /// PrePaintTreeWalk clears these bits on every object it visits
    /// (ClearSourcePaintFlags; pre_paint_tree_walk.cc:1584-1585), so a clean
    /// result means "unchanged since the last PrePaint walk that visited this
    /// object", as in Chromium. Bits set by a path that updates paint state
    /// without a walk (the retained scroll property update, the direct
    /// composited-animation update) stay set until the next walk.
    pub fn LivePrePaintInvalidation(&self) -> Option<PrePaintInvalidation> {
        let object = self.SourceObjectPointer();
        if object.is_null() {
            return None;
        }
        Some(
            crate::internal::boundary::layout_boundary::ExportSelectionPrePaintInvalidation(
                object,
            ),
        )
    }

    fn SourcePaintLayerPointer(&self) -> *mut PaintLayer {
        Self::PaintLayerOfObject(self.SourceObjectPointer())
    }

    fn PaintLayerOfObject(object: *mut LayoutObject) -> *mut PaintLayer {
        if object.is_null() {
            return std::ptr::null_mut();
        }
        let box_model = foundation::DynamicTo::<LayoutBoxModelObject>(object);
        if box_model.is_null() {
            return std::ptr::null_mut();
        }
        unsafe { (&*box_model).Layer() }
    }

    fn SourcePaintLayer(&self) -> Option<&PaintLayer> {
        unsafe { self.SourcePaintLayerPointer().as_ref() }
    }

    /// Ensure the authoritative source layer is ready before PrePaint reads
    /// its descendant aggregates. The root pass remains the normal path; this
    /// per-layer gate covers relationships newly exposed by Rechrom's derived
    /// (LayoutObject-backed) layer links during the same lifecycle update.
    pub fn UpdateSourcePaintLayerDescendantDependentFlags(&self) {
        let layer = self.SourcePaintLayerPointer();
        if layer.is_null() {
            return;
        }
        unsafe { &mut *layer }.UpdateDescendantDependentFlags();
    }

    /// The bound source LayoutObject as an opaque PrePaint handle, for
    /// PaintInvalidator's pending delayed list (paint_invalidator.h
    /// pending_delayed_paint_invalidations_). `None` for detached fixtures.
    pub fn PrePaintSourceObject(&self) -> Option<PrePaintSourceObject> {
        let object = self.SourceObjectPointer();
        (!object.is_null()).then_some(PrePaintSourceObject(object))
    }

    /// BoxPaintInvalidator::SavePreviousBoxGeometriesIfNeeded
    /// (core/paint/box_paint_invalidator.cc:416-427,429-463,512-537) and the
    /// LayoutView half of ComputeViewBackgroundInvalidation
    /// (box_paint_invalidator.cc:254-260), on the bound source LayoutBox.
    /// PrePaint calls this at the end of the object's paint invalidation,
    /// after every invalidator has read the previous geometry ("This is for
    /// the next invalidatePaintIfNeeded so must be at the end").
    pub fn SaveSourcePreviousBoxGeometries(&self) {
        let object = self.SourceObjectPointer();
        if object.is_null() {
            return;
        }
        let layout_box = foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object);
        if layout_box.is_null() {
            return;
        }
        let box_ = unsafe { &mut *layout_box };
        let style = box_.StyleRef();
        // box_paint_invalidator.cc:429-445 NeedsToSavePreviousContentBoxRect.
        let save_content = box_.IsLayoutReplaced()
            || ((style.BackgroundLayers().AnyLayerUsesContentBox()
                || style.MaskLayers().AnyLayerUsesContentBox())
                && box_.PhysicalContentBoxRect().size != box_.StitchedSize());
        // box_paint_invalidator.cc:447-463 NeedsToSavePreviousOverflowData.
        let save_overflow = box_.HasVisualOverflow()
            || box_.HasScrollableOverflow()
            || ((style.BackgroundLayers().AnyLayerHasLocalAttachmentImage()
                || box_.GetBackgroundPaintLocation()
                    != crate::internal::layout_object::kBackgroundPaintInBorderBoxSpace
                || box_.IsDocumentElement())
                && box_.ScrollableOverflowRect() != box_.PhysicalBorderBoxRect());
        let mut mutable = box_.GetMutableForPainting();
        mutable.SavePreviousSize();
        if save_overflow {
            mutable.SavePreviousOverflowData();
        } else {
            mutable.ClearPreviousOverflowData();
        }
        if save_content {
            mutable.SavePreviousContentBoxRect();
        } else {
            mutable.ClearPreviousContentBoxRect();
        }
        let view = foundation::DynamicTo::<crate::internal::layout_view::LayoutView>(object);
        if !view.is_null() {
            let view = unsafe { &*view };
            view.SetPreviousBackgroundRect(view.BackgroundRect());
        }
    }

    /// ObjectPaintInvalidatorWithContext::ComputePaintInvalidationReason's
    /// LayoutObject::UpdatePreviousVisibilityVisible
    /// (core/paint/object_paint_invalidator.cc:49-53). PrePaint computed the
    /// reason from the slot's snapshot of the previous value; record the
    /// current visibility for the next walk.
    pub fn UpdateSourcePreviousVisibilityVisible(&self) {
        let object = self.SourceObjectPointer();
        if !object.is_null() {
            unsafe { &mut *object }
                .GetMutableForPainting()
                .UpdatePreviousVisibilityVisible();
        }
    }

    /// LayoutObject::ClearPaintFlags for the bound source object after
    /// PrePaintTreeWalk::Walk visited it and its children
    /// (core/paint/pre_paint_tree_walk.cc:1584-1585;
    /// core/layout/layout_object.cc:5142-5161,5191-5204).
    ///
    /// Chromium's walk follows the LayoutObject tree, so it also reaches the
    /// inline-level objects that own no fragment: collapsed text is cleared
    /// directly (pre_paint_tree_walk.cc:1265-1279) and culled inlines are
    /// entered through their containing block's items
    /// (pre_paint_tree_walk.cc:1282-1300). Rechrom walks fragments, which do
    /// not represent those objects, so a visited block container clears them
    /// here, after its fragment children were walked. Leaving them marked
    /// would stop a later descendant mark at the stale ancestor
    /// (MarkContainerChainForPaintInvalidation) and hide it from PrePaint.
    pub fn ClearSourcePaintFlags(&self) {
        let object = self.SourceObjectPointer();
        if object.is_null() {
            return;
        }
        let object = unsafe { &mut *object };
        object.ClearPaintFlags();
        if object.ChildPrePaintBlockedByDisplayLock()
            || !object.IsLayoutBlockFlow()
            || !object.ChildrenInline()
        {
            return;
        }
        fn clear_fragmentless_inline_children(parent: &LayoutObject) {
            let mut child = parent.SlowFirstChild();
            while let Some(current) = unsafe { child.as_mut() } {
                child = current.NextSibling();
                // Boxes, atomic inlines included, are visited through their
                // own fragments.
                if current.IsBox() {
                    continue;
                }
                if !current.HasInlineFragments() {
                    current.ClearPaintFlags();
                }
                if current.IsLayoutInline() && !current.ChildPrePaintBlockedByDisplayLock() {
                    clear_fragmentless_inline_children(current);
                }
            }
        }
        clear_fragmentless_inline_children(object);
    }

    /// DisplayItemClient::DebugName() of the bound source LayoutObject
    /// (platform/graphics/paint/display_item_client.h; LayoutObject::DebugName,
    /// translated in layoutng/internal/layout_object_core_services.cc:489-499),
    /// computed on demand as Chromium does. Paint asks only while recording
    /// paint debug info or tracing. Detached fixtures have an empty name.
    pub fn DisplayItemClientDebugName(&self) -> Arc<str> {
        match self.SourceObject() {
            Some(object) => Arc::from(object.DebugName().Utf8()),
            None => {
                static EMPTY_DEBUG_NAME: OnceLock<Arc<str>> = OnceLock::new();
                Arc::clone(EMPTY_DEBUG_NAME.get_or_init(|| Arc::from("")))
            }
        }
    }

    pub fn HasSource(&self) -> bool {
        self.SourceObject().is_some() || self.has_source
    }

    // Source: platform/graphics/paint/display_item_client.h:58-88 and
    // core/paint/object_paint_invalidator.cc:30-46. Chromium's PrePaint and
    // PaintController read and invalidate the LayoutObject / LayoutText
    // selection / PaintLayer DisplayItemClient itself. Resolve the live client
    // whose identity this payload exported; the stored *_invalidation_reason,
    // *_is_cacheable and *_is_just_created fields are only the detached
    // payload's state (fixtures and payloads without a managed source).
    fn LiveDisplayItemClient(&self) -> Option<&DisplayItemClient> {
        let object = self.SourceObject()?;
        Some(unsafe { &*(object as *const LayoutObject).cast::<DisplayItemClient>() })
    }

    fn LiveSelectionClient(&self) -> Option<&DisplayItemClient> {
        let object = self.SourceObjectPointer();
        if object.is_null() {
            return None;
        }
        let text = foundation::DynamicTo::<LayoutText>(object);
        unsafe { text.as_ref()?.GetSelectionDisplayItemClient().as_ref() }
    }

    fn LivePaintLayerClient(&self) -> Option<&DisplayItemClient> {
        self.SourcePaintLayer().map(|layer| layer.DisplayItemClient())
    }

    /// The source PaintLayer's DisplayItemClient id, 0 without a layer
    /// (cpp: core/paint/paint_layer.h:168-171).
    pub fn PaintLayerClientId(&self) -> u64 {
        self.SourceObject()
            .map(|_| self.SourcePaintLayer().map_or(0, |layer| layer.Id()))
            .unwrap_or(self.paint_layer_client_id)
    }

    /// LayoutObject::InsideBlockingWheelEventHandler(), which PrePaint
    /// updates on the LayoutObject (paint_property_tree_builder.cc).
    pub fn InsideBlockingWheelEventHandler(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::InsideBlockingWheelEventHandler)
            .unwrap_or(self.inside_blocking_wheel_event_handler)
    }

    fn ClientState(
        live: Option<&DisplayItemClient>,
        reason: Option<PaintInvalidationReason>,
        cacheable: bool,
        just_created: bool,
    ) -> PaintClientState {
        live.map_or(
            PaintClientState {
                invalidation_reason: reason,
                is_cacheable: cacheable,
                is_just_created: just_created,
            },
            |client| PaintClientState {
                invalidation_reason: Some(client.GetPaintInvalidationReason()),
                is_cacheable: client.IsCacheable(),
                is_just_created: client.IsJustCreated(),
            },
        )
    }

    pub fn DisplayItemClientState(&self) -> PaintClientState {
        Self::ClientState(
            self.LiveDisplayItemClient(),
            self.display_item_client_invalidation_reason,
            self.display_item_client_is_cacheable,
            self.display_item_client_is_just_created.get(),
        )
    }

    pub fn SelectionClientState(&self) -> PaintClientState {
        Self::ClientState(
            self.LiveSelectionClient(),
            self.selection_client_invalidation_reason,
            self.selection_client_is_cacheable,
            false,
        )
    }

    pub fn PaintLayerClientState(&self) -> PaintClientState {
        Self::ClientState(
            self.LivePaintLayerClient(),
            self.paint_layer_client_invalidation_reason,
            false,
            false,
        )
    }

    /// The live scrollbar, scroll-corner or scrolling-background client of
    /// this payload's source PaintLayerScrollableArea with exported identity
    /// `id` (core/paint/paint_layer_scrollable_area.h).
    fn LiveScrollClient(&self, id: u64) -> Option<&DisplayItemClient> {
        if id == 0 {
            return None;
        }
        let object = self.SourceObjectPointer();
        if object.is_null() {
            return None;
        }
        let box_model = foundation::DynamicTo::<LayoutBoxModelObject>(object);
        let area = unsafe { box_model.as_ref()?.GetScrollableArea().as_ref()? };
        let matches = |client: &DisplayItemClient| client.Id() as u64 == id;
        for scrollbar in [area.HorizontalScrollbar(), area.VerticalScrollbar()] {
            if let Some(scrollbar) = unsafe { scrollbar.as_ref() } {
                let client = scrollbar.DisplayItemClient();
                if matches(client) {
                    return Some(unsafe { &*(client as *const DisplayItemClient) });
                }
            }
        }
        if let Some(client) = area.ScrollCornerDisplayItemClient() {
            if matches(client) {
                return Some(unsafe { &*(client as *const DisplayItemClient) });
            }
        }
        let client = area.GetScrollingBackgroundDisplayItemClient();
        matches(client).then(|| unsafe { &*(client as *const DisplayItemClient) })
    }

    /// State of a scroll-owned client record (`id` and its detached mirror).
    pub fn ScrollClientState(
        &self,
        id: u64,
        reason: Option<PaintInvalidationReason>,
        cacheable: bool,
        just_created: bool,
    ) -> PaintClientState {
        Self::ClientState(self.LiveScrollClient(id), reason, cacheable, just_created)
    }

    pub fn HasLiveScrollClient(&self, id: u64) -> bool {
        self.LiveScrollClient(id).is_some()
    }

    pub fn InvalidateLiveScrollClient(&self, id: u64, reason: PaintInvalidationReason) -> bool {
        self.LiveScrollClient(id)
            .map(|client| client.Invalidate(reason))
            .is_some()
    }

    pub fn HasLiveDisplayItemClient(&self) -> bool {
        self.LiveDisplayItemClient().is_some()
    }

    pub fn HasLiveSelectionClient(&self) -> bool {
        self.LiveSelectionClient().is_some()
    }

    pub fn HasLivePaintLayerClient(&self) -> bool {
        self.LivePaintLayerClient().is_some()
    }

    /// DisplayItemClient::Invalidate on the live client. Returns false when
    /// this payload has no live client and the caller owns detached state.
    pub fn InvalidateLiveDisplayItemClient(&self, reason: PaintInvalidationReason) -> bool {
        self.LiveDisplayItemClient()
            .map(|client| client.Invalidate(reason))
            .is_some()
    }

    pub fn InvalidateLiveSelectionClient(&self, reason: PaintInvalidationReason) -> bool {
        self.LiveSelectionClient()
            .map(|client| client.Invalidate(reason))
            .is_some()
    }

    pub fn InvalidateLivePaintLayerClient(&self, reason: PaintInvalidationReason) -> bool {
        self.LivePaintLayerClient()
            .map(|client| client.Invalidate(reason))
            .is_some()
    }
    pub fn UpdateInsideBlockingWheelEventHandler(&mut self, inside: bool) -> bool {
        let changed = self.InsideBlockingWheelEventHandler() != inside;
        let object = self.SourceObjectPointer();
        if let Some(object) = unsafe { object.as_mut() } {
            object.UpdateInsideBlockingWheelEventHandler(inside);
            if changed {
                object.SetShouldInvalidatePaintForHitTest();
            }
        }
        self.inside_blocking_wheel_event_handler = inside;
        changed
    }
    pub fn InvalidatePaintForHitTest(&self) {
        self.SetPaintingLayerNeedsRepaint();
    }
    /// `context_.painting_layer->SetNeedsRepaint()`: Chromium marks the
    /// painting layer whenever PrePaint invalidates one of its clients
    /// (object_paint_invalidator.cc:95-121), so PaintLayerPainter's
    /// subsequence check sees the change. A detached payload has no layer.
    pub fn SetPaintingLayerNeedsRepaint(&self) {
        let layer = self
            .SourceObject()
            .map_or(std::ptr::null_mut(), LayoutObject::PaintingLayerDefault);
        if let Some(layer) = unsafe { layer.as_mut() } {
            layer.SetNeedsRepaint();
        }
    }

    pub fn IsLayoutView(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutView)
            .unwrap_or(self.is_layout_view)
    }

    pub fn IsGlobalRootScroller(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsGlobalRootScroller)
            .unwrap_or(self.is_layout_view)
    }

    pub fn NeedsLayout(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::NeedsLayout)
            .unwrap_or(self.needs_layout)
    }

    pub fn IsInclusiveDescendantOfUnboundedElement(&self) -> bool {
        self.SourceObject().map(LayoutObject::IsInclusiveDescendantOfUnboundedElement)
            .unwrap_or(false)
    }

    pub fn IsPrinting(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsPrintingForLayout)
            .unwrap_or(self.is_printing)
    }

    pub fn ShouldPrintBackgrounds(&self) -> bool {
        self.SourceObject().map(LayoutObject::ShouldPrintBackgroundsForLayout)
            .unwrap_or(self.should_print_backgrounds)
    }

    pub fn CanvasDrawElementEnabled(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.CanvasDrawElementEnabledForLayout()
            })
            .unwrap_or(self.canvas_draw_element_enabled)
    }

    /// Source NeedsElementCanvasTransform, paint_property_tree_builder.cc:592-608.
    /// The identity node is required even before an element transform is set.
    pub fn ElementCanvasTransformInput(&self) -> Option<(Option<usize>, TransformMatrix)> {
        let object = self.SourceObject()?;
        if object.IsText()
            || object.IsSVGChild()
            || !object.IsBoxModelObject()
            || !object.IsInCanvasSubtree()
        {
            return None;
        }
        let node = unsafe { object.GetNode().as_ref()? };
        if !node.IsElementNode() || !node.InputElementCanvasTransformEnabled() {
            return None;
        }
        let canvas = object.CanvasForDrawingLayoutObjectInfo()?;
        let element = foundation::DynamicTo::<crate::internal::layout_node_metadata::Element>(
            object.GetNode(),
        );
        let matrix = unsafe { element.as_ref()? }
            .GetUsedCanvasTransform()
            .map_or_else(
                || foundation::gfx::Transform::default().GetColMajor(),
                |m| m.GetColMajor(),
            );
        Some((
            (!canvas.is_null()).then_some(canvas as usize),
            TransformMatrix { values: matrix },
        ))
    }
    pub fn CanvasLayoutObjectIdentity(&self) -> Option<usize> {
        let object = self.SourceObject()?;
        object
            .IsCanvas()
            .then_some(object as *const LayoutObject as usize)
    }
    pub fn IsInCanvasSubtree(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsInCanvasSubtree)
            .unwrap_or(self.is_in_canvas_subtree)
    }

    pub fn IsOverscrollAreaParent(&self) -> bool {
        self.SourceObject().map(LayoutObject::IsOverscrollAreaParent)
            .unwrap_or(self.is_overscroll_area_parent)
    }

    pub fn IsOverscrollContainer(&self) -> bool {
        self.SourceObject().map(LayoutObject::IsOverscrollContainer)
            .unwrap_or(false)
    }

    pub fn ChildLayoutBlockedByDisplayLock(&self) -> bool {
        self.SourceObject().map(LayoutObject::ChildLayoutBlockedByDisplayLock)
            .unwrap_or(self.child_layout_blocked_by_display_lock)
    }

    pub fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        self.SourceObject().map(LayoutObject::ChildPaintBlockedByDisplayLock)
            .unwrap_or(self.child_paint_blocked_by_display_lock)
    }

    pub fn IsFragmentLessBox(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsFragmentLessBox)
            .unwrap_or(self.is_fragment_less_box)
    }

    pub fn AdditionalCompositingReasons(&self) -> foundation::CompositingReasons {
        self.SourceObject()
            .map(|object| {
                if object.CanHaveAdditionalCompositingReasons() {
                    object.AdditionalCompositingReasons()
                } else {
                    foundation::CompositingReasons::new()
                }
            })
            .unwrap_or(self.additional_compositing_reasons)
    }

    pub fn InForcedColorsMode(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.StyleRef().InForcedColorsMode()
            })
            .unwrap_or(self.in_forced_colors_mode)
    }

    pub fn IsBackgroundAttachmentFixedObject(&self) -> bool {
        self.SourceObject().map(LayoutObject::IsBackgroundAttachmentFixedObject)
            .unwrap_or(false)
    }

    /// Source: LocalFrameView::RequiresMainThreadScrollingForBackgroundAttachmentFixed
    /// (core/frame/local_frame_view.cc:1504-1527), read live from the
    /// LayoutView that owns the fixed-background object set. None for any
    /// other source object; hand-built fixtures keep the stored value.
    pub fn RequiresMainThreadScrollingForBackgroundAttachmentFixed(&self) -> Option<bool> {
        match self.SourceObject() {
            Some(object) => unsafe {
                foundation::DynamicTo::<crate::internal::layout_view::LayoutView>(
                    object as *const LayoutObject,
                )
                .as_ref()
            }
            .map(|view| view.RequiresMainThreadScrollingForBackgroundAttachmentFixed()),
            None => self.requires_main_thread_scrolling_for_background_attachment_fixed,
        }
    }

    pub fn IsBoxModelObject(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsBoxModelObject)
            .unwrap_or(self.is_box_model_object)
    }

    // Stage 2 of the export removal: LayoutObject facts read live (Chromium
    // PrePaint/Paint read them from the LayoutObject); the stored copy is
    // only the detached-fixture fallback.
    pub fn ReplacedIsImage(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutImage)
            .unwrap_or(self.replaced_is_image)
    }

    pub fn PaintLayerIsInTopOrViewTransitionLayer(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && object.IsInTopOrViewTransitionLayer()
            })
            .unwrap_or(false)
    }

    pub fn PaintLayerIsDirectLayoutViewChild(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && unsafe { object.Parent().as_ref() }.is_some_and(LayoutObject::IsLayoutView)
            })
            .unwrap_or(false)
    }

    pub fn PaintLayerCanContainAbsolutePositionObjects(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && object.CanContainAbsolutePositionObjects()
            })
            .unwrap_or(false)
    }

    pub fn PaintLayerCanContainFixedPositionObjects(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && object.CanContainFixedPositionObjects()
            })
            .unwrap_or(false)
    }

    pub fn PaintLayerHasOverlayOverflowControls(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && unsafe {
                        (&*foundation::DynamicTo::<LayoutBoxModelObject>(object))
                            .GetScrollableArea()
                            .as_ref()
                    }
                    .is_some_and(|area| area.HasOverlayOverflowControls())
            })
            .unwrap_or(false)
    }

    pub fn BackgroundPaintLocation(&self) -> BackgroundPaintLocation {
        self.SourceObject()
            .and_then(|object| {
                unsafe { foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object).as_ref() }
                    .map(crate::internal::boundary::layout_boundary::LiveBackgroundPaintLocation)
            })
            .unwrap_or(self.background_paint_location)
    }

    pub fn CssClip(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .and_then(|object| {
                unsafe { foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object).as_ref() }
                    .map(crate::internal::boundary::layout_boundary::LiveCssClip)
            })
            .flatten()
    }

    pub fn VisibleToHitTesting(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::VisibleToHitTesting)
            .unwrap_or(self.visible_to_hit_testing)
    }

    pub fn IsSelected(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsSelected)
            .unwrap_or(false)
    }

    pub fn EffectiveAllowedTouchAction(&self) -> foundation::TouchAction {
        self.SourceObject()
            .map(LayoutObject::EffectiveAllowedTouchAction)
            .unwrap_or(self.effective_allowed_touch_action)
    }

    pub fn BackgroundIsKnownToBeObscured(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::BackgroundIsKnownToBeObscured)
            .unwrap_or(self.background_is_known_to_be_obscured)
    }

    /// LayoutBoxModelObject::BackgroundTransfersToView()
    /// (core/layout/layout_box_model_object.cc:1123-1151) of the source
    /// object: the document element, or the first body whose background
    /// propagates to the canvas. BoxPainter skips its own background then and
    /// ViewPainter paints it. Rechrom's isolated SVG documents opt out through
    /// ViewPainter::ShouldApplyRootBackgroundBehavior (view_painter.cc:461).
    pub fn BackgroundTransfersToView(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                let model = foundation::DynamicTo::<LayoutBoxModelObject>(object);
                !model.is_null()
                    && (object.IsDocumentElement() || object.IsBody())
                    && object.ShouldApplyRootBackgroundBehaviorForLayout()
                    && unsafe { &*model }.BackgroundTransfersToViewDefault()
            })
            .unwrap_or(self.background_transfers_to_view)
    }

    pub fn IsSVGChild(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsSVGChild)
            .unwrap_or(self.is_svg_child)
    }

    pub fn IsSVGHiddenContainer(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsSVGHiddenContainer)
            .unwrap_or(self.svg_hidden_container)
    }

    pub fn SVGDescendantMayHaveTransformRelatedOperations(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::SVGDescendantMayHaveTransformRelatedOperations)
            .unwrap_or(self.svg_descendant_may_have_transform_related_operations)
    }

    pub fn BackgroundShouldAlwaysBeClipped(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                let box_ = foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object);
                !box_.is_null() && unsafe { &*box_ }.BackgroundShouldAlwaysBeClipped()
            })
            .unwrap_or(false)
    }

    pub fn CanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object).is_null()
                    && object.CanCompositeBackgroundAttachmentFixed()
            })
            .unwrap_or(false)
    }

    pub fn IsLayoutBox(&self) -> bool {
        self.SourceObject().map(|object| {
            !foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object).is_null()
        })
            .unwrap_or(self.is_layout_box)
    }

    pub fn IsLayoutInline(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutInline)
            .unwrap_or(self.is_layout_inline)
    }

    pub fn ShouldApplyPaintContainment(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.ShouldApplyPaintContainment()
            })
            .unwrap_or(false)
    }

    pub fn ShouldApplyStyleContainment(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.ShouldApplyStyleContainment()
            })
            .unwrap_or(false)
    }

    pub fn ShouldApplyLayoutContainment(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.ShouldApplyLayoutContainment()
            })
            .unwrap_or(false)
    }

    pub fn ShouldClipOverflowAlongEitherAxis(&self) -> bool {
        self.SourceObject().map(LayoutObject::ShouldClipOverflowAlongEitherAxis)
            .unwrap_or(self.should_clip_overflow_along_either_axis)
    }

    pub fn ShouldClipOverflowAlongBothAxes(&self) -> bool {
        self.SourceObject().map(LayoutObject::ShouldClipOverflowAlongBothAxis)
            .unwrap_or(self.should_clip_overflow_along_both_axes)
    }

    pub fn IsLayoutBlock(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutBlock)
            .unwrap_or(false)
    }

    pub fn IsLayoutBlockFlow(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutBlockFlow)
            .unwrap_or(self.is_layout_block_flow)
    }

    /// Every live PaintLayer fact PrePaint reads per visit, resolved through
    /// one validated source object and layer lookup instead of one per fact.
    /// Each field equals the matching single-fact accessor below.
    pub fn LivePaintLayerFacts(&self) -> LivePaintLayerFacts {
        let object_pointer = self.SourceObjectPointer();
        let object = unsafe { object_pointer.as_ref() };
        let layer = unsafe { Self::PaintLayerOfObject(object_pointer).as_ref() };
        LivePaintLayerFacts {
            authoritative: object.is_some() || self.paint_layer_state_is_authoritative,
            is_self_painting: layer.map(|layer| {
                layer.IsSelfPaintingLayer()
            })
            .unwrap_or(self.paint_layer_is_self_painting),
            invisible_for_position_visibility: layer.map(PaintLayer::InvisibleForPositionVisibility)
            .unwrap_or(false),
            has_ancestor_invisible_for_position_visibility: layer.map(PaintLayer::HasAncestorInvisibleForPositionVisibility)
            .unwrap_or(false),
            has_visible_content: layer.map(|layer| {
                layer.HasVisibleContent()
            })
            .unwrap_or(false),
            has_self_painting_layer_descendant: layer.map(PaintLayer::HasSelfPaintingLayerDescendant)
            .unwrap_or(false),
            is_under_svg_hidden_container: layer.map(PaintLayer::IsUnderSVGHiddenContainer)
            .unwrap_or(false),
            supports_subsequence_caching: layer.map(PaintLayer::SupportsSubsequenceCaching)
            .unwrap_or(false),
            self_or_descendant_needs_repaint: layer.map(PaintLayer::SelfOrDescendantNeedsRepaint)
            .unwrap_or(true),
            is_fragmented: layer.map(|layer| {
                layer.GetLayoutObject().IsFragmented()
            })
            .unwrap_or(self.paint_layer_is_fragmented),
            known_to_clip_subtree_to_padding_box: layer.map(PaintLayer::KnownToClipSubtreeToPaddingBox)
            .unwrap_or(false),
            child_paint_blocked_by_display_lock: object.map(LayoutObject::ChildPaintBlockedByDisplayLock)
            .unwrap_or(self.child_paint_blocked_by_display_lock),
            is_z_order_list_visible: layer.map(PaintLayer::IsZOrderListVisible)
            .unwrap_or(false),
        }
    }

    pub fn PaintLayerStateIsAuthoritative(&self) -> bool {
        self.SourceObject().is_some() || self.paint_layer_state_is_authoritative
    }

    pub fn PaintLayerIsSelfPainting(&self) -> bool {
        self.SourcePaintLayer()
            .map(|layer| {
                layer.IsSelfPaintingLayer()
            })
            .unwrap_or(self.paint_layer_is_self_painting)
    }

    pub fn PaintLayerInvisibleForPositionVisibility(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::InvisibleForPositionVisibility)
            .unwrap_or(false)
    }

    pub fn PaintLayerHasAncestorInvisibleForPositionVisibility(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::HasAncestorInvisibleForPositionVisibility)
            .unwrap_or(false)
    }

    pub fn PaintLayerHasVisibleContent(&self) -> bool {
        self.SourcePaintLayer()
            .map(|layer| {
                layer.HasVisibleContent()
            })
            .unwrap_or(false)
    }

    pub fn PaintLayerHasSelfPaintingLayerDescendant(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::HasSelfPaintingLayerDescendant)
            .unwrap_or(false)
    }

    pub fn PaintLayerHasNonIsolatedDescendantWithBlendMode(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::HasNonIsolatedDescendantWithBlendMode)
            .unwrap_or(false)
    }

    pub fn PaintLayerIsUnderSvgHiddenContainer(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::IsUnderSVGHiddenContainer)
            .unwrap_or(false)
    }

    pub fn PaintLayerSupportsSubsequenceCaching(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::SupportsSubsequenceCaching)
            .unwrap_or(false)
    }

    pub fn PaintLayerSelfOrDescendantNeedsRepaint(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::SelfOrDescendantNeedsRepaint)
            .unwrap_or(true)
    }

    pub fn PaintLayerIsFragmented(&self) -> bool {
        self.SourcePaintLayer()
            .map(|layer| {
                layer.GetLayoutObject().IsFragmented()
            })
            .unwrap_or(self.paint_layer_is_fragmented)
    }

    pub fn PaintLayerKnownToClipSubtreeToPaddingBox(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::KnownToClipSubtreeToPaddingBox)
            .unwrap_or(false)
    }

    pub fn PaintLayerIsStackingContext(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.IsStackingContext()
            })
            .unwrap_or(self.paint_layer_is_stacking_context)
    }

    pub fn PaintLayerIsStacked(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsStacked)
            .unwrap_or(self.paint_layer_is_stacked)
    }

    pub fn PaintLayerStackingLevel(&self) -> i32 {
        self.SourceObject()
            .map(|object| {
                object.StyleRef().EffectiveZIndex()
            })
            .unwrap_or(self.paint_layer_stacking_level)
    }

    pub fn PaintLayerIsZOrderListVisible(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::IsZOrderListVisible)
            .unwrap_or(false)
    }
    pub fn CanPaintFragmentOccurrence(&self) -> bool {
        // The payload's CanPaintMultipleFragments() fallback was never
        // exported (always true); resident boxes answer through
        // ResidentBoxGeometry::ViewCanPaintMultipleFragments.
        true
    }

    // LayoutObject / LayoutBox / PaintLayer / Element geometry facts, read
    // live as Chromium's PrePaint and painters do (stage 2 of the export
    // removal). The stored fields remain only for fragments without a source.

    pub fn CssMaskBoundingBox(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveCssMaskBoundingBox)
            .unwrap_or(self.css_mask_bounding_box)
    }

    pub fn ScrollableOverflow(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveScrollableOverflow)
            .unwrap_or(self.scrollable_overflow)
    }

    pub fn PaintLayerFirstFragmentVisualRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(|_| {
                self.SourcePaintLayer().map(
                    crate::internal::boundary::layout_boundary::LivePaintLayerFirstFragmentVisualRect,
                )
            })
            .unwrap_or(self.paint_layer_first_fragment_visual_rect)
    }

    pub fn PaintLayerContentsVisualRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(|_| {
                self.SourcePaintLayer().and_then(
                    crate::internal::boundary::layout_boundary::LivePaintLayerContentsVisualRect,
                )
            })
            .unwrap_or(self.paint_layer_contents_visual_rect)
    }

    pub fn PaintLayerFilterReferenceBox(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(|_| {
                self.SourcePaintLayer().and_then(
                    crate::internal::boundary::layout_boundary::LivePaintLayerFilterReferenceBox,
                )
            })
            .unwrap_or(self.paint_layer_filter_reference_box)
    }

    pub fn PaintLayerFilterViewport(&self) -> Option<Size> {
        self.SourceObject()
            .map(|_| {
                self.SourcePaintLayer().and_then(
                    crate::internal::boundary::layout_boundary::LivePaintLayerFilterViewport,
                )
            })
            .unwrap_or(self.paint_layer_filter_viewport)
    }

    pub fn ReplacedSelectionState(&self) -> SelectionState {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveReplacedSelectionState)
            .unwrap_or(self.replaced_selection_state)
    }

    pub fn ReplacedLocalSelectionVisualRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveReplacedLocalSelectionVisualRect)
            .unwrap_or(self.replaced_local_selection_visual_rect)
    }

    pub fn RegionCaptureCropId(&self) -> Option<[u64; 2]> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveRegionCaptureCropId)
            .unwrap_or(self.region_capture_crop_id)
    }

    pub fn TrackedElementSubRects(
        &self,
    ) -> &[crate::internal::layout_input::TrackedElementSubRect] {
        match self.SourceObject() {
            Some(object) => {
                crate::internal::boundary::layout_boundary::LiveTrackedElementSubRects(object)
            }
            None => &self.tracked_element_sub_rects,
        }
    }

    // Source Node / Element facts. `None` from the live query means the
    // LayoutObject has no source Node (anonymous), for which ExportProperties
    // exported nothing; the stored defaults are used then.

    pub fn SourceKind(&self) -> NodeKind {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveSourceKind)
            .unwrap_or(self.source_kind)
    }

    pub fn ImageDecodingMode(&self) -> image_resource::ImageDecodingMode {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveImageDecodingMode)
            .unwrap_or(self.image_decoding_mode)
    }

    pub fn VideoFrameResourceId(&self) -> Option<image_resource::ImageId> {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveVideoFrameResourceId)
            .unwrap_or(self.video_frame_resource_id)
    }

    pub fn FocusedImageMapArea(&self) -> Option<&FocusedImageMapArea> {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveFocusedImageMapArea)
            .flatten()
    }

    pub fn SvgViewBoxIsEmpty(&self) -> bool {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveSvgViewBoxIsEmpty)
            .unwrap_or(self.svg_view_box_is_empty)
    }

    pub fn TextControlHost(&self) -> Option<u64> {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveTextControlHost)
            .flatten()
    }

    // Previous-frame geometry is Chromium's LayoutBox / LayoutView state,
    // saved by the layout commit after the artifact commits.

    pub fn PreviousBoxGeometry(&self) -> Option<PreviousBoxGeometry> {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LivePreviousBoxGeometry)
            .unwrap_or(self.previous_box_geometry)
    }

    pub fn ViewBackgroundRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveViewBackgroundRect)
            .unwrap_or(self.view_background_rect)
    }

    pub fn PreviousViewBackgroundRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LivePreviousViewBackgroundRect)
            .flatten()
    }

    pub fn DisplayItemRasterEffectOutset(&self) -> RasterEffectOutset {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveRasterEffectOutset)
            .unwrap_or(RasterEffectOutset::kNone)
    }

    // DisplayItemClient identity. Chromium's PaintController and PrePaint
    // use the LayoutObject (and LayoutText selection) DisplayItemClient
    // itself (display_item_client.h:33-35,71-82).

    pub fn DisplayItemClientId(&self) -> u64 {
        self.SourceObject()
            .map(|object| {
                unsafe { &*(object as *const LayoutObject as *const DisplayItemClient) }.Id() as u64
            })
            .unwrap_or(self.display_item_client_id)
    }

    pub fn DisplayItemClientOwnerNodeId(&self) -> i32 {
        self.SourceObject()
            .map(|object| object.OwnerNodeIdDefault())
            .unwrap_or(0)
    }

    pub fn SelectionClientId(&self) -> u64 {
        self.SourceObject()
            .map(|object| {
                let text = foundation::DynamicTo::<LayoutText>(
                    object as *const LayoutObject as *mut LayoutObject,
                );
                unsafe { text.as_ref() }
                    .and_then(|text| unsafe { text.GetSelectionDisplayItemClient().as_ref() })
                    .map_or(0, |client| client.Id() as u64)
            })
            .unwrap_or(self.selection_client_id)
    }

    /// The control element's theme-painting inputs, built from the source
    /// Element and the fragment's effective style.
    pub fn FormControl(&self) -> Option<Arc<FormControlPaintData>> {
        match self.SourceObject() {
            Some(object) => crate::internal::boundary::layout_boundary::LiveFormControl(
                object,
                self.FactStyle().unwrap_or_else(|| object.StyleRef()),
            ),
            None => self.form_control.clone(),
        }
    }

    /// The sticky translation, read from the LayoutBoxModelObject's current
    /// StickyPositionOffset (PaintPropertyTreeBuilder::UpdateStickyTranslation).
    pub fn StickyOffset(&self) -> Offset {
        match self.SourceObject() {
            Some(object) => {
                crate::internal::boundary::layout_boundary::LiveStickyPaintProperties(
                    object as *const LayoutObject,
                )
                .0
            }
            None => self.sticky_offset,
        }
    }

    /// The compositor's sticky constraint basis from the same sticky update.
    pub fn StickyConstraint(&self) -> Option<StickyPositionPaintData> {
        match self.SourceObject() {
            Some(object) => {
                crate::internal::boundary::layout_boundary::LiveStickyPaintProperties(
                    object as *const LayoutObject,
                )
                .1
            }
            None => self.sticky_constraint.clone(),
        }
    }

    pub fn SvgShape(&self) -> Option<&Arc<SvgShapeData>> {
        match self
            .SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveSvgShape)
        {
            Some(live) => live,
            None => self.svg_shape.as_ref(),
        }
    }

    fn SvgPaintGeometry(
        &self,
    ) -> Option<(Option<TransformMatrix>, Option<SvgViewportGeometry>, Option<FragmentPaintRect>)>
    {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveSvgPaintGeometry)
    }

    pub fn SvgLocalTransform(&self) -> Option<TransformMatrix> {
        self.SvgPaintGeometry().and_then(|(transform, _, _)| transform)
    }

    pub fn SvgViewport(&self) -> Option<SvgViewportGeometry> {
        match self.SvgPaintGeometry() {
            Some((_, viewport, _)) => viewport,
            None => self.svg_viewport.clone(),
        }
    }

    pub fn SvgVisualRect(&self) -> Option<FragmentPaintRect> {
        self.SvgPaintGeometry().and_then(|(_, _, visual_rect)| visual_rect)
    }

    pub fn SvgViewBox(&self) -> Option<SvgViewBoxData> {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveSvgViewBox)
            .flatten()
    }

    /// Whether the source is an anonymous LayoutTextCombine wrapper.
    pub fn IsTextCombine(&self) -> bool {
        match self.SourceObject() {
            Some(object) => !foundation::DynamicTo::<
                crate::internal::layout_text_combine::LayoutTextCombine,
            >(object as *const LayoutObject)
            .is_null(),
            None => self.text_combine.is_some(),
        }
    }

    pub fn TextCombine(&self) -> Option<TextCombinePaintData> {
        match self.SourceObject() {
            Some(object) => crate::internal::boundary::layout_boundary::LiveTextCombine(object),
            None => self.text_combine.clone(),
        }
    }

    /// The source scroll offset (PaintLayerScrollableArea::GetScrollOffset or
    /// the element's committed input). The stored copy is only the fallback
    /// for hand-built fixture slots without a bound source.
    pub fn ScrollOffset(&self) -> Offset {
        match self.SourceObject() {
            Some(object) => crate::internal::boundary::layout_boundary::LiveScrollOffset(object),
            None => self.scroll_offset,
        }
    }

    /// The source PaintLayerScrollableArea of this payload's LayoutObject.
    fn SourceScrollableArea(
        &self,
    ) -> Option<&crate::internal::layout_scrollable_area::PaintLayerScrollableArea> {
        let object = self.SourceObjectPointer();
        if object.is_null() {
            return None;
        }
        let box_model = foundation::DynamicTo::<LayoutBoxModelObject>(object);
        unsafe { box_model.as_ref()?.GetScrollableArea().as_ref() }
    }

    /// PaintLayerScrollableArea::LastCullRectUpdateScrollPosition() of the
    /// source scroll container: the scroll offset its contents CullRect was
    /// last computed for (core/paint/cull_rect_updater.cc:193-206). Paint
    /// owns this bookkeeping on the scroll owner; slots without a source or
    /// scrollable area have none.
    pub fn LastCullRectUpdateScrollPosition(&self) -> Option<Offset> {
        self.SourceScrollableArea()?
            .LastCullRectUpdateScrollPosition()
            .map(|(x, y)| Offset { x, y })
    }

    /// Record the scroll offset the contents CullRect was computed for
    /// (CullRectUpdater::UpdateForSelf ->
    /// PaintLayerScrollableArea::SetLastCullRectUpdateScrollPosition).
    /// Returns whether the recorded position changed.
    pub fn SetLastCullRectUpdateScrollPosition(&self, position: Offset) -> bool {
        self.SourceScrollableArea().is_some_and(|area| {
            area.SetLastCullRectUpdateScrollPosition((position.x, position.y))
        })
    }

    /// PaintLayerStackingNode's flex/grid `order` comparison inputs: the
    /// shared parent's UniqueObjectId and the order value.
    pub fn PaintLayerOrder(&self) -> (Option<u64>, i32) {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LivePaintLayerOrder)
            .unwrap_or((self.paint_layer_order_parent_unique_id, self.paint_layer_order))
    }

    /// The source Node id of the LayoutObject's nearest ancestor with a Node.
    pub fn LogicalParentNodeId(&self) -> Option<u64> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveLogicalParentNodeId)
            .flatten()
    }

    /// The source LayoutObject's resident document-order position: the
    /// 1-based preorder index among LayoutObjects with a Node, maintained on
    /// the object by the layout boundary object index after each topology
    /// mutation (LayoutObject::LogicalTreeOrder). 0 when the source has no
    /// Node. Hand-built fixtures without a live source keep the stored value.
    pub fn LogicalTreeOrder(&self) -> u64 {
        self.SourceObject()
            .map(LayoutObject::LogicalTreeOrder)
            .unwrap_or(self.logical_tree_order)
    }

    pub(crate) fn BindLiveStyleFacts(&mut self, live: bool) {
        self.live_style_facts = live;
    }

    pub(crate) fn LiveStyleFacts(&self) -> bool {
        self.live_style_facts
    }

    /// The fragment's effective ComputedStyle (LayoutObject::StyleRef(), or
    /// the ::first-line style for first-line fragments) when the facts below
    /// were projected from it.
    fn FactStyle(&self) -> Option<&layout_style::style::computed_style::ComputedStyle> {
        self.live_style_facts.then(|| self.style.Source()).flatten()
    }

    fn FactLayoutBox(&self) -> Option<&crate::internal::layout_box::LayoutBox> {
        if !self.live_style_facts {
            return None;
        }
        self.SourceObject().map(|object| unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object).as_ref()
        })?
    }

    pub fn Margin(&self) -> Edges {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleMargin)
            .unwrap_or(self.margin)
    }

    pub fn BorderStyles(&self) -> [BorderLineStyle; 4] {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleBorderStyles)
            .unwrap_or(self.border_styles)
    }

    pub fn Display(&self) -> Display {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleDisplay)
            .unwrap_or(self.display)
    }

    pub fn Position(&self) -> Position {
        self.FactStyle()
            .map(crate::style_projection::FragmentStylePosition)
            .unwrap_or(self.position)
    }

    pub fn Floating(&self) -> FloatSide {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleFloating)
            .unwrap_or(self.floating)
    }

    pub fn WritingMode(&self) -> WritingMode {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleWritingMode)
            .unwrap_or(self.writing_mode)
    }

    pub fn Direction(&self) -> TextDirection {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleDirection)
            .unwrap_or(self.direction)
    }

    pub fn OverflowX(&self) -> Overflow {
        self.FactStyle()
            .map(|style| crate::style_projection::FragmentStyleOverflow(style.OverflowX()))
            .unwrap_or(self.overflow_x)
    }

    pub fn OverflowY(&self) -> Overflow {
        self.FactStyle()
            .map(|style| crate::style_projection::FragmentStyleOverflow(style.OverflowY()))
            .unwrap_or(self.overflow_y)
    }

    pub fn OverscrollBehaviorX(&self) -> OverscrollBehavior {
        self.FactStyle()
            .map(|style| {
                crate::internal::boundary::layout_boundary::LiveOverscrollBehavior(
                    style.OverscrollBehaviorX(),
                )
            })
            .unwrap_or(self.overscroll_behavior_x)
    }

    pub fn OverscrollBehaviorY(&self) -> OverscrollBehavior {
        self.FactStyle()
            .map(|style| {
                crate::internal::boundary::layout_boundary::LiveOverscrollBehavior(
                    style.OverscrollBehaviorY(),
                )
            })
            .unwrap_or(self.overscroll_behavior_y)
    }

    // Source: core/paint/paint_property_tree_builder.cc:3634-3635.
    pub fn PreventScrollAxisLocking(&self) -> bool {
        self.FactStyle()
            .map(|style| style.ScrollAxisLock() == foundation::EScrollAxisLock::kNone)
            .unwrap_or(self.prevent_scroll_axis_locking)
    }

    pub fn EffectiveZoom(&self) -> f32 {
        self.FactStyle()
            .map(|style| style.EffectiveZoom())
            .unwrap_or(self.effective_zoom)
    }

    pub fn HasBackground(&self) -> bool {
        self.FactStyle()
            .map(|style| style.HasBackground())
            .unwrap_or(self.has_background)
    }

    pub fn BackgroundFillLayerCount(&self) -> usize {
        self.FactStyle()
            .map(crate::internal::boundary::paint_style_export::BackgroundFillLayerCount)
            .unwrap_or(self.background_fill_layer_count)
    }

    pub fn BackgroundAnyLocalAttachmentImage(&self) -> bool {
        self.FactStyle()
            .map(|style| style.BackgroundLayers().AnyLayerHasLocalAttachmentImage())
            .unwrap_or(self.background_any_local_attachment_image)
    }

    pub fn BackgroundAnyDefaultAttachmentImage(&self) -> bool {
        self.FactStyle()
            .map(|style| style.BackgroundLayers().AnyLayerHasDefaultAttachmentImage())
            .unwrap_or(self.background_any_default_attachment_image)
    }

    pub fn BackgroundAnyLayerUsesContentBox(&self) -> bool {
        self.FactStyle()
            .map(|style| style.BackgroundLayers().AnyLayerUsesContentBox())
            .unwrap_or(self.background_any_layer_uses_content_box)
    }

    pub fn FixedToView(&self) -> bool {
        self.live_style_facts
            .then(|| self.FactLayoutBox().is_some_and(|box_| box_.IsFixedToView(std::ptr::null())))
            .unwrap_or(self.fixed_to_view)
    }

    pub fn NeedsAnchorPositionScrollAdjustment(&self) -> bool {
        self.live_style_facts
            .then(|| {
                self.FactLayoutBox()
                    .is_some_and(|box_| box_.NeedsAnchorPositionScrollAdjustment())
            })
            .unwrap_or(self.needs_anchor_position_scroll_adjustment)
    }

    // C++ chained assignment evaluates the right-hand assignment first.
    // cpp: layoutng_fragment_tree/fragment_tree.h:96
    pub fn SetOverflow(&mut self, value: Overflow) {
        self.overflow_y = value;
        self.overflow_x = value;
    }
}
