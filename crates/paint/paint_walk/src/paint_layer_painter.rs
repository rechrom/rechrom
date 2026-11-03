#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::fragment_tree::FragmentKind;
use layoutng_assembly::internal::layout_input::FloatSide;
use layoutng_assembly::internal::paint_input::PaintFilterType;

use crate::box_fragment_painter::BoxFragmentPainter;
use crate::geometry_mapper::{MapRectFromRoot, MapRectToRoot};
use crate::ignore_paint_timing_scope::IgnorePaintTimingScope;
use crate::paint_context::PaintContext;
use crate::paint_info::PaintInfo;
use crate::paint_layer_paint_order_iterator::{
    CollectPaintLayerChildren, PaintLayerPaintOrder, PaintLayerPaintOrderEntry,
};
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintPhase;
use crate::PaintRect;
use paint_model::PaintResult;
use paint_record::{
    box_fragment_painter::GetHitTestOpaqueness,
    paint_chunker::DisplayItemClientInfo,
    paint_engine::PaintHitTestOpaqueness,
    scoped_paint_state::{ContentsVisualRect, CullRectInTransform},
    SubsequenceRecorder,
};

fn IsFloat(node: &PaintTreeNode<'_>) -> bool {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    fragment.paint.HasSource() && fragment.paint.floating != FloatSide::kNone
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
fn ShouldSkipForLayout(fragment: &layoutng_assembly::fragment_tree::FragmentPaintData) -> bool {
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

fn FragmentRect(rect: layoutng_assembly::fragment_tree::FragmentPaintRect) -> PaintRect {
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
        || layer.properties.nodes.effect.HasCanvasChildState()
        || fragment.paint.IsInclusiveDescendantOfUnboundedElement()
    {
        return (
            PaintResult::kMayBeClippedByCullRect,
            should_paint_content,
            false,
        );
    }

    let (Some(visual_rect), Some(self_cull_rect)) = (
        fragment.paint.paint_layer_first_fragment_visual_rect,
        layer.self_cull_rect,
    ) else {
        // Hand-built fragments and incomplete lifecycle input retain the
        // pre-existing admission. Chromium layers always export both values.
        return (PaintResult::kFullyPainted, should_paint_content, false);
    };
    let visual_rect = FragmentRect(visual_rect);
    let self_cull_rect = CullRectInTransform(self_cull_rect, &layer.properties.nodes.transform);
    let self_intersects = Intersects(self_cull_rect, visual_rect);
    let mut result = if self_cull_rect.contains(visual_rect) {
        PaintResult::kFullyPainted
    } else {
        PaintResult::kMayBeClippedByCullRect
    };

    let contents_intersects = if fragment.paint.IsLayoutBox() {
        if let (Some(contents), Some(contents_cull_rect)) =
            (ContentsVisualRect(layer), layer.cull_rect)
        {
            let contents_cull_rect = CullRectInTransform(
                contents_cull_rect,
                &layer.contents_properties.nodes.transform,
            );
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

fn HasEscapingChildLayer(parent: &PaintTreeNode<'_>) -> bool {
    for child_owner in &parent.children {
        let child = child_owner.as_ref();
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

fn CollectDirectPaintLayerChildren<'n, 'f>(
    root: &'n PaintTreeNode<'f>,
    output: &mut Vec<&'n PaintTreeNode<'f>>,
) {
    for child in &root.children {
        if child.is_paint_layer {
            output.push(child);
        } else {
            CollectDirectPaintLayerChildren(child, output);
        }
    }
}

/// Source: `paint_layer_painter.cc:221-235,239-284`.
///
/// These source branches cover property boundaries, non-trivial descendant
/// layer trees, compositor hit-test opaqueness and sparse merged bounds
/// without creating proxy scopes.
/// The ordinary Rechrom paint walk has no special kOmitCompositingInfo entry;
/// that source-only admission branch remains unclaimed in the manifest.
fn ShouldCreateSubsequence(layer: &PaintTreeNode<'_>, context: &RefCell<PaintContext<'_>>) -> bool {
    let fragment = layer
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if fragment.paint.IsPrinting()
        || context.borrow().IsSkippingCache()
        || !layer.supports_subsequence_caching
    {
        return false;
    }
    if context.borrow().NumNewChunks() > 0
        && context
            .borrow()
            .LastChunkProperties()
            .is_some_and(|properties| !properties.same_nodes(&layer.properties.nodes))
    {
        return true;
    }
    let mut children = Vec::new();
    CollectDirectPaintLayerChildren(layer, &mut children);
    if children.len() >= 2 {
        return true;
    }
    if children.first().is_some_and(|first| {
        let mut grandchildren = Vec::new();
        CollectDirectPaintLayerChildren(first, &mut grandchildren);
        !grandchildren.is_empty()
    }) {
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
            fragment.paint.paint_layer_first_fragment_visual_rect,
            fragment.paint.resources.as_deref(),
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

pub struct PaintLayerPainter<'n, 'f, 'c, 'o> {
    layer: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
    inherited_geometry: Vec<&'n PaintTreeNode<'f>>,
}

impl<'n, 'f, 'c, 'o> PaintLayerPainter<'n, 'f, 'c, 'o> {
    pub fn new(
        layer: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
        inherited_geometry: Vec<&'n PaintTreeNode<'f>>,
    ) -> Self {
        Self {
            layer,
            context,
            inherited_geometry,
        }
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

    pub fn Paint(&self) -> PaintResult {
        self.PaintInternal(true, false)
    }

    fn PaintInternal(
        &self,
        include_descendant_layers: bool,
        suppress_overlay_overflow_controls: bool,
    ) -> PaintResult {
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
            self.layer.previous_paint_result.set(result);
            return result;
        }
        // ScopedBoxContentsPaintState may downgrade this while phases paint.
        // Seed it with this traversal's result rather than the previous frame.
        self.layer.previous_paint_result.set(result);
        // Source: paint_layer_painter.cc:412-424. The scope is declared before
        // all paint-state scopes so its Drop closes the subsequence last.
        let subsequence_recorder = if should_paint_content
            && ShouldCreateSubsequence(self.layer, self.context)
        {
            let fragment = self
                .layer
                .fragment
                .as_deref()
                .expect("subsequence PaintLayer fragment");
            let client = DisplayItemClientInfo::from_paint_layer(fragment)
                .expect("subsequence requires a source PaintLayer client");
            let debug = Some((
                fragment.paint.display_item_client_owner_node_id,
                fragment.paint.display_item_client_debug_name.clone(),
            ));
            if !self.layer.self_or_descendant_needs_repaint {
                if let Some(cached_result) = SubsequenceRecorder::UseCachedSubsequenceIfPossible(
                    self.context,
                    fragment.paint.paint_layer_client_id,
                    client,
                    debug.clone(),
                ) {
                    self.layer.previous_paint_result.set(cached_result);
                    return cached_result;
                }
            }
            Some(SubsequenceRecorder::new(
                self.context,
                fragment.paint.paint_layer_client_id,
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
                self.layer,
                crate::display_item_id::DisplayItemIdType::kLayerChunk,
            )
        });
        // Source: core/paint/paint_layer_painter.cc:457-468. Reference filters
        // must retain a property-bearing chunk even if this layer records no
        // drawing. Canvas-child forcing remains a separate unsupported branch.
        if should_paint_content
            && (self.layer.properties.nodes.effect.HasCanvasChildState()
                || HasReferenceFilter(self.layer))
        {
            self.context.borrow_mut().EnsureChunk();
        }
        let effect_contains_escaping_child =
            !self.layer.escapes_ancestor_geometry && HasEscapingChildLayer(self.layer);
        for ancestor in &self.inherited_geometry {
            self.context.borrow_mut().BeginGeometry(ancestor);
            self.context.borrow_mut().BeginEffects(ancestor);
            if !effect_contains_escaping_child && self.UsesInheritedGeometry(ancestor) {
                self.context
                    .borrow_mut()
                    .BeginEscapableGeometry(ancestor, None);
            }
        }
        self.context.borrow_mut().BeginGeometry(self.layer);
        self.context.borrow_mut().BeginEffects(self.layer);
        if effect_contains_escaping_child {
            for ancestor in &self.inherited_geometry {
                if self.UsesInheritedGeometry(ancestor) {
                    self.context
                        .borrow_mut()
                        .BeginEscapableGeometry(ancestor, self.InheritedClipInLayerSpace(ancestor));
                }
            }
        }

        if should_paint_content {
            BoxFragmentPainter::PaintFragment(
                self.layer,
                &PaintInfo::new(self.context, PaintPhase::kSelfBlockBackgroundOnly),
            );
        }
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(self.layer, None);
        // The immutable paint tree has the same descendants in both z-order
        // phases. Collect/sort once, then consume each group at its original point.
        let paint_order = if include_descendant_layers
            && (!self.layer.paint_layer_facts_are_authoritative
                || (self.layer.has_self_painting_layer_descendant
                    && !self.layer.child_paint_blocked_by_display_lock))
        {
            Some(CollectPaintLayerChildren(self.layer))
        } else {
            None
        };
        let children = paint_order
            .as_ref()
            .map(|order| order.entries.as_slice())
            .unwrap_or_default();
        let negative_count = children
            .partition_point(|child| child.layer.is_stacked && child.layer.stacking_level < 0);
        if let Some(order) = &paint_order {
            result.Include(self.PaintLayerChildren(&children[..negative_count], order));
        }
        if should_paint_content {
            let _foreground_properties = crate::paint_context::ScopedPaintChunkProperties::layer(
                self.context,
                self.layer,
                crate::display_item_id::DisplayItemIdType::kLayerChunkForeground,
            );
            self.PaintForegroundPhases();
        }
        // Blink paints the self outline with LocalBorderBoxProperties, outside
        // ScopedBoxContentsPaintState (box_fragment_painter.cc:759-799). Mirror
        // that scope in the flat recording too; children still need this layer's
        // overflow clip when their own paint phases resume below.
        self.context.borrow_mut().EndEscapableGeometry(self.layer);
        if self.layer.is_self_painting_layer && self.layer.paints_outline {
            BoxFragmentPainter::PaintFragment(
                self.layer,
                &PaintInfo::new(self.context, PaintPhase::kSelfOutlineOnly),
            );
        }
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(self.layer, None);
        if let Some(order) = &paint_order {
            result.Include(self.PaintLayerChildren(&children[negative_count..], order));
        }

        // Source: paint_layer_painter.cc:507-514. Overlay controls are above
        // normal-flow and positive-z descendants unless StackingNode moved
        // them after the highest contained stacked descendant.
        if should_paint_content
            && self.layer.has_overlay_overflow_controls
            && !suppress_overlay_overflow_controls
        {
            self.PaintWithPhase(
                self.layer,
                &PaintInfo::new(self.context, PaintPhase::kOverlayOverflowControls),
            );
        }

        self.context.borrow_mut().EndEscapableGeometry(self.layer);
        if effect_contains_escaping_child {
            for ancestor in self.inherited_geometry.iter().rev() {
                if self.UsesInheritedGeometry(ancestor) {
                    self.context.borrow_mut().EndEscapableGeometry(ancestor);
                }
            }
        }
        // Mask's parent is the content Effect, outside a separate Filter node.
        // Finish the flat filter before recording the source Mask drawing, as
        // PaintFragmentWithPhase(kMask) uses the Mask child's property state.
        self.context.borrow_mut().EndFilter(self.layer);
        self.context.borrow_mut().EndCssClip(self.layer);
        self.context.borrow_mut().EndMaskClip(self.layer);
        if should_paint_content && self.layer.applies_mask {
            BoxFragmentPainter::PaintFragment(
                self.layer,
                &PaintInfo::new(self.context, PaintPhase::kMask),
            );
        }
        self.context.borrow_mut().EndEffectsAfterFilter(self.layer);
        self.context.borrow_mut().EndGeometry(self.layer);
        for ancestor in self.inherited_geometry.iter().rev() {
            if !effect_contains_escaping_child && self.UsesInheritedGeometry(ancestor) {
                self.context.borrow_mut().EndEscapableGeometry(ancestor);
            }
            self.context.borrow_mut().EndEffects(ancestor);
            self.context.borrow_mut().EndGeometry(ancestor);
        }
        result.Include(self.layer.previous_paint_result.get());
        self.layer.previous_paint_result.set(result);
        if let Some(recorder) = &subsequence_recorder {
            recorder.SetPaintResult(result);
        }
        result
    }

    fn SuspendGeometryForEscapingChild(&self) {
        if !self.layer.is_root {
            self.context.borrow_mut().EndEscapableGeometry(self.layer);
        }
        for ancestor in self.inherited_geometry.iter().rev() {
            self.context.borrow_mut().EndEscapableGeometry(ancestor);
        }
    }

    fn ResumeGeometryAfterEscapingChild(&self) {
        for ancestor in &self.inherited_geometry {
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(ancestor, self.InheritedClipInLayerSpace(ancestor));
        }
        if !self.layer.is_root {
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(self.layer, None);
        }
    }

    // Source: paint_layer_painter.cc:790-810.
    fn PaintForegroundPhases(&self) {
        self.PaintWithPhase(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kDescendantBlockBackgroundsOnly),
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
                &PaintInfo::new(self.context, PaintPhase::kForcedColorsModeBackplate),
            );
        }
        if self.layer.needs_float_phase {
            self.PaintWithPhase(
                self.layer,
                &PaintInfo::new(self.context, PaintPhase::kFloat),
            );
        }
        self.PaintWithPhase(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kForeground),
        );
        if self.layer.needs_outline_phase {
            self.PaintWithPhase(
                self.layer,
                &PaintInfo::new(self.context, PaintPhase::kDescendantOutlinesOnly),
            );
        }
    }

    fn PaintWithPhase(&self, node: &PaintTreeNode<'f>, paint_info: &PaintInfo<'c, 'o>) {
        self.context.borrow_mut().walk_metrics.phase_visits += 1;
        if node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment")
            .paint
            .hidden
        {
            self.context.borrow_mut().walk_metrics.skipped_hidden += 1;
            return;
        }
        if !node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment")
            .paint
            .CanPaintFragmentOccurrence()
        {
            self.context.borrow_mut().walk_metrics.skipped_phase += 1;
            return;
        }
        let phase = paint_info.phase;
        if !crate::scoped_svg_paint_state::ScopedSVGPaintState::PaintsPhase(node, phase) {
            self.context.borrow_mut().walk_metrics.skipped_phase += 1;
            return;
        }
        if !crate::scoped_svg_paint_state::ScopedSVGPaintState::ShouldPaint(node) {
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
        let is_svg_container = crate::svg_container_painter::SVGContainerPainter::IsContainer(node);
        if is_svg_container
            && !crate::svg_container_painter::SVGContainerPainter::ShouldPaintSubtree(node)
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
        if !is_svg_container && !BoxFragmentPainter::ShouldPaintFragment(node, paint_info) {
            return;
        }
        let enter_node = !std::ptr::eq(node, self.layer);
        if !enter_node
            && matches!(
                phase,
                PaintPhase::kForeground | PaintPhase::kForcedColorsModeBackplate
            )
        {
            BoxFragmentPainter::PaintFragment(node, paint_info);
        }
        let paint_self_outside_clip = enter_node
            && node.applies_overflow_clip
            && !node.is_paint_layer
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
                context.BeginGeometry(node);
                context.BeginEffects(node);
                drop(context);
                BoxFragmentPainter::PaintFragment(node, paint_info);
                self.context.borrow_mut().BeginEscapableGeometry(node, None);
            } else {
                self.context.borrow_mut().BeginNode(node);
                BoxFragmentPainter::PaintFragment(node, paint_info);
            }
        }

        let unscaled_text_combine_caret = phase == PaintPhase::kForeground
            && node.applies_transform
            && node
                .fragment
                .as_deref()
                .is_some_and(|fragment| fragment.paint.text_combine.is_some())
            && self.context.borrow().caret.is_some();
        if unscaled_text_combine_caret {
            self.context.borrow_mut().EndGeometry(node);
            {
                let _parent_properties =
                    crate::paint_context::ScopedPaintChunkProperties::text_combine_parent(
                        self.context,
                        node,
                        phase,
                    );
                BoxFragmentPainter::PaintFragmentBeforeChildren(node, paint_info);
            }
            self.context.borrow_mut().BeginGeometry(node);
        } else {
            BoxFragmentPainter::PaintFragmentBeforeChildren(node, paint_info);
        }

        let contents_paint_state =
            paint_record::scoped_paint_state::ScopedBoxContentsPaintState::new(paint_info, node);
        // Chromium's child/item traversal consumes the PaintInfo owned by
        // ScopedBoxContentsPaintState, not the border-box input PaintInfo. Keep
        // the adjusted contents CullRect alive across the resident child walk.
        let contents_paint_info = *contents_paint_state.GetPaintInfo();
        if phase == PaintPhase::kForeground
            && crate::svg_container_painter::SVGContainerPainter::NeedsReferenceFilterChunk(node)
        {
            self.context.borrow_mut().EnsureChunk();
        }

        for child_owner in &node.children {
            let child = child_owner.as_ref();
            if child.is_paint_layer {
                continue;
            }
            // Chromium's early child cull here belongs to FragmentItems. A
            // physical child may establish its own PaintOffsetTranslation;
            // its recursive ShouldPaintFragment maps the cull rect into that
            // property space before testing its ink overflow.
            let child_is_fragment_item = matches!(
                child
                    .fragment
                    .as_deref()
                    .expect("PrePaint node must own a fragment")
                    .kind,
                FragmentKind::kLine | FragmentKind::kText | FragmentKind::kGeneratedText
            ) || child
                .fragment
                .as_deref()
                .expect("PrePaint node must own a fragment")
                .paint
                .box_fragment_is_inline_box;
            if child_is_fragment_item
                && !BoxFragmentPainter::ChildIntersectsCullRect(child, &contents_paint_info)
            {
                continue;
            }
            let is_atomic = child
                .fragment
                .as_deref()
                .expect("PrePaint node must own a fragment")
                .paint
                .painted_atomically;
            match crate::phase::ClassifyChildPhase(phase, IsFloat(child), is_atomic) {
                crate::phase::ChildPhaseAction::Skip => {}
                crate::phase::ChildPhaseAction::PaintAtomically => {
                    let mut atomic_info = contents_paint_info.ForDescendants();
                    if IsFloat(child) {
                        atomic_info.phase = crate::phase::FloatPaintPhase(atomic_info.phase);
                    }
                    self.PaintAllPhasesAtomically(child, &atomic_info);
                }
                crate::phase::ChildPhaseAction::Recurse(child_phase) => {
                    let mut child_info = contents_paint_info.ForDescendants();
                    child_info.phase = child_phase;
                    self.PaintWithPhase(child, &child_info);
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
                .is_some_and(|fragment| fragment.paint.scrollbars.is_some());
        if paint_controls {
            self.context.borrow_mut().EndEscapableGeometry(node);
        }
        let restore_text_combine = enter_node
            && node.applies_transform
            && node
                .fragment
                .as_deref()
                .is_some_and(|fragment| fragment.paint.text_combine.is_some());
        if restore_text_combine {
            // BoxFragmentPainter restores the text-combine CTM before
            // TextCombinePainter draws the parent's decorations/emphasis.
            self.context.borrow_mut().EndGeometry(node);
        }
        BoxFragmentPainter::PaintFragmentAfterChildren(node, paint_info);
        if paint_controls {
            self.context.borrow_mut().BeginEscapableGeometry(node, None);
        }
        if enter_node && !restore_text_combine {
            if crate::scoped_svg_paint_state::ScopedSVGPaintState::OwnsPaintScope(node) {
                debug_assert_eq!(phase, PaintPhase::kForeground);
                self.context.borrow_mut().EndEscapableGeometry(node);
                crate::scoped_svg_paint_state::ScopedSVGPaintState::FinishEffectsAfterContents(
                    node,
                    self.context,
                );
                if crate::svg_container_painter::SVGContainerPainter::IsContainer(node) {
                    crate::svg_container_painter::SVGContainerPainter::PaintOutlineAfterEffects(
                        node,
                        self.context,
                    );
                } else {
                    crate::svg_model_object_painter::SVGModelObjectPainter::PaintOutline(
                        node,
                        self.context,
                    );
                }
                self.context.borrow_mut().EndGeometry(node);
            } else {
                self.context.borrow_mut().EndNode(node);
            }
        }
    }

    // Source: core/paint/box_fragment_painter.cc:1805-1836.
    fn PaintAllPhasesAtomically(&self, node: &PaintTreeNode<'f>, paint_info: &PaintInfo<'c, 'o>) {
        if matches!(
            paint_info.phase,
            PaintPhase::kSelectionDragImage | PaintPhase::kTextClip
        ) {
            self.PaintWithPhase(node, paint_info);
            return;
        }
        if paint_info.phase != PaintPhase::kForeground {
            return;
        }
        let mut local_info = *paint_info;
        for phase in crate::phase::ATOMIC_PHASES {
            local_info.phase = phase;
            self.PaintWithPhase(node, &local_info);
        }
    }

    fn EntryEscapesCurrentGeometry(&self, child: &PaintLayerPaintOrderEntry<'n, 'f>) -> bool {
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
            self.layer,
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        for ancestor in &self.inherited_geometry {
            self.context.borrow_mut().BeginGeometry(ancestor);
            self.context.borrow_mut().BeginEffects(ancestor);
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(ancestor, None);
        }
        self.context.borrow_mut().BeginGeometry(self.layer);
        self.context.borrow_mut().BeginEffects(self.layer);
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(self.layer, None);
        self.PaintWithPhase(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kOverlayOverflowControls),
        );
        self.context.borrow_mut().EndEscapableGeometry(self.layer);
        self.context.borrow_mut().EndFilter(self.layer);
        self.context.borrow_mut().EndCssClip(self.layer);
        self.context.borrow_mut().EndMaskClip(self.layer);
        self.context.borrow_mut().EndEffectsAfterFilter(self.layer);
        self.context.borrow_mut().EndGeometry(self.layer);
        for ancestor in self.inherited_geometry.iter().rev() {
            self.context.borrow_mut().EndEscapableGeometry(ancestor);
            self.context.borrow_mut().EndEffects(ancestor);
            self.context.borrow_mut().EndGeometry(ancestor);
        }
    }

    fn PaintLayerChildren(
        &self,
        children: &[PaintLayerPaintOrderEntry<'n, 'f>],
        paint_order: &PaintLayerPaintOrder<'n, 'f>,
    ) -> PaintResult {
        let mut result = PaintResult::kFullyPainted;
        for child in children {
            let escapes_current_geometry = self.EntryEscapesCurrentGeometry(child);
            if escapes_current_geometry {
                self.SuspendGeometryForEscapingChild();
            }
            let child_painter =
                PaintLayerPainter::new(child.layer, self.context, child.property_ancestors.clone());
            if child.layer.is_stacking_context {
                result.Include(child_painter.Paint());
            } else {
                result.Include(child_painter.PaintInternal(
                    false,
                    paint_order.NeedsReorderOverlayOverflowControls(child.layer),
                ));
            }
            if escapes_current_geometry {
                self.ResumeGeometryAfterEscapingChild();
            }

            // Source: core/paint/paint_layer_painter.cc:645-681. A target can
            // carry more than one scroller's controls, in collection order.
            for scroller in paint_order.ControlsAfter(child.layer) {
                let escapes_current_geometry = self.EntryEscapesCurrentGeometry(scroller);
                if escapes_current_geometry {
                    self.SuspendGeometryForEscapingChild();
                }
                PaintLayerPainter::new(
                    scroller.layer,
                    self.context,
                    scroller.property_ancestors.clone(),
                )
                .PaintOverlayOverflowControls();
                result.Include(scroller.layer.previous_paint_result.get());
                if escapes_current_geometry {
                    self.ResumeGeometryAfterEscapingChild();
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::fragment_tree::{FragmentPaintData, FragmentPaintRect, PaintResources};
    use layoutng_assembly::internal::layout_input::{Offset, Size};
    use paint_record::{DisplayItem, PaintRecording};

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

    #[test]
    fn child_incomplete_paint_result_propagates_to_parent_layer() {
        let full = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let inner = PaintRect {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };
        let mut parent = cull_layer(inner, inner, full, full);
        parent.paint_layer_facts_are_authoritative = true;
        parent.is_paint_layer = true;
        parent.is_self_painting_layer = true;
        parent.has_self_painting_layer_descendant = true;
        parent.is_stacking_context = true;

        let mut child = cull_layer(inner, inner, full, full);
        child.paint_layer_facts_are_authoritative = true;
        child.is_paint_layer = true;
        child.is_self_painting_layer = true;
        child.is_stacking_context = true;
        child.is_z_order_list_visible = true;
        child
            .fragment
            .as_mut()
            .unwrap()
            .paint
            .paint_layer_is_fragmented = true;
        parent.children.push(Box::new(child));

        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        assert_eq!(
            PaintLayerPainter::new(&parent, &context, Vec::new()).Paint(),
            PaintResult::kMayBeClippedByCullRect
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
    fn subsequence_admission_requires_source_support_and_nontrivial_children() {
        let fragment = FragmentPaintData::default();
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.supports_subsequence_caching = true;
        let mut first = PaintTreeNode::default();
        first.is_paint_layer = true;
        let mut second = PaintTreeNode::default();
        second.is_paint_layer = true;
        node.children.push(Box::new(first));
        node.children.push(Box::new(second));
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));

        assert!(ShouldCreateSubsequence(&node, &context));
        context.borrow_mut().BeginSkippingCache();
        assert!(!ShouldCreateSubsequence(&node, &context));
        context.borrow_mut().EndSkippingCache();
        node.supports_subsequence_caching = false;
        assert!(!ShouldCreateSubsequence(&node, &context));
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

        assert!(ShouldCreateSubsequence(&node, &context));
    }

    #[test]
    fn subsequence_admission_separates_sparse_merged_bounds() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.SetPointerEventsNoneForTesting(true);
        fragment.paint.resources = Some(std::sync::Arc::new(PaintResources::default()));
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

        assert!(!ShouldCreateSubsequence(&node, &context));
        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .paint_layer_first_fragment_visual_rect
            .as_mut()
            .unwrap()
            .offset
            .x = 2000.0;
        assert!(ShouldCreateSubsequence(&node, &context));
    }
}
