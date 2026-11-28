#![allow(non_snake_case)]

use layout::fragment::ResidentBoxGeometry as _;
use std::cell::RefCell;

use layout::fragment::FragmentKind;
use layout::input::FloatSide;
use layout::views::paint_layer::PaintLayer;
use paint_input_types::paint_input::PaintFilterType;

use crate::box_fragment_painter::BoxFragmentPainter;
use crate::geometry_mapper::{MapRectFromRoot, MapRectToRoot};
use crate::ignore_paint_timing_scope::IgnorePaintTimingScope;
use crate::paint_context::PaintContext;
use crate::paint_info::PaintInfo;
use crate::paint_layer_paint_order_iterator::{
    PaintLayerIteration, PaintLayerPaintOrderEntry, PaintLayerPaintOrderIterator,
    ResidentLayerNodes, SourcePaintLayer,
};
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintPhase;
use crate::PaintRect;
use paint_model::paint_flags::{PaintFlag, PaintFlags};
use paint_model::PaintResult;
use paint_record::{
    box_fragment_painter::GetHitTestOpaqueness, paint_chunker::DisplayItemClientInfo,
    paint_engine::PaintHitTestOpaqueness,
    paint_object::{PaintObject, PaintWalkContext},
    scoped_paint_state::ContentsVisualRect,
    SubsequenceRecorder,
};

fn IsFloat(node: &PaintTreeNode<'_>) -> bool {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    fragment.paint.HasSource() && fragment.paint.Floating() != FloatSide::kNone
}

