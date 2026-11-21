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
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::PaintInvalidationReason;
use layoutng::internal::layout_text::LayoutText;

/// A DisplayItemClient's PaintController-visible state, read from the live
/// source client (or from a detached payload's stored state).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintClientState {
    pub invalidation_reason: Option<PaintInvalidationReason>,
    pub is_cacheable: bool,
    pub is_just_created: bool,
}
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
/// The same selection read live from the resident root/body fragments
/// (PaintFragmentCursor::ViewBackground / CanvasBackground); the style is the
/// selected fragment's own PaintStyle projection, borrowed, not copied.
pub type CanvasBackgroundPaintView<'a> =
    layout_fragment_boundary::CanvasBackgroundPaintData<&'a PaintStyle>;

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
    /// Fixture-only DebugName for payloads without a bound source object;
    /// read through DisplayItemClientDebugName, never written by population.
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
    /// colors in Paint. Read through SelectionPaintStyle(), which resolves it
    /// from the live source; stored only for detached fixtures.
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
    /// phase color; it must not reconstruct highlight inheritance. Read
    /// through SelectionTextDecoration(); stored only for detached fixtures.
    pub selection_text_decoration: Option<TextDecorationPaint>,
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
    /// LayoutView::StyleRef() projected onto each page border box. Chromium
    /// paints the document background there instead of the anonymous page
    /// fragment's own style. None means that no laid-out document element was
    /// available, in which case the page border box paints no background.
    pub page_document_background_style: Option<PaintStyle>,
    /// Page geometry: the global paint pass stores only the LayoutView-global
    /// facts; ResidentBoxGeometry::Page*BackgroundGeometry() completes them
    /// from the resident page fragment (fixtures keep the whole stored value).
    pub page_document_background_geometry: Option<PageBackgroundPaintData>,
    pub page_container_background_geometry: Option<PageContainerBackgroundPaintData>,
    /// Detached-fixture canvas background. Production Paint reads the live
    /// selection through PaintFragmentCursor::CanvasBackground().
    pub canvas_background: Option<CanvasBackgroundPaintData>,
    /// Root-element geometry/style retained even when the canvas background is
    /// transparent, for LayoutView background invalidation. Captured at
    /// publication because BoxPaintInvalidator compares it with the previous
    /// slot's value.
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
    /// A sliced inline box's strip, written by PrePaint
    /// (PopulateInlineStitchedDecorations), and the fixture fallback. Other
    /// resident boxes read it live (ViewStitchedDecoration).
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
            live_style_facts: false,
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
            is_ellipsis: false,
            text_fit_scale: 1.0,
            text_combine: None,
            text_combine_inline_offset: 0.0,
            replaced_content: None,
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
    /// from the resident fragment or item.
    fn ViewInkOverflows(&self) -> [Option<FragmentPaintRect>; 4];
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
    /// Scroll size, CSSOM client size and ScrollsOverflow of the resident box.
    fn ViewScrollGeometry(&self) -> (Size, Option<Size>, bool);
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
    /// across its LayoutBox fragments; inline boxes keep PrePaint's stitched
    /// inline geometry (InlineBoxFragmentPainter).
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
        self.ViewInkOverflows()[0]
    }
    fn ViewContentsInkOverflow(&self) -> Option<FragmentPaintRect> {
        self.ViewInkOverflows()[1]
    }
    fn ViewInkOverflow(&self) -> Option<FragmentPaintRect> {
        self.ViewInkOverflows()[2]
    }
    fn ViewFilteredInkOverflow(&self) -> Option<FragmentPaintRect> {
        self.ViewInkOverflows()[3]
    }
    fn Borders(&self) -> Edges;
    fn Padding(&self) -> Edges;
    fn BorderSides(&self) -> layout_fragment_boundary::FragmentBoxSides;
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
    /// attachment fact stay published by the global paint pass.
    fn PageContainerBackgroundGeometry(&self) -> Option<PageContainerBackgroundPaintData>;
    /// BoxBackgroundPaintContext geometry of a page border box. The page
    /// area's OffsetFromRootFragmentationContext() is read from the resident
    /// fragment; the LayoutView-global facts stay published (see above).
    fn PageDocumentBackgroundGeometry(&self) -> Option<PageBackgroundPaintData>;
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
    image: impl FnOnce(&layoutng_style::style::computed_style::ComputedStyle)
        -> &layoutng_style::style::nine_piece_image::NinePieceImage,
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
const DERIVED_INSTANCE_ID_BIT: u64 = 1 << 63;
const FRAGMENTAINER_INSTANCE_ID_BIT: u64 = 1 << 62;
const HASHED_INSTANCE_ID_BIT: u64 = 1 << 61;
const PACKED_CLIENT_LIMIT: u64 = 1 << 29;

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
    // splitmix64 finalizer over the pair.
    let mut z = client.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ u64::from(fragment);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    DERIVED_INSTANCE_ID_BIT | kind | HASHED_INSTANCE_ID_BIT | (z & (HASHED_INSTANCE_ID_BIT - 1))
}

fn ClientInstanceId(object: *const LayoutObject) -> Option<u64> {
    (!object.is_null()).then(|| unsafe { &*(object as *const DisplayItemClient) }.Id() as u64)
}

/// Identity of a fragmentainer box (column or page area): its owner's
/// DisplayItemClient and the display-item fragment BoxFragmentPainter
/// scopes it with (break token SequenceNumber + 1, 0 for the last one).
/// Descendants are inside this fragmentainer only through the tree walk
/// (Chromium's PrePaintTreeWalk `fragmentainer_idx`); a walk propagates it.
pub fn FragmentainerInstanceId(
    fragment: &crate::physical_box_fragment::PhysicalBoxFragment,
) -> u64 {
    debug_assert!(fragment.IsFragmentainerBox());
    // A fragmentainer is not a CSS box, so GetLayoutObject() hides the
    // fragmentation-context owner it was laid out for.
    let client = ClientInstanceId(fragment.layout_object_ptr()).unwrap_or(0);
    let token = fragment.GetBreakToken();
    let sequence = if token.is_null() {
        0
    } else {
        unsafe { &*token }.SequenceNumber() + 1
    };
    DerivedFragmentInstanceId(client, true, sequence)
}

