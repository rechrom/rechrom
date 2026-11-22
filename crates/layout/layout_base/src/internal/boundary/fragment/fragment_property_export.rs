//! Chromium LayoutObject-to-fragment paint-property projection.
//!
//! This assembly-only owner keeps translated source types available while
//! assigning per-object projection and retained refresh self time to
//! `layout_fragment` in sampling profiles.

#![allow(non_snake_case)]

use super::*;
use crate::fragment_data::FragmentData;
use layout_style::style::style_transform::{
    ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
    ApplyTransformOrigin,
};

#[inline(never)]
pub(super) fn ExportPaintProperties(
    object: *const LayoutObject,
    effective_style: *const StyleComputedStyle,
) -> PaintProperties {
    ExportProperties(
        object,
        std::ptr::null_mut(),
        effective_style,
        LayoutUpdateKind::Paint,
        None,
    )
}

/// The StyleResourceData paired with `effective_style` for `object`, whose
/// own Node is `node`: the object's own resources for its own style, and the
/// ::first-line resources of the Node whose first-line style it is (the
/// fragment's object or the ancestor whose ::first-line it inherits), the
/// same pairing the Layout boundary's style index records.
fn EffectiveStyleResources<'a>(
    object: *const LayoutObject,
    node: &'a Node,
    effective_style: *const StyleComputedStyle,
) -> &'a crate::internal::layout_input::StyleResourceData {
    let own = node.InputStyleResources();
    if effective_style.is_null() {
        return own;
    }
    let mut current = object;
    while let Some(candidate) = unsafe { current.as_ref() } {
        if let Some(candidate_node) = unsafe { candidate.GetNode().as_ref() } {
            if std::ptr::eq(candidate_node.InputFirstLineStyle(), effective_style) {
                // SAFETY: the Node outlives this paint export like `node`.
                let candidate_node = unsafe { &*(candidate_node as *const Node) };
                return candidate_node
                    .InputFirstLineStyleResources()
                    .as_deref()
                    .unwrap_or_else(|| candidate_node.InputStyleResources());
            }
            if std::ptr::eq(candidate.StyleRef() as *const StyleComputedStyle, effective_style) {
                let candidate_node = unsafe { &*(candidate_node as *const Node) };
                return candidate_node.InputStyleResources();
            }
        }
        current = candidate.Parent() as *const LayoutObject;
    }
    own
}

// Source: core/paint/paint_property_tree_builder.cc:464-470. Compositor
// element ids belong to the active FragmentData, not to its LayoutObject's
// first fragment. This distinction is observable for fragmented boxes: each
// FragmentData owns a different UniqueObjectId while all fragments share the
// same LayoutObject.
pub(crate) fn PropertyFragmentUniqueId(
    object: *const LayoutObject,
    fragment_data: *mut FragmentData,
) -> u64 {
    let fragment_data = if fragment_data.is_null() {
        unsafe { &mut *object.cast_mut() }
            .GetMutableForPainting()
            .FirstFragment() as *mut FragmentData
    } else {
        fragment_data
    };
    let fragment_data = unsafe { &mut *fragment_data };
    fragment_data.EnsureId();
    fragment_data.UniqueId()
}

pub(super) fn ExportPrePaintInvalidation(
    object: *const LayoutObject,
) -> crate::fragment_tree::PrePaintInvalidation {
    if object.is_null() {
        return Default::default();
    }
    let object = unsafe { &*object };
    crate::fragment_tree::PrePaintInvalidation {
        blocking_wheel_event_handler_changed: object.GetPrePaintSubtreeWalkReasons().contains(crate::internal::pre_paint_subtree_walk_reasons::PrePaintSubtreeWalkReason::kBlockingWheelEventHandler),
        descendant_blocking_wheel_event_handler_changed: object.GetDescendantPrePaintSubtreeWalkReasons().contains(crate::internal::pre_paint_subtree_walk_reasons::PrePaintSubtreeWalkReason::kBlockingWheelEventHandler),
        needs_paint_property_update: object.NeedsPaintPropertyUpdate(),
        descendant_needs_paint_property_update: object.DescendantNeedsPaintPropertyUpdate(),
        child_pre_paint_blocked_by_display_lock: object.ChildPrePaintBlockedByDisplayLock(),
        should_check_for_paint_invalidation: object.ShouldCheckForPaintInvalidation(),
        background_needs_full_paint_invalidation: object.BackgroundNeedsFullPaintInvalidation(),
        full_paint_invalidation_reason: object
            .ShouldDoFullPaintInvalidation()
            .then(|| object.PaintInvalidationReasonForPrePaint()),
        should_check_layout_for_paint_invalidation: object.ShouldCheckLayoutForPaintInvalidation(),
        descendant_should_check_layout_for_paint_invalidation: object
            .DescendantShouldCheckLayoutForPaintInvalidation(),
        subtree_should_check_for_paint_invalidation: object
            .SubtreeShouldCheckForPaintInvalidation(),
        subtree_should_do_full_paint_invalidation: object.SubtreeShouldDoFullPaintInvalidation(),
        should_delay_full_paint_invalidation: object.ShouldDelayFullPaintInvalidation(),
        previous_visibility_visible: object.PreviousVisibilityVisible(),
        visual_rect_respects_visibility: object.VisualRectRespectsVisibility(),
        should_invalidate_selection: object.ShouldInvalidateSelection(),
        can_be_selection_leaf: object.CanBeSelectionLeaf(),
    }
}

