use std::sync::{Arc, OnceLock};

pub use layout_fragment_boundary::{
    BackgroundPaintLocation, CollapsedTableEdge, CollapsedTablePaintData, CollapsedTableSection,
    ColumnRule, ColumnRulePaintData, FieldsetPaintData, FormControlPaintData, FragmentBoxSides,
    FragmentKind, FragmentPaintRect, FrameSetPaintData, GapRule, GapRulePaintData, GapRuleSide,
    InlineGeometry, InlineGeometryRect, LayoutResultHandle, MathMLPaintData, MathMLPaintKind,
    PaintGlyph, PaintGlyphRun, PaintResources, PrePaintInvalidation, PreviousBoxGeometry,
    ReplacedContentPaintData, ScrollContainerPaintData, ScrollSnapAxisPaintData,
    ScrollSnapStrictnessPaintData, ScrollSnapTypePaintData, ScrollbarPaintAxis, ScrollbarPaintData,
    SelfOutlinePaintData, StickyAxisPaintData, StickyPositionPaintData, StitchedDecorationData,
    SvgTextPaintData, SvgViewportGeometry, TextCombineEmphasisPaintData, TextCombinePaintData,
};

#[path = "pre_paint_revision.rs"]
pub mod pre_paint_revision;
pub use pre_paint_revision::SamePrePaintInput;

pub use foundation::graphics_types::graphics::paint::display_item_client_types::RasterEffectOutset;
use foundation::{LayoutOffset as Offset, LayoutSize as Size};
use layoutng::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng::internal::layout_input::{
    BorderLineStyle, BoxDecorationBreak, Display, Edges, FloatSide, FocusedImageMapArea,
    ListStyleType, NodeKind, Overflow, OverscrollBehavior, PaintStyle, Position, SvgShapeData,
    SvgViewBoxData, TextDecorationPaint, TextDirection, TransformMatrix, WritingMode,
};
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::paint_layer::PaintLayer;
use layoutng::internal::selection_state::SelectionState;

// The C++ definitions are public API; all layoutng::* names above retain their
// owner in the later layoutng crate. The Cargo cycle needs an interface split.

// cpp: layoutng_fragment_tree/fragment_tree.h:25
// Defaulted C++ equality maps to the derived fieldwise comparison above.

// cpp: layoutng_fragment_tree/fragment_tree.h:131
// Defaulted C++ equality maps to the derived fieldwise comparison above.

/// Assembly specialization of fragment-owned table geometry with LayoutNG's
/// immutable paint-style snapshot. Existing consumers keep the original API.
pub type TablePaintColumn = layout_fragment_boundary::TablePaintColumn<PaintStyle>;
pub type TablePaintData = layout_fragment_boundary::TablePaintData<PaintStyle>;

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
    layout_fragment_boundary::CanvasBackgroundPaintData<PaintStyle>;

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
/// fragment-local storage and sharing remain owned by `layout_fragment`.
pub type PaintPropertiesHandle = layout_fragment_boundary::PaintPropertiesHandle<PaintProperties>;
pub type FragmentPaintData = layout_fragment_boundary::FragmentPaintData<PaintProperties>;

