// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/input/scroll_elasticity_helper.h:1-89; cc/input/scroll_elasticity_helper.cc:1-20.
// ScrollElasticityHelper is based on WebKit's ScrollElasticityController.
// Source: cc/input/scroll_elasticity_helper.h:21-44.
/* Copyright (C) 2011 Apple Inc. All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS BE LIABLE FOR
 * ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
 * ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
use crate::adapter::{
    elasticity_context::*,
    geometry::{PointF, Size, Vector2dF},
    input_types::ElementId,
    property_tree::PropertyTreeTransform,
    scroll_elasticity::LayerTreeHostImpl,
};
use crate::cc::trees::{
    damage_reason::DamageReason,
    property_ids::kInvalidPropertyNodeId,
    property_tree::{ScrollTree, TransformTree, TransformTreeContext},
    scroll_elasticity_utils::ShouldAllowOverscrollEffect,
    scroll_node::ScrollNode,
};
// Source: cc/input/scroll_elasticity_helper.h:46-85.
pub trait ScrollElasticityHelper {
    fn ConstrainOverscrollDelta<C: ElasticityQueries>(
        &self,
        host: &C,
        id: ElementId,
        delta: &Vector2dF,
    ) -> Vector2dF;
    fn IsUserOverscrollable<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> bool;
    fn ScrollBounds<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> Size;
    fn StretchAmount<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> Vector2dF;
    fn SetStretchAmount<C: ElasticityContext>(
        &mut self,
        host: &mut C,
        id: ElementId,
        amount: &Vector2dF,
    );
    fn ResetStretchAmounts<C: ElasticityContext>(&mut self, host: &mut C);
    fn ApplyStretchAmountsToPending<C: ElasticityContext>(&mut self, host: &mut C);
    fn ApplyStretchAmountsToActive<C: ElasticityContext>(&mut self, host: &mut C);
    fn ScrollOffset<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> PointF;
    fn MaxScrollOffset<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> PointF;
    fn ScrollBy<C: ElasticityContext>(&mut self, host: &mut C, id: ElementId, delta: &Vector2dF);
    fn RequestOneBeginFrame<C: ElasticityContext>(&mut self, host: &mut C);
    fn AnimationFinished<C: ElasticityAnimationContext>(&mut self, host: &mut C, id: ElementId);
}
// Source: cc/input/scroll_elasticity_helper.cc:21-63.
fn ApplyDrawnStretchAmount<T: PropertyTreeTransform>(
    tree: &mut TransformTree<T>,
    context: &mut TransformTreeContext<'_, T>,
    node: &ScrollNode,
    amount: &Vector2dF,
    is_active: bool,
    tree_effects: &mut dyn ElasticityTreeEffects,
    host: Option<&mut dyn ElasticityHostEffects>,
) {
    if !tree.SetDrawnElasticOverscroll(node.element_id, amount) {
        return;
    }
    let is_root = node.scrolls_inner_viewport || node.scrolls_outer_viewport;
    if node.transform_id != kInvalidPropertyNodeId {
        let transform = tree.MutableNode(node.transform_id);
        #[cfg(target_os = "android")]
        {
            let has_stretch = !amount.IsZero();
            transform.has_potential_animation = has_stretch;
        }
        transform.needs_local_transform_update = true;
        transform.SetTransformChanged(DamageReason::kCompositorScroll);
        tree.set_needs_update(true, context);
    }
    tree_effects.set_needs_update_draw_properties();
    if let Some(host) = host {
        host.SetNeedsRedraw(false, false);
        if is_active && is_root {
            host.SetFullViewportDamage();
        }
    }
}
// Source: cc/input/scroll_elasticity_helper.cc:67-95.
pub struct ScrollElasticityHelperImpl {
    // Rust identity mapping of source non-owning raw_ptr. The current call
    // supplies its live host; no Page/HostImpl reference is retained.
    host_impl_: crate::adapter::layer_tree_impl::LayerTreeHostId,
}
impl ScrollElasticityHelperImpl {
    // Source: cc/input/scroll_elasticity_helper.cc:97-101.
    // The defaulted C++ destructor has the same empty-body Rust drop behavior.
    pub fn new(owner: crate::adapter::layer_tree_impl::LayerTreeHostId) -> Self {
        Self { host_impl_: owner }
    }
    pub fn owner_id(&self) -> crate::adapter::layer_tree_impl::LayerTreeHostId {
        self.host_impl_
    }
    fn ValidateHost<C: ElasticityOwner>(&self, host: &C) {
        assert_eq!(self.host_impl_, host.identity());
    }
    // Source: cc/input/scroll_elasticity_helper.cc:103-108.
    pub fn IsRoot<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> bool {
        self.ValidateHost(host);
        let inner = host
            .InnerViewportScrollNodeId()
            .map(|i| host.GetScrollTree().Node(i));
        let outer = host
            .OuterViewportScrollNodeId()
            .map(|i| host.GetScrollTree().Node(i));
        inner.is_some_and(|n| id == n.element_id) || outer.is_some_and(|n| id == n.element_id)
    }
    // Source: cc/input/scroll_elasticity_helper.cc:110-117.
    // Both const and mutable overloads identify this same live canonical node.
    // The only mutable use (ScrollBy) passes its stable id to the canonical borrow
    // bridge rather than forming a second mutable borrow of the resident tree.
    pub fn GetNode<'a, C: ElasticityQueries>(
        &self,
        host: &'a C,
        id: ElementId,
    ) -> Option<&'a ScrollNode> {
        self.ValidateHost(host);
        host.GetScrollTree().FindNodeFromElementId(id)
    }
    // Source: cc/input/scroll_elasticity_helper.cc:200-253.
    fn ApplyStretchAmountsTo<T: PropertyTreeTransform>(
        active: &ScrollTree,
        target: &ScrollTree,
        transform: &mut TransformTree<T>,
        context: &mut TransformTreeContext<'_, T>,
        is_active: bool,
        tree_effects: &mut dyn ElasticityTreeEffects,
        host_policy: &dyn LayerTreeHostImpl,
        mut host: Option<&mut dyn ElasticityHostEffects>,
    ) {
        let active_map = active.elastic_overscroll();
        let mut to_remove = Vec::new();
        for (&id, _) in transform.drawn_elastic_overscroll() {
            if !active_map.contains_key(&id) {
                to_remove.push(id);
            }
        }
        for id in to_remove {
            let Some(node) = target.FindNodeFromElementId(id) else {
                continue;
            };
            let effect = host
                .as_mut()
                .map(|h| &mut **h as &mut dyn ElasticityHostEffects);
            ApplyDrawnStretchAmount(
                transform,
                context,
                node,
                &Vector2dF::default(),
                is_active,
                tree_effects,
                effect,
            );
        }
        for (&id, amount) in active_map {
            let Some(node) = target.FindNodeFromElementId(id) else {
                continue;
            };
            if !ShouldAllowOverscrollEffect(node, target, Some(host_policy)) {
                continue;
            }
            if is_active && !target.CanRealizeScrollsOnActiveTree(node) {
                continue;
            }
            let effect = host
                .as_mut()
                .map(|h| &mut **h as &mut dyn ElasticityHostEffects);
            ApplyDrawnStretchAmount(
                transform,
                context,
                node,
                amount,
                is_active,
                tree_effects,
                effect,
            );
        }
    }
}
impl ScrollElasticityHelper for ScrollElasticityHelperImpl {
    // Source: cc/input/scroll_elasticity_helper.cc:119-137.
    fn ConstrainOverscrollDelta<C: ElasticityQueries>(
        &self,
        host: &C,
        id: ElementId,
        delta: &Vector2dF,
    ) -> Vector2dF {
        self.ValidateHost(host);
        let node = if self.IsRoot(host, id) {
            host.OuterViewportScrollNodeId()
                .map(|i| host.GetScrollTree().Node(i))
        } else {
            self.GetNode(host, id)
        };
        let Some(node) = node else {
            return Vector2dF::default();
        };
        let mut constrained = *delta;
        if !node.user_scrollable_horizontal {
            constrained.set_x(0.0);
        }
        if !node.user_scrollable_vertical {
            constrained.set_y(0.0);
        }
        constrained
    }
    // Source: cc/input/scroll_elasticity_helper.cc:139-146.
    fn IsUserOverscrollable<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> bool {
        self.ValidateHost(host);
        if self.IsRoot(host, id) {
            return host.enable_elastic_overscroll_on_root();
        }
        host.enable_elastic_overscroll_for_subscroll()
    }
    // Source: cc/input/scroll_elasticity_helper.cc:148-165.
    fn StretchAmount<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> Vector2dF {
        self.ValidateHost(host);
        let node = if self.IsRoot(host, id) {
            host.InnerViewportScrollNodeId()
                .map(|i| host.GetScrollTree().Node(i))
        } else {
            self.GetNode(host, id)
        };
        let Some(node) = node else {
            return Vector2dF::default();
        };
        let stretch = host.active_scroll_tree().GetElasticOverscroll(node);
        assert!(
            ShouldAllowOverscrollEffect(node, host.GetScrollTree(), Some(host)) || stretch.IsZero()
        );
        stretch
    }
    // Source: cc/input/scroll_elasticity_helper.cc:167-172.
    fn ScrollBounds<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> Size {
        self.ValidateHost(host);
        let node = if self.IsRoot(host, id) {
            host.OuterViewportScrollNodeId()
                .map(|i| host.GetScrollTree().Node(i))
        } else {
            self.GetNode(host, id)
        };
        node.map_or(Size::default(), |n| n.container_bounds)
    }
    // Source: cc/input/scroll_elasticity_helper.cc:174-186.
    fn ResetStretchAmounts<C: ElasticityContext>(&mut self, host: &mut C) {
        self.ValidateHost(host);
        let to_remove: Vec<_> = host
            .active_scroll_tree()
            .elastic_overscroll()
            .iter()
            .map(|(&id, &v)| (id, v))
            .collect();
        for (id, _) in to_remove {
            self.SetStretchAmount(host, id, &Vector2dF::default());
        }
        host.WithActiveTreeMutation(|tree| tree.scroll_tree.elastic_overscroll_mut().clear());
    }
    // Source: cc/input/scroll_elasticity_helper.cc:188-194.
    fn ApplyStretchAmountsToPending<C: ElasticityContext>(&mut self, host: &mut C) {
        self.ValidateHost(host);
        if host.CommitsToActiveTree() {
            return;
        }
        host.WithActiveAndPendingMutation(|active, mut target| {
            let context = &mut target.context;
            let mut transforms = TransformTreeContext {
                owner: context.owner,
                scroll_tree: target.scroll_tree,
                clip_tree: context.clip_tree,
                state: context.state,
                cached_data: context.cached_data,
            };
            Self::ApplyStretchAmountsTo(
                active,
                target.scroll_tree,
                context.transform_tree,
                &mut transforms,
                target.is_active_tree,
                target.tree_effects,
                target.host_policy,
                target.host_effects,
            );
        });
    }
    // Source: cc/input/scroll_elasticity_helper.cc:196-198.
    fn ApplyStretchAmountsToActive<C: ElasticityContext>(&mut self, host: &mut C) {
        self.ValidateHost(host);
        host.WithActiveTreeMutation(|mut target| {
            let context = &mut target.context;
            let mut transforms = TransformTreeContext {
                owner: context.owner,
                scroll_tree: target.scroll_tree,
                clip_tree: context.clip_tree,
                state: context.state,
                cached_data: context.cached_data,
            };
            Self::ApplyStretchAmountsTo(
                target.scroll_tree,
                target.scroll_tree,
                context.transform_tree,
                &mut transforms,
                target.is_active_tree,
                target.tree_effects,
                target.host_policy,
                target.host_effects,
            );
        });
    }
    // Source: cc/input/scroll_elasticity_helper.cc:255-290.
    fn SetStretchAmount<C: ElasticityContext>(
        &mut self,
        host: &mut C,
        id: ElementId,
        amount: &Vector2dF,
    ) {
        self.ValidateHost(host);
        let node = if self.IsRoot(host, id) {
            host.InnerViewportScrollNodeId()
                .map(|i| host.GetScrollTree().Node(i))
        } else {
            self.GetNode(host, id)
        };
        let Some(node) = node else {
            return;
        };
        let node_id = node.id;
        let effective = if ShouldAllowOverscrollEffect(node, host.GetScrollTree(), Some(host)) {
            *amount
        } else {
            Vector2dF::default()
        };
        host.WithActiveTreeMutation(|mut tree| {
            if !tree
                .scroll_tree
                .SetElasticOverscrollById(node_id, &effective)
            {
                return;
            }
            let node = tree.scroll_tree.Node(node_id);
            if tree.scroll_tree.CanRealizeScrollsOnActiveTree(node) {
                let context = &mut tree.context;
                let mut transforms = TransformTreeContext {
                    owner: context.owner,
                    scroll_tree: tree.scroll_tree,
                    clip_tree: context.clip_tree,
                    state: context.state,
                    cached_data: context.cached_data,
                };
                ApplyDrawnStretchAmount(
                    context.transform_tree,
                    &mut transforms,
                    node,
                    &effective,
                    tree.is_active_tree,
                    tree.tree_effects,
                    tree.host_effects,
                );
            } else if tree.scroll_tree.CanRealizeScrollsOnPendingTree(node) {
                tree.host_effects
                    .expect("helper host")
                    .RequestImplSideInvalidationForRasterInducingScroll(id);
            }
        });
    }
    // Source: cc/input/scroll_elasticity_helper.cc:291-297.
    fn ScrollOffset<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> PointF {
        self.ValidateHost(host);
        if self.IsRoot(host, id) {
            return host.TotalScrollOffset(None);
        }
        host.TotalScrollOffset(Some(id))
    }
    // Source: cc/input/scroll_elasticity_helper.cc:298-304.
    fn MaxScrollOffset<C: ElasticityQueries>(&self, host: &C, id: ElementId) -> PointF {
        self.ValidateHost(host);
        if self.IsRoot(host, id) {
            return host.TotalMaxScrollOffset(None);
        }
        host.TotalMaxScrollOffset(Some(id))
    }
    // Source: cc/input/scroll_elasticity_helper.cc:306-322.
    fn ScrollBy<C: ElasticityContext>(&mut self, host: &mut C, id: ElementId, delta: &Vector2dF) {
        self.ValidateHost(host);
        let node_id = if self.IsRoot(host, id) {
            host.OuterViewportScrollNodeId()
                .or_else(|| host.InnerViewportScrollNodeId())
        } else {
            self.GetNode(host, id).map(|n| n.id)
        };
        if let Some(node_id) = node_id {
            host.ScrollByNodeId(node_id, delta);
        }
    }
    // Source: cc/input/scroll_elasticity_helper.cc:324-326.
    fn RequestOneBeginFrame<C: ElasticityContext>(&mut self, host: &mut C) {
        self.ValidateHost(host);
        host.SetNeedsOneBeginImplFrame();
    }
    // Source: cc/input/scroll_elasticity_helper.cc:328-330.
    fn AnimationFinished<C: ElasticityAnimationContext>(&mut self, host: &mut C, id: ElementId) {
        self.ValidateHost(host);
        host.ElasticOverscrollAnimationFinished(id);
    }
}
// Source: cc/input/scroll_elasticity_helper.h:52-53; cc/input/scroll_elasticity_helper.cc:333-336.
pub fn CreateForLayerTreeHostImpl<C: ElasticityOwner>(host: &C) -> Box<ScrollElasticityHelperImpl> {
    Box::new(ScrollElasticityHelperImpl::new(host.identity()))
}
