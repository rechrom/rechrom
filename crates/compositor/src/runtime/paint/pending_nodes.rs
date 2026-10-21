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
    runtime::types::CpuSkiaBackend,
};
fn element(id: foundation::CompositorElementId) -> ElementId {
    let id = id.GetInternalValue();
    if id == 0 {
        ElementId::default()
    } else {
        ElementId::new(id)
    }
}
fn blend(n: &native::EffectPaintPropertyNode) -> BlendMode {
    if n.is_mask {
        return BlendMode::kDstIn;
    }
    use native::PaintBlendMode::*;
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
impl<R: CpuSkiaBackend> crate::adapter::pending_layer::PendingTransformNode<PaintPropertyChange>
    for TransformNode<R>
{
    type Matrix = crate::runtime::geometry::Transform<R>;
    fn CcNodeId(&self, s: i32) -> i32 {
        self.node().lifecycle.cc_node_id(s)
    }
    fn SetCcNodeId(&self, s: i32, id: i32) {
        self.node().lifecycle.set_cc_node_id(s, id)
    }
    fn IsRoot(&self) -> bool {
        self.node().parent.is_none()
    }
    fn UnaliasedParent(&self) -> Option<Self> {
        self.node()
            .parent
            .as_ref()
            .map(|n| Self::from_arc(n.clone()))
    }
    fn ParentUnalias(&self) -> Self {
        self.UnaliasedParent().expect("source parent")
    }
    fn IsIdentityOr2dTranslation(&self) -> bool {
        crate::adapter::paint_geometry::TransformCacheNode::IsIdentityOr2dTranslation(self)
    }
    fn HasScrollNode(&self) -> bool {
        self.node().scroll.is_some()
    }
    fn HasDirectCompositingReasonsOtherThan3dTransform(&self) -> bool {
        self.node().direct_compositing_reasons.iter().any(|r| {
            !matches!(
                *r,
                "3DTransform" | "3DScale" | "3DRotate" | "3DTranslate" | "Trivial3DTransform"
            )
        })
    }
    fn FlattensInheritedTransformSameAsParent(&self) -> bool {
        self.node().parent.as_ref().is_none_or(|p| {
            p.flattens_inherited_transform == self.node().flattens_inherited_transform
        })
    }
    fn BackfaceVisibilitySameAsParent(&self) -> bool {
        let Some(parent) = self.UnaliasedParent() else {
            return true;
        };
        self.DelegatesToParentForBackface()
            || self.node().backface_visibility == parent.node().backface_visibility
            || self.IsBackfaceHidden() == parent.IsBackfaceHidden()
    }
    fn HasStickyConstraint(&self) -> bool {
        self.node().compositor.sticky_constraint.is_some()
    }
    fn HasAnchorPositionScrollData(&self) -> bool {
        self.node().compositor.anchor_position_scroll_data.is_some()
    }
    fn IsAffectedByOuterViewportBoundsDelta(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"AffectedByOuterViewportBoundsDelta")
    }
    fn IsAffectedBySafeAreaBottom(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"AffectedBySafeAreaBottom")
    }
    fn IsInSubtreeOfPageScale(&self) -> bool {
        self.node().compositor.in_subtree_of_page_scale
    }
    fn RequiresCompositingForFixedToViewport(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"UndoOverscroll")
    }
    fn FlattensInheritedTransform(&self) -> bool {
        self.node().flattens_inherited_transform
    }
    fn RenderingContextId(&self) -> i32 {
        self.node().rendering_context_id as i32
    }
    fn DelegatesToParentForBackface(&self) -> bool {
        self.node().backface_visibility == native::PaintBackfaceVisibility::Inherited
    }
    fn RequiresCompositingForWillChangeTransform(&self) -> bool {
        self.node().direct_compositing_reasons.iter().any(|r| {
            matches!(
                *r,
                "WillChangeTransform"
                    | "WillChangeScale"
                    | "WillChangeRotate"
                    | "WillChangeTranslate"
            )
        })
    }
    fn IsForSVGChild(&self) -> bool {
        self.node().compositor.is_for_svg_child
    }
    fn ScrollParentScrollTranslation(&self) -> Option<Self> {
        self.node()
            .scroll_parent_scroll_translation
            .as_ref()
            .map(|n| Self::from_arc(n.clone()))
    }
    fn PropertyTreeNearestScrollTranslationNode(&self) -> Self {
        crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache(self,|c|c.nearest_scroll_translation())
    }
    fn GetStickyConstraint(
        &self,
    ) -> Option<crate::cc::trees::sticky_position_constraint::StickyPositionConstraint> {
        self.node().compositor.sticky_constraint.as_ref().map(|v| {
            let r = |p: paint::paint_engine::PaintRect| {
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
    }
    fn GetAnchorPositionScrollData(
        &self,
    ) -> Option<crate::cc::trees::property_tree::AnchorPositionScrollData> {
        self.node()
            .compositor
            .anchor_position_scroll_data
            .as_ref()
            .map(
                |v| crate::cc::trees::property_tree::AnchorPositionScrollData {
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
                },
            )
    }
    fn GetCompositorElementId(&self) -> ElementId {
        element(self.node().compositor.compositor_element_id)
    }
    fn GetVisibleFrameElementId(&self) -> ElementId {
        element(self.node().compositor.visible_frame_element_id)
    }
    fn IsFramePaintOffsetTranslation(&self) -> bool {
        self.node().compositor.is_frame_paint_offset_translation
    }
    fn IsBackfaceHidden(&self) -> bool {
        crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache(self,|c|c.is_backface_hidden())
    }
    fn HasActiveTransformAnimation(&self) -> bool {
        crate::adapter::paint_geometry::TransformCacheNode::HasActiveTransformAnimation(self)
    }
    fn TransformAnimationIsAxisAligned(&self) -> bool {
        self.node().compositor.animation_is_axis_aligned
    }
    fn PropertyTreeLowestCommonAncestor(&self, other: &Self) -> Self {
        Self::from_arc(paint::property_node_geometry::LowestCommonAncestor(
            self.as_arc(),
            other.as_arc(),
        ))
    }
    fn Get2dTranslation(&self) -> Vector2dF {
        crate::adapter::paint_geometry::TransformCacheNode::Get2dTranslation(self)
    }
    fn Matrix(&self) -> Self::Matrix {
        crate::adapter::paint_geometry::GeometryMapperNode::Matrix(self)
    }
    fn Origin(&self) -> crate::adapter::geometry::Point3F {
        let o = self.node().origin;
        crate::adapter::geometry::Point3F::new(o[0] as f32, o[1] as f32, o[2] as f32)
    }
    fn NodeChanged(&self) -> PaintPropertyChange {
        self.node().lifecycle.node_changed()
    }
    fn ScrollCompositorElementId(&self) -> ElementId {
        element(
            self.node()
                .scroll
                .as_ref()
                .expect("source ScrollNode")
                .compositor
                .compositor_element_id,
        )
    }
    fn ScrollContainerRect(&self) -> crate::adapter::geometry::Rect {
        crate::adapter::paint_geometry::GeometryMapperNode::ScrollContainerRect(self)
    }
}
impl<R: CpuSkiaBackend> PendingEffectNode<TransformNode<R>, usize> for EffectNode<R> {
    type Filters = paint::cc::paint::filter_operations::FilterOperations;
    type Path = R::Path;
    type PaintPropertyChange = native::PaintPropertyChangeType;
    type ClipNode = ClipNode<R>;
    fn CcNodeId(&self, s: i32) -> i32 {
        self.node().lifecycle.cc_node_id(s)
    }
    fn SetCcNodeId(&self, s: i32, id: i32) {
        self.node().lifecycle.set_cc_node_id(s, id)
    }
    fn IsRoot(&self) -> bool {
        self.node().parent.is_none()
    }
    fn Identity(&self) -> usize {
        Arc::as_ptr(&self.node().lifecycle.identity) as usize
    }
    fn ParentUnalias(&self) -> Self {
        Self::from_arc(
            self.node()
                .parent
                .as_ref()
                .expect("source effect parent")
                .clone(),
        )
    }
    fn LocalTransformSpaceUnalias(&self) -> TransformNode<R> {
        TransformNode::from_arc(self.node().local_transform_space.clone())
    }
    fn BlendMode(&self) -> BlendMode {
        blend(self.node())
    }
    fn Opacity(&self) -> f32 {
        self.node().opacity
    }
    fn HasActiveOpacityAnimation(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"ActiveOpacityAnimation")
    }
    fn HasFilter(&self) -> bool {
        self.node().compositor.has_filter_info
    }
    fn RequiresCompositingForWillChangeFilter(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"WillChangeFilter")
    }
    fn HasActiveFilterAnimation(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"ActiveFilterAnimation")
    }
    fn HasBackdropFilter(&self) -> bool {
        self.node().compositor.has_backdrop_filter_info
    }
    fn RequiresCompositingForWillChangeBackdropFilter(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"WillChangeBackdropFilter")
    }
    fn HasActiveBackdropFilterAnimation(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"ActiveBackdropFilterAnimation")
    }
    fn HasValidViewTransitionElementResourceId(&self) -> bool {
        self.ViewTransitionElementResourceId().IsValid()
    }
    fn RequiresCompositingForUnboundedElement(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"UnboundedElement")
    }
    fn FlattensAtLeafOf3DScene(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"Transform3DSceneLeaf")
    }
    fn HasElementCaptureId(&self) -> bool {
        self.node().compositor.restriction_target_id != [0, 0]
    }
    fn LowestCommonAncestor(&self, other: &Self) -> Self {
        Self::from_arc(paint::property_node_geometry::LowestCommonAncestor(
            self.as_arc(),
            other.as_arc(),
        ))
    }
    // Source: effect_paint_property_node.h:355-385.
    fn MayHaveBackdropEffect(&self) -> bool {
        self.BlendMode() != BlendMode::kSrcOver
            || self.HasBackdropFilter()
            || self.HasActiveBackdropFilterAnimation()
            || self.RequiresCompositingForWillChangeBackdropFilter()
    }
    fn OutputClip(&self) -> Option<ClipNode<R>> {
        self.node()
            .output_clip
            .as_ref()
            .map(|c| ClipNode::from_arc(c.clone()))
    }
    fn GetCompositorElementId(&self) -> ElementId {
        element(self.node().compositor.compositor_element_id)
    }
    fn NeedsEffectFor2DScaleTransform(&self) -> bool {
        self.node().compositor.needs_effect_for_2d_scale_transform
    }
    fn FilterOperations(&self) -> Option<Self::Filters> {
        self.HasFilter().then(|| {
            self.node()
                .filters
                .canonical()
                .expect("validated PaintInput native filter")
                .clone()
        })
    }
    fn BackdropFilterOperations(&self) -> Option<Self::Filters> {
        self.HasBackdropFilter().then(|| {
            self.node()
                .backdrop_filters
                .canonical()
                .expect("validated PaintInput native backdrop filter")
                .clone()
        })
    }
    fn PropertyTreeBackdropFilterBounds(&self) -> Option<Self::Path> {
        self.node()
            .backdrop_filter_bounds
            .as_ref()
            .map(|path| R::FromPaintPath(path.as_ref()))
    }
    fn BackdropMaskElementId(&self) -> ElementId {
        element(self.node().compositor.backdrop_mask_element_id)
    }
    fn NodeChanged(&self) -> Self::PaintPropertyChange {
        self.node().lifecycle.node_changed()
    }
    fn ViewTransitionElementResourceId(&self) -> ViewTransitionElementResourceId {
        match self.node().compositor.view_transition_element_resource_id {
            Some((token, local_id, for_scope_snapshot)) => ViewTransitionElementResourceId {
                transition_token: Some(token),
                local_id,
                for_scope_snapshot,
            },
            None => ViewTransitionElementResourceId::default(),
        }
    }
    fn SubtreeCaptureId(&self) -> SubtreeCaptureId {
        SubtreeCaptureId {
            token: self.node().compositor.restriction_target_id,
        }
    }
}
impl<R: CpuSkiaBackend> PendingClipNode<TransformNode<R>, usize> for ClipNode<R> {
    type EffectNode = EffectNode<R>;
    fn CcNodeId(&self, sequence: i32) -> i32 {
        self.node().lifecycle.cc_node_id(sequence)
    }
    fn SetCcNodeId(&self, sequence: i32, id: i32) {
        self.node().lifecycle.set_cc_node_id(sequence, id)
    }
    fn IsRoot(&self) -> bool {
        self.node().parent.is_none()
    }
    fn Identity(&self) -> usize {
        Arc::as_ptr(&self.node().lifecycle.identity) as usize
    }
    fn ParentUnalias(&self) -> Self {
        Self::from_arc(
            self.node()
                .parent
                .as_ref()
                .expect("source clip parent")
                .clone(),
        )
    }
    fn LocalTransformSpaceUnalias(&self) -> TransformNode<R> {
        TransformNode::from_arc(self.node().local_transform_space.clone())
    }
    fn PaintClipRectRect(&self) -> RectF {
        let c = &self.node().rect;
        c.map(|r|RectF::new(r.x as f32,r.y as f32,r.width as f32,r.height as f32)).unwrap_or_else(||*crate::third_party::blink::renderer::platform::graphics::paint::float_clip_rect::FloatClipRect::default().Rect())
    }
    fn LowestCommonAncestor(&self, other: &Self) -> Self {
        Self::from_arc(paint::property_node_geometry::LowestCommonAncestor(
            self.as_arc(),
            other.as_arc(),
        ))
    }
    // Source: paint_property_node.h:159-168. Walk actual parent identities.
    fn IsAncestorOf(&self, other: &Self) -> bool {
        let mut current = Some(other.node());
        while let Some(node) = current {
            if self.node().lifecycle.same_node(&node.lifecycle) {
                return true;
            }
            current = node.parent.as_deref();
        }
        false
    }
    fn PaintClipRectIsRounded(&self) -> bool {
        self.node().radii.HasRadius()
    }
    fn PaintClipRRect(&self) -> Option<RRectF> {
        self.node().rect.map(|_| RRectF {
            rect: self.PaintClipRectRect(),
            corner_radii: [
                self.node().radii.top_left,
                self.node().radii.top_right,
                self.node().radii.bottom_right,
                self.node().radii.bottom_left,
            ]
            .map(|r| Vector2dF::new(r.x as f32, r.y as f32)),
        })
    }
    fn HasClipPath(&self) -> bool {
        self.node().clip_path.is_some()
    }
    fn PixelMovingFilter(&self) -> Option<EffectNode<R>> {
        self.node()
            .pixel_moving_filter
            .as_ref()
            .map(|e| EffectNode::from_arc(e.clone()))
    }
}