// Source: core/paint/text_painter.cc:743-764
// (SvgTextPaintState::TextDecorationObject) and :766-787 (the decorating
// object's style, or its ::first-line style, supplies fill and stroke).
// Chromium walks the live LayoutObject ancestors while painting; Paint calls
// this through PaintProperties::SvgTextDecorationStyle instead of reading a
// copy frozen at Layout publication.
pub(crate) fn LiveSvgTextDecorationStyle(
    object: &LayoutObject,
    effective_style: &StyleComputedStyle,
) -> Option<PaintStyle> {
    if !object.IsSVGInlineText() {
        return None;
    }
    let use_first_line = !std::ptr::eq(effective_style, object.StyleRef());
    let mut result = object.Parent();
    while !result.is_null() {
        let ancestor = unsafe { &*result };
        let style = if use_first_line {
            ancestor.FirstLineStyleRef()
        } else {
            ancestor.StyleRef()
        };
        if style.GetTextDecorationLine() != TextDecorationLine::kNone {
            let node = ancestor.GetNode();
            if node.is_null() {
                return None;
            }
            let node = unsafe { &*node };
            // The decorating node's document-resolved style resources; its
            // ::first-line variant carries its own resources when present
            // (the same pairing LayoutBoundaryCache::style_sources records).
            let first_line_resources = if use_first_line
                && std::ptr::eq(style as *const StyleComputedStyle, node.InputFirstLineStyle())
            {
                node.InputFirstLineStyleResources().as_deref()
            } else {
                None
            };
            let resources = first_line_resources.unwrap_or_else(|| node.InputStyleResources());
            return Some(
                crate::internal::boundary::paint_style_export::ExportStyleWithInputResources(
                    style, resources,
                ),
            );
        }
        result = ancestor.Parent();
    }
    None
}

