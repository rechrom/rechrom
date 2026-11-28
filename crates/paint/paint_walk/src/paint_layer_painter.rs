#![allow(non_snake_case)]

//! PaintLayerPainter over Layout's PaintLayer tree and the fragment cursor.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: third_party/blink/renderer/core/paint/paint_layer_painter.cc
//! Source: third_party/blink/renderer/core/paint/box_fragment_painter.cc
//!         (PaintObject, PaintBlockChildren, PaintInlineItems)
//! Source: third_party/blink/renderer/core/paint/frame_painter.cc,
//!         embedded_content_painter.cc
//!
//! A painter paints one PaintLayer occurrence (a PhysicalBoxFragment of the
//! layer's LayoutObject, reached through its fragment cursor): its own
//! phases through BoxFragmentPainter, its child layers through the
//! PaintLayer stacking lists (PaintLayerPaintOrderIterator), and the
//! non-layer descendants through the fragment cursor's children, skipping
//! those that paint as their own layer. Every painted object is a
//! `PaintObject` built from its fragment cursor, FragmentData and the walk
//! context (the PaintInfo cull rect, the in-flight PaintLayer paint result).

use layout::fragment::ResidentBoxGeometry as _;
use std::cell::{Cell, RefCell};

use layout::fragment::FragmentKind;
use layout::input::FloatSide;
use layout::paint::{FragmentPaintData, PaintFragmentCursor};
use layout::views::paint_layer::PaintLayer;
use paint_input_types::paint_input::PaintFilterType;

use crate::box_fragment_painter::BoxFragmentPainter;
use crate::geometry_mapper::{MapRectFromRoot, MapRectToRoot};
use crate::ignore_paint_timing_scope::IgnorePaintTimingScope;
use crate::paint_context::PaintContext;
use crate::paint_info::PaintInfo;
use crate::paint_layer_paint_order_iterator::{
    LayerOccurrence, LayerOccurrences, PaintLayerIteration, PaintLayerPaintOrderIterator,
};
use crate::pre_paint_tree_walk::PrePaintTreeWalk;
use crate::PaintPhase;
use crate::PaintRect;
use paint_model::paint_flags::{PaintFlag, PaintFlags};
use paint_model::paint_property_tree::PropertyTreeState;
use paint_model::PaintResult;
use paint_record::{
    box_fragment_painter::GetHitTestOpaqueness,
    paint_chunker::DisplayItemClientInfo,
    paint_engine::PaintHitTestOpaqueness,
    paint_object::{LogicalPaintChildren, PaintObject, PaintWalkContext, ResolvedFragment},
    scoped_paint_state::ContentsVisualRect,
    SubsequenceRecorder,
};