#[derive(Clone, PartialEq)]
pub struct PaintProperties {
    // Non-owning reference to the canonical LayoutObject. Resident fragment
    // leases keep the source layout result alive through PrePaint and Paint.
    // Production getters below read object facts through this identity rather
    // than copying them into the fragment boundary.
    source_object_address: usize,
    /// Chromium LayoutObject dirty bits consumed by Chromium's
    /// PrePaintTreeWalk. They are lifecycle facts, not drawing inputs: the
    /// layout boundary snapshots them before Paint and clears them only after
    /// the corresponding artifact commits.
    pub pre_paint_invalidation: PrePaintInvalidation,
    pub property_fragment_unique_id: u64,
    /// Chromium PhysicalBoxFragment::IsFirstForNode() and the canonical
    /// CanPaintMultipleFragments() result. Paint consumes these directly
    /// instead of duplicating LayoutObject-type fragmentation policy.
    pub is_first_for_node: bool,
    pub can_paint_multiple_fragments: bool,
    pub is_layout_view: bool,
    /// Exact LayoutObject/Document facts consumed by
    /// CullRectUpdater::ShouldUseInfiniteCullRect. PrePaint must not infer
    /// these from style or fragment geometry.
    pub is_inclusive_descendant_of_unbounded_element: bool,
    pub is_printing: bool,
    /// Host Settings::GetShouldPrintBackgrounds() snapshot. This is distinct
    /// from printing itself because print-economy color rewriting is disabled
    /// when the caller explicitly requests backgrounds.
    pub should_print_backgrounds: bool,
    pub canvas_draw_element_enabled: bool,
    pub is_in_canvas_subtree: bool,
    pub is_overscroll_area_parent: bool,
    pub is_overscroll_container: bool,
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
    pub should_apply_paint_containment: bool,
    pub should_apply_style_containment: bool,
    pub should_apply_layout_containment: bool,
    pub should_clip_overflow_along_either_axis: bool,
    pub should_clip_overflow_along_both_axes: bool,
    pub is_layout_block: bool,
    pub is_layout_block_flow: bool,
    /// PhysicalBoxFragment::IsInlineFormattingContext(). Forced-colors
    /// backplates belong to the block that owns FragmentItems, not to an
    /// arbitrary ancestor reconstructed from the flattened paint tree.
    pub is_inline_formatting_context: bool,
    pub in_forced_colors_mode: bool,
    pub is_background_attachment_fixed_object: bool,
    /// Source: core/frame/local_frame_view.cc:1504-1527. Published only by the
    /// actual LayoutView after querying source fixed-background membership.
    pub requires_main_thread_scrolling_for_background_attachment_fixed: Option<bool>,
    /// PaintController snapshots the client's actual reason when appending an
    /// item (paint_controller.cc:531-545). None is an unexported external input.
    pub display_item_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub display_item_client_debug_name: std::sync::Arc<str>,
    pub paint_layer_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    // None means source ink overflow has not been computed. Never substitute
    // a frame rect here: text glyphs and decorations can extend outside it.
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
    /// Content box used only by LineBoxFragmentPainter for a `::first-line`
    /// background. None means this line owns no line-box decoration item.
    pub first_line_decoration_rect: Option<FragmentPaintRect>,
    // Baseline-adjusted, writing-mode converted glyph bounds. This remains
    // available when full text decoration/emphasis overflow is not implemented.
    pub text_glyph_ink: Option<FragmentPaintRect>,
    /// FragmentItem::ComputeTextBoundsRectForHitTest() in item-local
    /// coordinates for ordinary point-event hit testing. SVG text uses its
    /// transformed quad path and therefore leaves this unset.
    pub text_hit_test_bounds: Option<FragmentPaintRect>,
    // BoxPainter::VisualRect uses LayoutBox::SelfVisualOverflowRect(), which
    // is a stitched LayoutBox rect rather than a physical fragment ink rect.
    pub box_self_visual_overflow: Option<FragmentPaintRect>,
    /// Exact FragmentPainter outline geometry. Some(empty) is authoritative:
    /// Chromium returns without recording an outline for that fragment.
    pub self_outline: Option<SelfOutlinePaintData>,
    pub previous_box_geometry: Option<PreviousBoxGeometry>,
    pub box_fragment_is_inline_box: bool,
    pub display_item_raster_effect_outset: RasterEffectOutset,
    // Chromium DisplayItemClient identity and lifecycle, independently of DOM
    // node ids and the source occurrence's fragment_instance_id.
    // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
    pub display_item_client_id: u64,
    /// Chromium DisplayItemClient::OwnerNodeId(), kept distinct from the
    /// layout/export node identity above. DOMNodeId is a signed 32-bit type.
    pub display_item_client_owner_node_id: i32,
    // A distinct persistent source PaintLayer DisplayItemClient, when present.
    // cpp: core/paint/paint_layer.h:168-171
    /// Chromium layer ownership was queried, including the authoritative no-layer result.
    /// False is reserved for standalone fragments without a source Layout owner.
    pub paint_layer_state_is_authoritative: bool,
    pub paint_layer_client_id: u64,
    pub paint_layer_client_is_cacheable: bool,
    pub paint_layer_client_is_just_created: std::cell::Cell<bool>,
    pub paint_layer_is_self_painting: bool,
    pub paint_layer_invisible_for_position_visibility: bool,
    pub paint_layer_has_ancestor_invisible_for_position_visibility: bool,
    /// PaintLayer::HasVisibleContent(), kept separate from z-list visibility:
    /// the latter also includes visible self-painting descendants and view
    /// transition names, while PaintLayerPainter gates this layer's own phases
    /// on visible content only.
    pub paint_layer_has_visible_content: bool,
    pub paint_layer_has_self_painting_layer_descendant: bool,
    /// PaintLayer::HasNonIsolatedDescendantWithBlendMode(). Chromium creates
    /// an isolation Effect only when this aggregate is true; `isolation:
    /// isolate` by itself is only a stacking-context boundary.
    pub paint_layer_has_non_isolated_descendant_with_blend_mode: bool,
    pub paint_layer_is_under_svg_hidden_container: bool,
    /// Source: PaintLayer::SupportsSubsequenceCaching().
    pub paint_layer_supports_subsequence_caching: bool,
    /// Source: PaintLayer::SelfOrDescendantNeedsRepaint().
    pub paint_layer_self_or_descendant_needs_repaint: bool,
    /// Exact inputs to PaintLayerPainter's PaintResult/cull admission. These
    /// are source LayoutObject/PaintLayer facts, not reconstructable from a
    /// physical fragment or projected CSS style.
    pub paint_layer_is_fragmented: bool,
    pub paint_layer_known_to_clip_subtree_to_padding_box: bool,
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
    pub paint_layer_is_z_order_list_visible: bool,
    pub paint_layer_is_in_top_or_view_transition_layer: bool,
    pub paint_layer_is_direct_layout_view_child: bool,
    /// Chromium PaintLayerStackingNode inputs for overlay overflow-control
    /// reordering. These are exported by Layout because Paint must not infer
    /// containing-block or flex/grid order semantics from flattened style.
    pub paint_layer_can_contain_absolute_position_objects: bool,
    pub paint_layer_can_contain_fixed_position_objects: bool,
    pub paint_layer_has_overlay_overflow_controls: bool,
    /// UniqueId of the flex/grid ordering ancestor's parent. None means the
    /// PaintLayer is not a child/grandchild covered by OrderLessThan().
    pub paint_layer_order_parent_unique_id: Option<u64>,
    pub paint_layer_order: i32,
    pub display_item_client_is_cacheable: bool,
    pub display_item_client_is_just_created: std::cell::Cell<bool>,
    pub selection_client_id: u64,
    pub selection_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub selection_client_is_cacheable: bool,
    pub selection_client_is_just_created: std::cell::Cell<bool>,
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
    pub logical_parent_node_id: Option<u64>,
    pub logical_parent_fragment_instance_id: Option<u64>,
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
    /// element fact distinct from `svg_view_box`, which is later reused for
    /// the nearest resolved viewport dimensions of every SVG child.
    pub svg_view_box_is_empty: bool,
    pub painted_atomically: bool,
    pub has_collapsed_borders: bool,
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
    /// Focused HTMLAreaElement shape/style facts associated by DOM with this
    /// LayoutImage. ImagePainter supplies container geometry and clipping.
    pub focused_image_map_area: Option<FocusedImageMapArea>,
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
    /// colors in Paint.
    pub selection_style: PaintStyle,
    /// LayoutReplaced::GetSelectionState and LocalSelectionVisualRect.  The
    /// latter may aggregate multiple inline fragment items, so Paint must not
    /// reconstruct it from the physical box fragment.
    /// cpp: core/layout/layout_replaced.cc:599-617
    /// LayoutObject::IsSelected(), including LayoutTextFragment virtual query.
    pub is_selected: bool,
    pub replaced_selection_state: SelectionState,
    pub replaced_local_selection_visual_rect: Option<FragmentPaintRect>,
    /// HighlightStyleUtils::SelectionTextDecoration result. Paint compares its
    /// line set with each originating decoration before applying the selection
    /// phase color; it must not reconstruct highlight inheritance.
    pub selection_text_decoration: Option<TextDecorationPaint>,
    /// TextPainter::SvgTextPaintState::TextDecorationObject() style.  SVG text
    /// decorations use the fill/stroke of the first ancestor that declares the
    /// decoration, which can differ from the leaf text's own style.
    pub svg_text_decoration_style: Option<PaintStyle>,
    /// True for an element whose CSS background is painted by the root canvas
    /// painter. Borders, shadows and outlines remain ordinary box decoration.
    pub background_transfers_to_view: bool,
    /// LayoutBox::BackgroundIsKnownToBeObscured(). Layout computes this after
    /// foreground opacity is known; Paint uses the retained decision instead
    /// of re-running replaced-content coverage analysis.
    pub background_is_known_to_be_obscured: bool,
    /// LayoutView::StyleRef() projected onto each page border box. Chromium
    /// paints the document background there instead of the anonymous page
    /// fragment's own style. None means that no laid-out document element was
    /// available, in which case the page border box paints no background.
    pub page_document_background_style: Option<PaintStyle>,
    pub page_document_background_geometry: Option<PageBackgroundPaintData>,
    pub page_container_background_geometry: Option<PageContainerBackgroundPaintData>,
    /// Present only on the root LayoutView fragment.
    pub canvas_background: Option<CanvasBackgroundPaintData>,
    /// Root-element geometry/style retained even when the canvas background is
    /// transparent, for LayoutView background invalidation.
    pub view_background_invalidation: Option<CanvasBackgroundPaintData>,
    pub background_paint_location: BackgroundPaintLocation,
    /// Number of source FillLayers, including image:none slots omitted from
    /// the drawing projection. BoxPaintInvalidator treats multiple layers as
    /// size-dependent before checking whether the first image can render.
    pub background_fill_layer_count: usize,
    pub has_background: bool,
    /// LayoutBox::BackgroundShouldAlwaysBeClipped(), exported as the virtual
    /// Layout decision rather than reconstructed from style in Paint.
    pub background_should_always_be_clipped: bool,
    pub background_any_local_attachment_image: bool,
    pub background_any_default_attachment_image: bool,
    pub background_any_layer_uses_content_box: bool,
    /// LayoutBox::ScrollableOverflowRect(), retained so PrePaint can compare
    /// the current positioning area with the previous committed fragment.
    pub scrollable_overflow: Option<FragmentPaintRect>,
    /// LayoutView::BackgroundRect(), the positioning area for fixed canvas
    /// backgrounds. Present only on the LayoutView.
    pub view_background_rect: Option<FragmentPaintRect>,
    pub previous_view_background_rect: Option<FragmentPaintRect>,
    /// LayoutObject::CanCompositeBackgroundAttachmentFixed is the canonical
    /// eligibility decision. It includes the one-layer, clip, shadow,
    /// appearance and border-radius restrictions required by BackgroundClip.
    pub can_composite_background_attachment_fixed: bool,
    pub margin: Edges,
    pub border: Edges,
    pub border_sides: FragmentBoxSides,
    pub border_styles: [BorderLineStyle; 4],
    pub padding: Edges,
    pub stitched_decoration: Option<StitchedDecorationData>,
    pub box_decoration_break: BoxDecorationBreak,
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
    /// CSS 2.1 `clip: rect(...)` in the box's local stitched coordinate space.
    /// This is distinct from overflow clipping: it clips the box's own paint
    /// as well as descendants and is ordered between Effect and Filter.
    pub css_clip: Option<FragmentPaintRect>,
    pub overflow_clip_margin_outsets: Option<Edges>,
    pub scroll_offset: Offset,
    /// PhysicalBoxFragment::OverflowClipRect with overlay scrollbar space
    /// excluded for hit testing. None means this fragment does not clip
    /// overflow along either axis.
    pub hit_test_overflow_clip: Option<FragmentPaintRect>,
    /// PhysicalBoxFragment::PixelSnappedScrolledContentOffset(). Paint and
    /// Interaction consume the source snap instead of re-rounding scroll_offset.
    pub pixel_snapped_scrolled_content_offset: Offset,
    pub scroll_size: Size,
    /// Exact LayoutBox::ScrollsOverflow() input used to choose whether gap
    /// decorations paint in border or scrolling-contents space. This is not
    /// equivalent to CSS overflow or merely having a ScrollTranslation node.
    pub scrolls_overflow: bool,
    /// Chromium CSSOM client dimensions, excluding classic scrollbar gutters.
    pub client_size: Option<Size>,
    // Chromium scrollable area, including viewport-propagated overflow. CSS
    // input overflow alone does not identify the LayoutView's scrolling owner.
    pub scroll_container: Option<ScrollContainerPaintData>,
    pub writing_mode: WritingMode,
    pub direction: TextDirection,
    pub first_baseline: Option<f64>,
    pub text_line_top_offset: Option<f64>,
    /// `TextFragmentPainter::ComputePhysicalBoxGeometry` line-top adjustment,
    /// captured from the source fragment's `paint_offset + parent_offset`.
    /// Flattened PaintTreeNode offsets cannot reconstruct that pair after a
    /// paint offset translation without changing the rounding phase.
    pub text_line_snap_delta: Option<f64>,
    pub text_line_height: Option<f64>,
    /// FragmentItem::IsEllipsis(). TextDecorationPainter must not infer this
    /// from generated-text kind because other generated runs can be decorated.
    pub is_ellipsis: bool,
    /// FragmentItem::GetTextFitScale(). SVG text keeps 1 because its separate
    /// SVG transform/scaled-font path owns that scaling.
    pub text_fit_scale: f64,
    pub text_combine: Option<TextCombinePaintData>,
    /// LayoutTextCombine::AdjustTextLeftForPaint, quantized as LayoutUnit.
    pub text_combine_inline_offset: f64,
    pub replaced_content: Option<ReplacedContentPaintData>,
    pub resources: Option<Arc<PaintResources>>,
    pub glyph_runs: Arc<Vec<PaintGlyphRun>>,
    pub text_control_host: Option<u64>,
    pub list_marker_symbol: Option<ListStyleType>,
    pub list_marker_inside: bool,
    pub mathml: Option<Arc<MathMLPaintData>>,
    pub frame_set: Option<Arc<FrameSetPaintData>>,
    pub fieldset: Option<Arc<FieldsetPaintData>>,
    pub column_rules: Option<Arc<ColumnRulePaintData>>,
    /// Final physical gap-rule segments in Chromium's rule-overlap order.
    pub gap_rules: Option<Arc<GapRulePaintData>>,
    pub collapsed_table: Option<Arc<CollapsedTablePaintData>>,
    pub table: Option<Arc<TablePaintData>>,
    pub scrollbars: Option<Arc<ScrollbarPaintData>>,
    pub form_control: Option<Arc<FormControlPaintData>>,
    pub svg_shape: Option<Arc<SvgShapeData>>,
    pub svg_view_box: Option<SvgViewBoxData>,
    pub svg_local_transform: Option<TransformMatrix>,
    pub svg_viewport: Option<SvgViewportGeometry>,
    /// LayoutSVGContainer::VisualRectInLocalSVGCoordinates(), aggregated from
    /// the final child layout in SVG user coordinates.  This is distinct from
    /// the container's zero-sized anonymous NG box overflow.
    pub svg_visual_rect: Option<FragmentPaintRect>,
    pub svg_text: Option<Arc<SvgTextPaintData>>,
    pub table_cell_column: Option<usize>,
}