// Source: core/paint/paint_layer_painter.cc:167-190.
fn PaintedOutputInvisible(node: &PaintTreeNode<'_>) -> bool {
    let style = &node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment")
        .paint
        .style;
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
// scope, so the immutable Paint walk does not rebuild a mutable PaintLayer
// parent chain.
fn HiddenForPositionVisibility(node: &PaintTreeNode<'_>) -> bool {
    node.paint_layer_facts_are_authoritative
        && (node.invisible_for_position_visibility
            || node.has_ancestor_invisible_for_position_visibility)
}

// Source: core/paint/paint_layer_painter.cc:289-295,300-302.
fn ShouldSkipForLayout(fragment: &layout::paint::FragmentPaintData) -> bool {
    (fragment.paint.NeedsLayout() && !fragment.paint.ChildLayoutBlockedByDisplayLock())
        || fragment.paint.IsFragmentLessBox()
}

fn HasReferenceFilter(node: &PaintTreeNode<'_>) -> bool {
    let style = &node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment")
        .paint
        .style;
    let filters = style.filters();
    let backdrop_filters = style.backdrop_filters();
    filters
        .iter()
        .chain(backdrop_filters.iter())
        .any(|operation| operation.r#type == PaintFilterType::kReference)
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
/// The source boundary supplies LayoutBox's stitched contents overflow. The
/// resident property tree applies ReplacedContentTransform because it is the
/// authoritative transform tree consumed by the rest of Rechrom Paint.
fn CullAdmission(
    layer: &PaintTreeNode<'_>,
    mut should_paint_content: bool,
) -> (PaintResult, bool, bool) {
    let fragment = layer
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if fragment.paint.PaintLayerIsFragmented()
        || (fragment.paint.IsLayoutView() && fragment.paint.IsPrinting())
        || paint_model::property_cell::with_access(|access| layer.properties.nodes.effect.HasCanvasChildState(access))
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
        layer.self_cull_rect,
    ) else {
        // Hand-built fragments and incomplete lifecycle input retain the
        // pre-existing admission. Chromium layers always export both values.
        return (PaintResult::kFullyPainted, should_paint_content, false);
    };
    let mut visual_rect = FragmentRect(visual_rect);
    // Source: PaintLayerPainter::FirstFragmentVisualRect(). Layout exports
    // the local virtual overflow; apply this PrePaint pass's FragmentData
    // paint offset here, after property-tree construction, as Chromium does.
    visual_rect.x += layer.paint_offset.x;
    visual_rect.y += layer.paint_offset.y;
    let self_intersects = Intersects(self_cull_rect, visual_rect);
    let mut result = if self_cull_rect.contains(visual_rect) {
        PaintResult::kFullyPainted
    } else {
        PaintResult::kMayBeClippedByCullRect
    };

    let contents_intersects = if fragment.paint.IsLayoutBox() {
        if let (Some(contents), Some(contents_cull_rect)) =
            (ContentsVisualRect(PaintObject::new(layer)), layer.cull_rect)
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
        if layer.known_to_clip_subtree_to_padding_box {
            return (PaintResult::kMayBeClippedByCullRect, false, true);
        }
        should_paint_content = false;
    }
    (result, should_paint_content, false)
}

/// BoxFragmentPainter::PaintFragment, skipping its ShouldPaint test when the
/// walk's own gate already admitted this node with this PaintInfo.
fn PaintGatedFragment(object: PaintObject<'_, '_>, paint_info: &PaintInfo<'_, '_>, admitted: bool) {
    if admitted {
        BoxFragmentPainter::PaintFragmentAdmitted(object, paint_info);
    } else {
        BoxFragmentPainter::PaintFragment(object, paint_info);
    }
}

fn HasEscapingChildLayer(parent: &PaintTreeNode<'_>) -> bool {
    for child_owner in &parent.children {
        let child = child_owner.as_ref();
        if child.is_embedded_local_frame_root {
            continue;
        }
        if child.is_paint_layer {
            if child.escapes_ancestor_geometry {
                return true;
            }
            if child.is_stacking_context {
                continue;
            }
        }
        if HasEscapingChildLayer(child) {
            return true;
        }
    }
    false
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
/// without creating proxy scopes.
/// Special recording preserves source's kOmitCompositingInfo cache exclusion.
fn ShouldCreateSubsequence(
    layer: &PaintTreeNode<'_>,
    context: &RefCell<PaintContext<'_>>,
    paint_flags: PaintFlags,
) -> bool {
    let fragment = layer
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if fragment.paint.IsPrinting()
        || paint_flags & PaintFlag::kOmitCompositingInfo != 0
        || context.borrow().IsSkippingCache()
        || !layer.supports_subsequence_caching
    {
        return false;
    }
    if context.borrow().NumNewChunks() > 0
        && !context
            .borrow()
            .LastChunkPropertiesSameNodes(&layer.properties.nodes)
    {
        return true;
    }
    if SourcePaintLayer(layer).is_some_and(HasTwoDescendantPaintLayers) {
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
            let visual_rect = PaintRect {
                x: visual_rect.offset.x,
                y: visual_rect.offset.y,
                width: visual_rect.size.width,
                height: visual_rect.size.height,
            };
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

pub struct PaintLayerPainter<'g, 'n, 'f, 'c, 'o> {
    layer: &'n PaintTreeNode<'f>,
    /// Chromium's `paint_layer_`: the Layout-owned PaintLayer of the
    /// resident node's LayoutObject. None for hand-built fixture nodes.
    paint_layer: Option<&'n PaintLayer>,
    context: &'c RefCell<PaintContext<'o>>,
    /// The property ancestors a child layer re-enters. A child painter
    /// borrows its paint-order entry's list (PaintLayerPainter is a stack
    /// object over the parent's z-order list in Chromium) instead of copying
    /// it for every child layer.
    inherited_geometry: std::borrow::Cow<'g, [&'n PaintTreeNode<'f>]>,
    paint_flags: PaintFlags,
}

impl<'g, 'n, 'f, 'c, 'o> PaintLayerPainter<'g, 'n, 'f, 'c, 'o> {
    pub fn new(
        layer: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
        inherited_geometry: Vec<&'n PaintTreeNode<'f>>,
    ) -> Self {
        Self {
            layer,
            paint_layer: SourcePaintLayer(layer),
            context,
            inherited_geometry: std::borrow::Cow::Owned(inherited_geometry),
            paint_flags: PaintFlag::kNoFlag,
        }
    }

    fn with_inherited_geometry(
        layer: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
        inherited_geometry: &'g [&'n PaintTreeNode<'f>],
    ) -> Self {
        Self {
            layer,
            paint_layer: SourcePaintLayer(layer),
            context,
            inherited_geometry: std::borrow::Cow::Borrowed(inherited_geometry),
            paint_flags: PaintFlag::kNoFlag,
        }
    }

    /// PaintLayer::SetPreviousPaintResult (paint_layer.cc:2827-2832), written
    /// by PaintLayerPainter::Paint. The node's cell stays the in-flight value
    /// ScopedBoxContentsPaintState downgrades and the remaining node readers
    /// see until the PrePaint node tree is removed. The source skips the
    /// PaintLayer write while CullRectUpdater::IsOverridingCullRects(); in
    /// Rechrom only the special recordings (drag images, resource subtrees)
    /// override cull rects, and they all paint with kOmitCompositingInfo.
    fn SetPreviousPaintResult(&self, result: PaintResult) {
        self.layer.previous_paint_result.set(result);
        if self.paint_flags & PaintFlag::kOmitCompositingInfo == 0 {
            if let Some(paint_layer) = self.paint_layer {
                paint_layer.SetPreviousPaintResult(result);
            }
        }
    }

    /// PaintLayer::NeedsReorderOverlayOverflowControls (paint_layer.h:
    /// 427-431), set by the owning PaintLayerStackingNode's rebuild. The
    /// node cell is not written here: its one remaining reader, the resident
    /// hit-test cursor, pairs it with the overlay-control map of its own
    /// collector (EnsureResidentHitTestLayerOrder), which writes it.
    fn NeedsReorderOverlayOverflowControls(&self) -> bool {
        self.paint_layer
            .is_some_and(PaintLayer::NeedsReorderOverlayOverflowControls)
    }

    /// PaintLayer::NeedsPaintPhaseFloat / NeedsPaintPhaseDescendantOutlines
    /// (paint_layer.h:490-507), set by PaintInvalidator::UpdatePaintingLayer.
    /// A fixture node without a PaintLayer keeps its aggregated node flags.
    fn NeedsPaintPhaseFloat(&self) -> bool {
        self.paint_layer
            .map_or(self.layer.needs_float_phase, PaintLayer::NeedsPaintPhaseFloat)
    }

    fn NeedsPaintPhaseDescendantOutlines(&self) -> bool {
        self.paint_layer.map_or(
            self.layer.needs_outline_phase,
            PaintLayer::NeedsPaintPhaseDescendantOutlines,
        )
    }

    /// PaintLayerPainter::Paint(context, paint_flags), source.cc:289-333.
    pub fn WithFlags(mut self, flags: PaintFlags) -> Self {
        self.paint_flags = flags;
        self
    }
    fn PaintInfo(&self, phase: PaintPhase) -> PaintInfo<'c, 'o> {
        PaintInfo::new_with_flags(self.context, phase, false, self.paint_flags)
    }

    /// The walk context of this painter's layer object: the in-flight
    /// PaintLayer::PreviousPaintResult this painter seeds and reads, and no
    /// inherited cull rect, since a PaintLayer object paints with its
    /// FragmentData cull rects (CullRectUpdater). A painted object without a
    /// Layout PaintLayer (a masked SVG child, a resource root, a fixture)
    /// enters with the PaintInfo cull rect PrePaint gave it.
    fn LayerWalk(&self) -> PaintWalkContext<'n> {
        let inherited_cull_rect = if self.paint_layer.is_some() {
            None
        } else {
            self.layer.cull_rect
        };
        PaintWalkContext::new(inherited_cull_rect, Some(&self.layer.previous_paint_result))
    }

    /// The painted object of this painter's layer, built from its fragment
    /// cursor, FragmentData and the walk context.
    fn LayerObject(&self) -> PaintObject<'n, 'f> {
        PaintObject::Walked(self.layer, self.LayerWalk())
    }

    fn UsesInheritedGeometry(&self, ancestor: &PaintTreeNode<'f>) -> bool {
        if !self.layer.escapes_ancestor_geometry {
            return true;
        }
        let logical_position = self
            .inherited_geometry
            .iter()
            .position(|candidate| std::ptr::eq(*candidate, ancestor));
        let Some(logical_position) = logical_position else {
            return false;
        };
        self.inherited_geometry[logical_position..]
            .iter()
            .any(|candidate| {
                let pointer = *candidate as *const PaintTreeNode<'f>;
                self.layer.physical_ancestors.contains(&pointer)
            })
    }

    fn InheritedClipInLayerSpace(&self, ancestor: &PaintTreeNode<'f>) -> Option<PaintRect> {
        let local_clip = ancestor.local_clip?;
        let Some(root_clip) = MapRectToRoot(local_clip, &ancestor.transforms) else {
            return Some(PaintRect::default());
        };
        Some(MapRectFromRoot(root_clip, &self.layer.transforms).unwrap_or_default())
    }

    /// PaintLayerPainter::Paint, paint_layer_painter.cc:287-596.
    pub fn Paint(&self) -> PaintResult {
        self.context.borrow_mut().walk_metrics.layer_visits += 1;
        // Source: core/paint/paint_layer_painter.cc:289-295,300-302.
        let fragment = self
            .layer
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if ShouldSkipForLayout(fragment) {
            self.context.borrow_mut().walk_metrics.skipped_hidden += 1;
            return PaintResult::kFullyPainted;
        }
        // Source: core/paint/paint_layer_painter.cc:304-308. A layer with no
        // content may still have self-painting descendants; fragment-hidden is
        // handled by the fragment painter and must not stand in for these
        // PaintLayer aggregates.
        if self.layer.paint_layer_facts_are_authoritative
            && !self.layer.is_self_painting_layer
            && !self.layer.has_self_painting_layer_descendant
        {
            self.context.borrow_mut().walk_metrics.skipped_hidden += 1;
            return PaintResult::kFullyPainted;
        }
        if HiddenForPositionVisibility(self.layer) {
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
        let should_paint_content = !self.layer.paint_layer_facts_are_authoritative
            || (self.layer.has_visible_content
                && !self.layer.is_under_svg_hidden_container
                && self.layer.is_self_painting_layer);
        let (mut result, should_paint_content, clipped_out) =
            CullAdmission(self.layer, should_paint_content);
        if clipped_out {
            self.SetPreviousPaintResult(result);
            return result;
        }
        // ScopedBoxContentsPaintState may downgrade this while phases paint.
        // Seed it with this traversal's result rather than the previous frame.
        self.layer.previous_paint_result.set(result);
        // Source: paint_layer_painter.cc:412-424. The scope is declared before
        // all paint-state scopes so its Drop closes the subsequence last.
        let subsequence_recorder = if should_paint_content
            && ShouldCreateSubsequence(self.layer, self.context, self.paint_flags)
        {
            let fragment = self
                .layer
                .fragment
                .as_deref()
                .expect("subsequence PaintLayer fragment");
            let client = DisplayItemClientInfo::from_paint_layer(fragment)
                .expect("subsequence requires a source PaintLayer client");
            let debug = self.context.borrow().DebugInfoEnabled().then(|| {
                (
                    fragment.paint.DisplayItemClientOwnerNodeId(),
                    fragment.paint.DisplayItemClientDebugName(),
                )
            });
            self.context.borrow_mut().walk_metrics.subsequence_layers += 1;
            if self.layer.SelfOrDescendantNeedsRepaint() {
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
        let _effectively_invisible = PaintedOutputInvisible(self.layer).then(|| {
            crate::scoped_effectively_invisible::ScopedEffectivelyInvisible::new(self.context)
        });
        let _layer_chunk_properties = should_paint_content.then(|| {
            crate::paint_context::ScopedPaintChunkProperties::layer(
                self.context,
                self.LayerObject(),
                crate::display_item_id::DisplayItemIdType::kLayerChunk,
            )
        });
        // Source: core/paint/paint_layer_painter.cc:457-468. Reference filters
        // must retain a property-bearing chunk even if this layer records no
        // drawing. Canvas-child forcing remains a separate unsupported branch.
        if should_paint_content
            && (paint_model::property_cell::with_access(|access| {
                self.layer.properties.nodes.effect.HasCanvasChildState(access)
            })
                || HasReferenceFilter(self.layer))
        {
            self.context.borrow_mut().EnsureChunk();
        }
        // Every consumer of this flag iterates `inherited_geometry`, so the
        // recursive descendant scan is only needed when there is inherited
        // geometry to re-enter; the root and most layers have none.
        let effect_contains_escaping_child = !self.inherited_geometry.is_empty()
            && !self.layer.escapes_ancestor_geometry
            && HasEscapingChildLayer(self.layer);
        for ancestor in self.inherited_geometry.iter() {
            self.context.borrow_mut().BeginGeometry(PaintObject::new(ancestor));
            self.context.borrow_mut().BeginEffects(PaintObject::new(ancestor));
            if !effect_contains_escaping_child && self.UsesInheritedGeometry(ancestor) {
                self.context
                    .borrow_mut()
                    .BeginEscapableGeometry(PaintObject::new(ancestor), None);
            }
        }
        self.context.borrow_mut().BeginGeometry(self.LayerObject());
        self.context.borrow_mut().BeginEffects(self.LayerObject());
        if effect_contains_escaping_child {
            for ancestor in self.inherited_geometry.iter() {
                if self.UsesInheritedGeometry(ancestor) {
                    self.context
                        .borrow_mut()
                        .BeginEscapableGeometry(PaintObject::new(ancestor), self.InheritedClipInLayerSpace(ancestor));
                }
            }
        }

        if should_paint_content && !selection_drag_image_only {
            BoxFragmentPainter::PaintFragment(
                self.LayerObject(),
                &self.PaintInfo(PaintPhase::kSelfBlockBackgroundOnly),
            );
        }
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(self.LayerObject(), None);
        // Source: paint_layer_painter.cc:640-650. PaintChildren returns early
        // without self-painting descendants or under a display lock. The
        // resident occurrences of the child layers are looked up in one map
        // shared by both PaintChildren calls of this layer.
        let child_layers = match self.paint_layer {
            Some(paint_layer)
                if self.layer.has_self_painting_layer_descendant
                    && !self.layer.child_paint_blocked_by_display_lock =>
            {
                Some((
                    paint_layer,
                    ResidentLayerNodes::Collect(self.layer, paint_layer.StackingNode().is_some()),
                ))
            }
            _ => None,
        };
        if let Some((paint_layer, nodes)) = &child_layers {
            result.Include(self.PaintChildren(
                paint_layer,
                PaintLayerIteration::kNegativeZOrderChildren,
                nodes,
            ));
        }
        if should_paint_content {
            let _foreground_properties = crate::paint_context::ScopedPaintChunkProperties::layer(
                self.context,
                self.LayerObject(),
                crate::display_item_id::DisplayItemIdType::kLayerChunkForeground,
            );
            if selection_drag_image_only {
                self.PaintWithPhase(self.layer, self.LayerWalk(), &self.PaintInfo(PaintPhase::kSelectionDragImage));
            } else {
                self.PaintForegroundPhases();
            }
        }
        // Blink paints the self outline with LocalBorderBoxProperties, outside
        // ScopedBoxContentsPaintState (box_fragment_painter.cc:759-799). Mirror
        // that scope in the flat recording too; children still need this layer's
        // overflow clip when their own paint phases resume below.
        self.context.borrow_mut().EndEscapableGeometry(self.LayerObject());
        if self.layer.is_self_painting_layer && self.layer.paints_outline {
            BoxFragmentPainter::PaintFragment(
                self.LayerObject(),
                &self.PaintInfo(PaintPhase::kSelfOutlineOnly),
            );
        }
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(self.LayerObject(), None);
        if let Some((paint_layer, nodes)) = &child_layers {
            result.Include(self.PaintChildren(
                paint_layer,
                PaintLayerIteration::kNormalFlowAndPositiveZOrderChildren,
                nodes,
            ));
        }

        // Source: paint_layer_painter.cc:507-514. Overlay controls are above
        // normal-flow and positive-z descendants unless StackingNode moved
        // them after the highest contained stacked descendant.
        if should_paint_content
            && self.layer.has_overlay_overflow_controls
            && !self.NeedsReorderOverlayOverflowControls()
        {
            self.PaintWithPhase(
                self.layer,
                self.LayerWalk(),
                &self.PaintInfo(PaintPhase::kOverlayOverflowControls),
            );
        }

        self.context.borrow_mut().EndEscapableGeometry(self.LayerObject());
        if effect_contains_escaping_child {
            for ancestor in self.inherited_geometry.iter().rev() {
                if self.UsesInheritedGeometry(ancestor) {
                    self.context.borrow_mut().EndEscapableGeometry(PaintObject::new(ancestor));
                }
            }
        }
        // Mask's parent is the content Effect, outside a separate Filter node.
        // Finish the flat filter before recording the source Mask drawing, as
        // PaintFragmentWithPhase(kMask) uses the Mask child's property state.
        self.context.borrow_mut().EndFilter(self.LayerObject());
        self.context.borrow_mut().EndCssClip(self.LayerObject());
        self.context.borrow_mut().EndMaskClip(self.LayerObject());
        if should_paint_content && self.layer.applies_mask {
            BoxFragmentPainter::PaintFragment(self.LayerObject(), &self.PaintInfo(PaintPhase::kMask));
        }
        self.context.borrow_mut().EndEffectsAfterFilter(self.LayerObject());
        self.context.borrow_mut().EndGeometry(self.LayerObject());
        for ancestor in self.inherited_geometry.iter().rev() {
            if !effect_contains_escaping_child && self.UsesInheritedGeometry(ancestor) {
                self.context.borrow_mut().EndEscapableGeometry(PaintObject::new(ancestor));
            }
            self.context.borrow_mut().EndEffects(PaintObject::new(ancestor));
            self.context.borrow_mut().EndGeometry(PaintObject::new(ancestor));
        }
        result.Include(self.layer.previous_paint_result.get());
        self.SetPreviousPaintResult(result);
        if let Some(recorder) = &subsequence_recorder {
            recorder.SetPaintResult(result);
        }
        result
    }

    fn SuspendGeometryForEscapingChild(&self) {
        if !self.layer.is_root {
            self.context.borrow_mut().EndEscapableGeometry(self.LayerObject());
        }
        for ancestor in self.inherited_geometry.iter().rev() {
            self.context.borrow_mut().EndEscapableGeometry(PaintObject::new(ancestor));
        }
    }

    fn ResumeGeometryAfterEscapingChild(&self) {
        for ancestor in self.inherited_geometry.iter() {
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(PaintObject::new(ancestor), self.InheritedClipInLayerSpace(ancestor));
        }
        if !self.layer.is_root {
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(self.LayerObject(), None);
        }
    }

    // Source: paint_layer_painter.cc:790-810.
    fn PaintForegroundPhases(&self) {
        self.PaintWithPhase(
            self.layer,
            self.LayerWalk(),
            &self.PaintInfo(PaintPhase::kDescendantBlockBackgroundsOnly),
        );
        if self
            .layer
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment")
            .paint
            .InForcedColorsMode()
        {
            self.PaintWithPhase(
                self.layer,
                self.LayerWalk(),
                &self.PaintInfo(PaintPhase::kForcedColorsModeBackplate),
            );
        }
        if self.NeedsPaintPhaseFloat() {
            self.PaintWithPhase(self.layer, self.LayerWalk(), &self.PaintInfo(PaintPhase::kFloat));
        }
        self.PaintWithPhase(self.layer, self.LayerWalk(), &self.PaintInfo(PaintPhase::kForeground));
        if self.NeedsPaintPhaseDescendantOutlines() {
            self.PaintWithPhase(
                self.layer,
                self.LayerWalk(),
                &self.PaintInfo(PaintPhase::kDescendantOutlinesOnly),
            );
        }
    }

    /// `walk` is the walk context the paint parent (or this painter, for its
    /// own layer object) passes down with `paint_info`; every painter of
    /// this visit receives the object built from `node`'s fragment cursor,
    /// FragmentData and `walk`. The resident node still provides the
    /// traversal (its children, embedded frame roots) and is the oracle the
    /// object's values are checked against.
    fn PaintWithPhase<'x, 'p>(
        &self,
        node: &'x PaintTreeNode<'f>,
        walk: PaintWalkContext<'x>,
        paint_info: &PaintInfo<'p, 'o>,
    ) {
        let object = PaintObject::Walked(node, walk);
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
        if let Some(source) = node.SourceCursor().filter(|source| source.IsSvgUse()) {
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
            node.needs_float_phase,
            node.needs_outline_phase,
            node.needs_overflow_controls_phase,
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
        // the complete resident subtree, not merely its own display items.
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
        // did not already answer it for the same node and PaintInfo.
        let admitted = !is_svg_container;
        let _text_paint_timing = paint_record::paint_context::ScopedTextPaintAggregation::new(
            self.context,
            object,
            phase == PaintPhase::kForeground && !paint_info.IsRenderingResourceSubtree(),
        );
        let enter_node = !std::ptr::eq(node, self.layer) || paint_info.IsRenderingResourceSubtree();
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
                // it is still inside the node's transform, clip-path and
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

        let unscaled_text_combine_caret = phase == PaintPhase::kForeground
            && object.applies_transform()
            && node
                .fragment
                .as_deref()
                .is_some_and(|fragment| fragment.paint.IsTextCombine())
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
        let contents_paint_state = (!node.children.is_empty() || needs_reference_filter_chunk)
            .then(|| {
                paint_record::scoped_paint_state::ScopedBoxContentsPaintState::new(
                    paint_info, object,
                )
            });
        if needs_reference_filter_chunk {
            self.context.borrow_mut().EnsureChunk();
        }
        // Chromium's child/item traversal consumes the PaintInfo owned by
        // ScopedBoxContentsPaintState, not the border-box input PaintInfo. Keep
        // the adjusted contents CullRect alive across the resident child walk.
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
        let child_cull_rect = if node.children.is_empty() {
            None
        } else {
            object.cull_rect()
        };

        for child_owner in &node.children {
            let child = child_owner.as_ref();
            let child_walk =
                PaintWalkContext::new(child_cull_rect, Some(&child.previous_paint_result));
            if child.is_embedded_local_frame_root {
                // EmbeddedContentPainter::PaintReplaced -> LocalFrameView::Paint
                // -> FramePainter::Paint: the child's entire PaintLayer tree
                // paints at the replaced foreground position using this same
                // PaintController/GraphicsContext. Properties already carry
                // the frame location, so no bitmap or extra offset is applied.
                if phase == PaintPhase::kForeground
                    && !node.child_paint_blocked_by_display_lock
                    && node.fragment.as_deref().unwrap().paint.style.visible()
                    && !(paint_info.GetPaintFlags() & PaintFlag::kPrivacyPreserving != 0
                        && child.embedded_frame_is_cross_origin)
                {
                    PaintLayerPainter::new(child, self.context, Vec::new())
                        .WithFlags(paint_info.GetPaintFlags())
                        .Paint();
                }
                continue;
            }
            if child.is_paint_layer {
                if contents_paint_info.IsRenderingResourceSubtree() {
                    self.PaintAllPhasesAtomically(child, child_walk, &contents_paint_info);
                }
                continue;
            }
            let child_fragment = child
                .fragment
                .as_deref()
                .expect("PrePaint node must own a fragment");
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
                && !BoxFragmentPainter::ChildIntersectsCullRect(
                    PaintObject::Walked(child, child_walk),
                    &contents_paint_info,
                )
            {
                continue;
            }
            // PhysicalFragment::IsPaintedAtomically of the resident box
            // (atomic inlines, flex/grid items, floats), read live as
            // BoxFragmentPainter::PaintFragment / Paint read it
            // (box_fragment_painter.cc:636-651). The stored
            // `paint.painted_atomically` is only a detached-fixture fallback
            // and is never populated for resident fragments. A
            // self-painting-layer child was already left to its own layer
            // above, matching `!box_fragment_.HasSelfPaintingLayer()` there.
            let is_atomic = child_fragment.ViewPaintedAtomically();
            let child_is_float = IsFloat(child);
            match crate::phase::ClassifyChildPhase(phase, child_is_float, is_atomic) {
                crate::phase::ChildPhaseAction::Skip => {}
                crate::phase::ChildPhaseAction::PaintAtomically => {
                    let mut atomic_info = contents_paint_info.ForDescendants();
                    if child_is_float {
                        atomic_info.phase = crate::phase::FloatPaintPhase(atomic_info.phase);
                    }
                    self.PaintAllPhasesAtomically(child, child_walk, &atomic_info);
                }
                crate::phase::ChildPhaseAction::Recurse(child_phase) => {
                    let mut child_info = contents_paint_info.ForDescendants();
                    child_info.phase = child_phase;
                    self.PaintWithPhase(child, child_walk, &child_info);
                }
            }
        }
        drop(contents_paint_state);
        // Blink's overflow controls use border-box properties, outside their
        // owner's OverflowClip/InnerBorderRadiusClip (paint property builder,
        // UpdateOverflowControlEffects). Keep the flat recording in that same
        // space; an extra fractional AA overflow clip is not redundant.
        let paint_controls = phase == PaintPhase::kOverlayOverflowControls
            && node
                .fragment
                .as_deref()
                .is_some_and(|fragment| fragment.ViewScrollbarsPaintAsOverlay().is_some());
        if paint_controls {
            self.context.borrow_mut().EndEscapableGeometry(object);
        }
        let restore_text_combine = enter_node
            && object.applies_transform()
            && node
                .fragment
                .as_deref()
                .is_some_and(|fragment| fragment.paint.IsTextCombine());
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
        node: &'x PaintTreeNode<'f>,
        walk: PaintWalkContext<'x>,
        paint_info: &PaintInfo<'p, 'o>,
    ) {
        if matches!(
            paint_info.phase,
            PaintPhase::kSelectionDragImage | PaintPhase::kTextClip
        ) {
            self.PaintWithPhase(node, walk, paint_info);
            return;
        }
        if paint_info.phase != PaintPhase::kForeground {
            return;
        }
        let mut local_info = *paint_info;
        for phase in crate::phase::ATOMIC_PHASES {
            local_info.phase = phase;
            self.PaintWithPhase(node, walk, &local_info);
        }
    }

    fn EntryEscapesCurrentGeometry(&self, child: &PaintLayerPaintOrderEntry<'_, 'n, 'f>) -> bool {
        let escapes_geometry = child.layer.escapes_ancestor_geometry;
        let has_physical_ancestor_below_current =
            child.property_ancestors.iter().any(|candidate| {
                let pointer = *candidate as *const PaintTreeNode<'f>;
                child.layer.physical_ancestors.contains(&pointer)
            });
        let current_pointer = self.layer as *const PaintTreeNode<'f>;
        let current_is_physical_ancestor =
            child.layer.physical_ancestors.contains(&current_pointer);
        escapes_geometry && !current_is_physical_ancestor && !has_physical_ancestor_below_current
    }

    // Source: core/paint/paint_layer_painter.cc:692-698. This enters the
    // reparented scroller's normal layer/property scope but records only its
    // overlay overflow-control phase.
    fn PaintOverlayOverflowControls(&self) {
        let _effectively_invisible = PaintedOutputInvisible(self.layer).then(|| {
            crate::scoped_effectively_invisible::ScopedEffectivelyInvisible::new(self.context)
        });
        let _layer_chunk_properties = crate::paint_context::ScopedPaintChunkProperties::layer(
            self.context,
            self.LayerObject(),
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        for ancestor in self.inherited_geometry.iter() {
            self.context.borrow_mut().BeginGeometry(PaintObject::new(ancestor));
            self.context.borrow_mut().BeginEffects(PaintObject::new(ancestor));
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(PaintObject::new(ancestor), None);
        }
        self.context.borrow_mut().BeginGeometry(self.LayerObject());
        self.context.borrow_mut().BeginEffects(self.LayerObject());
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(self.LayerObject(), None);
        self.PaintWithPhase(
            self.layer,
            self.LayerWalk(),
            &self.PaintInfo(PaintPhase::kOverlayOverflowControls),
        );
        self.context.borrow_mut().EndEscapableGeometry(self.LayerObject());
        self.context.borrow_mut().EndFilter(self.LayerObject());
        self.context.borrow_mut().EndCssClip(self.LayerObject());
        self.context.borrow_mut().EndMaskClip(self.LayerObject());
        self.context.borrow_mut().EndEffectsAfterFilter(self.LayerObject());
        self.context.borrow_mut().EndGeometry(self.LayerObject());
        for ancestor in self.inherited_geometry.iter().rev() {
            self.context.borrow_mut().EndEscapableGeometry(PaintObject::new(ancestor));
            self.context.borrow_mut().EndEffects(PaintObject::new(ancestor));
            self.context.borrow_mut().EndGeometry(PaintObject::new(ancestor));
        }
    }

    /// Source: paint_layer_painter.cc:623-690 (PaintChildren). `root` is this
    /// painter's PaintLayer, or a child PaintLayer without a resident layer
    /// node whose children this painter paints in its place (see
    /// PaintLayerWithoutResidentNode).
    fn PaintChildren(
        &self,
        root: &PaintLayer,
        children_to_visit: PaintLayerIteration,
        nodes: &ResidentLayerNodes<'n, 'f>,
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
            let mut has_resident_node = false;
            for occurrence in nodes.Occurrences(child) {
                has_resident_node = true;
                result.Include(self.PaintChildOccurrence(&occurrence));
            }
            if !has_resident_node {
                result.Include(self.PaintLayerWithoutResidentNode(child, nodes));
            }

            // Source: paint_layer_painter.cc:671-685. A target can carry more
            // than one scroller's controls, in collection order.
            if let Some(scrollers) = iterator.LayersPaintingOverlayOverflowControlsAfter(child) {
                for scroller in scrollers.iter() {
                    // Members of the live stacking node's map.
                    let scroller = unsafe { &*scroller.Get() };
                    debug_assert!(scroller.NeedsReorderOverlayOverflowControls());
                    for occurrence in nodes.Occurrences(scroller) {
                        let escapes_current_geometry =
                            self.EntryEscapesCurrentGeometry(&occurrence);
                        if escapes_current_geometry {
                            self.SuspendGeometryForEscapingChild();
                        }
                        PaintLayerPainter::with_inherited_geometry(
                            occurrence.layer,
                            self.context,
                            occurrence.property_ancestors,
                        )
                        .WithFlags(self.paint_flags)
                        .PaintOverlayOverflowControls();
                        result.Include(occurrence.layer.previous_paint_result.get());
                        if escapes_current_geometry {
                            self.ResumeGeometryAfterEscapingChild();
                        }
                    }
                }
            }
        }
        result
    }

    /// PaintLayerPainter(*child).Paint() for one resident occurrence of the
    /// child layer, re-entering the property scopes between this layer and it.
    fn PaintChildOccurrence(&self, child: &PaintLayerPaintOrderEntry<'_, 'n, 'f>) -> PaintResult {
        let escapes_current_geometry = self.EntryEscapesCurrentGeometry(child);
        if escapes_current_geometry {
            self.SuspendGeometryForEscapingChild();
        }
        let result = PaintLayerPainter::with_inherited_geometry(
            child.layer,
            self.context,
            child.property_ancestors,
        )
        .WithFlags(self.paint_flags)
        .Paint();
        if escapes_current_geometry {
            self.ResumeGeometryAfterEscapingChild();
        }
        result
    }

    /// PaintLayerPainter(*layer).Paint() for a PaintLayer PrePaint built no
    /// layer node for: a non-self-painting layer, or an inline-level layer
    /// whose content Rechrom paints with its line box. Such a layer records
    /// no content of its own here, so Paint reduces to its early returns and
    /// its two PaintChildren calls (paint_layer_painter.cc:304-308,481-483,
    /// 499-501,640-650). Its descendants' resident nodes lie below this
    /// painter's node, so they are painted from here with their recorded
    /// property ancestors.
    fn PaintLayerWithoutResidentNode(
        &self,
        layer: &PaintLayer,
        nodes: &ResidentLayerNodes<'n, 'f>,
    ) -> PaintResult {
        if !layer.HasSelfPaintingLayerDescendant()
            || layer.GetLayoutObject().ChildPaintBlockedByDisplayLock()
        {
            return PaintResult::kFullyPainted;
        }
        let result = self.PaintChildren(layer, PaintLayerIteration::kAllChildren, nodes);
        if self.paint_flags & PaintFlag::kOmitCompositingInfo == 0 {
            layer.SetPreviousPaintResult(result);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout::paint::FragmentPaintData;
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
            CullAdmission(&contained, true),
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
            CullAdmission(&clipped, true),
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
            CullAdmission(&layer, true),
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
            CullAdmission(&layer, true),
            (PaintResult::kFullyPainted, true, false)
        );
    }

    #[test]
    fn known_padding_clip_returns_before_paint_when_both_rects_are_outside() {
        let mut layer = cull_layer(
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
        layer.known_to_clip_subtree_to_padding_box = true;
        assert_eq!(
            CullAdmission(&layer, true),
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
        assert!(PaintedOutputInvisible(&node));

        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .style
            .will_change_opacity = true;
        assert!(!PaintedOutputInvisible(&node));

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
        assert!(!PaintedOutputInvisible(&node));
    }

    #[test]
    fn ancestor_position_visibility_hides_the_layer() {
        let mut node = PaintTreeNode::default();
        node.paint_layer_facts_are_authoritative = true;
        node.has_ancestor_invisible_for_position_visibility = true;
        assert!(HiddenForPositionVisibility(&node));

        node.has_ancestor_invisible_for_position_visibility = false;
        assert!(!HiddenForPositionVisibility(&node));
        node.invisible_for_position_visibility = true;
        assert!(HiddenForPositionVisibility(&node));
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
        node.supports_subsequence_caching = true;
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        context.borrow_mut().update_chunk_id_and_properties(
            &node,
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        context.borrow_mut().EnsureChunk();

        assert!(ShouldCreateSubsequence(&node, &context, PaintFlag::kNoFlag));
        context.borrow_mut().BeginSkippingCache();
        assert!(!ShouldCreateSubsequence(
            &node,
            &context,
            PaintFlag::kNoFlag
        ));
        context.borrow_mut().EndSkippingCache();
        node.supports_subsequence_caching = false;
        assert!(!ShouldCreateSubsequence(
            &node,
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
        node.supports_subsequence_caching = true;
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        context.borrow_mut().update_chunk_id_and_properties(
            &node,
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        context.borrow_mut().EnsureChunk();

        assert!(ShouldCreateSubsequence(&node, &context, PaintFlag::kNoFlag));
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
        node.supports_subsequence_caching = true;
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        context.borrow_mut().update_chunk_id_and_properties(
            &node,
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
            &node,
        );

        assert!(!ShouldCreateSubsequence(
            &node,
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
        assert!(ShouldCreateSubsequence(&node, &context, PaintFlag::kNoFlag));
    }
}

// Source SVGObjectPainter::PaintResourceSubtree (cc:93-102) and explicit
// SVGContainerPainter(marker) dispatch (svg_shape_painter.cc:282-323).
fn PaintSvgResourceSubtree(node: PaintObject<'_, '_>, info: &PaintInfo<'_, '_>) {
    let node = node.BackingNode();
    let painter = PaintLayerPainter::new(node, info.context, Vec::new()).WithFlags(info.GetPaintFlags());
    painter.PaintWithPhase(node, painter.LayerWalk(), info);
}