/// Identity of a resident physical fragment: FragmentData::UniqueId of a CSS
/// box (its own FragmentData, else its object's first fragment), the
/// fragmentainer identity of a column/page area, and 0 for a line box
/// fragment, which owns no Chromium paint identity.
pub fn PhysicalFragmentInstanceId(fragment: &crate::physical_fragment::PhysicalFragment) -> u64 {
    if fragment.IsFragmentainerBox() {
        return FragmentainerInstanceId(unsafe {
            &*(fragment as *const crate::physical_fragment::PhysicalFragment
                as *const crate::physical_box_fragment::PhysicalBoxFragment)
        });
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() {
        return 0;
    }
    let fragment_data = if fragment.IsBox() {
        fragment.GetFragmentData().cast_mut()
    } else {
        std::ptr::null_mut()
    };
    crate::internal::boundary::layout_boundary::PropertyFragmentUniqueId(object, fragment_data)
}

/// Identity of a resident inline item: (DisplayItemClient id,
/// FragmentItem::FragmentId()). Stable across relayout while the object
/// keeps the same fragment sequence number, independent of the item's index,
/// text offsets or container FragmentData.
pub fn InlineItemInstanceId(item: &crate::fragment_item::FragmentItem) -> u64 {
    ClientInstanceId(item.GetLayoutObject())
        .map_or(0, |client| DerivedFragmentInstanceId(client, false, item.FragmentId()))
}

impl ResidentBoxGeometry for FragmentPaintData {
    fn ViewEstablishesPaintState(&self) -> bool {
        use layout_fragment_boundary::ResidentFragmentOwner;
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
        let layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } =
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
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { .. }
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
        match self.ResidentBoxFragment() {
            Some(box_fragment) if !box_fragment.IsInlineBox() => {
                crate::internal::boundary::layout_boundary::ExportStitchedDecorationData(
                    box_fragment,
                )
            }
            _ => self.paint.stitched_decoration.clone(),
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
        use layout_fragment_boundary::ResidentFragmentOwner;
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
        use layout_fragment_boundary::ResidentFragmentOwner;
        match self.resident_owner {
            ResidentFragmentOwner::None => self.fragment_instance_id,
            ResidentFragmentOwner::Physical(address) => PhysicalFragmentInstanceId(unsafe {
                &*(address as *const crate::physical_fragment::PhysicalFragment)
            }),
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items.Items().get(index).map_or(0, InlineItemInstanceId)
            }
        }
    }

    fn ViewTextHitTestBounds(&self) -> Option<FragmentPaintRect> {
        match self.resident_owner {
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items
                    .Items()
                    .get(index)
                    .and_then(crate::internal::boundary::layout_boundary::LiveTextHitTestBounds)
            }
            _ => self.paint.text_hit_test_bounds,
        }
    }

    fn ViewCollapsedTable(&self) -> Option<Arc<CollapsedTablePaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::ExportCollapsedTablePaintData(
                    box_fragment,
                )
            }
            None => self.paint.collapsed_table.clone(),
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
            .map_or(self.paint.has_collapsed_borders, |box_fragment| {
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
            .map_or(self.paint.can_paint_multiple_fragments, |box_fragment| {
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
            None => self.paint.table_cell_column,
        }
    }

    fn ViewBoxDecorationBreak(&self) -> BoxDecorationBreak {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveBoxDecorationBreak(box_fragment)
            }
            None => self.paint.box_decoration_break,
        }
    }

    fn ViewTable(&self) -> Option<Arc<TablePaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::ExportTablePaintData(box_fragment)
            }
            None => self.paint.table.clone(),
        }
    }

    fn ViewOverflowClipMarginOutsets(&self) -> Option<Edges> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::ExportOverflowClipMargin(box_fragment)
            }
            None => self.paint.overflow_clip_margin_outsets,
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
            foundation::DynamicTo::<layoutng::internal::layout_box::LayoutBox>(
                box_fragment.GetLayoutObject(),
            )
            .as_ref()
        });
        match layout_box {
            Some(layout_box) => {
                crate::internal::boundary::layout_boundary::LiveScrollGeometry(layout_box)
            }
            None => (
                self.paint.scroll_size,
                self.paint.client_size,
                self.paint.scrolls_overflow,
            ),
        }
    }

    fn ViewScrollContainer(&self) -> Option<ScrollContainerPaintData> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                crate::internal::boundary::layout_boundary::LiveScrollContainer(box_fragment)
            }
            None => self.paint.scroll_container.clone(),
        }
    }

    fn ViewReplacedContent(&self) -> Option<ReplacedContentPaintData> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => {
                let replaced = foundation::DynamicTo::<layoutng::internal::layout_replaced::LayoutReplaced>(
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
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                match items.Items().get(index) {
                    Some(item) => crate::internal::boundary::layout_boundary::ExportSvgTextPaintData(
                        item,
                        *item.RectInContainerFragment(),
                    ),
                    None => self.paint.svg_text.clone(),
                }
            }
            _ => self.paint.svg_text.clone(),
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
                    .or(self.paint.first_baseline)
            }
            None => self.paint.first_baseline,
        }
    }

    fn ViewColumnRules(&self) -> Option<Arc<ColumnRulePaintData>> {
        match self.ResidentBoxFragment() {
            Some(box_fragment) => crate::internal::boundary::layout_boundary::ExportColumnRulePaintData(
                box_fragment,
                &self.paint,
            ),
            None => self.paint.column_rules.clone(),
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
            None => self.paint.frame_set.clone(),
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
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                crate::internal::boundary::layout_boundary::LiveTextLineGeometry(items, index)
            }
            _ => None,
        }
    }

    fn ViewFirstLineDecorationRect(&self) -> Option<FragmentPaintRect> {
        match self.resident_owner {
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } => {
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
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items
                    .Items()
                    .get(index)
                    .map_or(self.paint.display_item_fragment, |item| item.FragmentId())
            }
            _ => self.paint.display_item_fragment,
        }
    }

    fn ViewHidden(&self) -> bool {
        match self.resident_owner {
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items.Items().get(index).map_or(self.paint.hidden, |item| {
                    item.IsHiddenForPaint()
                        || self.paint.SvgViewport().is_some_and(|viewport| viewport.disabled)
                })
            }
            layout_fragment_boundary::ResidentFragmentOwner::Physical(address) => {
                let fragment =
                    unsafe { &*(address as *const crate::physical_fragment::PhysicalFragment) };
                IsHiddenForPaintIncludingInlineItem(fragment)
                    || self.paint.SvgViewport().is_some_and(|viewport| viewport.disabled)
            }
            layout_fragment_boundary::ResidentFragmentOwner::None => self.paint.hidden,
        }
    }

    fn ViewTextItemFacts(&self) -> (bool, f64, Option<ListStyleType>, bool, f64) {
        let stored = (
            self.paint.is_ellipsis,
            self.paint.text_fit_scale,
            self.paint.list_marker_symbol,
            self.paint.list_marker_inside,
            self.paint.text_combine_inline_offset,
        );
        match self.resident_owner {
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } => {
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
            layout_fragment_boundary::ResidentFragmentOwner::InlineItem { items, index } => {
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

    fn ViewInkOverflows(&self) -> [Option<FragmentPaintRect>; 4] {
        use layout_fragment_boundary::ResidentFragmentOwner;
        let stored = [
            self.paint.self_ink_overflow,
            self.paint.contents_ink_overflow,
            self.paint.ink_overflow,
            self.paint.filtered_ink_overflow,
        ];
        match self.resident_owner {
            ResidentFragmentOwner::None => stored,
            ResidentFragmentOwner::Physical(_) => match self.ResidentBoxFragment() {
                Some(box_fragment) => {
                    crate::internal::boundary::layout_boundary::LiveBoxInkOverflow(box_fragment)
                }
                None => stored,
            },
            ResidentFragmentOwner::InlineItem { items, index } => {
                let items = unsafe { &*(items as *const crate::fragment_items::FragmentItems) };
                items
                    .Items()
                    .get(index)
                    .and_then(crate::internal::boundary::layout_boundary::LiveItemInkOverflow)
                    .unwrap_or(stored)
            }
        }
    }

    fn ViewKind(&self) -> FragmentKind {
        use crate::fragment_item::ItemType;
        use layout_fragment_boundary::ResidentFragmentOwner;
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
        use layout_fragment_boundary::ResidentFragmentOwner;
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
        if matches!(self.resident_owner, layout_fragment_boundary::ResidentFragmentOwner::None) {
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
        use layout_fragment_boundary::ResidentFragmentOwner;
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
        use layout_fragment_boundary::ResidentFragmentOwner;
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
        use layout_fragment_boundary::ResidentFragmentOwner;
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
                layoutng::internal::layout_replaced::LayoutReplaced,
            >(box_fragment.GetLayoutObject())
            .is_null(),
            None => self.paint.replaced_content.is_some(),
        }
    }

    fn ResidentBoxFragment(&self) -> Option<&crate::physical_box_fragment::PhysicalBoxFragment> {
        use crate::physical_box_fragment::PhysicalBoxFragment;
        use layout_fragment_boundary::ResidentFragmentOwner;
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

    fn BorderSides(&self) -> layout_fragment_boundary::FragmentBoxSides {
        match self.ResidentBoxFragment() {
            Some(fragment) => {
                let sides = fragment.SidesToInclude();
                layout_fragment_boundary::FragmentBoxSides {
                    top: sides.top,
                    right: sides.right,
                    bottom: sides.bottom,
                    left: sides.left,
                }
            }
            None => self.paint.border_sides,
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
            ..self.paint.page_container_background_geometry.unwrap_or_default()
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
            ..self.paint.page_document_background_geometry.unwrap_or_default()
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
        match self.style.Source() {
            Some(source) => {
                crate::internal::boundary::layout_boundary::ExportSelectionTextDecoration(source)
            }
            None => self.selection_text_decoration.clone(),
        }
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

    /// The bound source LayoutObject's current PrePaint dirty bits, read live
    /// (LayoutObject::NeedsPaintPropertyUpdate, ShouldCheckForPaintInvalidation,
    /// their descendant/subtree variants and the PrePaint subtree-walk
    /// reasons; core/layout/layout_object.h), the same facts PrePaintTreeWalk
    /// admits nodes by (core/paint/pre_paint_tree_walk.cc:418-436). `None`
    /// for hand-built projections without a live source; callers then use
    /// the slot's `pre_paint_invalidation` snapshot.
    ///
    /// Rechrom clears these bits in the paint client commit
    /// (LayoutObject::ClearPaintFlags from layout_commit's client_commit.rs)
    /// rather than inside the PrePaint walk as Chromium does
    /// (pre_paint_tree_walk.cc:107). A clean result therefore means
    /// "unchanged since the last full paint commit", not "since the last
    /// PrePaint walk".
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

    /// DisplayItemClient::DebugName() of the bound source LayoutObject
    /// (platform/graphics/paint/display_item_client.h; LayoutObject::DebugName,
    /// translated in layoutng/internal/layout_object_core_services.cc:489-499),
    /// computed on demand as Chromium does. Paint asks only while recording
    /// paint debug info or tracing. Detached fixtures keep their stored name.
    pub fn DisplayItemClientDebugName(&self) -> Arc<str> {
        match self.SourceObject() {
            Some(object) => Arc::from(object.DebugName().Utf8()),
            None => Arc::clone(&self.display_item_client_debug_name),
        }
    }

    pub fn HasSource(&self) -> bool {
        let live = self.SourceObject().is_some();
        if live && !self.has_source && PaintSourceCheckEnabled() {
            browser_tracing::instant(
                "layout",
                "layout_fragment.PaintSourceMismatch",
                &[("has_source", 1.0)],
            );
        }
        live || self.has_source
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
            .SourceCheckedOr(self.paint_layer_client_id, "paint_layer_client_id")
    }

    /// LayoutObject::InsideBlockingWheelEventHandler(), which PrePaint
    /// updates on the LayoutObject (paint_property_tree_builder.cc).
    pub fn InsideBlockingWheelEventHandler(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::InsideBlockingWheelEventHandler)
            .SourceCheckedOr(
                self.inside_blocking_wheel_event_handler,
                "inside_blocking_wheel_event_handler",
            )
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
            self.selection_client_is_just_created.get(),
        )
    }

    pub fn PaintLayerClientState(&self) -> PaintClientState {
        Self::ClientState(
            self.LivePaintLayerClient(),
            self.paint_layer_client_invalidation_reason,
            self.paint_layer_client_is_cacheable,
            self.paint_layer_client_is_just_created.get(),
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
            .SourceCheckedOr(self.is_layout_view, "is_layout_view")
    }

    pub fn IsGlobalRootScroller(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsGlobalRootScroller)
            .SourceCheckedOr(self.is_layout_view, "is_layout_view")
    }

    pub fn NeedsLayout(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::NeedsLayout)
            .SourceCheckedOr(self.needs_layout, "needs_layout")
    }

    pub fn IsInclusiveDescendantOfUnboundedElement(&self) -> bool {
        self.SourceObject().map(LayoutObject::IsInclusiveDescendantOfUnboundedElement)
            .SourceCheckedOr(self.is_inclusive_descendant_of_unbounded_element, "is_inclusive_descendant_of_unbounded_element")
    }

    pub fn IsPrinting(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsPrintingForLayout)
            .SourceCheckedOr(self.is_printing, "is_printing")
    }

    pub fn ShouldPrintBackgrounds(&self) -> bool {
        self.SourceObject().map(LayoutObject::ShouldPrintBackgroundsForLayout)
            .SourceCheckedOr(self.should_print_backgrounds, "should_print_backgrounds")
    }

    pub fn CanvasDrawElementEnabled(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.CanvasDrawElementEnabledForLayout()
            })
            .SourceCheckedOr(self.canvas_draw_element_enabled, "canvas_draw_element_enabled")
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
            .map(LayoutObject::IsInCanvasSubtree)
            .SourceCheckedOr(self.is_in_canvas_subtree, "is_in_canvas_subtree")
    }

    pub fn IsOverscrollAreaParent(&self) -> bool {
        self.SourceObject().map(LayoutObject::IsOverscrollAreaParent)
            .SourceCheckedOr(self.is_overscroll_area_parent, "is_overscroll_area_parent")
    }

    pub fn IsOverscrollContainer(&self) -> bool {
        self.SourceObject().map(LayoutObject::IsOverscrollContainer)
            .SourceCheckedOr(self.is_overscroll_container, "is_overscroll_container")
    }

    pub fn ChildLayoutBlockedByDisplayLock(&self) -> bool {
        self.SourceObject().map(LayoutObject::ChildLayoutBlockedByDisplayLock)
            .SourceCheckedOr(self.child_layout_blocked_by_display_lock, "child_layout_blocked_by_display_lock")
    }

    pub fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        self.SourceObject().map(LayoutObject::ChildPaintBlockedByDisplayLock)
            .SourceCheckedOr(self.child_paint_blocked_by_display_lock, "child_paint_blocked_by_display_lock")
    }

    pub fn IsFragmentLessBox(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsFragmentLessBox)
            .SourceCheckedOr(self.is_fragment_less_box, "is_fragment_less_box")
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
            .SourceCheckedOr(self.additional_compositing_reasons, "additional_compositing_reasons")
    }

    pub fn InForcedColorsMode(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.StyleRef().InForcedColorsMode()
            })
            .SourceCheckedOr(self.in_forced_colors_mode, "in_forced_colors_mode")
    }

    pub fn IsBackgroundAttachmentFixedObject(&self) -> bool {
        self.SourceObject().map(LayoutObject::IsBackgroundAttachmentFixedObject)
            .SourceCheckedOr(self.is_background_attachment_fixed_object, "is_background_attachment_fixed_object")
    }

    pub fn IsBoxModelObject(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsBoxModelObject)
            .SourceCheckedOr(self.is_box_model_object, "is_box_model_object")
    }

    // Stage 2 of the export removal: LayoutObject facts read live (Chromium
    // PrePaint/Paint read them from the LayoutObject); the stored copy is
    // only the detached-fixture fallback.
    pub fn ReplacedIsImage(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutImage)
            .SourceCheckedOr(self.replaced_is_image, "replaced_is_image")
    }

    pub fn PaintLayerIsInTopOrViewTransitionLayer(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && object.IsInTopOrViewTransitionLayer()
            })
            .SourceCheckedOr(self.paint_layer_is_in_top_or_view_transition_layer, "paint_layer_is_in_top_or_view_transition_layer")
    }

    pub fn PaintLayerIsDirectLayoutViewChild(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && unsafe { object.Parent().as_ref() }.is_some_and(LayoutObject::IsLayoutView)
            })
            .SourceCheckedOr(self.paint_layer_is_direct_layout_view_child, "paint_layer_is_direct_layout_view_child")
    }

    pub fn PaintLayerCanContainAbsolutePositionObjects(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && object.CanContainAbsolutePositionObjects()
            })
            .SourceCheckedOr(self.paint_layer_can_contain_absolute_position_objects, "paint_layer_can_contain_absolute_position_objects")
    }

    pub fn PaintLayerCanContainFixedPositionObjects(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !Self::PaintLayerOfObject(object as *const LayoutObject as *mut LayoutObject)
                    .is_null()
                    && object.CanContainFixedPositionObjects()
            })
            .SourceCheckedOr(self.paint_layer_can_contain_fixed_position_objects, "paint_layer_can_contain_fixed_position_objects")
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
            .SourceCheckedOr(self.paint_layer_has_overlay_overflow_controls, "paint_layer_has_overlay_overflow_controls")
    }

    pub fn BackgroundPaintLocation(&self) -> BackgroundPaintLocation {
        self.SourceObject()
            .and_then(|object| {
                unsafe { foundation::DynamicTo::<layoutng::internal::layout_box::LayoutBox>(object).as_ref() }
                    .map(crate::internal::boundary::layout_boundary::LiveBackgroundPaintLocation)
            })
            .SourceCheckedOr(self.background_paint_location, "background_paint_location")
    }

    pub fn CssClip(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .and_then(|object| {
                unsafe { foundation::DynamicTo::<layoutng::internal::layout_box::LayoutBox>(object).as_ref() }
                    .map(crate::internal::boundary::layout_boundary::LiveCssClip)
            })
            .SourceCheckedOr(self.css_clip, "css_clip")
    }

    pub fn VisibleToHitTesting(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::VisibleToHitTesting)
            .SourceCheckedOr(self.visible_to_hit_testing, "visible_to_hit_testing")
    }

    pub fn IsSelected(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsSelected)
            .SourceCheckedOr(self.is_selected, "is_selected")
    }

    pub fn EffectiveAllowedTouchAction(&self) -> foundation::TouchAction {
        self.SourceObject()
            .map(LayoutObject::EffectiveAllowedTouchAction)
            .SourceCheckedOr(self.effective_allowed_touch_action, "effective_allowed_touch_action")
    }

    pub fn BackgroundIsKnownToBeObscured(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::BackgroundIsKnownToBeObscured)
            .SourceCheckedOr(self.background_is_known_to_be_obscured, "background_is_known_to_be_obscured")
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
            .SourceCheckedOr(self.background_transfers_to_view, "background_transfers_to_view")
    }

    pub fn IsSVGChild(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsSVGChild)
            .SourceCheckedOr(self.is_svg_child, "is_svg_child")
    }

    pub fn IsSVGHiddenContainer(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsSVGHiddenContainer)
            .SourceCheckedOr(self.svg_hidden_container, "svg_hidden_container")
    }

    pub fn SVGDescendantMayHaveTransformRelatedOperations(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::SVGDescendantMayHaveTransformRelatedOperations)
            .SourceCheckedOr(self.svg_descendant_may_have_transform_related_operations, "svg_descendant_may_have_transform_related_operations")
    }

    pub fn BackgroundShouldAlwaysBeClipped(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                let box_ = foundation::DynamicTo::<layoutng::internal::layout_box::LayoutBox>(object);
                !box_.is_null() && unsafe { &*box_ }.BackgroundShouldAlwaysBeClipped()
            })
            .SourceCheckedOr(self.background_should_always_be_clipped, "background_should_always_be_clipped")
    }

    pub fn CanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                !foundation::DynamicTo::<layoutng::internal::layout_box::LayoutBox>(object).is_null()
                    && object.CanCompositeBackgroundAttachmentFixed()
            })
            .SourceCheckedOr(self.can_composite_background_attachment_fixed, "can_composite_background_attachment_fixed")
    }

    pub fn IsLayoutBox(&self) -> bool {
        self.SourceObject().map(|object| {
            !foundation::DynamicTo::<layoutng::internal::layout_box::LayoutBox>(object).is_null()
        })
            .SourceCheckedOr(self.is_layout_box, "is_layout_box")
    }

    pub fn IsLayoutInline(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutInline)
            .SourceCheckedOr(self.is_layout_inline, "is_layout_inline")
    }

    pub fn ShouldApplyPaintContainment(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.ShouldApplyPaintContainment()
            })
            .SourceCheckedOr(self.should_apply_paint_containment, "should_apply_paint_containment")
    }

    pub fn ShouldApplyStyleContainment(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.ShouldApplyStyleContainment()
            })
            .SourceCheckedOr(self.should_apply_style_containment, "should_apply_style_containment")
    }

    pub fn ShouldApplyLayoutContainment(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.ShouldApplyLayoutContainment()
            })
            .SourceCheckedOr(self.should_apply_layout_containment, "should_apply_layout_containment")
    }

    pub fn ShouldClipOverflowAlongEitherAxis(&self) -> bool {
        self.SourceObject().map(LayoutObject::ShouldClipOverflowAlongEitherAxis)
            .SourceCheckedOr(self.should_clip_overflow_along_either_axis, "should_clip_overflow_along_either_axis")
    }

    pub fn ShouldClipOverflowAlongBothAxes(&self) -> bool {
        self.SourceObject().map(LayoutObject::ShouldClipOverflowAlongBothAxis)
            .SourceCheckedOr(self.should_clip_overflow_along_both_axes, "should_clip_overflow_along_both_axes")
    }

    pub fn IsLayoutBlock(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutBlock)
            .SourceCheckedOr(self.is_layout_block, "is_layout_block")
    }

    pub fn IsLayoutBlockFlow(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsLayoutBlockFlow)
            .SourceCheckedOr(self.is_layout_block_flow, "is_layout_block_flow")
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
            .SourceCheckedOr(self.paint_layer_is_self_painting, "paint_layer_is_self_painting"),
            invisible_for_position_visibility: layer.map(PaintLayer::InvisibleForPositionVisibility)
            .SourceCheckedOr(self.paint_layer_invisible_for_position_visibility, "paint_layer_invisible_for_position_visibility"),
            has_ancestor_invisible_for_position_visibility: layer.map(PaintLayer::HasAncestorInvisibleForPositionVisibility)
            .SourceCheckedOr(self.paint_layer_has_ancestor_invisible_for_position_visibility, "paint_layer_has_ancestor_invisible_for_position_visibility"),
            has_visible_content: layer.map(|layer| {
                layer.HasVisibleContent()
            })
            .SourceCheckedOr(self.paint_layer_has_visible_content, "paint_layer_has_visible_content"),
            has_self_painting_layer_descendant: layer.map(PaintLayer::HasSelfPaintingLayerDescendant)
            .SourceCheckedOr(self.paint_layer_has_self_painting_layer_descendant, "paint_layer_has_self_painting_layer_descendant"),
            is_under_svg_hidden_container: layer.map(PaintLayer::IsUnderSVGHiddenContainer)
            .SourceCheckedOr(self.paint_layer_is_under_svg_hidden_container, "paint_layer_is_under_svg_hidden_container"),
            supports_subsequence_caching: layer.map(PaintLayer::SupportsSubsequenceCaching)
            .SourceCheckedOr(self.paint_layer_supports_subsequence_caching, "paint_layer_supports_subsequence_caching"),
            self_or_descendant_needs_repaint: layer.map(PaintLayer::SelfOrDescendantNeedsRepaint)
            .SourceCheckedOr(self.paint_layer_self_or_descendant_needs_repaint, "paint_layer_self_or_descendant_needs_repaint"),
            is_fragmented: layer.map(|layer| {
                layer.GetLayoutObject().IsFragmented()
            })
            .SourceCheckedOr(self.paint_layer_is_fragmented, "paint_layer_is_fragmented"),
            known_to_clip_subtree_to_padding_box: layer.map(PaintLayer::KnownToClipSubtreeToPaddingBox)
            .SourceCheckedOr(self.paint_layer_known_to_clip_subtree_to_padding_box, "paint_layer_known_to_clip_subtree_to_padding_box"),
            child_paint_blocked_by_display_lock: object.map(LayoutObject::ChildPaintBlockedByDisplayLock)
            .SourceCheckedOr(self.child_paint_blocked_by_display_lock, "child_paint_blocked_by_display_lock"),
            is_z_order_list_visible: layer.map(PaintLayer::IsZOrderListVisible)
            .SourceCheckedOr(self.paint_layer_is_z_order_list_visible, "paint_layer_is_z_order_list_visible"),
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
            .SourceCheckedOr(self.paint_layer_is_self_painting, "paint_layer_is_self_painting")
    }

    pub fn PaintLayerInvisibleForPositionVisibility(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::InvisibleForPositionVisibility)
            .SourceCheckedOr(self.paint_layer_invisible_for_position_visibility, "paint_layer_invisible_for_position_visibility")
    }

    pub fn PaintLayerHasAncestorInvisibleForPositionVisibility(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::HasAncestorInvisibleForPositionVisibility)
            .SourceCheckedOr(self.paint_layer_has_ancestor_invisible_for_position_visibility, "paint_layer_has_ancestor_invisible_for_position_visibility")
    }

    pub fn PaintLayerHasVisibleContent(&self) -> bool {
        self.SourcePaintLayer()
            .map(|layer| {
                layer.HasVisibleContent()
            })
            .SourceCheckedOr(self.paint_layer_has_visible_content, "paint_layer_has_visible_content")
    }

    pub fn PaintLayerHasSelfPaintingLayerDescendant(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::HasSelfPaintingLayerDescendant)
            .SourceCheckedOr(self.paint_layer_has_self_painting_layer_descendant, "paint_layer_has_self_painting_layer_descendant")
    }

    pub fn PaintLayerHasNonIsolatedDescendantWithBlendMode(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::HasNonIsolatedDescendantWithBlendMode)
            .SourceCheckedOr(self.paint_layer_has_non_isolated_descendant_with_blend_mode, "paint_layer_has_non_isolated_descendant_with_blend_mode")
    }

    pub fn PaintLayerIsUnderSvgHiddenContainer(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::IsUnderSVGHiddenContainer)
            .SourceCheckedOr(self.paint_layer_is_under_svg_hidden_container, "paint_layer_is_under_svg_hidden_container")
    }

    pub fn PaintLayerSupportsSubsequenceCaching(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::SupportsSubsequenceCaching)
            .SourceCheckedOr(self.paint_layer_supports_subsequence_caching, "paint_layer_supports_subsequence_caching")
    }

    pub fn PaintLayerSelfOrDescendantNeedsRepaint(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::SelfOrDescendantNeedsRepaint)
            .SourceCheckedOr(self.paint_layer_self_or_descendant_needs_repaint, "paint_layer_self_or_descendant_needs_repaint")
    }

    pub fn PaintLayerIsFragmented(&self) -> bool {
        self.SourcePaintLayer()
            .map(|layer| {
                layer.GetLayoutObject().IsFragmented()
            })
            .SourceCheckedOr(self.paint_layer_is_fragmented, "paint_layer_is_fragmented")
    }

    pub fn PaintLayerKnownToClipSubtreeToPaddingBox(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::KnownToClipSubtreeToPaddingBox)
            .SourceCheckedOr(self.paint_layer_known_to_clip_subtree_to_padding_box, "paint_layer_known_to_clip_subtree_to_padding_box")
    }

    pub fn PaintLayerIsStackingContext(&self) -> bool {
        self.SourceObject()
            .map(|object| {
                object.IsStackingContext()
            })
            .SourceCheckedOr(self.paint_layer_is_stacking_context, "paint_layer_is_stacking_context")
    }

    pub fn PaintLayerIsStacked(&self) -> bool {
        self.SourceObject()
            .map(LayoutObject::IsStacked)
            .SourceCheckedOr(self.paint_layer_is_stacked, "paint_layer_is_stacked")
    }

    pub fn PaintLayerStackingLevel(&self) -> i32 {
        self.SourceObject()
            .map(|object| {
                object.StyleRef().EffectiveZIndex()
            })
            .SourceCheckedOr(self.paint_layer_stacking_level, "paint_layer_stacking_level")
    }

    pub fn PaintLayerIsZOrderListVisible(&self) -> bool {
        self.SourcePaintLayer().map(PaintLayer::IsZOrderListVisible)
            .SourceCheckedOr(self.paint_layer_is_z_order_list_visible, "paint_layer_is_z_order_list_visible")
    }
    pub fn CanPaintFragmentOccurrence(&self) -> bool {
        self.is_first_for_node || self.can_paint_multiple_fragments
    }

    // LayoutObject / LayoutBox / PaintLayer / Element geometry facts, read
    // live as Chromium's PrePaint and painters do (stage 2 of the export
    // removal). The stored fields remain only for fragments without a source.

    pub fn CssMaskBoundingBox(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveCssMaskBoundingBox)
            .SourceCheckedOr(self.css_mask_bounding_box, "css_mask_bounding_box")
    }

    pub fn ScrollableOverflow(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveScrollableOverflow)
            .SourceCheckedOr(self.scrollable_overflow, "scrollable_overflow")
    }

    pub fn PaintLayerFirstFragmentVisualRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(|_| {
                self.SourcePaintLayer().map(
                    crate::internal::boundary::layout_boundary::LivePaintLayerFirstFragmentVisualRect,
                )
            })
            .SourceCheckedOr(
                self.paint_layer_first_fragment_visual_rect,
                "paint_layer_first_fragment_visual_rect",
            )
    }

    pub fn PaintLayerContentsVisualRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(|_| {
                self.SourcePaintLayer().and_then(
                    crate::internal::boundary::layout_boundary::LivePaintLayerContentsVisualRect,
                )
            })
            .SourceCheckedOr(
                self.paint_layer_contents_visual_rect,
                "paint_layer_contents_visual_rect",
            )
    }

    pub fn PaintLayerFilterReferenceBox(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(|_| {
                self.SourcePaintLayer().and_then(
                    crate::internal::boundary::layout_boundary::LivePaintLayerFilterReferenceBox,
                )
            })
            .SourceCheckedOr(
                self.paint_layer_filter_reference_box,
                "paint_layer_filter_reference_box",
            )
    }

    pub fn PaintLayerFilterViewport(&self) -> Option<Size> {
        self.SourceObject()
            .map(|_| {
                self.SourcePaintLayer().and_then(
                    crate::internal::boundary::layout_boundary::LivePaintLayerFilterViewport,
                )
            })
            .SourceCheckedOr(self.paint_layer_filter_viewport, "paint_layer_filter_viewport")
    }

    pub fn ReplacedSelectionState(&self) -> SelectionState {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveReplacedSelectionState)
            .SourceCheckedOr(self.replaced_selection_state, "replaced_selection_state")
    }

    pub fn ReplacedLocalSelectionVisualRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveReplacedLocalSelectionVisualRect)
            .SourceCheckedOr(
                self.replaced_local_selection_visual_rect,
                "replaced_local_selection_visual_rect",
            )
    }

    pub fn RegionCaptureCropId(&self) -> Option<[u64; 2]> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveRegionCaptureCropId)
            .SourceCheckedOr(self.region_capture_crop_id, "region_capture_crop_id")
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
            .SourceCheckedOr(self.source_kind, "source_kind")
    }

    pub fn ImageDecodingMode(&self) -> image_resource::ImageDecodingMode {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveImageDecodingMode)
            .SourceCheckedOr(self.image_decoding_mode, "image_decoding_mode")
    }

    pub fn VideoFrameResourceId(&self) -> Option<image_resource::ImageId> {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveVideoFrameResourceId)
            .SourceCheckedOr(self.video_frame_resource_id, "video_frame_resource_id")
    }

    pub fn FocusedImageMapArea(&self) -> Option<&FocusedImageMapArea> {
        match self
            .SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveFocusedImageMapArea)
        {
            Some(live) => live,
            None => self.focused_image_map_area.as_ref(),
        }
    }

    pub fn SvgViewBoxIsEmpty(&self) -> bool {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveSvgViewBoxIsEmpty)
            .SourceCheckedOr(self.svg_view_box_is_empty, "svg_view_box_is_empty")
    }

    pub fn TextControlHost(&self) -> Option<u64> {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveTextControlHost)
            .SourceCheckedOr(self.text_control_host, "text_control_host")
    }

    // Previous-frame geometry is Chromium's LayoutBox / LayoutView state,
    // saved by the layout commit after the artifact commits.

    pub fn PreviousBoxGeometry(&self) -> Option<PreviousBoxGeometry> {
        self.SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LivePreviousBoxGeometry)
            .SourceCheckedOr(self.previous_box_geometry, "previous_box_geometry")
    }

    pub fn ViewBackgroundRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveViewBackgroundRect)
            .SourceCheckedOr(self.view_background_rect, "view_background_rect")
    }

    pub fn PreviousViewBackgroundRect(&self) -> Option<FragmentPaintRect> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LivePreviousViewBackgroundRect)
            .SourceCheckedOr(
                self.previous_view_background_rect,
                "previous_view_background_rect",
            )
    }

    pub fn DisplayItemRasterEffectOutset(&self) -> RasterEffectOutset {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveRasterEffectOutset)
            .SourceCheckedOr(
                self.display_item_raster_effect_outset,
                "display_item_raster_effect_outset",
            )
    }

    // DisplayItemClient identity. Chromium's PaintController and PrePaint
    // use the LayoutObject (and LayoutText selection) DisplayItemClient
    // itself (display_item_client.h:33-35,71-82).

    pub fn DisplayItemClientId(&self) -> u64 {
        self.SourceObject()
            .map(|object| {
                unsafe { &*(object as *const LayoutObject as *const DisplayItemClient) }.Id() as u64
            })
            .SourceCheckedOr(self.display_item_client_id, "display_item_client_id")
    }

    pub fn DisplayItemClientOwnerNodeId(&self) -> i32 {
        self.SourceObject()
            .map(|object| object.OwnerNodeIdDefault())
            .SourceCheckedOr(
                self.display_item_client_owner_node_id,
                "display_item_client_owner_node_id",
            )
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
            .SourceCheckedOr(self.selection_client_id, "selection_client_id")
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
        match self.SvgPaintGeometry() {
            Some((transform, _, _)) => transform,
            None => self.svg_local_transform.clone(),
        }
    }

    pub fn SvgViewport(&self) -> Option<SvgViewportGeometry> {
        match self.SvgPaintGeometry() {
            Some((_, viewport, _)) => viewport,
            None => self.svg_viewport.clone(),
        }
    }

    pub fn SvgVisualRect(&self) -> Option<FragmentPaintRect> {
        match self.SvgPaintGeometry() {
            Some((_, _, visual_rect)) => visual_rect,
            None => self.svg_visual_rect,
        }
    }

    pub fn SvgViewBox(&self) -> Option<SvgViewBoxData> {
        match self
            .SourceObject()
            .and_then(crate::internal::boundary::layout_boundary::LiveSvgViewBox)
        {
            Some(live) => live,
            None => self.svg_view_box,
        }
    }

    /// Whether the source is an anonymous LayoutTextCombine wrapper.
    pub fn IsTextCombine(&self) -> bool {
        match self.SourceObject() {
            Some(object) => !foundation::DynamicTo::<
                layoutng::internal::layout_text_combine::LayoutTextCombine,
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
    ) -> Option<&layoutng::internal::layout_scrollable_area::PaintLayerScrollableArea> {
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
            .SourceCheckedOr(
                (self.paint_layer_order_parent_unique_id, self.paint_layer_order),
                "paint_layer_order",
            )
    }

    /// The source Node id of the LayoutObject's nearest ancestor with a Node.
    pub fn LogicalParentNodeId(&self) -> Option<u64> {
        self.SourceObject()
            .map(crate::internal::boundary::layout_boundary::LiveLogicalParentNodeId)
            .SourceCheckedOr(self.logical_parent_node_id, "logical_parent_node_id")
    }

    /// The source LayoutObject's resident document-order position: the
    /// 1-based preorder index among LayoutObjects with a Node, maintained on
    /// the object by the layout boundary object index after each topology
    /// mutation (LayoutObject::LogicalTreeOrder). 0 when the source has no
    /// Node. Hand-built fixtures without a live source keep the stored value.
    pub fn LogicalTreeOrder(&self) -> u64 {
        self.SourceObject()
            .map(LayoutObject::LogicalTreeOrder)
            .SourceCheckedOr(self.logical_tree_order, "logical_tree_order")
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
    fn FactStyle(&self) -> Option<&layoutng_style::style::computed_style::ComputedStyle> {
        self.live_style_facts.then(|| self.style.Source()).flatten()
    }

    fn FactLayoutBox(&self) -> Option<&layoutng::internal::layout_box::LayoutBox> {
        if !self.live_style_facts {
            return None;
        }
        self.SourceObject().map(|object| unsafe {
            foundation::DynamicTo::<layoutng::internal::layout_box::LayoutBox>(object).as_ref()
        })?
    }

    pub fn Margin(&self) -> Edges {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleMargin)
            .SourceCheckedOr(self.margin, "margin")
    }

    pub fn BorderStyles(&self) -> [BorderLineStyle; 4] {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleBorderStyles)
            .SourceCheckedOr(self.border_styles, "border_styles")
    }

    pub fn Display(&self) -> Display {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleDisplay)
            .SourceCheckedOr(self.display, "display")
    }

    pub fn Position(&self) -> Position {
        self.FactStyle()
            .map(crate::style_projection::FragmentStylePosition)
            .SourceCheckedOr(self.position, "position")
    }

    pub fn Floating(&self) -> FloatSide {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleFloating)
            .SourceCheckedOr(self.floating, "floating")
    }

    pub fn WritingMode(&self) -> WritingMode {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleWritingMode)
            .SourceCheckedOr(self.writing_mode, "writing_mode")
    }

    pub fn Direction(&self) -> TextDirection {
        self.FactStyle()
            .map(crate::style_projection::FragmentStyleDirection)
            .SourceCheckedOr(self.direction, "direction")
    }

    pub fn OverflowX(&self) -> Overflow {
        self.FactStyle()
            .map(|style| crate::style_projection::FragmentStyleOverflow(style.OverflowX()))
            .SourceCheckedOr(self.overflow_x, "overflow_x")
    }

    pub fn OverflowY(&self) -> Overflow {
        self.FactStyle()
            .map(|style| crate::style_projection::FragmentStyleOverflow(style.OverflowY()))
            .SourceCheckedOr(self.overflow_y, "overflow_y")
    }

    pub fn OverscrollBehaviorX(&self) -> OverscrollBehavior {
        self.FactStyle()
            .map(|style| {
                crate::internal::boundary::layout_boundary::LiveOverscrollBehavior(
                    style.OverscrollBehaviorX(),
                )
            })
            .SourceCheckedOr(self.overscroll_behavior_x, "overscroll_behavior_x")
    }

    pub fn OverscrollBehaviorY(&self) -> OverscrollBehavior {
        self.FactStyle()
            .map(|style| {
                crate::internal::boundary::layout_boundary::LiveOverscrollBehavior(
                    style.OverscrollBehaviorY(),
                )
            })
            .SourceCheckedOr(self.overscroll_behavior_y, "overscroll_behavior_y")
    }

    // Source: core/paint/paint_property_tree_builder.cc:3634-3635.
    pub fn PreventScrollAxisLocking(&self) -> bool {
        self.FactStyle()
            .map(|style| style.ScrollAxisLock() == foundation::EScrollAxisLock::kNone)
            .SourceCheckedOr(self.prevent_scroll_axis_locking, "prevent_scroll_axis_locking")
    }

    pub fn EffectiveZoom(&self) -> f32 {
        self.FactStyle()
            .map(|style| style.EffectiveZoom())
            .SourceCheckedOr(self.effective_zoom, "effective_zoom")
    }

    pub fn HasBackground(&self) -> bool {
        self.FactStyle()
            .map(|style| style.HasBackground())
            .SourceCheckedOr(self.has_background, "has_background")
    }

    pub fn BackgroundFillLayerCount(&self) -> usize {
        self.FactStyle()
            .map(crate::internal::boundary::paint_style_export::BackgroundFillLayerCount)
            .SourceCheckedOr(self.background_fill_layer_count, "background_fill_layer_count")
    }

    pub fn BackgroundAnyLocalAttachmentImage(&self) -> bool {
        self.FactStyle()
            .map(|style| style.BackgroundLayers().AnyLayerHasLocalAttachmentImage())
            .SourceCheckedOr(
                self.background_any_local_attachment_image,
                "background_any_local_attachment_image",
            )
    }

    pub fn BackgroundAnyDefaultAttachmentImage(&self) -> bool {
        self.FactStyle()
            .map(|style| style.BackgroundLayers().AnyLayerHasDefaultAttachmentImage())
            .SourceCheckedOr(
                self.background_any_default_attachment_image,
                "background_any_default_attachment_image",
            )
    }

    pub fn BackgroundAnyLayerUsesContentBox(&self) -> bool {
        self.FactStyle()
            .map(|style| style.BackgroundLayers().AnyLayerUsesContentBox())
            .SourceCheckedOr(
                self.background_any_layer_uses_content_box,
                "background_any_layer_uses_content_box",
            )
    }

    pub fn FixedToView(&self) -> bool {
        self.live_style_facts
            .then(|| self.FactLayoutBox().is_some_and(|box_| box_.IsFixedToView(std::ptr::null())))
            .SourceCheckedOr(self.fixed_to_view, "fixed_to_view")
    }

    pub fn NeedsAnchorPositionScrollAdjustment(&self) -> bool {
        self.live_style_facts
            .then(|| {
                self.FactLayoutBox()
                    .is_some_and(|box_| box_.NeedsAnchorPositionScrollAdjustment())
            })
            .SourceCheckedOr(
                self.needs_anchor_position_scroll_adjustment,
                "needs_anchor_position_scroll_adjustment",
            )
    }

    // C++ chained assignment evaluates the right-hand assignment first.
    // cpp: layoutng_fragment_tree/fragment_tree.h:96
    pub fn SetOverflow(&mut self, value: Overflow) {
        self.overflow_y = value;
        self.overflow_x = value;
    }
}

/// Stage-0 check harness for the Layout -> Paint export removal. With
/// RECHROM_PAINT_SOURCE_CHECK set, every PaintProperties getter that reads its
/// live LayoutObject / PaintLayer also compares the exported copy and emits a
/// `layout_fragment.PaintSourceMismatch` instant naming the field on a
/// difference. The live value is always returned; fixtures without a source
/// keep the exported copy.
pub(crate) trait SourceCheckedOr<T> {
    fn SourceCheckedOr(self, stored: T, field: &'static str) -> T;
}

impl<T: PartialEq> SourceCheckedOr<T> for Option<T> {
    #[inline]
    fn SourceCheckedOr(self, stored: T, field: &'static str) -> T {
        match self {
            None => stored,
            Some(live) => {
                if PaintSourceCheckEnabled() && live != stored {
                    browser_tracing::instant(
                        "layout",
                        "layout_fragment.PaintSourceMismatch",
                        &[(field, 1.0)],
                    );
                }
                live
            }
        }
    }
}

pub(crate) fn PaintSourceCheckEnabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RECHROM_PAINT_SOURCE_CHECK").is_some())
}