impl Default for PaintProperties {
    fn default() -> Self {
        static EMPTY_DEBUG_NAME: OnceLock<Arc<str>> = OnceLock::new();
        static EMPTY_GLYPH_RUNS: OnceLock<Arc<Vec<PaintGlyphRun>>> = OnceLock::new();
        Self {
            source_object_address: 0,
            pre_paint_invalidation: PrePaintInvalidation::default(),
            property_fragment_unique_id: 0,
            is_first_for_node: true,
            can_paint_multiple_fragments: true,
            is_layout_view: false,
            is_inclusive_descendant_of_unbounded_element: false,
            is_printing: false,
            should_print_backgrounds: false,
            canvas_draw_element_enabled: false,
            is_in_canvas_subtree: false,
            is_overscroll_area_parent: false,
            is_overscroll_container: false,
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
            should_apply_paint_containment: false,
            should_apply_style_containment: false,
            should_apply_layout_containment: false,
            should_clip_overflow_along_either_axis: false,
            should_clip_overflow_along_both_axes: false,
            is_layout_block: false,
            is_layout_block_flow: false,
            is_inline_formatting_context: false,
            in_forced_colors_mode: false,
            is_background_attachment_fixed_object: false,
            requires_main_thread_scrolling_for_background_attachment_fixed: None,
            display_item_client_invalidation_reason: None,
            display_item_client_debug_name: Arc::clone(
                EMPTY_DEBUG_NAME.get_or_init(|| Arc::from("")),
            ),
            paint_layer_client_invalidation_reason: None,
            self_ink_overflow: None,
            contents_ink_overflow: None,
            ink_overflow: None,
            filtered_ink_overflow: None,
            css_mask_bounding_box: None,
            first_line_decoration_rect: None,
            text_glyph_ink: None,
            text_hit_test_bounds: None,
            box_self_visual_overflow: None,
            self_outline: None,
            previous_box_geometry: None,
            box_fragment_is_inline_box: false,
            display_item_raster_effect_outset: RasterEffectOutset::kNone,
            display_item_client_id: 0,
            display_item_client_owner_node_id: 0,
            paint_layer_state_is_authoritative: false,
            paint_layer_client_id: 0,
            paint_layer_client_is_cacheable: false,
            paint_layer_client_is_just_created: std::cell::Cell::new(false),
            paint_layer_is_self_painting: false,
            paint_layer_invisible_for_position_visibility: false,
            paint_layer_has_ancestor_invisible_for_position_visibility: false,
            paint_layer_has_visible_content: false,
            paint_layer_has_self_painting_layer_descendant: false,
            paint_layer_has_non_isolated_descendant_with_blend_mode: false,
            paint_layer_is_under_svg_hidden_container: false,
            paint_layer_supports_subsequence_caching: false,
            paint_layer_self_or_descendant_needs_repaint: true,
            paint_layer_is_fragmented: false,
            paint_layer_known_to_clip_subtree_to_padding_box: false,
            paint_layer_first_fragment_visual_rect: None,
            paint_layer_contents_visual_rect: None,
            paint_layer_filter_reference_box: None,
            paint_layer_filter_viewport: None,
            child_paint_blocked_by_display_lock: false,
            paint_layer_is_stacking_context: false,
            paint_layer_is_stacked: false,
            paint_layer_stacking_level: 0,
            paint_layer_is_z_order_list_visible: false,
            paint_layer_is_in_top_or_view_transition_layer: false,
            paint_layer_is_direct_layout_view_child: false,
            paint_layer_can_contain_absolute_position_objects: false,
            paint_layer_can_contain_fixed_position_objects: false,
            paint_layer_has_overlay_overflow_controls: false,
            paint_layer_order_parent_unique_id: None,
            paint_layer_order: 0,
            display_item_client_is_cacheable: false,
            display_item_client_is_just_created: std::cell::Cell::new(false),
            selection_client_id: 0,
            selection_client_invalidation_reason: None,
            selection_client_is_cacheable: false,
            selection_client_is_just_created: std::cell::Cell::new(false),
            display_item_fragment: 0,
            has_source: false,
            visible_to_hit_testing: true,
            effective_allowed_touch_action: foundation::TouchAction::kAuto,
            inside_blocking_wheel_event_handler: false,
            region_capture_crop_id: None,
            tracked_element_sub_rects: Vec::new(),
            is_svg_child: false,
            logical_parent_node_id: None,
            logical_parent_fragment_instance_id: None,
            logical_tree_order: 0,
            establishes_paint_state: false,
            hidden: false,
            is_line_break: false,
            svg_hidden_container: false,
            svg_descendant_may_have_transform_related_operations: false,
            svg_view_box_is_empty: false,
            painted_atomically: false,
            has_collapsed_borders: false,
            source_kind: NodeKind::kBox,
            replaced_is_image: false,
            video_frame_resource_id: None,
            image_decoding_mode: Default::default(),
            image_last_painted_id: std::cell::Cell::new(image_resource::PaintImageId::INVALID),
            focused_image_map_area: None,
            image_last_paint_rect: std::cell::Cell::new(None),
            effective_zoom: 1.0,
            style: PaintStyle::default(),
            selection_style: PaintStyle::default(),
            is_selected: false,
            replaced_selection_state: SelectionState::kNone,
            replaced_local_selection_visual_rect: None,
            selection_text_decoration: None,
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
            background_should_always_be_clipped: false,
            background_any_local_attachment_image: false,
            background_any_default_attachment_image: false,
            background_any_layer_uses_content_box: false,
            scrollable_overflow: None,
            view_background_rect: None,
            previous_view_background_rect: None,
            can_composite_background_attachment_fixed: false,
            margin: Edges::default(),
            border: Edges::default(),
            border_sides: FragmentBoxSides::default(),
            border_styles: [BorderLineStyle::kNone; 4],
            padding: Edges::default(),
            stitched_decoration: None,
            box_decoration_break: BoxDecorationBreak::kSlice,
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
            css_clip: None,
            overflow_clip_margin_outsets: None,
            scroll_offset: Offset::default(),
            hit_test_overflow_clip: None,
            pixel_snapped_scrolled_content_offset: Offset::default(),
            scroll_size: Size::default(),
            scrolls_overflow: false,
            client_size: None,
            scroll_container: None,
            writing_mode: WritingMode::kHorizontalTb,
            direction: TextDirection::kLtr,
            first_baseline: None,
            text_line_top_offset: None,
            text_line_snap_delta: None,
            text_line_height: None,
            is_ellipsis: false,
            text_fit_scale: 1.0,
            text_combine: None,
            text_combine_inline_offset: 0.0,
            replaced_content: None,
            resources: None,
            glyph_runs: Arc::clone(EMPTY_GLYPH_RUNS.get_or_init(|| Arc::new(Vec::new()))),
            text_control_host: None,
            list_marker_symbol: None,
            list_marker_inside: false,
            mathml: None,
            frame_set: None,
            fieldset: None,
            column_rules: None,
            gap_rules: None,
            collapsed_table: None,
            table: None,
            scrollbars: None,
            form_control: None,
            svg_shape: None,
            svg_view_box: None,
            svg_local_transform: None,
            svg_viewport: None,
            svg_visual_rect: None,
            svg_text: None,
            table_cell_column: None,
        }
    }
}