/// LayoutEmbeddedContent::ChildFrameView, as EmbeddedContentPainter reaches
/// a same-Page child LocalFrameView to paint it (embedded_content_painter.cc,
/// LocalFrameView::Paint -> FramePainter::Paint): the root fragment of the
/// frame embedded by the object whose DisplayItemClient is `owner_client`,
/// and whether that frame is cross-origin to its parent.
pub trait LocalFrameRoots {
    fn LocalFrameRoot(&self, owner_client: u64) -> Option<(PaintFragmentCursor<'_>, bool)>;
}

impl LocalFrameRoots for PrePaintTreeWalk {
    fn LocalFrameRoot(&self, owner_client: u64) -> Option<(PaintFragmentCursor<'_>, bool)> {
        let view = self.LocalFrameView(owner_client)?;
        Some((view.Root(), view.CrossOriginToParent()))
    }
}

fn IsFloat(fragment: &FragmentPaintData) -> bool {
    fragment.paint.HasSource() && fragment.paint.Floating() != FloatSide::kNone
}

// Source: core/paint/paint_layer_painter.cc:167-190.
fn PaintedOutputInvisible(fragment: &FragmentPaintData) -> bool {
    let style = &fragment.paint.style;
    if style.has_backdrop_filters() {
        return false;
    }
    if style.will_change_opacity() || style.active_opacity_animation() {
        return false;
    }
    style.opacity() < 0.0004
}

// Source: core/paint/paint_layer_painter.cc:310-318. Layout exports the exact
// ancestor-layer answer that Chromium computes inside its stacking-context
// scope (PaintLayer::InvisibleForPositionVisibility /
// HasAncestorInvisibleForPositionVisibility).
fn HiddenForPositionVisibility(
    facts_are_authoritative: bool,
    invisible_for_position_visibility: bool,
    has_ancestor_invisible_for_position_visibility: bool,
) -> bool {
    facts_are_authoritative
        && (invisible_for_position_visibility || has_ancestor_invisible_for_position_visibility)
}

// Source: core/paint/paint_layer_painter.cc:289-295,300-302.
fn ShouldSkipForLayout(fragment: &FragmentPaintData) -> bool {
    (fragment.paint.NeedsLayout() && !fragment.paint.ChildLayoutBlockedByDisplayLock())
        || fragment.paint.IsFragmentLessBox()
}

fn HasReferenceFilter(fragment: &FragmentPaintData) -> bool {
    let style = &fragment.paint.style;
    let filters = style.filters();
    let backdrop_filters = style.backdrop_filters();
    filters
        .iter()
        .chain(backdrop_filters.iter())
        .any(|operation| operation.r#type == PaintFilterType::kReference)
}

fn HasCanvasChildState(properties: &PropertyTreeState) -> bool {
    paint_model::property_cell::with_access(|access| properties.effect.HasCanvasChildState(access))
}

fn FragmentRect(rect: paint_input_types::FragmentPaintRect) -> PaintRect {
    PaintRect {
        x: rect.offset.x,
        y: rect.offset.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

fn Intersects(a: PaintRect, b: PaintRect) -> bool {
    !a.intersection(b).is_empty()
}

/// Source: `paint_layer_painter.cc:369-410` and `ContentsVisualRect():192-208`.
///
/// The layer's FragmentData supplies the cull rects (CullRectUpdater) and
/// PaintOffset; LayoutBox supplies its stitched contents overflow.
/// `known_to_clip_subtree_to_padding_box` is
/// PaintLayer::KnownToClipSubtreeToPaddingBox.
fn CullAdmission(
    layer: PaintObject<'_, '_>,
    mut should_paint_content: bool,
    known_to_clip_subtree_to_padding_box: bool,
) -> (PaintResult, bool, bool) {
    let fragment = layer.Fragment().expect("painted layer has a fragment");
    if fragment.paint.PaintLayerIsFragmented()
        || (fragment.paint.IsLayoutView() && fragment.paint.IsPrinting())
        || HasCanvasChildState(layer.properties())
        || fragment.paint.IsInclusiveDescendantOfUnboundedElement()
    {
        return (
            PaintResult::kMayBeClippedByCullRect,
            should_paint_content,
            false,
        );
    }

    let (Some(visual_rect), Some(self_cull_rect)) = (
        fragment.paint.PaintLayerFirstFragmentVisualRect(),
        layer.self_cull_rect(),
    ) else {
        // Hand-built fragments and incomplete lifecycle input retain the
        // pre-existing admission. Chromium layers always export both values.
        return (PaintResult::kFullyPainted, should_paint_content, false);
    };
    let mut visual_rect = FragmentRect(visual_rect);
    // Source: PaintLayerPainter::FirstFragmentVisualRect(). Layout exports
    // the local virtual overflow; apply this PrePaint pass's FragmentData
    // paint offset here, after property-tree construction, as Chromium does.
    let paint_offset = layer.FlatPaintOffset();
    visual_rect.x += paint_offset.x;
    visual_rect.y += paint_offset.y;
    let self_intersects = Intersects(self_cull_rect, visual_rect);
    let mut result = if self_cull_rect.contains(visual_rect) {
        PaintResult::kFullyPainted
    } else {
        PaintResult::kMayBeClippedByCullRect
    };

    let contents_intersects = if fragment.paint.IsLayoutBox() {
        if let (Some(contents), Some(contents_cull_rect)) =
            (ContentsVisualRect(layer), layer.cull_rect())
        {
            if !contents_cull_rect.contains(contents) {
                result = PaintResult::kMayBeClippedByCullRect;
            }
            Intersects(contents_cull_rect, contents)
        } else {
            true
        }
    } else {
        self_intersects
    };

    if !self_intersects && !contents_intersects {
        if known_to_clip_subtree_to_padding_box {
            return (PaintResult::kMayBeClippedByCullRect, false, true);
        }
        should_paint_content = false;
    }
    (result, should_paint_content, false)
}

/// BoxFragmentPainter::PaintFragment, skipping its ShouldPaint test when the
/// walk's own gate already admitted this object with this PaintInfo.
fn PaintGatedFragment(object: PaintObject<'_, '_>, paint_info: &PaintInfo<'_, '_>, admitted: bool) {
    if admitted {
        BoxFragmentPainter::PaintFragmentAdmitted(object, paint_info);
    } else {
        BoxFragmentPainter::PaintFragment(object, paint_info);
    }
}

/// Source: paint_layer_painter.cc:250-254. `paint_layer.FirstChild() &&
/// (FirstChild()->FirstChild() || FirstChild()->NextSibling())` over the
/// PaintLayer tree: whether the layer has at least two descendant layers.
fn HasTwoDescendantPaintLayers(layer: &PaintLayer) -> bool {
    let Some(first) = (unsafe { layer.FirstChild().as_ref() }) else {
        return false;
    };
    !first.FirstChild().is_null() || !first.NextSibling().is_null()
}

/// Source: `paint_layer_painter.cc:221-235,239-284`.
///
/// These source branches cover property boundaries, non-trivial descendant
/// layer trees, compositor hit-test opaqueness and sparse merged bounds
/// without creating proxy scopes. `supports_subsequence_caching` is
/// PaintLayer::SupportsSubsequenceCaching.
/// Special recording preserves source's kOmitCompositingInfo cache exclusion.
fn ShouldCreateSubsequence(
    layer: PaintObject<'_, '_>,
    paint_layer: Option<&PaintLayer>,
    supports_subsequence_caching: bool,
    context: &RefCell<PaintContext<'_>>,
    paint_flags: PaintFlags,
) -> bool {
    let fragment = layer.Fragment().expect("painted layer has a fragment");
    if fragment.paint.IsPrinting()
        || paint_flags & PaintFlag::kOmitCompositingInfo != 0
        || context.borrow().IsSkippingCache()
        || !supports_subsequence_caching
    {
        return false;
    }
    if context.borrow().NumNewChunks() > 0
        && !context
            .borrow()
            .LastChunkPropertiesSameNodes(layer.properties())
    {
        return true;
    }
    if paint_layer.is_some_and(HasTwoDescendantPaintLayers) {
        return true;
    }
    if context.borrow().NumNewChunks() > 0 {
        let transparent_to_hit_test =
            GetHitTestOpaqueness(fragment) == PaintHitTestOpaqueness::Transparent;
        if transparent_to_hit_test
            != context
                .borrow()
                .CurrentChunkIsNonEmptyAndTransparentToHitTest()
        {
            return true;
        }
        if let (Some(last_bounds), Some(visual_rect), Some(resources)) = (
            context.borrow().LastChunkBounds(),
            fragment.paint.PaintLayerFirstFragmentVisualRect(),
            layer.DocumentResources(),
        ) {
            let visual_rect = FragmentRect(visual_rect);
            let mut merged_bounds = last_bounds;
            merged_bounds.union(visual_rect);
            let area = |rect: PaintRect| rect.width * rect.height;
            let device_pixel_ratio = resources.device_pixel_ratio;
            if area(merged_bounds)
                > 10000.0 * device_pixel_ratio * device_pixel_ratio
                    + area(last_bounds)
                    + area(visual_rect)
            {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// PaintLayerPainter.
// ---------------------------------------------------------------------------

/// The child layer occurrences of one painted layer occurrence.
struct ChildLayers<'p, 'a> {
    paint_layer: &'p PaintLayer,
    occurrences: LayerOccurrences<'a>,
}

pub struct PaintLayerPainter<'g, 'a, 'c, 'o> {
    /// The painted layer occurrence: a PhysicalBoxFragment of `paint_layer_`'s
    /// LayoutObject, a frame's LayoutView, or an SVG resource subtree root;
    /// its sidecar and FragmentData resolved once for the painter's lifetime.
    layer: ResolvedFragment<'a>,
    /// Chromium's `paint_layer_`. None for an SVG resource root.
    paint_layer: Option<&'a PaintLayer>,
    context: &'c RefCell<PaintContext<'o>>,
    /// The child LocalFrameViews EmbeddedContentPainter paints.
    frames: Option<&'a dyn LocalFrameRoots>,
    /// The property ancestors this layer re-enters in the flat replay stream:
    /// the fragments between the painting parent layer and this one. A child
    /// painter borrows its parent's occurrence list (PaintLayerPainter is a
    /// stack object over the parent's z-order list in Chromium).
    inherited_geometry: &'g [ResolvedFragment<'a>],
    escapes_ancestor_geometry: bool,
    /// How many leading `inherited_geometry` entries this layer re-enters the
    /// escapable geometry of.
    uses_inherited_geometry: usize,
    paint_flags: PaintFlags,
    /// The in-flight PaintLayer::PreviousPaintResult this painter seeds and
    /// ScopedBoxContentsPaintState downgrades, written to the PaintLayer at
    /// the end of Paint (paint_layer_painter.cc:412-424,589-596).
    previous_paint_result: Cell<PaintResult>,
}

impl<'g, 'a, 'c, 'o> PaintLayerPainter<'g, 'a, 'c, 'o> {
    /// LocalFrameView::PaintTree / FramePainter::Paint: the painter of a
    /// frame's root layer (`root` is its LayoutView fragment), or of an SVG
    /// resource subtree root.
    pub fn new(
        root: PaintFragmentCursor<'a>,
        context: &'c RefCell<PaintContext<'o>>,
        frames: Option<&'a dyn LocalFrameRoots>,
    ) -> Self {
        Self {
            layer: ResolvedFragment::new(root),
            paint_layer: root.OwnPaintLayer(),
            context,
            frames,
            inherited_geometry: &[],
            escapes_ancestor_geometry: false,
            uses_inherited_geometry: 0,
            paint_flags: PaintFlag::kNoFlag,
            previous_paint_result: Cell::new(PaintResult::kMayBeClippedByCullRect),
        }
    }

    /// PaintLayerPainter::Paint(context, paint_flags), source.cc:289-333.
    pub fn WithFlags(mut self, flags: PaintFlags) -> Self {
        self.paint_flags = flags;
        self
    }

    /// The painter of one child layer occurrence.
    fn ChildPainter<'s>(&self, occurrence: &LayerOccurrence<'s, 'a>) -> PaintLayerPainter<'s, 'a, 'c, 'o> {
        PaintLayerPainter {
            layer: occurrence.layer,
            paint_layer: occurrence.layer.cursor.OwnPaintLayer(),
            context: self.context,
            frames: self.frames,
            inherited_geometry: occurrence.property_ancestors,
            escapes_ancestor_geometry: occurrence.escapes_ancestor_geometry,
            uses_inherited_geometry: occurrence.uses_inherited_geometry,
            paint_flags: self.paint_flags,
            previous_paint_result: Cell::new(PaintResult::kMayBeClippedByCullRect),
        }
    }

    /// PaintLayer::SetPreviousPaintResult (paint_layer.cc:2827-2832), written
    /// by PaintLayerPainter::Paint, after the in-flight value. The source
    /// skips the PaintLayer write while CullRectUpdater::IsOverridingCullRects();
    /// in Rechrom only the special recordings (drag images, resource
    /// subtrees) override cull rects, and they all paint with
    /// kOmitCompositingInfo.
    fn SetPreviousPaintResult(&self, result: PaintResult) {
        self.previous_paint_result.set(result);
        if self.paint_flags & PaintFlag::kOmitCompositingInfo == 0 {
            if let Some(paint_layer) = self.paint_layer {
                paint_layer.SetPreviousPaintResult(result);
            }
        }
    }

    /// PaintLayer::NeedsReorderOverlayOverflowControls (paint_layer.h:
    /// 427-431), set by the owning PaintLayerStackingNode's rebuild.
    fn NeedsReorderOverlayOverflowControls(&self) -> bool {
        self.paint_layer
            .is_some_and(PaintLayer::NeedsReorderOverlayOverflowControls)
    }

    /// PaintLayer::NeedsPaintPhaseFloat / NeedsPaintPhaseDescendantOutlines
    /// (paint_layer.h:490-507), set by PaintInvalidator::UpdatePaintingLayer.
    /// An SVG resource root without a PaintLayer keeps its subtree's phase
    /// needs.
    fn NeedsPaintPhaseFloat(&self) -> bool {
        self.paint_layer.map_or_else(
            || self.LayerObject().needs_float_phase(),
            PaintLayer::NeedsPaintPhaseFloat,
        )
    }

    fn NeedsPaintPhaseDescendantOutlines(&self) -> bool {
        self.paint_layer.map_or_else(
            || self.LayerObject().needs_outline_phase(),
            PaintLayer::NeedsPaintPhaseDescendantOutlines,
        )
    }

    fn PaintInfo(&self, phase: PaintPhase) -> PaintInfo<'c, 'o> {
        PaintInfo::new_with_flags(self.context, phase, false, self.paint_flags)
    }

    /// The walk context of this painter's layer object: the in-flight
    /// PaintLayer::PreviousPaintResult this painter seeds and reads, and no
    /// inherited cull rect. A PaintLayer object paints with its FragmentData
    /// cull rects (CullRectUpdater); an SVG resource root paints with an
    /// infinite cull rect (SVGObjectPainter::PaintResourceSubtree).
    fn LayerWalk(&self) -> PaintWalkContext<'_> {
        PaintWalkContext::new(None, Some(&self.previous_paint_result))
    }

    /// The painted object of this painter's layer, built from its fragment
    /// cursor, FragmentData and the walk context.
    fn LayerObject(&self) -> PaintObject<'_, 'static> {
        self.layer.Object(self.LayerWalk())
    }

    fn IsLayerObject(&self, object: PaintObject<'_, '_>) -> bool {
        object
            .Fragment()
            .is_some_and(|fragment| std::ptr::eq(fragment, self.layer.fragment))
    }

    /// Property ancestor `index` as a painted object (its scopes only).
    fn InheritedObject(&self, index: usize) -> PaintObject<'a, 'static> {
        self.inherited_geometry[index].Object(PaintWalkContext::default())
    }

    fn UsesInheritedGeometry(&self, index: usize) -> bool {
        !self.escapes_ancestor_geometry || index < self.uses_inherited_geometry
    }

    fn InheritedClipInLayerSpace(&self, ancestor: PaintObject<'_, '_>) -> Option<PaintRect> {
        let local_clip = ancestor.local_clip()?;
        let Some(root_clip) = MapRectToRoot(local_clip, ancestor.transforms()) else {
            return Some(PaintRect::default());
        };
        Some(MapRectFromRoot(root_clip, self.LayerObject().transforms()).unwrap_or_default())
    }

    /// The child layer occurrences PaintChildren paints (paint_layer_painter.cc:
    /// 640-650 returns early without self-painting descendants or under a
    /// display lock).
    fn CollectChildLayers(
        &self,
        has_self_painting_layer_descendant: bool,
        child_paint_blocked_by_display_lock: bool,
        resolve_escaping: bool,
    ) -> Option<ChildLayers<'a, 'a>> {
        let paint_layer = self.paint_layer?;
        if !has_self_painting_layer_descendant || child_paint_blocked_by_display_lock {
            return None;
        }
        let through_layers = paint_layer.StackingNode().is_some();
        Some(ChildLayers {
            paint_layer,
            occurrences: LayerOccurrences::Collect(
                self.layer,
                paint_layer,
                through_layers,
                resolve_escaping,
            ),
        })
    }

    /// PaintLayerPainter::Paint, paint_layer_painter.cc:287-596.
    pub fn Paint(&self) -> PaintResult {
        self.context.borrow_mut().walk_metrics.layer_visits += 1;
        let object = self.LayerObject();
        // Source: core/paint/paint_layer_painter.cc:289-295,300-302.
        let fragment = object.Fragment().expect("painted layer has a fragment");
        if ShouldSkipForLayout(fragment) {
            self.context.borrow_mut().walk_metrics.skipped_hidden += 1;
            return PaintResult::kFullyPainted;
        }
        // The live PaintLayer facts of the layer's LayoutObject, read once.
        let facts = fragment.paint.LivePaintLayerFacts();
        let is_self_painting_layer = object.is_self_painting_layer();
        // Source: core/paint/paint_layer_painter.cc:304-308. A layer with no
        // content may still have self-painting descendants; fragment-hidden is
        // handled by the fragment painter and must not stand in for these
        // PaintLayer aggregates.
        if facts.authoritative
            && !is_self_painting_layer
            && !facts.has_self_painting_layer_descendant
        {
            self.context.borrow_mut().walk_metrics.skipped_hidden += 1;
            return PaintResult::kFullyPainted;
        }
        if HiddenForPositionVisibility(
            facts.authoritative,
            facts.invisible_for_position_visibility,
            facts.has_ancestor_invisible_for_position_visibility,
        ) {
            self.context.borrow_mut().walk_metrics.skipped_hidden += 1;
            return PaintResult::kFullyPainted;
        }
        // Source: paint_layer_painter.cc:330-333. This is Layout's actual
        // selection state, not intersection with selection geometry.
        let selection_drag_image_only = self.paint_flags & PaintFlag::kSelectionDragImageOnly != 0;
        if selection_drag_image_only && !fragment.paint.IsSelected() {
            return PaintResult::kFullyPainted;
        }
        // Source: core/paint/paint_layer_painter.cc:334-348. The document-wide
        // opacity fact is resolved once by PaintEngine from the immutable root;
        // nested layer scopes retain Chromium's exact depth/reset behavior.
        let _ignore_paint_timing = IgnorePaintTimingScope::new();
        if fragment.paint.style.opacity() == 0.0 {
            IgnorePaintTimingScope::IncrementIgnoreDepth();
        }
        IgnorePaintTimingScope::SetIsDocumentElementInvisible(
            self.context.borrow().DocumentElementIsInvisible(),
        );
        // Source: core/paint/paint_layer_painter.cc:361-368.
        let should_paint_content = !facts.authoritative
            || (facts.has_visible_content
                && !facts.is_under_svg_hidden_container
                && is_self_painting_layer);
        let (mut result, should_paint_content, clipped_out) = CullAdmission(
            object,
            should_paint_content,
            facts.known_to_clip_subtree_to_padding_box,
        );
        if clipped_out {
            self.SetPreviousPaintResult(result);
            return result;
        }
        // ScopedBoxContentsPaintState may downgrade this while phases paint.
        // Seed it with this traversal's result rather than the previous frame.
        self.previous_paint_result.set(result);
        // Source: paint_layer_painter.cc:412-424. The scope is declared before
        // all paint-state scopes so its Drop closes the subsequence last.
        let subsequence_recorder = if should_paint_content
            && ShouldCreateSubsequence(
                object,
                self.paint_layer,
                facts.supports_subsequence_caching,
                self.context,
                self.paint_flags,
            ) {
            let client = DisplayItemClientInfo::from_paint_layer(fragment)
                .expect("subsequence requires a source PaintLayer client");
            let debug = self.context.borrow().DebugInfoEnabled().then(|| {
                (
                    fragment.paint.DisplayItemClientOwnerNodeId(),
                    fragment.paint.DisplayItemClientDebugName(),
                )
            });
            self.context.borrow_mut().walk_metrics.subsequence_layers += 1;
            if fragment.paint.PaintLayerSelfOrDescendantNeedsRepaint() {
                self.context.borrow_mut().walk_metrics.subsequence_needs_repaint += 1;
            } else {
                if let Some(cached_result) = SubsequenceRecorder::UseCachedSubsequenceIfPossible(
                    self.context,
                    fragment.paint.PaintLayerClientId(),
                    client,
                    debug.clone(),
                ) {
                    // Chromium returns PaintLayer::PreviousPaintResult(); the
                    // subsequence marker retains that same value.
                    self.SetPreviousPaintResult(cached_result);
                    return cached_result;
                }
            }
            Some(SubsequenceRecorder::new(
                self.context,
                fragment.paint.PaintLayerClientId(),
                debug,
            ))
        } else {
            None
        };
        // Source: core/paint/paint_layer_painter.cc:429-432. This marks chunks
        // as non-drawing while retaining their display items for later opacity
        // changes and invalidation comparison.
        let _effectively_invisible = PaintedOutputInvisible(fragment).then(|| {
            crate::scoped_effectively_invisible::ScopedEffectivelyInvisible::new(self.context)
        });
        let _layer_chunk_properties = should_paint_content.then(|| {
            crate::paint_context::ScopedPaintChunkProperties::layer(
                self.context,
                object,
                crate::display_item_id::DisplayItemIdType::kLayerChunk,
            )
        });
        // Source: core/paint/paint_layer_painter.cc:457-468. Reference filters
        // must retain a property-bearing chunk even if this layer records no
        // drawing. Canvas-child forcing remains a separate unsupported branch.
        if should_paint_content
            && (HasCanvasChildState(object.properties()) || HasReferenceFilter(fragment))
        {
            self.context.borrow_mut().EnsureChunk();
        }
        // Source: paint_layer_painter.cc:640-650. The child layer occurrences
        // are collected once for both PaintChildren calls of this layer.
        // Every consumer of the escaping-child fact iterates
        // `inherited_geometry`, so escaping layers of the subtree are only
        // resolved when there is inherited geometry to re-enter; the root and
        // most layers have none.
        let needs_escaping_scan =
            !self.inherited_geometry.is_empty() && !self.escapes_ancestor_geometry;
        let child_layers = self.CollectChildLayers(
            facts.has_self_painting_layer_descendant,
            facts.child_paint_blocked_by_display_lock,
            needs_escaping_scan,
        );
        let effect_contains_escaping_child = needs_escaping_scan
            && child_layers
                .as_ref()
                .is_some_and(|children| children.occurrences.HasEscapingLayer());
        for index in 0..self.inherited_geometry.len() {
            let ancestor = self.InheritedObject(index);
            self.context.borrow_mut().BeginGeometry(ancestor);
            self.context.borrow_mut().BeginEffects(ancestor);
            if !effect_contains_escaping_child && self.UsesInheritedGeometry(index) {
                self.context
                    .borrow_mut()
                    .BeginEscapableGeometry(ancestor, None);
            }
        }
        self.context.borrow_mut().BeginGeometry(object);
        self.context.borrow_mut().BeginEffects(object);
        if effect_contains_escaping_child {
            for index in 0..self.inherited_geometry.len() {
                if self.UsesInheritedGeometry(index) {
                    let ancestor = self.InheritedObject(index);
                    let clip = self.InheritedClipInLayerSpace(ancestor);
                    self.context
                        .borrow_mut()
                        .BeginEscapableGeometry(ancestor, clip);
                }
            }
        }

        if should_paint_content && !selection_drag_image_only {
            BoxFragmentPainter::PaintFragment(
                object,
                &self.PaintInfo(PaintPhase::kSelfBlockBackgroundOnly),
            );
        }
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(object, None);
        if let Some(children) = &child_layers {
            result.Include(self.PaintChildren(
                children.paint_layer,
                PaintLayerIteration::kNegativeZOrderChildren,
                children,
            ));
        }
        if should_paint_content {
            let _foreground_properties = crate::paint_context::ScopedPaintChunkProperties::layer(
                self.context,
                object,
                crate::display_item_id::DisplayItemIdType::kLayerChunkForeground,
            );
            if selection_drag_image_only {
                self.PaintWithPhase(object, &self.PaintInfo(PaintPhase::kSelectionDragImage));
            } else {
                self.PaintForegroundPhases(object);
            }
        }
        // Blink paints the self outline with LocalBorderBoxProperties, outside
        // ScopedBoxContentsPaintState (box_fragment_painter.cc:759-799). Mirror
        // that scope in the flat recording too; children still need this layer's
        // overflow clip when their own paint phases resume below.
        self.context.borrow_mut().EndEscapableGeometry(object);
        if is_self_painting_layer && object.paints_outline() {
            BoxFragmentPainter::PaintFragment(
                object,
                &self.PaintInfo(PaintPhase::kSelfOutlineOnly),
            );
        }
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(object, None);
        if let Some(children) = &child_layers {
            result.Include(self.PaintChildren(
                children.paint_layer,
                PaintLayerIteration::kNormalFlowAndPositiveZOrderChildren,
                children,
            ));
        }

        // Source: paint_layer_painter.cc:507-514. Overlay controls are above
        // normal-flow and positive-z descendants unless StackingNode moved
        // them after the highest contained stacked descendant.
        if should_paint_content
            && fragment.paint.PaintLayerHasOverlayOverflowControls()
            && !self.NeedsReorderOverlayOverflowControls()
        {
            self.PaintWithPhase(object, &self.PaintInfo(PaintPhase::kOverlayOverflowControls));
        }

        self.context.borrow_mut().EndEscapableGeometry(object);
        if effect_contains_escaping_child {
            for index in (0..self.inherited_geometry.len()).rev() {
                if self.UsesInheritedGeometry(index) {
                    self.context
                        .borrow_mut()
                        .EndEscapableGeometry(self.InheritedObject(index));
                }
            }
        }
        // Mask's parent is the content Effect, outside a separate Filter node.
        // Finish the flat filter before recording the source Mask drawing, as
        // PaintFragmentWithPhase(kMask) uses the Mask child's property state.
        self.context.borrow_mut().EndFilter(object);
        self.context.borrow_mut().EndCssClip(object);
        self.context.borrow_mut().EndMaskClip(object);
        if should_paint_content && object.applies_mask() {
            BoxFragmentPainter::PaintFragment(object, &self.PaintInfo(PaintPhase::kMask));
        }
        self.context.borrow_mut().EndEffectsAfterFilter(object);
        self.context.borrow_mut().EndGeometry(object);
        for index in (0..self.inherited_geometry.len()).rev() {
            let ancestor = self.InheritedObject(index);
            if !effect_contains_escaping_child && self.UsesInheritedGeometry(index) {
                self.context.borrow_mut().EndEscapableGeometry(ancestor);
            }
            self.context.borrow_mut().EndEffects(ancestor);
            self.context.borrow_mut().EndGeometry(ancestor);
        }
        result.Include(self.previous_paint_result.get());
        self.SetPreviousPaintResult(result);
        if let Some(recorder) = &subsequence_recorder {
            recorder.SetPaintResult(result);
        }
        result
    }

    fn SuspendGeometryForEscapingChild(&self) {
        if !self.LayerObject().is_root() {
            self.context
                .borrow_mut()
                .EndEscapableGeometry(self.LayerObject());
        }
        for index in (0..self.inherited_geometry.len()).rev() {
            self.context
                .borrow_mut()
                .EndEscapableGeometry(self.InheritedObject(index));
        }
    }

    fn ResumeGeometryAfterEscapingChild(&self) {
        for index in 0..self.inherited_geometry.len() {
            let ancestor = self.InheritedObject(index);
            let clip = self.InheritedClipInLayerSpace(ancestor);
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(ancestor, clip);
        }
        if !self.LayerObject().is_root() {
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(self.LayerObject(), None);
        }
    }

    // Source: paint_layer_painter.cc:790-810.
    fn PaintForegroundPhases(&self, object: PaintObject<'_, 'static>) {
        self.PaintWithPhase(
            object,
            &self.PaintInfo(PaintPhase::kDescendantBlockBackgroundsOnly),
        );
        if object
            .Fragment()
            .expect("painted layer has a fragment")
            .paint
            .InForcedColorsMode()
        {
            self.PaintWithPhase(
                object,
                &self.PaintInfo(PaintPhase::kForcedColorsModeBackplate),
            );
        }
        if self.NeedsPaintPhaseFloat() {
            self.PaintWithPhase(object, &self.PaintInfo(PaintPhase::kFloat));
        }
        self.PaintWithPhase(object, &self.PaintInfo(PaintPhase::kForeground));
        if self.NeedsPaintPhaseDescendantOutlines() {
            self.PaintWithPhase(
                object,
                &self.PaintInfo(PaintPhase::kDescendantOutlinesOnly),
            );
        }
    }

    /// BoxFragmentPainter::Paint of one object in one phase, then its
    /// children (box_fragment_painter.cc PaintObject -> PaintBlockChildren /
    /// PaintInlineItems): the fragment cursor's children in paint order,
    /// skipping those that paint as their own layer, then the embedded child
    /// LocalFrameView (EmbeddedContentPainter). Each child is built from its
    /// cursor, FragmentData and a walk context carrying this object's
    /// contents cull rect.
    fn PaintWithPhase<'x, 'p>(&self, object: PaintObject<'x, 'static>, paint_info: &PaintInfo<'p, 'o>) {
        let mut dispatched_info = *paint_info;
        dispatched_info.svg_resource_painter = Some(PaintSvgResourceSubtree);
        let paint_info = &dispatched_info;
        let fragment = object.Fragment().expect("native Paint owner");
        if fragment.paint.IsSVGHiddenContainer()
            && !paint_info.IsSvgResourceRoot(fragment.paint.DisplayItemClientId())
        {
            return;
        }
        let mut transformed_context = paint_info.GetSvgContextPaints().map(|paints| {
            let mut result = *paints;
            for transform in object.local_transforms() {
                result.transform =
                    crate::geometry_mapper::MultiplyTransforms(&result.transform, transform);
            }
            result
        });
        if let Some(source) = object.SourceCursor().filter(|source| source.IsSvgUse()) {
            let rect = source
                .SvgContextReferenceBox()
                .expect("native SVG use geometry");
            transformed_context = Some(paint_record::svg_context_paints::SvgContextPaints::New(
                &fragment.paint.style,
                PaintRect {
                    x: rect.x() as f64,
                    y: rect.y() as f64,
                    width: rect.width() as f64,
                    height: rect.height() as f64,
                },
                paint_info.GetSvgContextPaints(),
                Default::default(),
            ));
        }
        let paint_info = if let Some(paints) = &transformed_context {
            dispatched_info.WithSvgContextPaints(paints)
        } else {
            dispatched_info
        };
        let paint_info = &paint_info;
        self.context.borrow_mut().walk_metrics.phase_visits += 1;
        if fragment.ViewHidden() {
            self.context.borrow_mut().walk_metrics.skipped_hidden += 1;
            return;
        }
        if !fragment.ViewCanPaintFragmentOccurrence() {
            self.context.borrow_mut().walk_metrics.skipped_phase += 1;
            return;
        }
        let phase = paint_info.phase;
        // The SVG painter gates below all key on the source node kind; read
        // it once for this visit, as BoxFragmentPainter / the SVG painters
        // read their LayoutObject's type once per Paint call.
        let source_kind = fragment.paint.SourceKind();
        let owns_svg_paint_scope =
            crate::scoped_svg_paint_state::ScopedSVGPaintState::OwnsPaintScopeForKind(
                object,
                source_kind,
            );
        if owns_svg_paint_scope && phase != PaintPhase::kForeground {
            self.context.borrow_mut().walk_metrics.skipped_phase += 1;
            return;
        }
        if owns_svg_paint_scope
            && !crate::scoped_svg_paint_state::ScopedSVGPaintState::ShouldPaintForKind(
                object,
                source_kind,
                paint_info,
            )
        {
            return;
        }
        if crate::phase::ShouldSkipPhase(
            phase,
            object.needs_float_phase(),
            object.needs_outline_phase(),
            object.needs_overflow_controls_phase(),
        ) {
            self.context.borrow_mut().walk_metrics.skipped_phase += 1;
            return;
        }
        let is_svg_container =
            crate::svg_container_painter::SVGContainerPainter::IsContainerKind(source_kind);
        if is_svg_container
            && !crate::svg_container_painter::SVGContainerPainter::ShouldPaintSubtree(object)
        {
            return;
        }
        // SVGRootPainter::IsRoot: the source kind is kSvgRoot.
        let is_svg_root = source_kind == layout::input::NodeKind::kSvgRoot;
        if is_svg_root
            && !crate::svg_root_painter::SVGRootPainter::ShouldPaintSubtree(object, paint_info)
        {
            return;
        }
        // BoxFragmentPainter::PaintInternal owns child traversal in Chromium.
        // Rechrom keeps traversal in paint_walk, so mirror its ShouldPaint gate
        // here as well as at the recording entry; a culled box must suppress
        // the complete subtree, not merely its own display items.
        // SVGContainerPainter is the source owner for SVG container culling.
        // Its LayoutSVGModelObject visual rect is independent of the anonymous
        // zero-sized NG box fragment, so applying BoxFragmentPainter's CSS-box
        // filtered-ink cull afterwards would discard the admitted SVG subtree.
        // Chromium: svg_container_painter.cc:36-90 versus
        // box_fragment_painter.cc:615-641.
        if !is_svg_container && !BoxFragmentPainter::ShouldPaintFragment(object, paint_info) {
            return;
        }
        // The recording entry below repeats ShouldPaint only when this gate
        // did not already answer it for the same object and PaintInfo.
        let admitted = !is_svg_container;
        let _text_paint_timing = paint_record::paint_context::ScopedTextPaintAggregation::new(
            self.context,
            object,
            phase == PaintPhase::kForeground && !paint_info.IsRenderingResourceSubtree(),
        );
        let enter_node = !self.IsLayerObject(object) || paint_info.IsRenderingResourceSubtree();
        if !enter_node
            && matches!(
                phase,
                PaintPhase::kForeground | PaintPhase::kForcedColorsModeBackplate
            )
        {
            PaintGatedFragment(object, paint_info, admitted);
        }
        let paint_self_outside_clip = enter_node
            && object.applies_overflow_clip()
            && !object.is_paint_layer()
            && matches!(
                phase,
                PaintPhase::kBlockBackground
                    | PaintPhase::kDescendantBlockBackgroundsOnly
                    | PaintPhase::kOutline
                    | PaintPhase::kDescendantOutlinesOnly
            );
        if enter_node {
            if paint_self_outside_clip {
                // The box's own background is outside its overflow clip, but
                // it is still inside the object's transform, clip-path and
                // effect. Keep the same ordering as Blink's property-tree
                // conversion: outer geometry/effect, self paint, then the
                // escapable overflow clip used by descendants.
                let mut context = self.context.borrow_mut();
                context.BeginGeometry(object);
                context.BeginEffects(object);
                drop(context);
                PaintGatedFragment(object, paint_info, admitted);
                self.context.borrow_mut().BeginEscapableGeometry(object, None);
            } else {
                self.context.borrow_mut().BeginNode(object);
                PaintGatedFragment(object, paint_info, admitted);
            }
        }

        let is_text_combine = fragment.paint.IsTextCombine();
        let unscaled_text_combine_caret = phase == PaintPhase::kForeground
            && object.applies_transform()
            && is_text_combine
            && self.context.borrow().caret.is_some();
        if unscaled_text_combine_caret {
            self.context.borrow_mut().EndGeometry(object);
            {
                let _parent_properties =
                    crate::paint_context::ScopedPaintChunkProperties::text_combine_parent(
                        self.context,
                        object,
                        phase,
                    );
                BoxFragmentPainter::PaintFragmentBeforeChildren(object, paint_info);
            }
            self.context.borrow_mut().BeginGeometry(object);
        } else {
            BoxFragmentPainter::PaintFragmentBeforeChildren(object, paint_info);
        }

        // The children BoxFragmentPainter::PaintObject traverses: none while
        // the LayoutObject's child paint is blocked by a display lock
        // (PaintInfo::DescendantPaintingBlocked). The embedded child frame is
        // EmbeddedContentPainter's, painted after them.
        let blocked = fragment.paint.ChildPaintBlockedByDisplayLock();
        let children = match object.SourceCursor() {
            Some(cursor) if !blocked => LogicalPaintChildren(cursor),
            _ => Vec::new(),
        };
        let frame_root = self
            .frames
            .and_then(|frames| frames.LocalFrameRoot(fragment.paint.DisplayItemClientId()));
        let has_children = !children.is_empty() || frame_root.is_some();

        let needs_reference_filter_chunk = phase == PaintPhase::kForeground
            && ((is_svg_container
                && crate::svg_container_painter::SVGContainerPainter::NeedsReferenceFilterChunk(
                    object,
                ))
                || crate::scoped_svg_paint_state::ScopedSVGPaintState::NeedsReferenceFilterChunkForKind(
                    object,
                    source_kind,
                    paint_info,
                ));
        // BoxFragmentPainter::PaintObject enters ScopedBoxContentsPaintState
        // only for a box with children or items to traverse
        // (box_fragment_painter.cc:858-959, `!Children().empty() ||
        // HasItems()`); a leaf records nothing in the contents space. The
        // reference-filter chunk below still belongs to the contents state.
        let contents_paint_state = (has_children || needs_reference_filter_chunk).then(|| {
            paint_record::scoped_paint_state::ScopedBoxContentsPaintState::new(paint_info, object)
        });
        if needs_reference_filter_chunk {
            self.context.borrow_mut().EnsureChunk();
        }
        // Chromium's child/item traversal consumes the PaintInfo owned by
        // ScopedBoxContentsPaintState, not the border-box input PaintInfo. Keep
        // the adjusted contents CullRect alive across the child walk.
        let contents_paint_info = contents_paint_state
            .as_ref()
            .map_or(*paint_info, |state| *state.GetPaintInfo());
        // Chromium's PaintLineBoxes visits no ordinary line in the
        // block-background phases; only a block-in-inline line paints its
        // block background there (box_fragment_painter.cc:1050-1065,
        // PaintBoxDecorationBackgroundForBlockInInline). Atomic inlines in a
        // line paint all their phases from the foreground pass (PaintBoxItem
        // -> PaintFragment -> PaintAllPhasesAtomically,
        // box_fragment_painter.cc:615-641, 2034-2077), and inline boxes paint
        // their decorations in the foreground phase only.
        let skips_ordinary_lines =
            paint_model::paint_phase::ShouldPaintDescendantBlockBackgrounds(phase);
        // MathML operators suppress their inline children; read once per
        // visit, on the first fragment-item child.
        let mut suppresses_fragment_items: Option<bool> = None;
        // Every child paints with this object's contents cull rect, the
        // PaintInfo cull rect ScopedBoxContentsPaintState installed above.
        let child_cull_rect = if has_children { object.cull_rect() } else { None };

        for &child in &children {
            let child_object = child.Object(PaintWalkContext::new(child_cull_rect, None));
            // `!box_fragment_.HasSelfPaintingLayer()`: a child painting as its
            // own layer is painted by its PaintLayerPainter, except inside an
            // SVG resource subtree, which paints it atomically in place.
            if child_object.is_paint_layer() {
                if contents_paint_info.IsRenderingResourceSubtree() {
                    self.PaintAllPhasesAtomically(child_object, &contents_paint_info);
                }
                continue;
            }
            let child_fragment = child_object.Fragment().expect("painted child has a fragment");
            if skips_ordinary_lines && child_fragment.ViewIsLineOutsideBlockInInline() {
                continue;
            }
            // One read per child of its fragment-item type, shared by the
            // MathML operator test (ShouldPaintChildFragment) and the cull.
            let child_is_fragment_item = matches!(
                child_fragment.ViewKind(),
                FragmentKind::kLine | FragmentKind::kText | FragmentKind::kGeneratedText
            ) || child_fragment.ViewIsInlineBox();
            if child_is_fragment_item
                && *suppresses_fragment_items.get_or_insert_with(|| {
                    BoxFragmentPainter::SuppressesFragmentItemChildren(object)
                })
            {
                continue;
            }
            // Chromium's early child cull here belongs to FragmentItems. A
            // physical child may establish its own PaintOffsetTranslation;
            // its recursive ShouldPaintFragment maps the cull rect into that
            // property space before testing its ink overflow.
            if child_is_fragment_item
                && !BoxFragmentPainter::ChildIntersectsCullRect(child_object, &contents_paint_info)
            {
                continue;
            }
            // PhysicalFragment::IsPaintedAtomically of the box (atomic
            // inlines, flex/grid items, floats), read live as
            // BoxFragmentPainter::PaintFragment / Paint read it
            // (box_fragment_painter.cc:636-651).
            let is_atomic = child_fragment.ViewPaintedAtomically();
            let child_is_float = IsFloat(child_fragment);
            match crate::phase::ClassifyChildPhase(phase, child_is_float, is_atomic) {
                crate::phase::ChildPhaseAction::Skip => {}
                crate::phase::ChildPhaseAction::PaintAtomically => {
                    let mut atomic_info = contents_paint_info.ForDescendants();
                    if child_is_float {
                        atomic_info.phase = crate::phase::FloatPaintPhase(atomic_info.phase);
                    }
                    self.PaintAllPhasesAtomically(child_object, &atomic_info);
                }
                crate::phase::ChildPhaseAction::Recurse(child_phase) => {
                    let mut child_info = contents_paint_info.ForDescendants();
                    child_info.phase = child_phase;
                    self.PaintWithPhase(child_object, &child_info);
                }
            }
        }
        if let Some((frame_root, cross_origin_to_parent)) = frame_root {
            // EmbeddedContentPainter::PaintReplaced -> LocalFrameView::Paint
            // -> FramePainter::Paint: the child's entire PaintLayer tree
            // paints at the replaced foreground position using this same
            // PaintController/GraphicsContext. Properties already carry the
            // frame location, so no bitmap or extra offset is applied.
            if phase == PaintPhase::kForeground
                && !blocked
                && fragment.paint.style.visible()
                && !(paint_info.GetPaintFlags() & PaintFlag::kPrivacyPreserving != 0
                    && cross_origin_to_parent)
            {
                PaintLayerPainter::new(frame_root, self.context, self.frames)
                    .WithFlags(paint_info.GetPaintFlags())
                    .Paint();
            }
        }
        drop(contents_paint_state);
        // Blink's overflow controls use border-box properties, outside their
        // owner's OverflowClip/InnerBorderRadiusClip (paint property builder,
        // UpdateOverflowControlEffects). Keep the flat recording in that same
        // space; an extra fractional AA overflow clip is not redundant.
        let paint_controls = phase == PaintPhase::kOverlayOverflowControls
            && fragment.ViewScrollbarsPaintAsOverlay().is_some();
        if paint_controls {
            self.context.borrow_mut().EndEscapableGeometry(object);
        }
        let restore_text_combine = enter_node && object.applies_transform() && is_text_combine;
        if restore_text_combine {
            // BoxFragmentPainter restores the text-combine CTM before
            // TextCombinePainter draws the parent's decorations/emphasis.
            self.context.borrow_mut().EndGeometry(object);
        }
        BoxFragmentPainter::PaintFragmentAfterChildren(object, paint_info);
        if paint_controls {
            self.context.borrow_mut().BeginEscapableGeometry(object, None);
        }
        if enter_node && !restore_text_combine {
            if owns_svg_paint_scope {
                debug_assert_eq!(phase, PaintPhase::kForeground);
                self.context.borrow_mut().EndEscapableGeometry(object);
                crate::scoped_svg_paint_state::ScopedSVGPaintState::FinishEffectsAfterContents(
                    object,
                    self.context,
                );
                if is_svg_container {
                    crate::svg_container_painter::SVGContainerPainter::PaintOutlineAfterEffects(
                        object,
                        self.context,
                    );
                } else {
                    crate::svg_model_object_painter::SVGModelObjectPainter::PaintOutline(
                        object,
                        self.context,
                    );
                }
                self.context.borrow_mut().EndGeometry(object);
            } else {
                self.context.borrow_mut().EndNode(object);
            }
        }
    }

    // Source: core/paint/box_fragment_painter.cc:1805-1836.
    fn PaintAllPhasesAtomically<'x, 'p>(
        &self,
        object: PaintObject<'x, 'static>,
        paint_info: &PaintInfo<'p, 'o>,
    ) {
        if matches!(
            paint_info.phase,
            PaintPhase::kSelectionDragImage | PaintPhase::kTextClip
        ) {
            self.PaintWithPhase(object, paint_info);
            return;
        }
        if paint_info.phase != PaintPhase::kForeground {
            return;
        }
        let mut local_info = *paint_info;
        for phase in crate::phase::ATOMIC_PHASES {
            local_info.phase = phase;
            self.PaintWithPhase(object, &local_info);
        }
    }

    // Source: core/paint/paint_layer_painter.cc:692-698. This enters the
    // reparented scroller's normal layer/property scope but records only its
    // overlay overflow-control phase.
    fn PaintOverlayOverflowControls(&self) {
        let object = self.LayerObject();
        let fragment = object.Fragment().expect("painted layer has a fragment");
        let _effectively_invisible = PaintedOutputInvisible(fragment).then(|| {
            crate::scoped_effectively_invisible::ScopedEffectivelyInvisible::new(self.context)
        });
        let _layer_chunk_properties = crate::paint_context::ScopedPaintChunkProperties::layer(
            self.context,
            object,
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        for index in 0..self.inherited_geometry.len() {
            let ancestor = self.InheritedObject(index);
            self.context.borrow_mut().BeginGeometry(ancestor);
            self.context.borrow_mut().BeginEffects(ancestor);
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(ancestor, None);
        }
        self.context.borrow_mut().BeginGeometry(object);
        self.context.borrow_mut().BeginEffects(object);
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(object, None);
        self.PaintWithPhase(object, &self.PaintInfo(PaintPhase::kOverlayOverflowControls));
        self.context.borrow_mut().EndEscapableGeometry(object);
        self.context.borrow_mut().EndFilter(object);
        self.context.borrow_mut().EndCssClip(object);
        self.context.borrow_mut().EndMaskClip(object);
        self.context.borrow_mut().EndEffectsAfterFilter(object);
        self.context.borrow_mut().EndGeometry(object);
        for index in (0..self.inherited_geometry.len()).rev() {
            let ancestor = self.InheritedObject(index);
            self.context.borrow_mut().EndEscapableGeometry(ancestor);
            self.context.borrow_mut().EndEffects(ancestor);
            self.context.borrow_mut().EndGeometry(ancestor);
        }
    }

    /// Source: paint_layer_painter.cc:623-690 (PaintChildren). `root` is this
    /// painter's PaintLayer, or a child PaintLayer without an occurrence
    /// whose children this painter paints in its place (see
    /// PaintLayerWithoutOccurrence).
    fn PaintChildren(
        &self,
        root: &PaintLayer,
        children_to_visit: PaintLayerIteration,
        children: &ChildLayers<'_, 'a>,
    ) -> PaintResult {
        let mut result = PaintResult::kFullyPainted;
        let mut iterator = PaintLayerPaintOrderIterator::new(root, children_to_visit);
        while let Some(child) = iterator.Next() {
            // A normal-flow child outside every z-order list paints nothing:
            // no visible content, self-painting descendant or view-transition
            // name (PaintLayer::IsZOrderListVisible, paint_layer.cc:573-575).
            // The flat replay stream would still record its empty property
            // scopes, so it is not entered.
            if !child.GetLayoutObject().IsStacked() && !child.IsZOrderListVisible() {
                continue;
            }
            let mut painted = 0;
            for occurrence in children.occurrences.Occurrences(child) {
                painted += 1;
                result.Include(self.PaintChildOccurrence(&occurrence));
            }
            if painted == 0 {
                result.Include(self.PaintLayerWithoutOccurrence(child, children));
            }

            // Source: paint_layer_painter.cc:671-685. A target can carry more
            // than one scroller's controls, in collection order.
            if let Some(scrollers) = iterator.LayersPaintingOverlayOverflowControlsAfter(child) {
                for scroller in scrollers.iter() {
                    // Members of the live stacking node's map.
                    let scroller = unsafe { &*scroller.Get() };
                    debug_assert!(scroller.NeedsReorderOverlayOverflowControls());
                    for occurrence in children.occurrences.Occurrences(scroller) {
                        let escapes_current_geometry = occurrence.EscapesCurrentGeometry();
                        if escapes_current_geometry {
                            self.SuspendGeometryForEscapingChild();
                        }
                        self.ChildPainter(&occurrence).PaintOverlayOverflowControls();
                        // The scroller's own Paint ran earlier in this list
                        // and wrote its PaintLayer::PreviousPaintResult.
                        result.Include(scroller.PreviousPaintResult());
                        if escapes_current_geometry {
                            self.ResumeGeometryAfterEscapingChild();
                        }
                    }
                }
            }
        }
        result
    }

    /// PaintLayerPainter(*child).Paint() for one occurrence of the child
    /// layer, re-entering the property scopes between this layer and it.
    fn PaintChildOccurrence(&self, occurrence: &LayerOccurrence<'_, 'a>) -> PaintResult {
        let escapes_current_geometry = occurrence.EscapesCurrentGeometry();
        if escapes_current_geometry {
            self.SuspendGeometryForEscapingChild();
        }
        let result = self.ChildPainter(occurrence).Paint();
        if escapes_current_geometry {
            self.ResumeGeometryAfterEscapingChild();
        }
        result
    }

    /// PaintLayerPainter(*layer).Paint() for a PaintLayer without a
    /// fragment occurrence painting as a layer: a non-self-painting layer,
    /// or an inline-level layer whose content Rechrom paints with its line
    /// box. Such a layer records no content of its own here, so Paint
    /// reduces to its early returns and its two PaintChildren calls
    /// (paint_layer_painter.cc:304-308,481-483,499-501,640-650). Its
    /// descendants' occurrences lie below this painter's fragment, so they
    /// are painted from here with their recorded property ancestors.
    fn PaintLayerWithoutOccurrence(&self, layer: &PaintLayer, children: &ChildLayers<'_, 'a>) -> PaintResult {
        if !layer.HasSelfPaintingLayerDescendant()
            || layer.GetLayoutObject().ChildPaintBlockedByDisplayLock()
        {
            return PaintResult::kFullyPainted;
        }
        let result = self.PaintChildren(layer, PaintLayerIteration::kAllChildren, children);
        if self.paint_flags & PaintFlag::kOmitCompositingInfo == 0 {
            layer.SetPreviousPaintResult(result);
        }
        result
    }
}

// Source SVGObjectPainter::PaintResourceSubtree (cc:93-102) and explicit
// SVGContainerPainter(marker) dispatch (svg_shape_painter.cc:282-323): the
// resource LayoutObject's subtree, reached through Layout, painted with the
// resource PaintInfo.
fn PaintSvgResourceSubtree(object: PaintObject<'_, 'static>, info: &PaintInfo<'_, '_>) {
    let cursor = object
        .SourceCursor()
        .expect("SVG resource subtree root has a resident fragment");
    let painter = PaintLayerPainter::new(cursor, info.context, None).WithFlags(info.GetPaintFlags());
    painter.PaintWithPhase(painter.LayerObject(), info);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pre_paint_tree_walk::PaintTreeNode;
    use paint_input_types::{FragmentPaintRect, PaintResources};
    use paint_input_types::layout_input::{Offset, Size};
    use paint_model::geometry_mapper::TranslationTransform;
    use paint_model::paint_property_tree::TransformPaintPropertyNode;
    use paint_record::{DisplayItem, PaintRecording};
    use std::sync::Arc;

    fn cull_layer(
        self_visual_rect: PaintRect,
        contents_visual_rect: PaintRect,
        self_cull_rect: PaintRect,
        contents_cull_rect: PaintRect,
    ) -> PaintTreeNode<'static> {
        let rect = |rect: PaintRect| FragmentPaintRect {
            offset: Offset {
                x: rect.x,
                y: rect.y,
            },
            size: Size {
                width: rect.width,
                height: rect.height,
            },
        };
        let mut fragment = FragmentPaintData::default();
        fragment.paint.is_layout_box = true;
        fragment.paint.paint_layer_first_fragment_visual_rect = Some(rect(self_visual_rect));
        fragment.paint.paint_layer_contents_visual_rect = Some(rect(contents_visual_rect));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.self_cull_rect = Some(self_cull_rect);
        node.cull_rect = Some(contents_cull_rect);
        node
    }

    #[test]
    fn cull_admission_matches_self_and_contents_containment() {
        let contained = cull_layer(
            PaintRect {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 20.0,
            },
            PaintRect {
                x: 15.0,
                y: 15.0,
                width: 10.0,
                height: 10.0,
            },
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
        );
        assert_eq!(
            CullAdmission(PaintObject::Fixture(&contained), true, false),
            (PaintResult::kFullyPainted, true, false)
        );

        let clipped = cull_layer(
            PaintRect {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 20.0,
            },
            PaintRect {
                x: 15.0,
                y: 15.0,
                width: 100.0,
                height: 10.0,
            },
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 100.0,
            },
        );
        assert_eq!(
            CullAdmission(PaintObject::Fixture(&clipped), true, false),
            (PaintResult::kMayBeClippedByCullRect, true, false)
        );
    }

    #[test]
    fn cull_admission_compares_local_property_space_rectangles() {
        let local_visual_rect = PaintRect {
            x: 0.0,
            y: 3.0,
            width: 100.0,
            height: 20.0,
        };
        let local_cull_rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 52.0,
        };
        let mut layer = cull_layer(
            local_visual_rect,
            local_visual_rect,
            local_cull_rect,
            local_cull_rect,
        );

        // The property translation owns the integral offset, leaving a zero
        // residual FragmentData paint offset. The cull and resulting visual
        // rectangles therefore remain in this local property space.
        let translated = |parent: Arc<TransformPaintPropertyNode>, id| {
            Arc::new(TransformPaintPropertyNode {
                lifecycle: Default::default(),
                id,
                parent: Some(parent.clone()),
                matrix: TranslationTransform(0.0, 52.0),
                ..(*parent).clone()
            })
        };
        layer.properties.nodes.transform =
            translated(layer.properties.nodes.transform.clone(), 101);
        layer.contents_properties.nodes.transform =
            translated(layer.contents_properties.nodes.transform.clone(), 102);

        assert_eq!(
            CullAdmission(PaintObject::Fixture(&layer), true, false),
            (PaintResult::kFullyPainted, true, false)
        );
    }

    #[test]
    fn cull_admission_applies_current_fragment_paint_offset() {
        let local_visual_rect = PaintRect {
            x: 0.0,
            y: 3.0,
            width: 100.0,
            height: 20.0,
        };
        let property_space_cull = PaintRect {
            x: 0.0,
            y: 52.0,
            width: 320.0,
            height: 52.0,
        };
        let mut layer = cull_layer(
            local_visual_rect,
            local_visual_rect,
            property_space_cull,
            property_space_cull,
        );
        layer.paint_offset = Offset { x: 0.0, y: 52.25 };

        assert_eq!(
            CullAdmission(PaintObject::Fixture(&layer), true, false),
            (PaintResult::kFullyPainted, true, false)
        );
    }

    #[test]
    fn known_padding_clip_returns_before_paint_when_both_rects_are_outside() {
        let layer = cull_layer(
            PaintRect {
                x: 100.0,
                y: 100.0,
                width: 20.0,
                height: 20.0,
            },
            PaintRect {
                x: 100.0,
                y: 100.0,
                width: 20.0,
                height: 20.0,
            },
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
        );
        assert_eq!(
            CullAdmission(PaintObject::Fixture(&layer), true, true),
            (PaintResult::kMayBeClippedByCullRect, false, true)
        );
    }

    fn node_with_opacity(opacity: f32) -> PaintTreeNode<'static> {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.SetOpacityForTesting(opacity);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node
    }

    #[test]
    fn near_zero_opacity_is_effectively_invisible_unless_it_may_change() {
        let mut node = node_with_opacity(0.0003);
        assert!(PaintedOutputInvisible(node.fragment.as_deref().unwrap()));

        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .style
            .will_change_opacity = true;
        assert!(!PaintedOutputInvisible(node.fragment.as_deref().unwrap()));

        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .style
            .will_change_opacity = false;
        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .style
            .active_opacity_animation = true;
        assert!(!PaintedOutputInvisible(node.fragment.as_deref().unwrap()));
    }

    #[test]
    fn ancestor_position_visibility_hides_the_layer() {
        assert!(HiddenForPositionVisibility(true, false, true));
        assert!(!HiddenForPositionVisibility(true, false, false));
        assert!(HiddenForPositionVisibility(true, true, false));
    }

    #[test]
    fn layout_dirty_and_fragmentless_layers_follow_source_early_returns() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.needs_layout = true;
        assert!(ShouldSkipForLayout(&fragment));

        fragment.paint.child_layout_blocked_by_display_lock = true;
        assert!(!ShouldSkipForLayout(&fragment));

        fragment.paint.needs_layout = false;
        fragment.paint.is_fragment_less_box = true;
        assert!(ShouldSkipForLayout(&fragment));
    }

    #[test]
    fn subsequence_admission_requires_source_support() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.SetPointerEventsNoneForTesting(true);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let supports_subsequence_caching = true;
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        context.borrow_mut().update_chunk_id_and_properties(
            PaintObject::Fixture(&node),
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        context.borrow_mut().EnsureChunk();

        assert!(ShouldCreateSubsequence(
            PaintObject::Fixture(&node),
            None,
            supports_subsequence_caching,
            &context,
            PaintFlag::kNoFlag
        ));
        context.borrow_mut().BeginSkippingCache();
        assert!(!ShouldCreateSubsequence(
            PaintObject::Fixture(&node),
            None,
            supports_subsequence_caching,
            &context,
            PaintFlag::kNoFlag
        ));
        context.borrow_mut().EndSkippingCache();
        let supports_subsequence_caching = false;
        assert!(!ShouldCreateSubsequence(
            PaintObject::Fixture(&node),
            None,
            supports_subsequence_caching,
            &context,
            PaintFlag::kNoFlag
        ));
    }

    #[test]
    fn subsequence_admission_separates_transparent_hit_test_content() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.SetPointerEventsNoneForTesting(true);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let supports_subsequence_caching = true;
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        context.borrow_mut().update_chunk_id_and_properties(
            PaintObject::Fixture(&node),
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        context.borrow_mut().EnsureChunk();

        assert!(ShouldCreateSubsequence(
            PaintObject::Fixture(&node),
            None,
            supports_subsequence_caching,
            &context,
            PaintFlag::kNoFlag
        ));
    }

    #[test]
    fn subsequence_admission_separates_sparse_merged_bounds() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.SetPointerEventsNoneForTesting(true);
        fragment.resources = Some(std::sync::Arc::new(PaintResources::default()));
        let mut sparse_visual_rect = FragmentPaintRect::default();
        sparse_visual_rect.offset.x = 20.0;
        sparse_visual_rect.size.width = 10.0;
        sparse_visual_rect.size.height = 10.0;
        fragment.paint.paint_layer_first_fragment_visual_rect = Some(sparse_visual_rect);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let supports_subsequence_caching = true;
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        context.borrow_mut().update_chunk_id_and_properties(
            PaintObject::Fixture(&node),
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        context.borrow_mut().Append(
            DisplayItem {
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                },
                ..Default::default()
            },
            PaintObject::Fixture(&node),
        );

        assert!(!ShouldCreateSubsequence(
            PaintObject::Fixture(&node),
            None,
            supports_subsequence_caching,
            &context,
            PaintFlag::kNoFlag
        ));
        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .paint_layer_first_fragment_visual_rect
            .as_mut()
            .unwrap()
            .offset
            .x = 2000.0;
        assert!(ShouldCreateSubsequence(
            PaintObject::Fixture(&node),
            None,
            supports_subsequence_caching,
            &context,
            PaintFlag::kNoFlag
        ));
    }
}

