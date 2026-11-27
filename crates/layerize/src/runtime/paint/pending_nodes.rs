//! Call-scoped property-node reads. Algorithmic mapping/merging stays in the
//! canonical translated Paint stems; these handles retain actual node Arcs.
use super::*;
use crate::adapter::{
    input_types::ElementId,
    pending_layer::PendingEffectNode,
    property_types::{BlendMode, SubtreeCaptureId, ViewTransitionElementResourceId},
};
use crate::{
    adapter::{
        geometry::{RRectF, RectF, Vector2dF},
        pending_layer::PendingClipNode,
    },
    runtime::types::PaintSkiaBackend,
};
fn element(id: foundation::CompositorElementId) -> ElementId {
    let id = id.GetInternalValue();
    if id == 0 {
        ElementId::default()
    } else {
        ElementId::new(id)
    }
}
fn blend(n: &paint_property_tree::EffectPaintPropertyNodeState) -> BlendMode {
    if n.is_mask {
        return BlendMode::kDstIn;
    }
    use paint_property_tree::PaintBlendMode::*;
    match n.blend_mode {
        kNormal => BlendMode::kSrcOver,
        kMultiply => BlendMode::kMultiply,
        kScreen => BlendMode::kScreen,
        kOverlay => BlendMode::kOverlay,
        kDarken => BlendMode::kDarken,
        kLighten => BlendMode::kLighten,
        kColorDodge => BlendMode::kColorDodge,
        kColorBurn => BlendMode::kColorBurn,
        kHardLight => BlendMode::kHardLight,
        kSoftLight => BlendMode::kSoftLight,
        kDifference => BlendMode::kDifference,
        kExclusion => BlendMode::kExclusion,
        kHue => BlendMode::kHue,
        kSaturation => BlendMode::kSaturation,
        kColor => BlendMode::kColor,
        kLuminosity => BlendMode::kLuminosity,
        kPlusLighter => BlendMode::kPlus,
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/transform_paint_property_node.h:185-435.
// Every field is projected from its actual Paint owner. Transform-cache queries
// enter the canonical cache rather than materializing an ancestry snapshot.
impl<R: PaintSkiaBackend> crate::adapter::pending_layer::PendingTransformNode<PaintPropertyChange>
    for TransformNode<R>
{
    type Matrix = crate::runtime::geometry::Transform<R>;
    fn CcNodeId(&self, s: i32) -> i32 {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.cc_node_id(a, s))
    }
    fn SetCcNodeId(&self, s: i32, id: i32) {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.set_cc_node_id(a, s, id))
    }
    fn IsRoot(&self) -> bool {
        read(|a| {
            self.node().state(a).parent.is_none()
        })
    }
    fn UnaliasedParent(&self) -> Option<Self> {
        self.parent_handle()
    }
    fn ParentUnalias(&self) -> Self {
        self.UnaliasedParent().expect("source parent")
    }
    fn IsIdentityOr2dTranslation(&self) -> bool {
        read(|a| {
            self.node().IsIdentityOr2dTranslation(a)
        })
    }
    fn HasScrollNode(&self) -> bool {
        read(|a| {
            self.node().ScrollNode(a).is_some()
        })
    }
    fn HasDirectCompositingReasonsOtherThan3dTransform(&self) -> bool {
        read(|a| {
            self.node()
                .HasDirectCompositingReasonsOtherThan3dTransform(a)
        })
    }
    fn FlattensInheritedTransformSameAsParent(&self) -> bool {
        read(|a| {
            let node = self.node();
            node.Parent(a).is_none_or(|p| {
                p.FlattensInheritedTransform(a) == node.FlattensInheritedTransform(a)
            })
        })
    }
    fn BackfaceVisibilitySameAsParent(&self) -> bool {
        // The fields are read under one proof; only the GeometryMapper cache
        // fallback needs the retained parent handle.
        let parent = read(|a| {
            let node = self.node();
            let Some(parent) = node.Parent(a) else {
                return Ok(());
            };
            if node.DelegatesToParentForBackface(a)
                || node.state(a).backface_visibility == parent.state(a).backface_visibility
            {
                return Ok(());
            }
            Err(Self::from_arc(parent.clone()))
        });
        match parent {
            Ok(()) => true,
            Err(parent) => self.IsBackfaceHidden() == parent.IsBackfaceHidden(),
        }
    }
    fn HasStickyConstraint(&self) -> bool {
        read(|a| {
            self.node().GetStickyConstraint(a).is_some()
        })
    }
    fn HasAnchorPositionScrollData(&self) -> bool {
        read(|a| {
            self.node().GetAnchorPositionScrollData(a).is_some()
        })
    }
    fn IsAffectedByOuterViewportBoundsDelta(&self) -> bool {
        read(|a| {
            self.node().IsAffectedByOuterViewportBoundsDelta(a)
        })
    }
    fn IsAffectedBySafeAreaBottom(&self) -> bool {
        read(|a| {
            self.node().IsAffectedBySafeAreaBottom(a)
        })
    }
    fn IsInSubtreeOfPageScale(&self) -> bool {
        read(|a| {
            self.node().IsInSubtreeOfPageScale(a)
        })
    }
    fn RequiresCompositingForFixedToViewport(&self) -> bool {
        read(|a| {
            self.node().RequiresCompositingForFixedToViewport(a)
        })
    }
    fn FlattensInheritedTransform(&self) -> bool {
        read(|a| {
            self.node().FlattensInheritedTransform(a)
        })
    }
    fn RenderingContextId(&self) -> i32 {
        read(|a| {
            self.node().RenderingContextId(a) as i32
        })
    }
    fn DelegatesToParentForBackface(&self) -> bool {
        read(|a| {
            self.node().DelegatesToParentForBackface(a)
        })
    }
    fn RequiresCompositingForWillChangeTransform(&self) -> bool {
        read(|a| {
            self.node().RequiresCompositingForWillChangeTransform(a)
        })
    }
    fn IsForSVGChild(&self) -> bool {
        read(|a| {
            self.node().IsForSVGChild(a)
        })
    }
    fn ScrollParentScrollTranslation(&self) -> Option<Self> {
        read(|a| {
            self.node()
                .ScrollParentScrollTranslation(a)
                .map(|n| Self::from_arc(n.clone()))
        })
    }
    fn PropertyTreeNearestScrollTranslationNode(&self) -> Self {
        crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache(self,|c|c.nearest_scroll_translation())
    }
    fn GetStickyConstraint(
        &self,
    ) -> Option<crate::cc::trees::sticky_position_constraint::StickyPositionConstraint> {
        read(|a| {
            self.node().GetStickyConstraint(a).map(|v| {
                let r = |p: paint_model::PaintRect| {
                    RectF::new(p.x as f32, p.y as f32, p.width as f32, p.height as f32)
                };
                crate::cc::trees::sticky_position_constraint::StickyPositionConstraint {
                    is_anchored_left: v.anchored[0],
                    is_anchored_right: v.anchored[1],
                    is_anchored_top: v.anchored[2],
                    is_anchored_bottom: v.anchored[3],
                    left_offset: v.offsets[0],
                    right_offset: v.offsets[1],
                    top_offset: v.offsets[2],
                    bottom_offset: v.offsets[3],
                    constraint_box_rect: r(v.constraint_box_rect),
                    scroll_container_relative_sticky_box_rect: r(v.sticky_box_rect),
                    scroll_container_relative_containing_block_rect: r(v.containing_block_rect),
                    pixel_snap_offset: Vector2dF::new(v.pixel_snap_offset[0], v.pixel_snap_offset[1]),
                    x_scroll_ancestor_element_id: element(v.x_scroll_ancestor),
                    y_scroll_ancestor_element_id: element(v.y_scroll_ancestor),
                    nearest_element_shifting_sticky_box: element(v.nearest_shifting_sticky_box),
                    nearest_element_shifting_containing_block: element(
                        v.nearest_shifting_containing_block,
                    ),
                }
            })
        })
    }
    fn GetAnchorPositionScrollData(
        &self,
    ) -> Option<crate::cc::trees::property_tree::AnchorPositionScrollData> {
        read(|a| {
            self.node().GetAnchorPositionScrollData(a).map(|v| {
                crate::cc::trees::property_tree::AnchorPositionScrollData {
                    adjustment_container_ids: v
                        .adjustment_container_ids
                        .iter()
                        .copied()
                        .map(element)
                        .collect(),
                    accumulated_scroll_origin: crate::adapter::geometry::Vector2d::new(
                        v.accumulated_scroll_origin[0],
                        v.accumulated_scroll_origin[1],
                    ),
                    needs_scroll_adjustment_in_x: v.needs_scroll_adjustment_in_x,
                    needs_scroll_adjustment_in_y: v.needs_scroll_adjustment_in_y,
                }
            })
        })
    }
    fn GetCompositorElementId(&self) -> ElementId {
        read(|a| {
            element(self.node().GetCompositorElementId(a))
        })
    }
    fn GetVisibleFrameElementId(&self) -> ElementId {
        read(|a| {
            element(self.node().GetVisibleFrameElementId(a))
        })
    }
    fn IsFramePaintOffsetTranslation(&self) -> bool {
        read(|a| {
            self.node().IsFramePaintOffsetTranslation(a)
        })
    }
    fn IsBackfaceHidden(&self) -> bool {
        crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache(self,|c|c.is_backface_hidden())
    }
    fn HasActiveTransformAnimation(&self) -> bool {
        read(|a| {
            self.node().HasActiveTransformAnimation(a)
        })
    }
    fn TransformAnimationIsAxisAligned(&self) -> bool {
        read(|a| {
            self.node().TransformAnimationIsAxisAligned(a)
        })
    }
    fn PropertyTreeLowestCommonAncestor(&self, other: &Self) -> Self {
        self.lowest_common_ancestor(other)
    }
    fn Get2dTranslation(&self) -> Vector2dF {
        read(|a| {
            let translation = self.node().Get2dTranslation(a);
            Vector2dF::new(translation[0], translation[1])
        })
    }
    fn Matrix(&self) -> Self::Matrix {
        crate::adapter::paint_geometry::GeometryMapperNode::Matrix(self)
    }
    fn Origin(&self) -> crate::adapter::geometry::Point3F {
        read(|a| {
            let o = self.node().Origin(a);
            crate::adapter::geometry::Point3F::new(o[0] as f32, o[1] as f32, o[2] as f32)
        })
    }
    fn NodeChanged(&self) -> PaintPropertyChange {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.node_changed(a))
    }
    fn ScrollCompositorElementId(&self) -> ElementId {
        element(read(|a| {
            self.node().ScrollNode(a).expect("source ScrollNode").GetCompositorElementId(a)
        }))
    }
    fn ScrollContainerRect(&self) -> crate::adapter::geometry::Rect {
        crate::adapter::paint_geometry::GeometryMapperNode::ScrollContainerRect(self)
    }
}
impl<R: PaintSkiaBackend> PendingEffectNode<TransformNode<R>, usize> for EffectNode<R> {
    type Filters = paint_model::cc::paint::filter_operations::FilterOperations;
    type Path = R::Path;
    type PaintPropertyChange = paint_property_tree::PaintPropertyChangeType;
    type ClipNode = ClipNode<R>;
    fn CcNodeId(&self, s: i32) -> i32 {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.cc_node_id(a, s))
    }
    fn SetCcNodeId(&self, s: i32, id: i32) {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.set_cc_node_id(a, s, id))
    }
    fn IsRoot(&self) -> bool {
        read(|a| {
            self.node().state(a).parent.is_none()
        })
    }
    fn Identity(&self) -> usize {
        Arc::as_ptr(&self.node().lifecycle.identity) as usize
    }
    fn ParentUnalias(&self) -> Self {
        self.parent_handle().expect("source effect parent")
    }
    fn LocalTransformSpaceUnalias(&self) -> TransformNode<R> {
        read(|a| {
            super::TransformNode::from_arc(self.node().LocalTransformSpace(a).clone())
        })
    }
    fn BlendMode(&self) -> BlendMode {
        read(|a| blend(self.node().state(a)))
    }
    fn Opacity(&self) -> f32 {
        read(|a| {
            self.node().state(a).opacity
        })
    }
    fn HasActiveOpacityAnimation(&self) -> bool {
        read(|a| {
            self.node().HasActiveOpacityAnimation(a)
        })
    }
    fn HasFilter(&self) -> bool {
        read(|a| {
            self.node().Filter(a).is_some()
        })
    }
    fn RequiresCompositingForWillChangeFilter(&self) -> bool {
        read(|a| {
            self.node().RequiresCompositingForWillChangeFilter(a)
        })
    }
    fn HasActiveFilterAnimation(&self) -> bool {
        read(|a| {
            self.node().HasActiveFilterAnimation(a)
        })
    }
    fn HasBackdropFilter(&self) -> bool {
        read(|a| {
            self.node().BackdropFilter(a).is_some()
        })
    }
    fn RequiresCompositingForWillChangeBackdropFilter(&self) -> bool {
        read(|a| {
            self.node().RequiresCompositingForWillChangeBackdropFilter(a)
        })
    }
    fn HasActiveBackdropFilterAnimation(&self) -> bool {
        read(|a| {
            self.node().HasActiveBackdropFilterAnimation(a)
        })
    }
    fn HasValidViewTransitionElementResourceId(&self) -> bool {
        read(|a| {
            self.node().HasValidViewTransitionElementResourceId(a)
        })
    }
    fn RequiresCompositingForUnboundedElement(&self) -> bool {
        read(|a| {
            self.node().RequiresCompositingForUnboundedElement(a)
        })
    }
    fn FlattensAtLeafOf3DScene(&self) -> bool {
        read(|a| {
            self.node().FlattensAtLeafOf3DScene(a)
        })
    }
    fn HasElementCaptureId(&self) -> bool {
        read(|a| {
            self.node().HasElementCaptureId(a)
        })
    }
    fn LowestCommonAncestor(&self, other: &Self) -> Self {
        self.lowest_common_ancestor(other)
    }
    // Source: effect_paint_property_node.h:355-385.
    fn MayHaveBackdropEffect(&self) -> bool {
        read(|a| {
            self.node().MayHaveBackdropEffect(a)
        })
    }
    fn OutputClip(&self) -> Option<ClipNode<R>> {
        read(|a| {
            self.node().state(a)
                .output_clip
                .as_ref()
                .map(|c| super::ClipNode::from_arc(c.clone()))
        })
    }
    fn GetCompositorElementId(&self) -> ElementId {
        read(|a| {
            element(self.node().state(a).compositor.compositor_element_id)
        })
    }
    fn NeedsEffectFor2DScaleTransform(&self) -> bool {
        read(|a| {
            self.node().NeedsEffectFor2DScaleTransform(a)
        })
    }
    fn FilterOperations(&self) -> Option<Self::Filters> {
        read(|a| {
            self.node().Filter(a).is_some().then(|| {
                self.node().state(a)
                    .filters
                    .canonical()
                    .expect("validated PaintInput native filter")
                    .clone()
            })
        })
    }
    fn BackdropFilterOperations(&self) -> Option<Self::Filters> {
        read(|a| {
            self.node().BackdropFilter(a).is_some().then(|| {
                self.node().state(a)
                    .backdrop_filters
                    .canonical()
                    .expect("validated PaintInput native backdrop filter")
                    .clone()
            })
        })
    }
    fn PropertyTreeBackdropFilterBounds(&self) -> Option<Self::Path> {
        read(|a| {
            self.node().state(a)
                .backdrop_filter_bounds
                .as_ref()
                .map(|path| R::FromPaintPath(path.as_ref()))
        })
    }
    fn BackdropMaskElementId(&self) -> ElementId {
        read(|a| {
            element(self.node().state(a).compositor.backdrop_mask_element_id)
        })
    }
    fn NodeChanged(&self) -> Self::PaintPropertyChange {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.node_changed(a))
    }
    fn ViewTransitionElementResourceId(&self) -> ViewTransitionElementResourceId {
        read(|a| {
            match self.node().state(a).compositor.view_transition_element_resource_id {
                Some((token, local_id, for_scope_snapshot)) => ViewTransitionElementResourceId {
                    transition_token: Some(token),
                    local_id,
                    for_scope_snapshot,
                },
                None => ViewTransitionElementResourceId::default(),
            }
        })
    }
    fn SubtreeCaptureId(&self) -> SubtreeCaptureId {
        read(|a| {
            SubtreeCaptureId {
                token: self.node().state(a).compositor.restriction_target_id,
            }
        })
    }
}
impl<R: PaintSkiaBackend> PendingClipNode<TransformNode<R>, usize> for ClipNode<R> {
    type EffectNode = EffectNode<R>;
    fn CcNodeId(&self, sequence: i32) -> i32 {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.cc_node_id(a, sequence))
    }
    fn SetCcNodeId(&self, sequence: i32, id: i32) {
        paint_model::property_cell::with_access(|a| self.node().lifecycle.set_cc_node_id(a, sequence, id))
    }
    fn IsRoot(&self) -> bool {
        read(|a| self.node().Parent(a).is_none())
    }
    fn Identity(&self) -> usize {
        Arc::as_ptr(&self.node().lifecycle.identity) as usize
    }
    fn ParentUnalias(&self) -> Self {
        self.parent_handle().expect("source clip parent")
    }
    fn LocalTransformSpaceUnalias(&self) -> TransformNode<R> {
        read(|a| super::TransformNode::from_arc(self.node().LocalTransformSpace(a).clone()))
    }
    fn PaintClipRectRect(&self) -> RectF {
        read(|a| self.node().PaintClipRect(a)).map(|r|RectF::new(r.x as f32,r.y as f32,r.width as f32,r.height as f32)).unwrap_or_else(||*crate::third_party::blink::renderer::platform::graphics::paint::float_clip_rect::FloatClipRect::default().Rect())
    }
    fn LowestCommonAncestor(&self, other: &Self) -> Self {
        self.lowest_common_ancestor(other)
    }
    // Source: paint_property_node.h:159-168. Walk actual parent identities.
    fn IsAncestorOf(&self, other: &Self) -> bool {
        self.is_ancestor_of(other)
    }
    fn PaintClipRectIsRounded(&self) -> bool {
        read(|a| self.node().PaintClipRadii(a)).HasRadius()
    }
    fn PaintClipRRect(&self) -> Option<RRectF> {
        let (rect, radii) = read(|a| {
            let node = self.node();
            (node.PaintClipRect(a), node.PaintClipRadii(a))
        });
        rect.map(|r| RRectF {
            rect: RectF::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32),
            corner_radii: [
                radii.top_left,
                radii.top_right,
                radii.bottom_right,
                radii.bottom_left,
            ]
            .map(|r| Vector2dF::new(r.x as f32, r.y as f32)),
        })
    }
    fn HasClipPath(&self) -> bool {
        read(|a| self.node().ClipPath(a).is_some())
    }
    fn PixelMovingFilter(&self) -> Option<EffectNode<R>> {
        read(|a| self.node().PixelMovingFilter(a).map(|e| super::EffectNode::from_arc(e.clone())))
    }
}