#[allow(non_snake_case)]
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
    pub(crate) fn SourceObjectPointer(&self) -> *mut LayoutObject {
        let object = self.source_object_address as *mut LayoutObject;
        if object.is_null() || !foundation::IsManagedLayoutAddress(object) {
            std::ptr::null_mut()
        } else {
            object
        }
    }

    fn SourceObject(&self) -> Option<&LayoutObject> {
        unsafe { self.SourceObjectPointer().as_ref() }
    }

    fn SourcePaintLayerPointer(&self) -> *mut PaintLayer {
        let object = self.SourceObjectPointer();
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

    pub fn HasSource(&self) -> bool {
        self.SourceObject().is_some() || self.has_source
    }
    pub fn UpdateInsideBlockingWheelEventHandler(&mut self, inside: bool) -> bool {
        let changed = self.inside_blocking_wheel_event_handler != inside;
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
        let layer = self
            .SourceObject()
            .map_or(std::ptr::null_mut(), LayoutObject::PaintingLayerDefault);
        if let Some(layer) = unsafe { layer.as_mut() } {
            layer.SetNeedsRepaint();
        }
    }

    pub fn IsLayoutView(&self) -> bool {
        self.SourceObject()
            .map_or(self.is_layout_view, LayoutObject::IsLayoutView)
    }

    pub fn IsGlobalRootScroller(&self) -> bool {
        self.SourceObject()
            .map_or(self.is_layout_view, LayoutObject::IsGlobalRootScroller)
    }

    pub fn NeedsLayout(&self) -> bool {
        self.SourceObject()
            .map_or(self.needs_layout, LayoutObject::NeedsLayout)
    }

    pub fn IsInclusiveDescendantOfUnboundedElement(&self) -> bool {
        self.SourceObject().map_or(
            self.is_inclusive_descendant_of_unbounded_element,
            LayoutObject::IsInclusiveDescendantOfUnboundedElement,
        )
    }

    pub fn IsPrinting(&self) -> bool {
        self.SourceObject()
            .map_or(self.is_printing, LayoutObject::IsPrintingForLayout)
    }

    pub fn ShouldPrintBackgrounds(&self) -> bool {
        self.SourceObject().map_or(
            self.should_print_backgrounds,
            LayoutObject::ShouldPrintBackgroundsForLayout,
        )
    }

    pub fn CanvasDrawElementEnabled(&self) -> bool {
        self.SourceObject()
            .map_or(self.canvas_draw_element_enabled, |object| {
                object.CanvasDrawElementEnabledForLayout()
            })
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
        let element = foundation::DynamicTo::<layoutng::internal::layout_node_metadata::Element>(
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
            .map_or(self.is_in_canvas_subtree, LayoutObject::IsInCanvasSubtree)
    }

    pub fn IsOverscrollAreaParent(&self) -> bool {
        self.SourceObject().map_or(
            self.is_overscroll_area_parent,
            LayoutObject::IsOverscrollAreaParent,
        )
    }

    pub fn IsOverscrollContainer(&self) -> bool {
        self.SourceObject().map_or(
            self.is_overscroll_container,
            LayoutObject::IsOverscrollContainer,
        )
    }

    pub fn ChildLayoutBlockedByDisplayLock(&self) -> bool {
        self.SourceObject().map_or(
            self.child_layout_blocked_by_display_lock,
            LayoutObject::ChildLayoutBlockedByDisplayLock,
        )
    }

    pub fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        self.SourceObject().map_or(
            self.child_paint_blocked_by_display_lock,
            LayoutObject::ChildPaintBlockedByDisplayLock,
        )
    }

    pub fn IsFragmentLessBox(&self) -> bool {
        self.SourceObject()
            .map_or(self.is_fragment_less_box, LayoutObject::IsFragmentLessBox)
    }

    pub fn AdditionalCompositingReasons(&self) -> foundation::CompositingReasons {
        self.SourceObject()
            .map_or(self.additional_compositing_reasons, |object| {
                if object.CanHaveAdditionalCompositingReasons() {
                    object.AdditionalCompositingReasons()
                } else {
                    foundation::CompositingReasons::new()
                }
            })
    }

    pub fn InForcedColorsMode(&self) -> bool {
        self.SourceObject()
            .map_or(self.in_forced_colors_mode, |object| {
                object.StyleRef().InForcedColorsMode()
            })
    }

    pub fn IsBackgroundAttachmentFixedObject(&self) -> bool {
        self.SourceObject().map_or(
            self.is_background_attachment_fixed_object,
            LayoutObject::IsBackgroundAttachmentFixedObject,
        )
    }

    pub fn IsBoxModelObject(&self) -> bool {
        self.SourceObject()
            .map_or(self.is_box_model_object, LayoutObject::IsBoxModelObject)
    }

    pub fn IsLayoutBox(&self) -> bool {
        self.SourceObject().map_or(self.is_layout_box, |object| {
            !foundation::DynamicTo::<layoutng::internal::layout_box::LayoutBox>(object).is_null()
        })
    }

    pub fn IsLayoutInline(&self) -> bool {
        self.SourceObject()
            .map_or(self.is_layout_inline, LayoutObject::IsLayoutInline)
    }

    pub fn ShouldApplyPaintContainment(&self) -> bool {
        self.SourceObject()
            .map_or(self.should_apply_paint_containment, |object| {
                object.ShouldApplyPaintContainment()
            })
    }

    pub fn ShouldApplyStyleContainment(&self) -> bool {
        self.SourceObject()
            .map_or(self.should_apply_style_containment, |object| {
                object.ShouldApplyStyleContainment()
            })
    }

    pub fn ShouldApplyLayoutContainment(&self) -> bool {
        self.SourceObject()
            .map_or(self.should_apply_layout_containment, |object| {
                object.ShouldApplyLayoutContainment()
            })
    }

    pub fn ShouldClipOverflowAlongEitherAxis(&self) -> bool {
        self.SourceObject().map_or(
            self.should_clip_overflow_along_either_axis,
            LayoutObject::ShouldClipOverflowAlongEitherAxis,
        )
    }

    pub fn ShouldClipOverflowAlongBothAxes(&self) -> bool {
        self.SourceObject().map_or(
            self.should_clip_overflow_along_both_axes,
            LayoutObject::ShouldClipOverflowAlongBothAxis,
        )
    }

    pub fn IsLayoutBlock(&self) -> bool {
        self.SourceObject()
            .map_or(self.is_layout_block, LayoutObject::IsLayoutBlock)
    }

    pub fn IsLayoutBlockFlow(&self) -> bool {
        self.SourceObject()
            .map_or(self.is_layout_block_flow, LayoutObject::IsLayoutBlockFlow)
    }

    pub fn PaintLayerStateIsAuthoritative(&self) -> bool {
        self.SourceObject().is_some() || self.paint_layer_state_is_authoritative
    }

    pub fn PaintLayerIsSelfPainting(&self) -> bool {
        self.SourcePaintLayer()
            .map_or(self.paint_layer_is_self_painting, |layer| {
                layer.IsSelfPaintingLayer()
            })
    }

    pub fn PaintLayerInvisibleForPositionVisibility(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_invisible_for_position_visibility,
            PaintLayer::InvisibleForPositionVisibility,
        )
    }

    pub fn PaintLayerHasAncestorInvisibleForPositionVisibility(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_has_ancestor_invisible_for_position_visibility,
            PaintLayer::HasAncestorInvisibleForPositionVisibility,
        )
    }

    pub fn PaintLayerHasVisibleContent(&self) -> bool {
        self.SourcePaintLayer()
            .map_or(self.paint_layer_has_visible_content, |layer| {
                layer.HasVisibleContent()
            })
    }

    pub fn PaintLayerHasSelfPaintingLayerDescendant(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_has_self_painting_layer_descendant,
            PaintLayer::HasSelfPaintingLayerDescendant,
        )
    }

    pub fn PaintLayerHasNonIsolatedDescendantWithBlendMode(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_has_non_isolated_descendant_with_blend_mode,
            PaintLayer::HasNonIsolatedDescendantWithBlendMode,
        )
    }

    pub fn PaintLayerIsUnderSvgHiddenContainer(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_is_under_svg_hidden_container,
            PaintLayer::IsUnderSVGHiddenContainer,
        )
    }

    pub fn PaintLayerSupportsSubsequenceCaching(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_supports_subsequence_caching,
            PaintLayer::SupportsSubsequenceCaching,
        )
    }

    pub fn PaintLayerSelfOrDescendantNeedsRepaint(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_self_or_descendant_needs_repaint,
            PaintLayer::SelfOrDescendantNeedsRepaint,
        )
    }

    pub fn PaintLayerIsFragmented(&self) -> bool {
        self.SourcePaintLayer()
            .map_or(self.paint_layer_is_fragmented, |layer| {
                layer.GetLayoutObject().IsFragmented()
            })
    }

    pub fn PaintLayerKnownToClipSubtreeToPaddingBox(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_known_to_clip_subtree_to_padding_box,
            PaintLayer::KnownToClipSubtreeToPaddingBox,
        )
    }

    pub fn PaintLayerIsStackingContext(&self) -> bool {
        self.SourceObject()
            .map_or(self.paint_layer_is_stacking_context, |object| {
                object.IsStackingContext()
            })
    }

    pub fn PaintLayerIsStacked(&self) -> bool {
        self.SourceObject()
            .map_or(self.paint_layer_is_stacked, LayoutObject::IsStacked)
    }

    pub fn PaintLayerStackingLevel(&self) -> i32 {
        self.SourceObject()
            .map_or(self.paint_layer_stacking_level, |object| {
                object.StyleRef().EffectiveZIndex()
            })
    }

    pub fn PaintLayerIsZOrderListVisible(&self) -> bool {
        self.SourcePaintLayer().map_or(
            self.paint_layer_is_z_order_list_visible,
            PaintLayer::IsZOrderListVisible,
        )
    }
    pub fn CanPaintFragmentOccurrence(&self) -> bool {
        self.is_first_for_node || self.can_paint_multiple_fragments
    }

    // C++ chained assignment evaluates the right-hand assignment first.
    // cpp: layoutng_fragment_tree/fragment_tree.h:96
    pub fn SetOverflow(&mut self, value: Overflow) {
        self.overflow_y = value;
        self.overflow_x = value;
    }
}