#[inline(never)]
pub(super) fn ExportProperties(
    object: *const LayoutObject,
    fragment_data: *mut FragmentData,
    effective_style: *const StyleComputedStyle,
    purpose: LayoutUpdateKind,
    exported_styles: ExportedStyleCache,
) -> PaintProperties {
    let mut output = PaintProperties::default();
    if object.is_null() {
        return output;
    }
    output.BindSourceObject(object);
    if purpose == LayoutUpdateKind::Paint {
        // PaintInvalidator's readiness step (paint_invalidator.cc:222) runs
        // in the caller that snapshots the invalidation facts.
        output.property_fragment_unique_id = PropertyFragmentUniqueId(object, fragment_data);
        // DisplayItemClient::DebugName() is read from the bound source object
        // on demand (PaintProperties::DisplayItemClientDebugName).
    }
    let text_combine = DynamicTo::<LayoutTextCombine>(object);
    if !text_combine.is_null() {
        let text_combine = unsafe { &*text_combine };
        // TextCombinePainter paints decorations and emphasis from the parent
        // inline's style after restoring the compressed-text transform.  The
        // anonymous LayoutTextCombine has no source Node, so freeze that
        // parent-owned paint state into this wrapper's typed payload here.
        // cpp: core/paint/text_combine_painter.cc:35-48
        let parent = text_combine.Parent();
        let parent_style = unsafe { &*parent }.StyleRef();
        let parent_image_resource_id = unsafe { (*parent).GetNode().as_ref() }
            .and_then(|node| node.InputStyleResources().image_resource_id);
        output.style = if purpose == LayoutUpdateKind::Paint {
            crate::internal::boundary::paint_style_export::ExportStyle(
                parent_style,
                parent_image_resource_id,
            )
        } else {
            crate::internal::boundary::paint_style_export::ExportGeometryStyle(parent_style)
        };
        output.writing_mode = PublicWritingMode(parent_style.GetWritingMode());
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    // Only objects with a Node carry source paint inputs.
    let Some(source) = (unsafe { (*object).GetNode().as_ref() }) else {
        return output;
    };
    let style_resources = EffectiveStyleResources(object, source, effective_style);
    output.logical_tree_order = unsafe { &*object }.LogicalTreeOrder();
    let layout_style = if effective_style.is_null() {
        unsafe { &*object }.StyleRef()
    } else {
        unsafe { &*effective_style }
    };
    let style_facts = crate::style_projection::ReadFragmentStyleFacts(layout_style);
    if purpose == LayoutUpdateKind::Layout {
        output.style =
            crate::internal::boundary::paint_style_export::ExportGeometryStyle(layout_style);
        ApplyResolvedOffsetTransform(object, layout_style, &mut output.style);
    } else if layout_style.HasOffset() {
        // The motion-path offset is resolved against the LayoutBox geometry
        // this publication certifies; it is never deferred past it.
        output.style =
            ResolveFragmentPaintStyle(object, layout_style, style_resources, exported_styles);
    } else {
        // The slot initialization records only the projection's identity: the effective
        // ComputedStyle and its StyleResourceData. The values resolve through
        // ResolveFragmentPaintStyle on their first read.
        output.style = PaintStyle::FromPending(crate::internal::layout_input::PendingPaintStyle {
            style: layout_style as *const StyleComputedStyle as usize,
            object: object as usize,
            resources: RetainStyleResources(style_resources),
            cache: exported_styles,
        });
    }
    // Chromium resolves the ::selection painting style and decoration
    // (HighlightStyleUtils::HighlightPaintingStyle, SelectionTextDecoration)
    // and the SVG decorating-ancestor style (text_painter.cc:743-787) while
    // painting. Paint reads them live through PaintProperties::
    // SelectionPaintStyle, SelectionTextDecoration and SvgTextDecorationStyle;
    // nothing is frozen into the publication here.
    output.border = style_facts.border;
    output.padding = style_facts.padding;
    // Chromium's PrePaint and painters read these from the LayoutObject's
    // ComputedStyle and LayoutBox (e.g. paint_property_tree_builder.cc
    // UpdateOverflowClip/UpdateScrollAndScrollTranslation, box_painter_base.cc
    // PaintFillLayers). PaintProperties reads them through `style` when it is
    // this live ComputedStyle and stores copies only otherwise.
    let live_style_facts = output
        .style
        .Source()
        .is_some_and(|style| std::ptr::eq(style, layout_style));
    output.BindLiveStyleFacts(live_style_facts);
    if !live_style_facts {
        output.effective_zoom = layout_style.EffectiveZoom();
        if purpose == LayoutUpdateKind::Paint {
            output.background_fill_layer_count =
                crate::internal::boundary::paint_style_export::BackgroundFillLayerCount(
                    layout_style,
                );
            output.has_background = layout_style.HasBackground();
            let background_layers = layout_style.BackgroundLayers();
            output.background_any_local_attachment_image =
                background_layers.AnyLayerHasLocalAttachmentImage();
            output.background_any_default_attachment_image =
                background_layers.AnyLayerHasDefaultAttachmentImage();
            output.background_any_layer_uses_content_box =
                background_layers.AnyLayerUsesContentBox();
        }
        output.margin = style_facts.margin;
        output.border_styles = style_facts.border_styles;
        output.display = style_facts.display;
        output.position = style_facts.position;
        output.fixed_to_view =
            !layout_box.is_null() && unsafe { &*layout_box }.IsFixedToView(std::ptr::null());
        output.needs_anchor_position_scroll_adjustment = !layout_box.is_null()
            && unsafe { &*layout_box }.NeedsAnchorPositionScrollAdjustment();
        output.floating = style_facts.floating;
        output.writing_mode = style_facts.writing_mode;
        output.direction = style_facts.direction;
        output.overflow_x = style_facts.overflow_x;
        output.overflow_y = style_facts.overflow_y;
        output.overscroll_behavior_x =
            crate::internal::boundary::layout_boundary::LiveOverscrollBehavior(
                layout_style.OverscrollBehaviorX(),
            );
        output.overscroll_behavior_y =
            crate::internal::boundary::layout_boundary::LiveOverscrollBehavior(
                layout_style.OverscrollBehaviorY(),
            );
        // Source: core/paint/paint_property_tree_builder.cc:3634-3635.
        output.prevent_scroll_axis_locking =
            layout_style.ScrollAxisLock() == foundation::EScrollAxisLock::kNone;
    }
    // SVG shape, transform, viewport and viewBox are read live from the
    // LayoutSVG owners (PaintProperties::SvgShape / SvgViewport / ...).
    if purpose == LayoutUpdateKind::Layout {
        return output;
    }
    output
}

/// A counted handle on the StyleResourceData paired with a fragment's
/// effective ComputedStyle, so a pending projection resolves against exactly
/// the resources the slot initialization saw.
fn RetainStyleResources(
    resources: &crate::internal::layout_input::StyleResourceData,
) -> Arc<crate::internal::layout_input::StyleResourceData> {
    let pointer = resources as *const crate::internal::layout_input::StyleResourceData;
    // SAFETY: StyleResourceData reaches the Layout boundary only through
    // Node::InputStyleResources / InputFirstLineStyleResources (directly or
    // via LayoutBoundaryCache::style_sources), both views into a live
    // Arc<StyleResourceData> owned by the node's input data.
    unsafe {
        Arc::increment_strong_count(pointer);
        Arc::from_raw(pointer)
    }
}

/// The live LayoutObject of the SVG resource element `target` of `kind`, in
/// the LayoutObject tree containing `object`. The resource can live under a
/// different nested SVG from its client.
fn FindSvgResourceObject(
    object: *const LayoutObject,
    target: u64,
    kind: NodeKind,
) -> *const LayoutObject {
    let mut root = object.cast_mut();
    loop {
        let parent = unsafe { &*root }.Parent();
        if parent.is_null() {
            break;
        }
        root = parent;
    }
    let mut current = root;
    while !current.is_null() {
        let candidate = unsafe { &*current };
        let node = candidate.GetNode();
        if !node.is_null()
            && unsafe { &*node }.InputId() == target
            && unsafe { &*node }.InputKind() == kind
        {
            return current;
        }
        current = candidate.NextInPreOrder(root);
    }
    std::ptr::null()
}

/// Motion-path is resolved against the actual containing LayoutBox, so its
/// final matrix cannot be reconstructed from ComputedStyle alone
/// (SetResolvedOffsetTransform).
fn ApplyResolvedOffsetTransform(
    object: *const LayoutObject,
    layout_style: &StyleComputedStyle,
    style: &mut PaintStyle,
) {
    if !layout_style.HasOffset() {
        return;
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    if layout_box.is_null() || DynamicTo::<LayoutBoxModelObject>(object).is_null() {
        return;
    }
    let layout_box = unsafe { &*layout_box };
    let reference_box = crate::internal::transform_utils::ComputeReferenceBoxForLayoutBox(layout_box);
    let mut offset = foundation::gfx::Transform::default();
    layout_style.ApplyTransformPhysicalRect(
        &mut offset,
        Some(layout_box),
        &reference_box,
        ApplyTransformOperations::kExcludeTransformOperations,
        ApplyTransformOrigin::kExcludeTransformOrigin,
        ApplyMotionPath::kIncludeMotionPath,
        ApplyIndependentTransformProperties::kExcludeIndependentTransformProperties,
    );
    crate::internal::boundary::paint_style_export::SetResolvedOffsetTransform(style, offset);
}

/// The Paint projection of a fragment's effective ComputedStyle and the
/// StyleResourceData paired with it: the identity-keyed cache lookup, the
/// export, the SVG clip-path / mask resource viewports and the motion-path
/// offset transform. The slot initialization resolves through this directly only for an
/// offset style; every other fragment PaintStyle resolves through it on its
/// first projected-value read (ResolvePendingPaintStyle).
#[inline(never)]
pub(crate) fn ResolveFragmentPaintStyle(
    object: *const LayoutObject,
    layout_style: &StyleComputedStyle,
    style_resources: &crate::internal::layout_input::StyleResourceData,
    cache: ExportedStyleCache,
) -> PaintStyle {
    let key = (
        layout_style as *const StyleComputedStyle as usize,
        style_resources as *const crate::internal::layout_input::StyleResourceData as usize,
    );
    if let Some(data) =
        cache.and_then(|cache| crate::layout_boundary_support::LookupExportedStyle(cache, key))
    {
        // Only offset-free projections are memoized.
        return PaintStyle::FromSourceData(layout_style, data);
    }
    let mut style = crate::internal::boundary::paint_style_export::ExportStyleWithInputResources(
        layout_style,
        style_resources,
    );
    let viewport_size = crate::internal::layout_pass_scope::LayoutPassScope::ResidentSvgSupport()
        .viewport_size;
    if let Some(target) = style
        .clip_path
        .as_ref()
        .and_then(|clip| clip.reference_target_id)
    {
        let resource = FindSvgResourceObject(object, target, NodeKind::kSvgResourceClipper);
        if let (false, Some(resolve)) = (resource.is_null(), viewport_size) {
            let size = resolve(unsafe { &*resource });
            style.clip_path.as_mut().unwrap().reference_viewport = Some(Size {
                width: f64::from(size.width()),
                height: f64::from(size.height()),
            });
        } else {
            // Connected ID targets without an attached source resource are
            // unavailable (for example a display:none ancestor).
            style.clip_path = None;
        }
    }
    if style
        .svg_mask_bindings
        .iter()
        .any(|reference| reference.target_id.is_some())
    {
        for reference in &mut style.svg_mask_bindings {
            let Some(target) = reference.target_id else {
                continue;
            };
            let resource = FindSvgResourceObject(object, target, NodeKind::kSvgResourceMasker);
            if let (false, Some(resolve)) = (resource.is_null(), viewport_size) {
                let size = resolve(unsafe { &*resource });
                reference.viewport = Some(Size {
                    width: f64::from(size.width()),
                    height: f64::from(size.height()),
                });
            } else {
                reference.graph = None;
            }
        }
    }
    if layout_style.HasOffset() {
        // Layout-dependent: resolved per object, never shared.
        ApplyResolvedOffsetTransform(object, layout_style, &mut style);
    } else if let Some(cache) = cache {
        // Every other style payload is immutable and shared by all physical
        // and inline fragments referencing the same source/resources pair.
        crate::layout_boundary_support::StoreExportedStyle(cache, key, style.SharedData());
    }
    style
}

/// Resolve a fragment PaintStyle recorded by the slot initialization, on its first read.
pub(crate) fn ResolvePendingPaintStyle(
    pending: &crate::internal::layout_input::PendingPaintStyle,
) -> Arc<crate::internal::paint_input::PaintStyleData> {
    let layout_style = unsafe { &*(pending.style as *const StyleComputedStyle) };
    ResolveFragmentPaintStyle(
        pending.object as *const LayoutObject,
        layout_style,
        &pending.resources,
        pending.cache,
    )
    .SharedData()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_fragment_unique_id_uses_selected_fragment_data() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut first = FragmentData::default();
        let mut second = FragmentData::default();
        let first_id = PropertyFragmentUniqueId(std::ptr::null(), &mut first);
        let second_id = PropertyFragmentUniqueId(std::ptr::null(), &mut second);

        assert_ne!(first_id, second_id);
        assert_eq!(
            first_id,
            PropertyFragmentUniqueId(std::ptr::null(), &mut first)
        );
        assert_eq!(
            second_id,
            PropertyFragmentUniqueId(std::ptr::null(), &mut second)
        );
    }
}

/// Selection commit updates the same resident PrePaint sidecars after LayoutClean.
/// This is also the live projection behind
/// `PaintProperties::LivePrePaintInvalidation`.
pub(crate) fn ExportSelectionPrePaintInvalidation(
    object: *const LayoutObject,
) -> crate::fragment_tree::PrePaintInvalidation {
    ExportPrePaintInvalidation(object)
}

// Live LayoutObject / PaintLayer / Element paint facts read by the
// PaintProperties getters (stage 2 of the Layout -> Paint export removal).
// Chromium's PrePaint and painters query these on the LayoutObject itself.

/// Source: core/paint/css_mask_painter.cc:44-98 and
/// svg_background_paint_context.cc:29-33. Ordinary CSS boxes, including
/// SVGForeignObject, use one object-wide maximum mask region for
/// ObjectPaintProperties::MaskClip. The no-clip query unions the
/// foreignObject's self-painting descendants at its PaintLayer.
pub(crate) fn LiveCssMaskBoundingBox(object: &LayoutObject) -> Option<FragmentPaintRect> {
    let object_ptr = object as *const LayoutObject as *mut LayoutObject;
    let style = object.StyleRef();
    if !(style.HasMask()
        && object.IsBoxModelObject()
        && (!object.IsSVGChild() || object.IsSVGForeignObject()))
    {
        return None;
    }
    let no_clip = style.MaskLayers().LayersClipMax() == EFillBox::kNoClip;
    let mut bounds = if object.IsBox() {
        let box_ = unsafe { &*To::<LayoutBox>(object_ptr) };
        if no_clip {
            let layer = box_.Layer();
            assert!(!layer.is_null());
            unsafe { &*layer }.LocalBoundingBoxIncludingSelfPaintingDescendants()
        } else {
            box_.PhysicalBorderBoxRect()
        }
    } else {
        let inline = unsafe { &*To::<LayoutInline>(object_ptr) };
        if no_clip {
            let layer = inline.Layer();
            assert!(!layer.is_null());
            unsafe { &*layer }.LocalBoundingBoxIncludingSelfPaintingDescendants()
        } else {
            inline.PhysicalLinesBoundingBox()
        }
    };
    if style.HasMaskBoxImageOutsets() {
        bounds.Expand(&style.MaskBoxImageOutsets());
    }
    Some(ExportInkRect(bounds))
}

/// LayoutBox::ScrollableOverflowRect(); None for objects that are not boxes.
pub(crate) fn LiveScrollableOverflow(object: &LayoutObject) -> Option<FragmentPaintRect> {
    let layout_box = DynamicTo::<LayoutBox>(object as *const LayoutObject);
    unsafe { layout_box.as_ref() }.map(|box_| ExportInkRect(box_.ScrollableOverflowRect()))
}

/// Source: core/paint/paint_layer_painter.cc:192-219
/// (PaintLayerPainter::FirstFragmentVisualRect). The LayoutObject-local
/// filtered overflow; PaintLayerPainter applies the FragmentData paint offset.
pub(crate) fn LivePaintLayerFirstFragmentVisualRect(layer: &crate::internal::paint_layer::PaintLayer) -> FragmentPaintRect {
    let layout_object = layer.GetLayoutObject();
    let mut visual_rect = if layout_object.IsBox() {
        let layout_box = DynamicTo::<LayoutBox>(layout_object);
        debug_assert!(!layout_box.is_null());
        unsafe { &*layout_box }.SelfVisualOverflowRect()
    } else {
        layout_object.VisualOverflowRect()
    };
    visual_rect = layout_object.ApplyFiltersToRect(&visual_rect);
    let visual_rect = foundation::gfx::ToEnclosingRect(gfx::RectF::from(visual_rect));
    FragmentPaintRect {
        offset: Offset {
            x: visual_rect.x() as f64,
            y: visual_rect.y() as f64,
        },
        size: Size {
            width: visual_rect.width() as f64,
            height: visual_rect.height() as f64,
        },
    }
}

/// Source: core/paint/paint_layer_painter.cc:304-318 — the layer box's
/// ContentsVisualOverflowRect(); None when the layer object is not a box.
pub(crate) fn LivePaintLayerContentsVisualRect(layer: &crate::internal::paint_layer::PaintLayer) -> Option<FragmentPaintRect> {
    let layout_box = DynamicTo::<LayoutBox>(layer.GetLayoutObject());
    unsafe { layout_box.as_ref() }.map(|box_| ExportInkRect(box_.ContentsVisualOverflowRect()))
}

/// Source: core/paint/paint_layer.cc:2044-2047 (PaintLayer::FilterReferenceBox).
pub(crate) fn LivePaintLayerFilterReferenceBox(layer: &crate::internal::paint_layer::PaintLayer) -> Option<FragmentPaintRect> {
    layer
        .HasFilterThatMovesPixels()
        .then(|| ExportInkRect(layer.LocalBoundingBoxIncludingSelfPaintingDescendants()))
}

/// Source: core/paint/paint_layer.cc (PaintLayer::FilterViewport).
pub(crate) fn LivePaintLayerFilterViewport(layer: &crate::internal::paint_layer::PaintLayer) -> Option<Size> {
    if !layer.HasFilterThatMovesPixels() {
        return None;
    }
    layer.ComputeFilterViewport().map(|viewport| Size {
        width: f64::from(viewport.width()),
        height: f64::from(viewport.height()),
    })
}

/// LayoutReplaced::GetSelectionState (layout_replaced.cc:599-617).
pub(crate) fn LiveReplacedSelectionState(object: &LayoutObject) -> crate::internal::selection_state::SelectionState {
    let replaced = DynamicTo::<crate::internal::layout_replaced::LayoutReplaced>(
        object as *const LayoutObject,
    );
    unsafe { replaced.as_ref() }.map_or(crate::internal::selection_state::SelectionState::kNone, |replaced| {
        replaced.GetSelectionState()
    })
}

/// LayoutReplaced::LocalSelectionVisualRect (layout_replaced.cc:599-617).
pub(crate) fn LiveReplacedLocalSelectionVisualRect(
    object: &LayoutObject,
) -> Option<FragmentPaintRect> {
    let replaced = DynamicTo::<crate::internal::layout_replaced::LayoutReplaced>(
        object as *const LayoutObject,
    );
    let rect = unsafe { replaced.as_ref() }?.LocalSelectionVisualRect();
    (!rect.IsEmpty()).then(|| ExportInkRect(rect))
}

fn LiveElement(object: &LayoutObject) -> Option<&Element> {
    unsafe { DynamicTo::<Element>(object.GetNode()).as_ref() }
}

/// Element::GetRegionCaptureCropId(), read as PaintController's
/// region-capture chunk metadata does (box_fragment_painter.cc).
pub(crate) fn LiveRegionCaptureCropId(object: &LayoutObject) -> Option<[u64; 2]> {
    LiveElement(object).and_then(|element| element.InputRegionCaptureCropId())
}

/// Element tracked sub-rects, read as PaintController's tracked-element
/// chunk metadata does.
pub(crate) fn LiveTrackedElementSubRects(
    object: &LayoutObject,
) -> &[crate::internal::layout_input::TrackedElementSubRect] {
    LiveElement(object).map_or(&[], |element| element.InputTrackedElementSubRects())
}

// DOM-sourced facts. Chromium's painters read them from the LayoutObject's
// Node / Element (e.g. ImagePainter's HTMLImageElement decoding mode,
// VideoPainter's HTMLVideoElement frame). `None` means the object has no
// source Node; ExportProperties exported no such facts for it either.

fn LiveSourceNode(object: &LayoutObject) -> Option<&Node> {
    unsafe { object.GetNode().as_ref() }
}

fn LiveElementInputData(
    object: &LayoutObject,
) -> Option<Option<&crate::internal::layout_input::ElementData>> {
    let node = LiveSourceNode(object)?;
    Some(if node.IsElementNode() {
        unsafe { &*(node as *const Node as *const Element) }
            .InputElementData()
            .as_ref()
    } else {
        None
    })
}

pub(crate) fn LiveSourceKind(object: &LayoutObject) -> Option<NodeKind> {
    LiveSourceNode(object).map(|node| node.InputKind())
}

pub(crate) fn LiveImageDecodingMode(
    object: &LayoutObject,
) -> Option<image_resource::ImageDecodingMode> {
    LiveElementInputData(object)
        .map(|data| data.map(|data| data.image_decoding_mode).unwrap_or_default())
}

pub(crate) fn LiveVideoFrameResourceId(
    object: &LayoutObject,
) -> Option<Option<image_resource::ImageId>> {
    LiveElementInputData(object).map(|data| data.and_then(|data| data.video_frame_resource_id))
}

pub(crate) fn LiveFocusedImageMapArea(
    object: &LayoutObject,
) -> Option<Option<&crate::internal::layout_input::FocusedImageMapArea>> {
    LiveElementInputData(object)
        .map(|data| data.and_then(|data| data.focused_image_map_area.as_ref()))
}

/// SVGViewportContainerElement::HasEmptyViewBox().
pub(crate) fn LiveSvgViewBoxIsEmpty(object: &LayoutObject) -> Option<bool> {
    LiveElementInputData(object).map(|data| {
        data.and_then(|data| data.svg_view_box)
            .is_some_and(|view_box| view_box.width <= 0.0 || view_box.height <= 0.0)
    })
}

/// The enclosing text control (form control) host of the source Node.
pub(crate) fn LiveTextControlHost(object: &LayoutObject) -> Option<Option<u64>> {
    let mut ancestor = LiveSourceNode(object)? as *const Node;
    while !ancestor.is_null() {
        let node = unsafe { &*ancestor };
        if node.InputKind() == NodeKind::kFormControl {
            return Some(Some(node.InputId()));
        }
        ancestor = node.parentNode().cast();
    }
    Some(None)
}

/// Source: BoxPaintInvalidator's previous geometry, kept on the LayoutBox
/// (LayoutBox::PreviousSize / PreviousPhysicalContentBoxRect /
/// PreviousSelfVisualOverflowRect / PreviousScrollableOverflowRect). The
/// layout commit saves them after the artifact commits
/// (SavePreviousBoxGeometriesIfNeeded), so PrePaint reads the previous values.
/// `Some(None)` for objects that are not boxes. A replaced box's previous
/// content rect uses its natural dimensions, which SVG roots answer through
/// the resident SVG queries outside a pass.
pub(crate) fn LivePreviousBoxGeometry(
    object: &LayoutObject,
) -> Option<Option<crate::fragment_tree::PreviousBoxGeometry>> {
    Some(ExportPreviousBoxGeometry(object))
}

/// LayoutBox previous geometry including a LayoutReplaced's previous
/// replaced content rect; valid inside a layout pass.
pub(crate) fn ExportPreviousBoxGeometry(
    object: &LayoutObject,
) -> Option<crate::fragment_tree::PreviousBoxGeometry> {
    let object_ptr = object as *const LayoutObject;
    let box_ = unsafe { DynamicTo::<LayoutBox>(object_ptr).as_ref() }?;
    let previous_size = box_.PreviousSize();
    let replaced = DynamicTo::<crate::internal::layout_replaced::LayoutReplaced>(object_ptr);
    Some(crate::fragment_tree::PreviousBoxGeometry {
        size: Size {
            width: Number(previous_size.width),
            height: Number(previous_size.height),
        },
        content_box: ExportInkRect(box_.PreviousPhysicalContentBoxRect()),
        self_visual_overflow: ExportInkRect(box_.PreviousSelfVisualOverflowRect()),
        scrollable_overflow: ExportInkRect(box_.PreviousScrollableOverflowRect()),
        replaced_content: (!replaced.is_null()).then(|| {
            ExportInkRect(
                unsafe { &*replaced }
                    .ReplacedContentRectFrom(&box_.PreviousPhysicalContentBoxRect()),
            )
        }),
    })
}

/// LayoutView::BackgroundRect(); None for other objects.
pub(crate) fn LiveViewBackgroundRect(object: &LayoutObject) -> Option<FragmentPaintRect> {
    unsafe { DynamicTo::<LayoutView>(object as *const LayoutObject).as_ref() }
        .map(|view| ExportInkRect(view.BackgroundRect()))
}

/// LayoutView::PreviousBackgroundRect(); None for other objects.
pub(crate) fn LivePreviousViewBackgroundRect(object: &LayoutObject) -> Option<FragmentPaintRect> {
    unsafe { DynamicTo::<LayoutView>(object as *const LayoutObject).as_ref() }
        .map(|view| ExportInkRect(view.PreviousBackgroundRect()))
}

/// DisplayItemClient::VisualRectOutsetForRasterEffects with its LayoutBox
/// (layout_box.cc:4099-4108) and LayoutSVGShape (layout_svg_shape.cc:664-674)
/// overrides.
pub(crate) fn LiveRasterEffectOutset(
    object: &LayoutObject,
) -> crate::fragment_tree::RasterEffectOutset {
    let object_ptr = object as *const LayoutObject;
    if let Some(box_) = unsafe { DynamicTo::<LayoutBox>(object_ptr).as_ref() } {
        return box_.VisualRectOutsetForRasterEffects();
    }
    if object.IsSVGShape() {
        let style = object.StyleRef();
        return if !style.HasVisibleStroke() {
            unsafe { &*(object_ptr as *const DisplayItemClient) }
                .VisualRectOutsetForRasterEffects()
        } else if style.CapStyle() == foundation::LineCap::kButtCap {
            crate::fragment_tree::RasterEffectOutset::kHalfPixel
        } else {
            crate::fragment_tree::RasterEffectOutset::kWholePixel
        };
    }
    unsafe { &*(object_ptr as *const DisplayItemClient) }.VisualRectOutsetForRasterEffects()
}

/// The source Node id of the nearest LayoutObject ancestor that has a Node:
/// the logical (LayoutObject-tree) parent PrePaint and the client-rect walk
/// reparent out-of-flow occurrences under.
pub(crate) fn LiveLogicalParentNodeId(object: &LayoutObject) -> Option<u64> {
    // Anonymous objects have no logical position of their own.
    LiveSourceNode(object)?;
    let mut parent = object.Parent();
    while let Some(current) = unsafe { parent.as_ref() } {
        if let Some(node) = LiveSourceNode(current) {
            return Some(node.InputId());
        }
        parent = current.Parent();
    }
    None
}

/// Source: core/paint/paint_layer_stacking_node.cc:187-254. The exact
/// comparison projection ForAllChildrenSortedByOrder() uses for a layer
/// whose LayoutObject (or its parent) is a flex/grid item: the shared
/// flex/grid parent's identity and the item's `order`. Chromium compares the
/// parent by LayoutObject identity (cc:218-236); Rechrom carries it as the
/// FragmentData UniqueObjectId, so the id is allocated on that owner first
/// (UniqueId() assumes EnsureId() has run).
pub(crate) fn LivePaintLayerOrder(object: &LayoutObject) -> (Option<u64>, i32) {
    let object_ptr = object as *const LayoutObject as *mut LayoutObject;
    let box_model = DynamicTo::<LayoutBoxModelObject>(object_ptr);
    if box_model.is_null() || unsafe { &*box_model }.Layer().is_null() {
        return (None, 0);
    }
    let parent = object.Parent();
    let is_flex_or_grid =
        |candidate: *mut LayoutObject| {
            let candidate = unsafe { &*candidate };
            candidate.IsFlexibleBox() || candidate.IsLayoutGridOrGridLanes()
        };
    let order_ancestor = if !parent.is_null() && is_flex_or_grid(parent) {
        object_ptr
    } else if !parent.is_null() {
        let grandparent = unsafe { &*parent }.Parent();
        if !grandparent.is_null() && is_flex_or_grid(grandparent) {
            parent
        } else {
            return (None, 0);
        }
    } else {
        return (None, 0);
    };
    let order_parent = unsafe { &*order_ancestor }.Parent();
    if order_parent.is_null() {
        return (None, 0);
    }
    unsafe { &mut *order_parent }
        .GetMutableForPainting()
        .EnsureId();
    let order = if order_ancestor == object_ptr
        && unsafe { &*order_ancestor }.IsOutOfFlowPositioned()
    {
        0
    } else {
        unsafe { &*order_ancestor }.StyleRef().Order()
    };
    (Some(unsafe { &*order_parent }.UniqueId()), order)
}

/// FormControlPaintData from the control element and its style (the
/// LayoutTheme / ThemePainter inputs Chromium reads from the Element).
pub(crate) fn LiveFormControl(
    object: &LayoutObject,
    layout_style: &StyleComputedStyle,
) -> Option<Arc<FormControlPaintData>> {
    let element_data = LiveElementInputData(object)?;
    let appearance = layout_style.EffectiveAppearance();
    if appearance != AppearanceValue::kNone
        || element_data.is_some_and(|element_data| element_data.control_focused)
    {
        let mut control = FormControlPaintData {
            appearance,
            ..FormControlPaintData::default()
        };
        if let Some(element_data) = element_data {
            control.r#type = element_data.form_control_type;
            control.checked = element_data.control_checked;
            control.indeterminate = element_data.control_indeterminate;
            control.is_progress = element_data.control_is_progress;
            control.disabled = element_data.control_disabled;
            control.read_only = element_data.control_read_only;
            control.hovered = element_data.control_hovered;
            control.active = element_data.control_active;
            control.focused = element_data.control_focused;
            // OutlinePainter uses the computed outline, including the UA
            // :focus-visible rule; focus alone must not override outline:none
            // or add an auto ring alongside an author's non-auto outline.
            control.auto_focus_ring = element_data.control_focused
                && appearance == AppearanceValue::kNone
                && layout_style.HasOutline()
                && layout_style.OutlineStyleIsAuto();
            control.autofilled = element_data.autofill_state != AutofillState::kNotFilled;
            control.value_ratio = element_data
                .control_value_ratio
                .or(element_data.range_value_ratio);
        }
        if !layout_style.AccentColor().IsAutoColor() {
            control.accent_color = crate::internal::boundary::paint_style_export::StyleColor(
                layout_style,
                layout_style.AccentColor().ToStyleColor(),
            );
        }
        Some(Arc::new(control))
    } else {
        None
    }
}

/// The scroll offset PaintPropertyTreeBuilder applies: the source element's
/// current, already-clamped offset; generated scroll owners without a source
/// Node read their own PaintLayerScrollableArea.
pub(crate) fn LiveScrollOffset(object: &LayoutObject) -> Offset {
    match LiveElementInputData(object) {
        Some(Some(data)) => data.scroll_offset,
        Some(None) => Offset::default(),
        None => {
            let layout_box = DynamicTo::<LayoutBox>(object as *const LayoutObject);
            let area = unsafe { layout_box.as_ref() }
                .map_or(std::ptr::null_mut(), |box_| box_.GetScrollableArea());
            unsafe { area.as_ref() }.map_or(Offset::default(), |area| {
                let current = area.GetScrollOffset();
                Offset {
                    x: current.x() as f64,
                    y: current.y() as f64,
                }
            })
        }
    }
}

/// TextCombinePainter's inputs for an anonymous LayoutTextCombine: the
/// compressed-text scale and spacing, and the parent inline's emphasis and
/// font size, which it paints after restoring the transform
/// (core/paint/text_combine_painter.cc:35-48).
pub(crate) fn LiveTextCombine(
    object: &LayoutObject,
) -> Option<crate::fragment_tree::TextCombinePaintData> {
    let text_combine =
        unsafe { DynamicTo::<LayoutTextCombine>(object as *const LayoutObject).as_ref() }?;
    let parent_style = unsafe { &*text_combine.Parent() }.StyleRef();
    let scale_x = text_combine.ScaleXForInline().map(f64::from);
    let inline_spacing = scale_x.map_or(0.0, |_| f64::from(text_combine.ComputeInlineSpacing()));
    let wrapper_font = unsafe { &*text_combine.StyleRef().GetFont() };
    let primary_font = wrapper_font.PrimaryFont();
    let text_top_offset = if primary_font.is_null() {
        0.0
    } else {
        let primary_font = unsafe { &*primary_font };
        Number(LayoutUnit::from_f32(
            primary_font.GetFontMetrics().FloatAscent() - primary_font.InternalLeading() / 2.0,
        ))
    };
    Some(crate::fragment_tree::TextCombinePaintData {
        scale_x,
        inline_spacing,
        text_top_offset,
        synthetic_oblique: text_combine.UsingSyntheticOblique(),
        font_size: unsafe { &*parent_style.GetFont() }
            .GetFontDescription()
            .ComputedSize() as f64,
        emphasis: ExportTextCombineEmphasis(parent_style),
    })
}

// SVG layout geometry. Chromium's SVG painters read these members of the
// LayoutSVG objects (LayoutSVGShape path, LocalSVGTransform, viewport). The
// SVG layout package keeps them on its owners and answers through its
// queries, which stay callable after the pass (ResidentSvgSupport). `None`
// means the object has no source Node, for which nothing was exported.

pub(crate) fn LiveSvgShape(
    object: &LayoutObject,
) -> Option<Option<&Arc<crate::internal::layout_input::SvgShapeData>>> {
    let data = LiveElementInputData(object)?;
    if data.is_none() || !object.IsSVGShape() {
        return Some(None);
    }
    Some(
        crate::internal::layout_pass_scope::LayoutPassScope::ResidentSvgSupport()
            .retained_shape
            .and_then(|query| query(object)),
    )
}

pub(crate) fn LiveSvgPaintGeometry(
    object: &LayoutObject,
) -> Option<(
    Option<crate::internal::layout_input::TransformMatrix>,
    Option<crate::fragment_tree::SvgViewportGeometry>,
    Option<FragmentPaintRect>,
)> {
    LiveSourceNode(object)?;
    if !object.IsSVG() {
        return Some((None, None, None));
    }
    let Some(query) =
        crate::internal::layout_pass_scope::LayoutPassScope::ResidentSvgSupport().paint_geometry
    else {
        return Some((None, None, None));
    };
    let (transform, viewport, visual_rect) = query(object);
    Some((
        transform,
        viewport,
        visual_rect.map(|rect| FragmentPaintRect {
            offset: Offset {
                x: rect.x() as f64,
                y: rect.y() as f64,
            },
            size: Size {
                width: rect.width() as f64,
                height: rect.height() as f64,
            },
        }),
    ))
}

/// The authored viewBox of the element, or for an SVG child the nearest
/// viewport's resolved size (SVGResources::ReferenceBoxForEffects(kViewBox)).
pub(crate) fn LiveSvgViewBox(
    object: &LayoutObject,
) -> Option<Option<crate::internal::layout_input::SvgViewBoxData>> {
    let data = LiveElementInputData(object)?;
    if object.IsSVGChild() {
        if let Some(resolve) =
            crate::internal::layout_pass_scope::LayoutPassScope::ResidentSvgSupport()
                .viewport_size
        {
            let size = resolve(object);
            return Some(Some(crate::internal::layout_input::SvgViewBoxData {
                width: size.width() as f64,
                height: size.height() as f64,
                ..Default::default()
            }));
        }
    }
    Some(data.and_then(|data| data.svg_view_box))
}

// Ink overflow read from the fragment itself, as Chromium's painters read
// PhysicalBoxFragment / FragmentItem InkOverflowRect (stage 3 of the
// borrowed-view boundary). Order: self, contents, ink, filtered ink.

/// A box fragment's computed ink overflow; all `None` until computed.
pub(crate) fn LiveBoxInkOverflow(fragment: &PhysicalBoxFragment) -> [Option<FragmentPaintRect>; 4] {
    if !fragment.IsInkOverflowComputed() {
        return [None; 4];
    }
    let ink_overflow = fragment.InkOverflowRect();
    let box_model = DynamicTo::<LayoutBoxModelObject>(fragment.GetLayoutObject());
    // Source: core/paint/box_fragment_painter.cc:2122-2133.
    let filtered = unsafe { box_model.as_ref() }
        .map(|box_model| ExportInkRect(box_model.ApplyFiltersToRect(&ink_overflow)));
    [
        Some(ExportInkRect(fragment.SelfInkOverflowRect())),
        Some(ExportInkRect(fragment.ContentsInkOverflowRect())),
        Some(ExportInkRect(ink_overflow)),
        filtered,
    ]
}

/// An inline item's ink overflow (FragmentItem::SelfInkOverflowRect /
/// InkOverflowRect). Chromium never paints an item whose ink overflow is
/// uncomputed: PaintLayer::UpdateDescendantDependentFlags runs
/// RecalcVisualOverflow before PrePaint (paint_layer.cc:386-514), and the
/// layout lifecycle marks the painting layer whenever ink is invalidated.
pub(crate) fn LiveItemInkOverflow(item: &FragmentItem) -> [Option<FragmentPaintRect>; 4] {
    if let Some(box_fragment) = unsafe { item.BoxFragment().as_ref() } {
        return LiveBoxInkOverflow(box_fragment);
    }
    [
        Some(ExportInkRect(item.SelfInkOverflowRect())),
        None,
        Some(ExportInkRect(item.InkOverflowRect())),
        None,
    ]
}

/// A text item's shaped glyph ink in item-local physical coordinates: the
/// glyph stage of InkOverflow::ComputeTextInkOverflow (ink_overflow.cc:
/// 439-489), without decoration, emphasis or shadow overflow. `None` for SVG
/// text, symbol markers and items without a ShapeResult.
pub(crate) fn LiveTextGlyphInk(
    item: &FragmentItem,
    items: &crate::fragment_items::FragmentItems,
) -> Option<FragmentPaintRect> {
    if !item.IsText() || item.IsSvgText() || item.IsSymbolMarker() {
        return None;
    }
    let text_info = item.TextPaintInfo(items);
    if text_info.shape_result.is_null() {
        return None;
    }
    let style = item.Style();
    let logical = crate::internal::ink_overflow::InkOverflow::ComputeTextGlyphInkOverflow(
        &item.GetUsedFont(),
        &text_info,
        style,
    );
    let converter = WritingModeConverter::new(
        foundation::WritingDirectionMode::new(
            style.GetWritingMode(),
            foundation::TextDirection::kLtr,
        ),
        item.Size(),
    );
    Some(ExportInkRect(converter.ToPhysicalRect(logical)))
}

/// Box-fragment geometry facts Chromium's box painter and hit testing read
/// from the PhysicalBoxFragment / LayoutBox: whether it is an inline box,
/// the hit-test overflow clip rect (only when `clips` — the box clips
/// overflow), the pixel-snapped scrolled content offset, and the
/// LayoutBox's self visual overflow (box_painter.cc:124-129).
pub(crate) fn LiveBoxGeometryFacts(
    fragment: &PhysicalBoxFragment,
    clips: bool,
) -> (bool, Option<FragmentPaintRect>, Offset, Option<FragmentPaintRect>) {
    let hit_test_overflow_clip = clips.then(|| {
        ExportInkRect(fragment.OverflowClipRect(
            foundation::OverlayScrollbarClipBehavior::kExcludeOverlayScrollbarSizeForHitTesting,
        ))
    });
    let object = fragment.GetLayoutObject();
    let scrolled = if object.is_null() {
        Offset::default()
    } else {
        let scrolled = fragment.PixelSnappedScrolledContentOffset();
        Offset {
            x: scrolled.x() as f64,
            y: scrolled.y() as f64,
        }
    };
    let self_visual_overflow = unsafe { DynamicTo::<LayoutBox>(object).as_ref() }
        .map(|layout_box| ExportInkRect(layout_box.SelfVisualOverflowRect()));
    (
        fragment.IsInlineBox(),
        hit_test_overflow_clip,
        scrolled,
        self_visual_overflow,
    )
}

/// Source: core/paint/box_fragment_painter.cc:2492-2510 and
/// fragment_item.cc:1090-1109. A text item's point-test rectangle relative to
/// the item: LayoutTextCombine's compressed bounds when present, otherwise
/// FragmentItem's pixel-snapped ordinary bounds.
pub(crate) fn LiveTextHitTestBounds(item: &FragmentItem) -> Option<FragmentPaintRect> {
    if !item.IsText() || item.IsSvgText() {
        return None;
    }
    let object = item.GetLayoutObject();
    let text_combine = if object.is_null() {
        std::ptr::null_mut()
    } else {
        DynamicTo::<LayoutTextCombine>(unsafe { &*object }.Parent())
    };
    let mut bounds = if text_combine.is_null() {
        item.ComputeTextBoundsRectForHitTest(PhysicalOffset::default(), false)
    } else {
        unsafe { &*text_combine }.ComputeTextBoundsRectForHitTest(item, &PhysicalOffset::default())
    };
    bounds.offset -= item.RectInContainerFragment().offset;
    Some(FragmentPaintRect {
        offset: Offset {
            x: Number(bounds.offset.left),
            y: Number(bounds.offset.top),
        },
        size: Size {
            width: Number(bounds.size.width),
            height: Number(bounds.size.height),
        },
    })
}
