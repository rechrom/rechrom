/*
 * Copyright (C) 2006, 2007, 2008, 2009, 2010, 2011, 2012 Apple Inc. All rights
 * reserved.
 *
 * Portions are Copyright (C) 1998 Netscape Communications Corporation.
 *
 * Other contributors:
 *   Robert O'Callahan <roc+@cs.cmu.edu>
 *   David Baron <dbaron@dbaron.org>
 *   Christian Biesinger <cbiesinger@web.de>
 *   Randall Jesup <rjesup@wgate.com>
 *   Roland Mainz <roland.mainz@informatik.med.uni-giessen.de>
 *   Josh Soref <timeless@mac.com>
 *   Boris Zbarsky <bzbarsky@mit.edu>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA
 *
 * Alternatively, the contents of this file may be used under the terms
 * of either the Mozilla Public License Version 1.1, found at
 * http://www.mozilla.org/MPL/ (the "MPL") or the GNU General Public
 * License Version 2.0, found at http://www.fsf.org/copyleft/gpl.html
 * (the "GPL"), in which case the provisions of the MPL or the GPL are
 * applicable instead of those above. If you wish to allow use of your
 * version of this file only under the terms of one of those two
 * licenses (the MPL or the GPL) and not to allow others to use your
 * version of this file under the LGPL, indicate your decision by
 * deleting the provisions above and replace them with the notice and
 * other provisions required by the MPL or the GPL, as the case may be.
 * If you do not delete the provisions above, a recipient may use your
 * version of this file under any of the LGPL, the MPL or the GPL.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/paint/paint_layer.h:250-257,566-664
// Source: third_party/blink/renderer/core/paint/paint_layer.cc:928-1116,1118-1124,1130-1275,1277-2042,2103-2193
// Source-aligned hit testing over the existing PaintLayer/FragmentData owners.

use std::{cell::Cell, rc::Rc};

use foundation::{gfx, PhysicalOffset, PhysicalRect};

use super::hit_testing_transform_state::{HitTestingTransformState, TransformPaintPropertyNode};
use super::paint_layer::PaintLayerIteration;
use crate::core::layout::geometry::physical_rect::PhysicalRectHitTestExt;
use crate::core::layout::hit_test_location::HitTestLocation;
use crate::core::layout::hit_test_phase::HitTestPhase;
use crate::core::layout::hit_test_request::HitTestRequest;
use crate::core::layout::hit_test_result::{HitTestNode, HitTestResult, NodeRef};
use crate::platform::geometry::path::PathHitTest;

// Source: paint_layer.cc:1039-1092,1326-1328,1576-1659,1940-1941
// These are the small request values consumed by layer traversal. Stop-owner
// tokens are the data addresses of the same resident owners used by Request.
#[derive(Clone, Copy)]
pub struct LayerRequest {
    pub active: bool,
    pub release: bool,
    pub child_frame: bool,
    pub ignore_clipping: bool,
    pub ignore_zero_opacity: bool,
    pub visual_overflow: bool,
    pub list_based: bool,
    pub stop_node: Option<usize>,
}

// A projection of the existing HitTestResult owner, not a second result model.
// The concrete implementation below routes to the canonical translated type.
pub trait LayerHitTestResult: Sized {
    fn Request(&self) -> LayerRequest;
    fn StopLayer(&self) -> Option<usize>;
    fn Fresh(&self, location: &HitTestLocation) -> Self;
    fn Assign(&mut self, other: &Self);
    fn Append(&mut self, other: &Self);
    fn InnerNode(&self) -> Option<NodeRef>;
    fn HasURLElement(&self) -> bool;
    fn SetURLElement(&mut self, node: Option<NodeRef>);
    fn SetInnerNode(&mut self, node: Option<NodeRef>);
    fn SetNodeAndPosition(&mut self, node: Option<NodeRef>, point: PhysicalOffset);
    fn SetCacheable(&mut self, cacheable: bool);
    fn ListSize(&self) -> usize;
}

impl<L: ?Sized> LayerHitTestResult for HitTestResult<'_, L> {
    fn Request(&self) -> LayerRequest {
        let request = self.GetHitTestRequest();
        LayerRequest {
            active: request.Active(),
            release: request.Release(),
            child_frame: request.IsChildFrameHitTest(),
            ignore_clipping: request.IgnoreClipping(),
            ignore_zero_opacity: request.GetType()
                & HitTestRequest::<L, dyn HitTestNode>::kIgnoreZeroOpacityObjects
                != 0,
            visual_overflow: request.IsHitTestVisualOverflow(),
            list_based: request.ListBased(),
            stop_node: request
                .GetStopNode()
                .map(|node| node as *const _ as *const () as usize),
        }
    }
    fn StopLayer(&self) -> Option<usize> {
        self.GetHitTestRequest()
            .GetStopLayer()
            .map(|layer| layer as *const L as *const () as usize)
    }
    fn Fresh(&self, location: &HitTestLocation) -> Self {
        Self::new(self.GetHitTestRequest(), location)
    }
    fn Assign(&mut self, other: &Self) {
        Self::Assign(self, other);
    }
    fn Append(&mut self, other: &Self) {
        Self::Append(self, other);
    }
    fn InnerNode(&self) -> Option<NodeRef> {
        Self::InnerNode(self).cloned()
    }
    fn HasURLElement(&self) -> bool {
        self.URLElement().is_some()
    }
    fn SetURLElement(&mut self, node: Option<NodeRef>) {
        Self::SetURLElement(self, node);
    }
    fn SetInnerNode(&mut self, node: Option<NodeRef>) {
        Self::SetInnerNode(self, node);
    }
    fn SetNodeAndPosition(&mut self, node: Option<NodeRef>, point: PhysicalOffset) {
        Self::SetNodeAndPosition(self, node, point);
    }
    fn SetCacheable(&mut self, cacheable: bool) {
        Self::SetCacheable(self, cacheable);
    }
    fn ListSize(&self) -> usize {
        self.ListBasedTestResult().len()
    }
}

// Source: paint_layer.cc:1304-1359,1383-1396,1426-1473,1578-1581,1890-1938
// Per-layer facts are read from its existing layout/style owner. This small
// value contains no tree, ComputedStyle, geometry cache, or engine reference.
#[derive(Clone, Copy)]
pub struct LayerLayoutFacts {
    pub layout_identity: usize,
    pub can_contain_fixed: bool,
    pub can_contain_absolute: bool,
    pub prepaint_clean: bool,
    pub layout_pending: bool,
    pub needs_layout: bool,
    pub child_layout_blocked: bool,
    pub fragment_less: bool,
    pub nonzero_effective_opacity: bool,
    pub stacking_context: bool,
    pub represented_by_transition_pseudo: bool,
    pub unbounded_descendant: bool,
    pub global_root_scroller: bool,
    pub svg_foreign_object: bool,
    pub has_clip_path: bool,
    pub has_border_shape: bool,
    pub clips_overflow_both_axes: bool,
    pub backface_hidden: bool,
    pub child_paint_blocked: bool,
    pub layout_inline: bool,
    pub can_traverse_fragments: bool,
    pub fragmented: bool,
    pub floating: bool,
    pub replaced_normal_flow_stacking_context: bool,
    pub canvas: bool,
    pub canvas_layout_subtree: bool,
    pub view_transition_root: bool,
    pub has_view_transition_pseudo: bool,
}

#[derive(Clone, Copy)]
pub struct LayerFacts {
    pub root: bool,
    pub self_painting: bool,
    pub self_painting_descendant: bool,
    pub invisible_for_position_visibility: bool,
    pub ancestor_invisible_for_position_visibility: bool,
    pub preserves_3d: bool,
    pub has_3d_transform: bool,
    pub has_3d_descendant: bool,
    pub scrollable_area: bool,
    pub needs_reorder_overlay_controls: bool,
}

#[derive(Clone, Copy)]
pub struct LayerHitTestFeatures {
    pub unbounded_element: bool,
    pub border_radius_for_stacking_context: bool,
    pub container_transform_for_preserve_3d: bool,
    pub stacking_context_is_not_stacked: bool,
    pub replaced_normal_flow_stacking_inline_paint: bool,
    pub canvas_draw_element: bool,
}

// Source: paint_layer.cc:947-951,1005-1009
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OverflowClipBehavior {
    Respect,
    Ignore,
}

#[derive(Clone, Copy)]
pub struct LayerClipRect {
    pub rect: PhysicalRect,
    pub has_radius: bool,
    pub is_infinite: bool,
}
impl LayerClipRect {
    fn Intersects(&self, location: &HitTestLocation) -> bool {
        self.is_infinite || location.Intersects(&self.rect)
    }
}

// Source: paint_layer.cc:1154-1204,1438-1455,2166-2187
pub trait LayerTransformNode: TransformPaintPropertyNode {
    fn NativeOwner(
        &self,
    ) -> &std::sync::Arc<paint::paint_property_tree::TransformPaintPropertyNode>;
    fn Identity(&self) -> usize;
    fn IsAncestorOf(&self, descendant: &dyn LayerTransformNode) -> bool;
}
pub trait LayerRadiusClip {
    fn LocalTransformSpace(&self) -> &dyn LayerTransformNode;
    fn IntersectsQuad(&self, quad: &gfx::QuadF) -> bool;
}
pub trait LayerPaintProperties {
    fn HasCSSTransformPropertyNode(&self) -> bool;
    fn Perspective(&self) -> Option<&dyn LayerTransformNode>;
    fn HasElementCanvasTransform(&self) -> bool;
    fn CSSTransformPropertiesOutsideToInside(
        &self,
    ) -> Box<dyn Iterator<Item = &dyn LayerTransformNode> + '_>;
    fn InnerBorderRadiusClip(&self) -> Option<&dyn LayerRadiusClip>;
}
pub trait LayerFragmentData {
    fn NativeOwner(&self) -> paint::resident_hit_test::ResidentPaintHitTest<'_>;
    fn Identity(&self) -> usize;
    fn PaintOffset(&self) -> PhysicalOffset;
    fn LocalBorderTransform(&self) -> &dyn LayerTransformNode;
    fn ContentsTransform(&self) -> &dyn LayerTransformNode;
    fn PreTransform(&self) -> &dyn LayerTransformNode;
    fn PaintProperties(&self) -> Option<&dyn LayerPaintProperties>;
}

// A physical fragment port invokes only the existing owning package's query.
// The same canonical result is passed to the translated BoxFragmentPainter.
pub trait LayerPhysicalFragment<R> {
    fn MayIntersect(&self, result: &R, location: &HitTestLocation, offset: PhysicalOffset) -> bool;
    fn NodeAtPoint(
        &self,
        result: &mut R,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
}

// Source PaintLayerFragment is already a derived, local query record. It keeps
// references to resident FragmentData/PhysicalBoxFragment, never their copies.
pub struct LayerFragment<'l, R> {
    pub fragment_data: Rc<dyn LayerFragmentData + 'l>,
    pub physical_fragment: Option<Rc<dyn LayerPhysicalFragment<R> + 'l>>,
    pub fragment_idx: usize,
    pub layer_offset: PhysicalOffset,
    pub background_rect: LayerClipRect,
    pub foreground_rect: LayerClipRect,
}
impl<R> Clone for LayerFragment<'_, R> {
    fn clone(&self) -> Self {
        Self {
            fragment_data: self.fragment_data.clone(),
            physical_fragment: self.physical_fragment.clone(),
            fragment_idx: self.fragment_idx,
            layer_offset: self.layer_offset,
            background_rect: self.background_rect,
            foreground_rect: self.foreground_rect,
        }
    }
}

pub trait LayerVisibilityScope {}

// Narrow source operations over the retained layer. Read-only iterator ports
// borrow native PaintOrder/Canvas/FragmentData links without copying lists.
pub trait PaintLayerHitTest<R: LayerHitTestResult> {
    fn Identity(&self) -> usize;
    fn Facts(&self) -> LayerFacts;
    fn LayoutFacts(&self) -> LayerLayoutFacts;
    fn Features(&self) -> LayerHitTestFeatures;
    fn Parent(&self) -> Option<Box<dyn PaintLayerHitTest<R> + '_>>;
    fn NearestAncestorElementLayoutIdentity(&self) -> Option<usize>;
    fn IsDescendantOfLayout(&self, identity: usize) -> bool;
    fn FirstFragment(&self) -> Rc<dyn LayerFragmentData + '_>;
    fn FragmentData(&self) -> Box<dyn Iterator<Item = Rc<dyn LayerFragmentData + '_>> + '_>;
    fn PhysicalFragment(&self, index: usize) -> Option<Rc<dyn LayerPhysicalFragment<R> + '_>>;
    fn HasLayoutBoxWithBlockFragments(&self) -> bool;
    fn CanPaintMultipleFragments(&self) -> bool;
    fn CalculateClipRects(
        &self,
        root: &dyn PaintLayerHitTest<R>,
        root_fragment: Rc<dyn LayerFragmentData + '_>,
        fragment: Rc<dyn LayerFragmentData + '_>,
        clip: OverflowClipBehavior,
    ) -> (PhysicalOffset, LayerClipRect, LayerClipRect);
    fn Projection(
        &self,
        source: &dyn LayerTransformNode,
        destination: &dyn LayerTransformNode,
    ) -> gfx::Transform;
    fn EnterAncestorPositionVisibilityScope(&self) -> Box<dyn LayerVisibilityScope + '_>;
    fn RootFrameToVisualViewport(&self, point: gfx::Point) -> gfx::Point;
    fn HitTestOverflowControl(
        &self,
        result: &mut R,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn LayoutNodeAtPoint(
        &self,
        result: &mut R,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
    fn UpdateHitTestResult(&self, result: &mut R, point: PhysicalOffset);
    fn LayoutAncestorNodes(&self) -> Box<dyn Iterator<Item = Option<NodeRef>> + '_>;
    fn EnclosingLinkEventParentOrSelf(&self, node: &NodeRef) -> Option<NodeRef>;
    fn VisualOverflowRect(&self) -> PhysicalRect;
    fn ReversePaintOrder(
        &self,
        children: PaintLayerIteration,
    ) -> Box<dyn Iterator<Item = Box<dyn PaintLayerHitTest<R> + '_>> + '_>;
    fn CanvasReversePaintOrder(
        &self,
    ) -> Box<dyn Iterator<Item = Box<dyn PaintLayerHitTest<R> + '_>> + '_>;
    fn OverlayOverflowControlsAfter(
        &self,
        child: &dyn PaintLayerHitTest<R>,
    ) -> Box<dyn DoubleEndedIterator<Item = Box<dyn PaintLayerHitTest<R> + '_>> + '_>;
    fn LocalOriginInAncestor(&self, ancestor: &dyn PaintLayerHitTest<R>) -> PhysicalOffset;
    fn ClipPathHitTest(&self, location: &HitTestLocation) -> bool;
    fn HasLayoutBox(&self) -> bool;
    fn BorderBoxRect(&self) -> PhysicalRect;
    fn ShouldApplyOverflowClipMargin(&self) -> bool;
    // The owning box applies its existing source BorderOutsetsForClipping.
    fn ExpandByBorderOutsetsForClipping(&self, rect: &mut PhysicalRect);
    // Existing Paint's ComputeBorderShapeReferenceRects / OuterPath result.
    fn BorderShapeOuterPath(&self, border_rect: PhysicalRect) -> Option<Box<dyn PathHitTest + '_>>;
    fn ContainerBorderRadiusClips(
        &self,
    ) -> Box<dyn Iterator<Item = (usize, Option<&dyn LayerRadiusClip>)> + '_>;
}

/// Source hit-layer identity and the exact facts consumed by depth ordering.
/// This call-scoped value replaces only the C++ result pointer, never a layer
/// owner, fragment graph, or retained geometry model.
#[derive(Clone, Copy)]
pub struct LayerHitCandidate {
    pub identity: usize,
    pub svg_foreign_object: bool,
    pub has_3d_transform: bool,
    pub occlusion_visual_overflow: Option<PhysicalRect>,
}
fn HitCandidate<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    occlusion: bool,
) -> LayerHitCandidate {
    let has_3d_transform = layer.Facts().has_3d_transform;
    LayerHitCandidate {
        identity: layer.Identity(),
        svg_foreign_object: layer.LayoutFacts().svg_foreign_object,
        has_3d_transform,
        occlusion_visual_overflow: (occlusion && has_3d_transform)
            .then(|| layer.VisualOverflowRect()),
    }
}

// Source: paint_layer.h:566-569; paint_layer.cc:928-954
pub fn AppendSingleFragmentForHitTesting<'l, R: LayerHitTestResult>(
    layer: &'l dyn PaintLayerHitTest<R>,
    fragments: &mut Vec<LayerFragment<'l, R>>,
    container_fragment: Option<&LayerFragment<'l, R>>,
    clip: OverflowClipBehavior,
) {
    let (data, physical, index) = if let Some(fragment) = container_fragment {
        (
            fragment.fragment_data.clone(),
            fragment.physical_fragment.clone(),
            fragment.fragment_idx,
        )
    } else {
        let physical = if layer.LayoutFacts().can_traverse_fragments && layer.HasLayoutBox() {
            layer.PhysicalFragment(0)
        } else {
            None
        };
        (layer.FirstFragment(), physical, 0)
    };
    let (offset, background, foreground) =
        layer.CalculateClipRects(layer, data.clone(), data.clone(), clip);
    fragments.push(LayerFragment {
        fragment_data: data,
        physical_fragment: physical,
        fragment_idx: index,
        layer_offset: offset,
        background_rect: background,
        foreground_rect: foreground,
    });
}

// Source: paint_layer.h:571-574; paint_layer.cc:956-1028
pub fn CollectFragments<'l, R: LayerHitTestResult>(
    layer: &'l dyn PaintLayerHitTest<R>,
    root: &dyn PaintLayerHitTest<R>,
    clip: OverflowClipBehavior,
    root_fragment_arg: Option<Rc<dyn LayerFragmentData + '_>>,
) -> Vec<LayerFragment<'l, R>> {
    let first_root_fragment_data = root.FirstFragment();
    let layout_box_with_fragments = layer.HasLayoutBoxWithBlockFragments();
    let multiple_fragments_allowed = layout_box_with_fragments || layer.CanPaintMultipleFragments();
    let mut fragments = Vec::new();
    for (physical_fragment_idx, fragment_data) in layer.FragmentData().enumerate() {
        let root_fragment_data = if let Some(root_fragment) = root_fragment_arg.as_ref() {
            debug_assert_ne!(layer.Identity(), root.Identity());
            if !root_fragment
                .ContentsTransform()
                .IsAncestorOf(fragment_data.LocalBorderTransform())
            {
                continue;
            }
            root_fragment.clone()
        } else if root.Identity() == layer.Identity() {
            fragment_data.clone()
        } else {
            first_root_fragment_data.clone()
        };
        let (offset, background, foreground) =
            layer.CalculateClipRects(root, root_fragment_data, fragment_data.clone(), clip);
        let physical = if layout_box_with_fragments {
            let physical = layer.PhysicalFragment(physical_fragment_idx);
            debug_assert!(physical.is_some());
            physical
        } else {
            None
        };
        fragments.push(LayerFragment {
            fragment_data,
            physical_fragment: physical,
            fragment_idx: physical_fragment_idx,
            layer_offset: offset,
            background_rect: background,
            foreground_rect: foreground,
        });
        if !multiple_fragments_allowed {
            break;
        }
    }
    fragments
}

// Source: paint_layer.h:576-588; paint_layer.cc:1030-1037
pub struct HitTestRecursionData<'r> {
    pub rect: &'r PhysicalRect,
    pub location: &'r HitTestLocation,
    pub original_location: &'r HitTestLocation,
    pub intersects_location: bool,
}
impl<'r> HitTestRecursionData<'r> {
    pub fn new(
        rect: &'r PhysicalRect,
        location: &'r HitTestLocation,
        original: &'r HitTestLocation,
    ) -> Self {
        Self {
            rect,
            location,
            original_location: original,
            intersects_location: location.Intersects(rect),
        }
    }
}

// Source: paint_layer.h:250-256; paint_layer.cc:1039-1094
pub fn HitTest<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    location: &HitTestLocation,
    result: &mut R,
    area: &PhysicalRect,
) -> bool {
    let layout = layer.LayoutFacts();
    debug_assert!(layout.can_contain_fixed && layout.can_contain_absolute);
    debug_assert!(!layout.layout_pending && !layout.needs_layout);
    let request = result.Request();
    let recursion = HitTestRecursionData::new(area, location, location);
    let mut inside = HitTestLayer(
        layer, layer, None, result, &recursion, false, None, None, false,
    )
    .is_some();
    if !inside && layer.Facts().root {
        let fallback = area.Contains(location.BoundingBox())
            || ((request.active || request.release) && !request.child_frame);
        if fallback {
            layer.UpdateHitTestResult(result, *location.Point());
            inside = true;
            result.SetCacheable(false);
        }
    }
    if let Some(node) = result.InnerNode() {
        if !result.HasURLElement() {
            result.SetURLElement(layer.EnclosingLinkEventParentOrSelf(&node));
        }
    }
    inside
}

// Source: paint_layer.h:257; paint_layer.cc:1096-1116
pub fn HitTestReplacedNormalFlowInline<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    result: &mut R,
    location: &HitTestLocation,
) -> bool {
    debug_assert!(ShouldPaintReplacedNormalFlowInline(layer));
    fn WithContainer<R: LayerHitTestResult>(
        current: &dyn PaintLayerHitTest<R>,
        query: &mut dyn FnMut(&dyn PaintLayerHitTest<R>) -> bool,
    ) -> bool {
        let Some(parent) = current.Parent() else {
            return false;
        };
        let facts = parent.LayoutFacts();
        if facts.can_contain_fixed && facts.can_contain_absolute {
            query(parent.as_ref())
        } else {
            WithContainer(parent.as_ref(), query)
        }
    }
    WithContainer(layer, &mut |container| {
        let recursion = HitTestRecursionData::new(location.BoundingBox(), location, location);
        HitTestLayer(
            layer, container, None, result, &recursion, false, None, None, false,
        )
        .is_some()
    })
}

// Source: paint_layer.cc:1118-1124
fn EnclosingNode<R: LayerHitTestResult>(layer: &dyn PaintLayerHitTest<R>) -> NodeRef {
    for node in layer.LayoutAncestorNodes() {
        if let Some(node) = node {
            return node;
        }
    }
    unreachable!("a hit layer must have an enclosing DOM node")
}

// Source: paint_layer.cc:1130-1147
pub fn ComputeZOffset(state: &HitTestingTransformState) -> f64 {
    let transform = state.AccumulatedTransform();
    if transform.Is2dTransform() {
        return 0.0;
    }
    let point = state.MappedPoint();
    transform.MapPoint3d([point.x(), point.y(), 0.0])[2] as f64
}

// Source: paint_layer.h:621-626; paint_layer.cc:1149-1206
pub fn CreateLocalTransformState<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    container: &dyn PaintLayerHitTest<R>,
    container_fragment: Rc<dyn LayerFragmentData + '_>,
    local_fragment: Rc<dyn LayerFragmentData + '_>,
    recursion: &HitTestRecursionData<'_>,
    container_state: Option<&HitTestingTransformState>,
) -> HitTestingTransformState {
    let mut state = container_state.cloned().unwrap_or_else(|| {
        HitTestingTransformState::new(
            *recursion.location.TransformedPoint(),
            *recursion.location.TransformedRect(),
            *recursion.rect,
        )
    });
    if container.Identity() == layer.Identity() {
        debug_assert!(container_state.is_none());
        return state;
    }
    if container_state.is_some()
        && (!container.Facts().preserves_3d
            || layer.NearestAncestorElementLayoutIdentity()
                != Some(container.LayoutFacts().layout_identity))
    {
        state.Flatten();
    }
    debug_assert_ne!(container_fragment.Identity(), local_fragment.Identity());
    let mut container_transform = container_fragment.LocalBorderTransform();
    if let Some(perspective) = container_fragment
        .PaintProperties()
        .and_then(|p| p.Perspective())
    {
        state.ApplyTransform(&perspective.MatrixWithOriginApplied());
        container_transform = perspective;
    }
    state.Translate(gfx::Vector2dF::from(-container_fragment.PaintOffset()));
    state.ApplyTransform(&layer.Projection(local_fragment.PreTransform(), container_transform));
    state.Translate(gfx::Vector2dF::from(local_fragment.PaintOffset()));
    if let Some(properties) = local_fragment.PaintProperties() {
        for transform in properties.CSSTransformPropertiesOutsideToInside() {
            state.ApplyTransform(&transform.MatrixWithOriginApplied());
        }
    }
    state
}

// Source: paint_layer.cc:1208-1267
pub fn IsHitCandidateForDepthOrder(
    hit_layer: Option<LayerHitCandidate>,
    can_depth_sort: bool,
    z_offset: Option<&Cell<f64>>,
    transform_state: Option<&HitTestingTransformState>,
    occlusion: bool,
) -> bool {
    let Some(hit_layer) = hit_layer else {
        return false;
    };
    if can_depth_sort {
        return true;
    }
    debug_assert!(z_offset.is_none() || transform_state.is_some() || hit_layer.svg_foreign_object);
    if let (Some(z_offset), Some(state)) = (z_offset, transform_state) {
        let child_z = if occlusion && hit_layer.has_3d_transform {
            let mut child_z = f64::NEG_INFINITY;
            let quad = gfx::QuadF::from(gfx::RectF::from(
                hit_layer
                    .occlusion_visual_overflow
                    .expect("occlusion candidate geometry"),
            ));
            let transform = state.AccumulatedTransform();
            for point in [quad.p1(), quad.p2(), quad.p3(), quad.p4()] {
                let z = transform.MapPoint3d([point.x(), point.y(), 0.0])[2] as f64;
                if z > child_z {
                    child_z = z;
                }
            }
            if child_z < z_offset.get() {
                return false;
            }
            child_z
        } else {
            ComputeZOffset(state)
        };
        if child_z > z_offset.get() {
            z_offset.set(child_z);
            return true;
        }
        return false;
    }
    true
}

// Source: paint_layer.cc:1269-1275
pub fn IsHitCandidateForStopNode<R: LayerHitTestResult>(
    candidate: &dyn PaintLayerHitTest<R>,
    stop_node: Option<usize>,
) -> bool {
    stop_node.is_none_or(|stop| {
        candidate.LayoutFacts().layout_identity == stop || !candidate.IsDescendantOfLayout(stop)
    })
}

// Source: paint_layer.h:590-597; paint_layer.cc:1277-1661
pub fn HitTestLayer<'l, R: LayerHitTestResult>(
    layer: &'l dyn PaintLayerHitTest<R>,
    container: &dyn PaintLayerHitTest<R>,
    container_fragment: Option<&LayerFragment<'_, R>>,
    result: &mut R,
    recursion: &HitTestRecursionData<'_>,
    applied_transform: bool,
    container_state: Option<&HitTestingTransformState>,
    z_offset: Option<&Cell<f64>>,
    overflow_controls_only: bool,
) -> Option<LayerHitCandidate> {
    let layout = layer.LayoutFacts();
    let facts = layer.Facts();
    let features = layer.Features();
    let request = result.Request();
    let container_fragment_data = container_fragment.map(|f| f.fragment_data.clone());
    let container_layout = container.LayoutFacts();
    debug_assert!(container_layout.can_contain_fixed && container_layout.can_contain_absolute);
    debug_assert!(layout.prepaint_clean);
    if layout.needs_layout && !layout.child_layout_blocked {
        return None;
    }
    if layout.fragment_less {
        return None;
    }
    if !facts.self_painting && !facts.self_painting_descendant {
        return None;
    }
    if request.ignore_zero_opacity && !layout.nonzero_effective_opacity {
        return None;
    }
    if facts.invisible_for_position_visibility || facts.ancestor_invisible_for_position_visibility {
        return None;
    }
    let _visibility_scope = layout
        .stacking_context
        .then(|| layer.EnterAncestorPositionVisibilityScope());
    if layout.represented_by_transition_pseudo {
        return None;
    }
    let clip =
        if request.ignore_clipping || (features.unbounded_element && layout.unbounded_descendant) {
            OverflowClipBehavior::Ignore
        } else {
            OverflowClipBehavior::Respect
        };
    if layout.global_root_scroller {
        debug_assert!(!facts.preserves_3d && !layout.has_clip_path);
        if facts.scrollable_area {
            let point = layer.RootFrameToVisualViewport(recursion.location.RoundedPoint());
            debug_assert!(layer.HasLayoutBox());
            if layer.HitTestOverflowControl(
                result,
                &HitTestLocation::FromPoint(point),
                PhysicalOffset::default(),
            ) {
                return Some(HitCandidate(layer, request.visual_overflow));
            }
        }
    }
    let use_transform = !layout.svg_foreign_object
        && layout.can_contain_fixed
        && layer
            .FirstFragment()
            .PaintProperties()
            .is_some_and(|properties| {
                properties.HasCSSTransformPropertyNode()
                    || properties.Perspective().is_some()
                    || properties.HasElementCanvasTransform()
            });
    if use_transform && !applied_transform {
        return HitTestTransformedLayerInFragments(
            layer,
            container,
            container_fragment,
            result,
            recursion,
            container_state,
            z_offset,
            overflow_controls_only,
            clip,
        );
    }
    if !request.visual_overflow
        && layout.has_clip_path
        && HitTestClippedOutByClipPath(layer, container, recursion.location)
    {
        return None;
    }
    if !request.visual_overflow
        && layout.has_border_shape
        && layout.clips_overflow_both_axes
        && HitTestClippedOutByBorderShape(layer, container, recursion.location)
    {
        return None;
    }
    let storage;
    let local_state = if applied_transform {
        debug_assert!(container_state.is_some());
        container_state
    } else if container_state.is_some() || facts.has_3d_descendant {
        debug_assert!(!facts.preserves_3d);
        let local_fragment = if let Some(data) = container_fragment_data.as_ref() {
            let Some(local) = layer.FragmentData().find(|fragment| {
                data.ContentsTransform()
                    .IsAncestorOf(fragment.LocalBorderTransform())
            }) else {
                return None;
            };
            local
        } else {
            layer.FirstFragment()
        };
        let container_data = container_fragment_data
            .clone()
            .unwrap_or_else(|| container.FirstFragment());
        storage = CreateLocalTransformState(
            layer,
            container,
            container_data,
            local_fragment,
            recursion,
            container_state,
        );
        Some(&storage)
    } else {
        None
    };
    if layout.backface_hidden
        && local_state.is_some_and(|state| state.AccumulatedTransform().IsBackFaceVisible())
    {
        return None;
    }
    let local_z_offset = Cell::new(f64::NEG_INFINITY);
    let depth_sort = facts.preserves_3d;
    let descendants_z = if depth_sort {
        Some(z_offset.unwrap_or(&local_z_offset))
    } else {
        None
    };
    let contents_z = if depth_sort {
        Some(z_offset.unwrap_or(&local_z_offset))
    } else {
        z_offset
    };
    let mut candidate = None;
    if layout.has_view_transition_pseudo {
        let hit = HitTestChildren(
            layer,
            PaintLayerIteration::kAllChildren,
            container,
            container_fragment,
            result,
            recursion,
            container_state,
            descendants_z,
            z_offset,
            local_state,
            depth_sort,
            true,
        );
        if hit.is_some() {
            if !depth_sort {
                return hit;
            }
            candidate = hit;
        }
    }
    let fragments = if recursion.intersects_location {
        let fragments = if applied_transform {
            debug_assert_eq!(container.Identity(), layer.Identity());
            let mut fragments = Vec::new();
            AppendSingleFragmentForHitTesting(layer, &mut fragments, container_fragment, clip);
            fragments
        } else {
            CollectFragments(layer, container, clip, container_fragment_data)
        };
        if facts.scrollable_area
            && fragments[0].background_rect.Intersects(recursion.location)
            && layer.HitTestOverflowControl(result, recursion.location, fragments[0].layer_offset)
        {
            if z_offset.is_none()
                || local_state.is_none()
                || IsHitCandidateForDepthOrder(
                    Some(HitCandidate(layer, request.visual_overflow)),
                    false,
                    z_offset,
                    local_state,
                    request.visual_overflow,
                )
            {
                return Some(HitCandidate(layer, request.visual_overflow));
            }
        }
        fragments
    } else {
        Vec::new()
    };
    if overflow_controls_only {
        return None;
    }
    // Source: paint_layer.cc:1553-1574, positive z before normal flow.
    for children in [
        PaintLayerIteration::kPositiveZOrderChildren,
        PaintLayerIteration::kNormalFlowChildren,
    ] {
        let hit = HitTestChildren(
            layer,
            children,
            container,
            container_fragment,
            result,
            recursion,
            container_state,
            descendants_z,
            z_offset,
            local_state,
            depth_sort,
            false,
        );
        if hit.is_some() {
            if !depth_sort {
                return hit;
            }
            candidate = hit;
        }
    }
    if recursion.intersects_location && facts.self_painting && !layout.child_paint_blocked {
        let mut temporary = result.Fresh(recursion.original_location);
        let mut inside_clip = false;
        if HitTestForegroundForFragments(
            layer,
            container,
            container_fragment,
            &fragments,
            &mut temporary,
            recursion.location,
            &mut inside_clip,
        ) && IsHitCandidateForDepthOrder(
            Some(HitCandidate(layer, request.visual_overflow)),
            false,
            contents_z,
            if features.container_transform_for_preserve_3d {
                container_state
            } else {
                local_state
            },
            temporary.Request().visual_overflow,
        ) && IsHitCandidateForStopNode(layer, request.stop_node)
        {
            if request.list_based {
                result.Append(&temporary);
            } else {
                result.Assign(&temporary);
            }
            if !depth_sort {
                return Some(HitCandidate(layer, request.visual_overflow));
            }
            candidate = Some(HitCandidate(layer, request.visual_overflow));
        } else if inside_clip
            && request.list_based
            && IsHitCandidateForStopNode(layer, request.stop_node)
        {
            result.Append(&temporary);
        }
    }
    let hit = HitTestChildren(
        layer,
        PaintLayerIteration::kNegativeZOrderChildren,
        container,
        container_fragment,
        result,
        recursion,
        container_state,
        descendants_z,
        z_offset,
        local_state,
        depth_sort,
        false,
    );
    if hit.is_some() {
        if !depth_sort {
            return hit;
        }
        candidate = hit;
    }
    if recursion.intersects_location && facts.self_painting {
        let mut temporary = result.Fresh(recursion.original_location);
        let mut inside_clip = false;
        if HitTestFragmentsWithPhase(
            layer,
            container,
            container_fragment,
            &fragments,
            &mut temporary,
            recursion.location,
            HitTestPhase::kSelfBlockBackground,
            &mut inside_clip,
        ) && IsHitCandidateForDepthOrder(
            Some(HitCandidate(layer, request.visual_overflow)),
            false,
            contents_z,
            if features.container_transform_for_preserve_3d {
                container_state
            } else {
                local_state
            },
            temporary.Request().visual_overflow,
        ) && IsHitCandidateForStopNode(layer, request.stop_node)
        {
            if request.list_based {
                result.Append(&temporary);
            } else {
                result.Assign(&temporary);
            }
            return Some(HitCandidate(layer, request.visual_overflow));
        }
        if inside_clip && request.list_based && IsHitCandidateForStopNode(layer, request.stop_node)
        {
            result.Append(&temporary);
        }
    }
    candidate
}

// Source: paint_layer.h:640-646; paint_layer.cc:1663-1689
pub fn HitTestForegroundForFragments<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    container: &dyn PaintLayerHitTest<R>,
    container_fragment: Option<&LayerFragment<'_, R>>,
    fragments: &[LayerFragment<'_, R>],
    result: &mut R,
    location: &HitTestLocation,
    inside_clip: &mut bool,
) -> bool {
    if HitTestFragmentsWithPhase(
        layer,
        container,
        container_fragment,
        fragments,
        result,
        location,
        HitTestPhase::kForeground,
        inside_clip,
    ) {
        return true;
    }
    if *inside_clip
        && HitTestFragmentsWithPhase(
            layer,
            container,
            container_fragment,
            fragments,
            result,
            location,
            HitTestPhase::kFloat,
            inside_clip,
        )
    {
        return true;
    }
    if *inside_clip
        && HitTestFragmentsWithPhase(
            layer,
            container,
            container_fragment,
            fragments,
            result,
            location,
            HitTestPhase::kDescendantBlockBackgrounds,
            inside_clip,
        )
    {
        return true;
    }
    false
}

// Source: paint_layer.h:633-639; paint_layer.cc:1691-1746
pub fn HitTestFragmentsWithPhase<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    container: &dyn PaintLayerHitTest<R>,
    container_fragment: Option<&LayerFragment<'_, R>>,
    fragments: &[LayerFragment<'_, R>],
    result: &mut R,
    location: &HitTestLocation,
    phase: HitTestPhase,
    inside_clip: &mut bool,
) -> bool {
    if fragments.is_empty() {
        return false;
    }
    for fragment in fragments.iter().rev() {
        let bounds = if phase == HitTestPhase::kSelfBlockBackground {
            &fragment.background_rect
        } else {
            &fragment.foreground_rect
        };
        if !bounds.Intersects(location) {
            continue;
        }
        if layer.Features().border_radius_for_stacking_context
            && bounds.has_radius
            && HitTestClippedOutByBorderRadius(
                layer,
                container,
                container_fragment,
                location,
                bounds,
            )
            && !result.Request().visual_overflow
        {
            continue;
        }
        *inside_clip = true;
        let facts = layer.LayoutFacts();
        if facts.layout_inline && facts.can_traverse_fragments {
            let fragment_location =
                HitTestLocation::WithFragmentIndex(location, fragment.fragment_idx);
            if HitTestFragmentWithPhase(
                layer,
                result,
                fragment.physical_fragment.as_deref(),
                fragment.layer_offset,
                &fragment_location,
                phase,
            ) {
                return true;
            }
        } else if HitTestFragmentWithPhase(
            layer,
            result,
            fragment.physical_fragment.as_deref(),
            fragment.layer_offset,
            location,
            phase,
        ) {
            return true;
        }
    }
    false
}

// Source: paint_layer.h:647-655; paint_layer.cc:1748-1791
pub fn HitTestTransformedLayerInFragments<'l, R: LayerHitTestResult>(
    layer: &'l dyn PaintLayerHitTest<R>,
    container: &dyn PaintLayerHitTest<R>,
    container_fragment: Option<&LayerFragment<'_, R>>,
    result: &mut R,
    recursion: &HitTestRecursionData<'_>,
    container_state: Option<&HitTestingTransformState>,
    z_offset: Option<&Cell<f64>>,
    overflow_controls_only: bool,
    clip: OverflowClipBehavior,
) -> Option<LayerHitCandidate> {
    let fragments = CollectFragments(
        layer,
        container,
        clip,
        container_fragment.map(|f| f.fragment_data.clone()),
    );
    for fragment in &fragments {
        if !fragment.background_rect.Intersects(recursion.location) {
            continue;
        }
        if layer.Features().border_radius_for_stacking_context
            && fragment.background_rect.has_radius
            && HitTestClippedOutByBorderRadius(
                layer,
                container,
                container_fragment,
                recursion.location,
                &fragment.background_rect,
            )
            && !result.Request().visual_overflow
        {
            continue;
        }
        let hit = HitTestLayerByApplyingTransform(
            layer,
            container,
            container_fragment,
            fragment,
            result,
            recursion,
            container_state,
            z_offset,
            overflow_controls_only,
        );
        if hit.is_some() {
            return hit;
        }
    }
    None
}

// Source: paint_layer.h:598-607; paint_layer.cc:1793-1840
pub fn HitTestLayerByApplyingTransform<'l, R: LayerHitTestResult>(
    layer: &'l dyn PaintLayerHitTest<R>,
    container: &dyn PaintLayerHitTest<R>,
    container_fragment: Option<&LayerFragment<'_, R>>,
    local_fragment: &LayerFragment<'_, R>,
    result: &mut R,
    recursion: &HitTestRecursionData<'_>,
    root_state: Option<&HitTestingTransformState>,
    z_offset: Option<&Cell<f64>>,
    overflow_controls_only: bool,
) -> Option<LayerHitCandidate> {
    // Source's translation_offset parameter is unused in its definition.
    let state = CreateLocalTransformState(
        layer,
        container,
        container_fragment.map_or_else(|| container.FirstFragment(), |f| f.fragment_data.clone()),
        local_fragment.fragment_data.clone(),
        recursion,
        root_state,
    );
    if !state.AccumulatedTransform().IsInvertible() {
        return None;
    }
    let local_point = state.MappedPoint();
    let bounds = state.BoundsOfMappedArea();
    let location = if recursion.location.IsRectBasedTest() {
        HitTestLocation::FromPointAndQuad(local_point, state.MappedQuad())
    } else {
        HitTestLocation::FromPointAndBoundingBox(local_point, state.BoundsOfMappedQuad())
    };
    let new_recursion = HitTestRecursionData::new(&bounds, &location, recursion.original_location);
    let new_container_fragment = layer.LayoutFacts().fragmented.then_some(local_fragment);
    HitTestLayer(
        layer,
        layer,
        new_container_fragment,
        result,
        &new_recursion,
        true,
        Some(&state),
        z_offset,
        overflow_controls_only,
    )
}

// Source: paint_layer.h:628-632; paint_layer.cc:1842-1888
pub fn HitTestFragmentWithPhase<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    result: &mut R,
    physical: Option<&dyn LayerPhysicalFragment<R>>,
    offset: PhysicalOffset,
    location: &HitTestLocation,
    phase: HitTestPhase,
) -> bool {
    let facts = layer.Facts();
    debug_assert!(facts.self_painting || facts.self_painting_descendant);
    let did_hit = if let Some(physical) = physical {
        physical.MayIntersect(result, location, offset)
            && physical.NodeAtPoint(result, location, offset, phase)
    } else {
        layer.LayoutNodeAtPoint(result, location, offset, phase)
    };
    if !did_hit {
        debug_assert!(
            result.InnerNode().is_none() || (result.Request().list_based && result.ListSize() > 0)
        );
        return false;
    }
    if result.InnerNode().is_none() {
        result.SetInnerNode(Some(EnclosingNode(layer)));
    }
    true
}

// Source: paint_layer.cc:1890-1897
pub fn IsReplacedNormalFlowStackingContext<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
) -> bool {
    if !layer.Features().stacking_context_is_not_stacked {
        return layer.LayoutFacts().svg_foreign_object;
    }
    layer.LayoutFacts().replaced_normal_flow_stacking_context
}
/// Scalar source facts shared by the native Layout/BoxFragment entry and the
/// PaintLayer entry. This keeps the source decision in its owning package.
#[derive(Clone, Copy)]
pub struct ReplacedNormalFlowInlineFacts {
    pub svg_foreign_object: bool,
    pub replaced_normal_flow_stacking_context: bool,
    pub floating: bool,
    pub fragmented: bool,
}
// Source: paint_layer.cc:1890-1907.
pub fn ShouldPaintReplacedNormalFlowInlineLayout(
    facts: ReplacedNormalFlowInlineFacts,
    features: LayerHitTestFeatures,
) -> bool {
    if !facts.svg_foreign_object && !features.replaced_normal_flow_stacking_inline_paint {
        return false;
    }
    let stacking_context = if features.stacking_context_is_not_stacked {
        facts.replaced_normal_flow_stacking_context
    } else {
        facts.svg_foreign_object
    };
    stacking_context && !facts.floating && !facts.fragmented
}
// Source: paint_layer.cc:1899-1907
pub fn ShouldPaintReplacedNormalFlowInline<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
) -> bool {
    let facts = layer.LayoutFacts();
    ShouldPaintReplacedNormalFlowInlineLayout(
        ReplacedNormalFlowInlineFacts {
            svg_foreign_object: facts.svg_foreign_object,
            replaced_normal_flow_stacking_context: facts.replaced_normal_flow_stacking_context,
            floating: facts.floating,
            fragmented: facts.fragmented,
        },
        layer.Features(),
    )
}

// Source: paint_layer.h:608-619; paint_layer.cc:1909-2042
pub fn HitTestChildren<'l, R: LayerHitTestResult>(
    layer: &'l dyn PaintLayerHitTest<R>,
    children: PaintLayerIteration,
    container: &dyn PaintLayerHitTest<R>,
    container_fragment: Option<&LayerFragment<'_, R>>,
    result: &mut R,
    recursion: &HitTestRecursionData<'_>,
    container_state: Option<&HitTestingTransformState>,
    descendants_z: Option<&Cell<f64>>,
    z_offset: Option<&Cell<f64>>,
    local_state: Option<&HitTestingTransformState>,
    depth_sort: bool,
    transition_pseudo_pass: bool,
) -> Option<LayerHitCandidate> {
    let facts = layer.LayoutFacts();
    if !layer.Facts().self_painting_descendant || facts.child_paint_blocked {
        return None;
    }
    if facts.canvas
        && (!layer.Features().canvas_draw_element
            || !facts.canvas_layout_subtree
            || children != PaintLayerIteration::kNormalFlowChildren)
    {
        return None;
    }
    let request = result.Request();
    let stop_layer = result.StopLayer();
    let mut result_layer = None;
    let iterator = if facts.canvas {
        layer.CanvasReversePaintOrder()
    } else {
        layer.ReversePaintOrder(children)
    };
    // Source: paint_layer.cc:1957-2007. Closure executes each candidate against
    // a source temporary result before committing it to the shared result.
    let mut hit_child = |child: &dyn PaintLayerHitTest<R>, controls_only: bool| -> bool {
        if ShouldPaintReplacedNormalFlowInline(child) {
            return false;
        }
        let scoped_transition =
            !facts.view_transition_root && child.LayoutFacts().view_transition_root;
        if scoped_transition != transition_pseudo_pass {
            return false;
        }
        if stop_layer == Some(layer.Identity())
            && !IsHitCandidateForStopNode(child, request.stop_node)
        {
            return false;
        }
        let mut temporary = result.Fresh(recursion.original_location);
        let hit = HitTestLayer(
            child,
            container,
            container_fragment,
            &mut temporary,
            recursion,
            false,
            container_state,
            descendants_z,
            controls_only,
        );
        if request.list_based {
            result.Append(&temporary);
        }
        if IsHitCandidateForDepthOrder(
            hit,
            depth_sort,
            z_offset,
            if layer.Features().container_transform_for_preserve_3d {
                container_state
            } else {
                local_state
            },
            request.visual_overflow,
        ) {
            result_layer = hit;
            if !request.list_based {
                result.Assign(&temporary);
            }
            if !depth_sort {
                return true;
            }
        }
        false
    };
    for child in iterator {
        let mut break_loop = false;
        for controls in layer.OverlayOverflowControlsAfter(child.as_ref()).rev() {
            debug_assert!(controls.Facts().needs_reorder_overlay_controls);
            if hit_child(controls.as_ref(), true) {
                break_loop = true;
                break;
            }
        }
        if break_loop || hit_child(child.as_ref(), false) {
            break;
        }
    }
    result_layer
}

// Source: paint_layer.h:656-657; paint_layer.cc:2103-2115
pub fn HitTestClippedOutByClipPath<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    root: &dyn PaintLayerHitTest<R>,
    location: &HitTestLocation,
) -> bool {
    debug_assert!(layer.LayoutFacts().has_clip_path && layer.Facts().self_painting);
    let origin = layer.LocalOriginInAncestor(root);
    !layer.ClipPathHitTest(&HitTestLocation::WithOffset(location, -origin))
}

// Source: paint_layer.h:658-659; paint_layer.cc:2117-2148
pub fn HitTestClippedOutByBorderShape<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    container: &dyn PaintLayerHitTest<R>,
    location: &HitTestLocation,
) -> bool {
    let facts = layer.LayoutFacts();
    debug_assert!(facts.has_border_shape && facts.clips_overflow_both_axes);
    if !layer.HasLayoutBox() {
        return false;
    }
    let origin = layer.LocalOriginInAncestor(container);
    let location = HitTestLocation::WithOffset(location, -origin);
    let mut border_rect = layer.BorderBoxRect();
    if layer.ShouldApplyOverflowClipMargin() {
        layer.ExpandByBorderOutsetsForClipping(&mut border_rect);
    }
    let Some(outer_path) = layer.BorderShapeOuterPath(border_rect) else {
        return false;
    };
    !location.IntersectsPath(outer_path.as_ref())
}

// Source: paint_layer.h:660-664; paint_layer.cc:2150-2193
pub fn HitTestClippedOutByBorderRadius<R: LayerHitTestResult>(
    layer: &dyn PaintLayerHitTest<R>,
    container: &dyn PaintLayerHitTest<R>,
    container_fragment: Option<&LayerFragment<'_, R>>,
    location: &HitTestLocation,
    clip: &LayerClipRect,
) -> bool {
    debug_assert!(clip.has_radius);
    let data =
        container_fragment.map_or_else(|| container.FirstFragment(), |f| f.fragment_data.clone());
    let transform = data.LocalBorderTransform();
    let container_id = container.LayoutFacts().layout_identity;
    for (current_id, clip) in layer.ContainerBorderRadiusClips() {
        let Some(clip) = clip else {
            continue;
        };
        let mut hit_rect = gfx::RectF::from(*location.BoundingBox());
        let paint_offset = gfx::Vector2dF::from(data.PaintOffset());
        hit_rect.Offset(paint_offset.x(), paint_offset.y());
        let projection = layer.Projection(transform, clip.LocalTransformSpace());
        let quad = gfx::QuadF::from(hit_rect);
        let mapped = gfx::QuadF::new(
            projection.MapPoint(*quad.p1()),
            projection.MapPoint(*quad.p2()),
            projection.MapPoint(*quad.p3()),
            projection.MapPoint(*quad.p4()),
        );
        if !clip.IntersectsQuad(&mapped) {
            return true;
        }
        if current_id == container_id {
            break;
        }
    }
    false
}
