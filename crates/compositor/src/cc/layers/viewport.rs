// Translation status: complete
// Source: cc/layers/viewport.h:1-142; cc/layers/viewport.cc:1-413.
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/viewport.h:1-35,107-142; cc/layers/viewport.cc:1-19,413.
// Rust live split contexts replace the non-owning host pointer. No whole tree,
// node, layer, or property output is copied to satisfy aliasing.
use crate::adapter::input_handler::InputScrollNodeId;
use crate::adapter::{
    geometry::{cpp_max, Point, PointF, SizeF, Vector2d, Vector2dF},
    layer_tree_impl::{LayerTreeHostId, LayerTreeTypes},
    time::TimeDelta,
    viewport::{ScrollbarOrientation, ViewportContext},
};
use crate::cc::{
    input::{
        input_handler::ViewportScrollResult, scroll_snap_data::SnapPositionType,
        scroll_state::ScrollState, snap_selection_strategy::SnapSelectionStrategy,
    },
    trees::{property_tree::ScrollTree, scroll_node::ScrollNode},
};
// Source: cc/layers/viewport.h:36.
pub type ScrollResult = ViewportScrollResult;
// Source: cc/layers/viewport.h:41.
pub const kPinchZoomSnapMarginDips: i32 = 100;
// Source: cc/layers/viewport.h:129-135. No Clone: source deletes copy/assignment.
pub struct Viewport {
    pinch_zoom_active_: bool,
    pinch_anchor_adjustment_: Vector2d,
}
impl Viewport {
    // Source: cc/layers/viewport.cc:22-30; cc/layers/viewport.h:43-46.
    pub fn Create(owner: LayerTreeHostId) -> Box<Self> {
        Box::new(Self::new(owner))
    }
    fn new(_owner: LayerTreeHostId) -> Self {
        Self {
            pinch_zoom_active_: false,
            pinch_anchor_adjustment_: Vector2d::default(),
        }
    }
    // Source: cc/layers/viewport.cc:32-39.
    pub fn Pan<B: LayerTreeTypes>(&mut self, delta: Vector2dF, c: &mut ViewportContext<'_, B>) {
        debug_assert!(Self::InnerScrollNode(c).is_some());
        let mut pending_delta = delta;
        let page_scale = c.runtime.CurrentPageScaleFactor();
        pending_delta.InvScale(page_scale);
        let id = Self::InnerScrollNode(c).unwrap().id;
        c.runtime.ScrollNodeBy(id, &pending_delta, self);
    }
    // Source: cc/layers/viewport.cc:41-80.
    pub fn ScrollBy<B: LayerTreeTypes>(
        &mut self,
        physical_delta: Vector2dF,
        viewport_point: Point,
        is_direct_manipulation: bool,
        affect_browser_controls: bool,
        scroll_outer_viewport: bool,
        is_inertial: bool,
        c: &mut ViewportContext<'_, B>,
    ) -> ScrollResult {
        if Self::OuterScrollNode(c).is_none() {
            return ScrollResult::default();
        }
        let mut scroll_node_delta = physical_delta;
        if affect_browser_controls && self.ShouldBrowserControlsConsumeScroll(physical_delta, c) {
            scroll_node_delta -= self.ScrollBrowserControls(physical_delta, is_inertial, c);
        }
        let mut pending_scroll_node_delta = scroll_node_delta;
        let inner = InputScrollNodeId {
            tree: c.runtime.ActiveTree().id,
            index: Self::InnerScrollNode(c).unwrap().id,
        };
        let inner_delta = c.runtime.ScrollSingleNode(
            inner,
            pending_scroll_node_delta,
            viewport_point,
            is_direct_manipulation,
            self,
        );
        pending_scroll_node_delta -= inner_delta;
        let mut outer_delta = Vector2dF::default();
        if scroll_outer_viewport {
            let outer = InputScrollNodeId {
                tree: c.runtime.ActiveTree().id,
                index: Self::OuterScrollNode(c).unwrap().id,
            };
            outer_delta = c.runtime.ScrollSingleNode(
                outer,
                pending_scroll_node_delta,
                viewport_point,
                is_direct_manipulation,
                self,
            );
            pending_scroll_node_delta -= outer_delta;
        }
        let mut result = ScrollResult::default();
        result.outer_viewport_scrolled_delta = outer_delta;
        result.inner_viewport_scrolled_delta = inner_delta;
        result.consumed_delta = physical_delta - Self::AdjustOverscroll(pending_scroll_node_delta);
        result.content_scrolled_delta = scroll_node_delta - pending_scroll_node_delta;
        result
    }
    // Source: cc/layers/viewport.cc:82-98.
    pub fn CanScroll<B: LayerTreeTypes>(
        &mut self,
        node: &InputScrollNodeId,
        scroll_state: &ScrollState,
        c: &mut ViewportContext<'_, B>,
    ) -> bool {
        debug_assert!(self.ShouldScroll(node, c));
        let mut result = c.runtime.CanConsumeDelta(
            scroll_state,
            InputScrollNodeId {
                tree: c.runtime.ActiveTree().id,
                index: Self::InnerScrollNode(c).unwrap().id,
            },
            self,
        );
        if c.runtime.ScrollNode(node).scrolls_inner_viewport {
            return result;
        }
        result |= c.runtime.CanConsumeDelta(
            scroll_state,
            InputScrollNodeId {
                tree: c.runtime.ActiveTree().id,
                index: Self::OuterScrollNode(c).unwrap().id,
            },
            self,
        );
        result
    }
    // Source: cc/layers/viewport.cc:100-127.
    pub fn SnapIfNeeded<B: LayerTreeTypes>(&mut self, c: &mut ViewportContext<'_, B>) {
        let Some(node) = Self::OuterScrollNode(c) else {
            return;
        };
        let Some(data) = node.snap_container_data.as_ref() else {
            return;
        };
        if Self::scroll_tree(c)
            .CurrentlyScrollingNode()
            .is_some_and(|n| n.id == node.id)
        {
            return;
        }
        let current_position = self.TotalScrollOffset(c);
        let snap = data.FindSnapPosition(&*<dyn SnapSelectionStrategy>::CreateForTargetElement(
            current_position,
        ));
        if snap.r#type == SnapPositionType::kNone {
            return;
        }
        let mut delta = snap.position - current_position;
        delta.Scale(c.runtime.PageScaleFactorForScroll());
        self.ScrollBy(delta, Point::default(), false, false, true, false, c);
    }
    // Source: cc/layers/viewport.cc:129-154.
    pub fn ComputeClampedDelta<B: LayerTreeTypes>(
        &mut self,
        scroll_delta: Vector2dF,
        c: &mut ViewportContext<'_, B>,
    ) -> Vector2dF {
        let mut source = RuntimeClampedDelta {
            viewport: self,
            context: c,
        };
        Self::ComputeClampedDeltaWith(&mut source, scroll_delta)
    }
    // Source: cc/layers/viewport.cc:156-174.
    pub fn GetInnerViewportSizeExcludingScrollbars<B: LayerTreeTypes>(
        &self,
        c: &mut ViewportContext<'_, B>,
    ) -> SizeF {
        debug_assert!(Self::InnerScrollNode(c).is_some());
        let inner_node = Self::InnerScrollNode(c).unwrap();
        let mut inner_bounds = SizeF::new(
            inner_node.container_bounds.width() as f32,
            inner_node.container_bounds.height() as f32,
        );
        let outer_node = Self::OuterScrollNode(c).unwrap();
        let scrollbars = c.runtime.ScrollbarsFor(outer_node.element_id);
        let mut scrollbars_size = SizeF::default();
        for handle in scrollbars {
            let orientation = c.runtime.ScrollbarOrientation(&handle);
            if orientation == ScrollbarOrientation::kVertical {
                scrollbars_size.set_width(c.runtime.ScrollbarBounds(&handle).width() as f32);
            } else {
                debug_assert_eq!(orientation, ScrollbarOrientation::kHorizontal);
                scrollbars_size.set_height(c.runtime.ScrollbarBounds(&handle).height() as f32);
            }
        }
        inner_bounds.set_width(inner_bounds.width() - scrollbars_size.width());
        inner_bounds.set_height(inner_bounds.height() - scrollbars_size.height());
        inner_bounds
    }
    // Source: cc/layers/viewport.cc:176-186.
    pub fn ScrollByInnerFirst<B: LayerTreeTypes>(
        &mut self,
        delta: Vector2dF,
        c: &mut ViewportContext<'_, B>,
    ) {
        debug_assert!(Self::InnerScrollNode(c).is_some());
        let inner = Self::InnerScrollNode(c).unwrap().id;
        let unused = c.runtime.ScrollNodeBy(inner, &delta, self);
        if let Some(outer) = Self::OuterScrollNode(c).map(|n| n.id) {
            if !unused.IsZero() {
                c.runtime.ScrollNodeBy(outer, &unused, self);
            }
        }
    }
    // Source: cc/layers/viewport.cc:188-195.
    fn ShouldAnimateViewport(viewport_delta: Vector2dF, pending_delta: Vector2dF) -> bool {
        let max_dim_viewport_delta = cpp_max(viewport_delta.x().abs(), viewport_delta.y().abs());
        let max_dim_pending_delta = cpp_max(pending_delta.x().abs(), pending_delta.y().abs());
        max_dim_viewport_delta > max_dim_pending_delta
    }
    // Source: cc/layers/viewport.cc:197-239.
    pub fn ScrollAnimated<B: LayerTreeTypes>(
        &mut self,
        delta: Vector2dF,
        delayed_by: TimeDelta,
        c: &mut ViewportContext<'_, B>,
    ) -> ScrollResult {
        let Some(outer) = Self::OuterScrollNode(c).map(|n| n.id) else {
            return ScrollResult::default();
        };
        let outer = InputScrollNodeId {
            tree: c.runtime.ActiveTree().id,
            index: outer,
        };
        let scale_factor = c.runtime.CurrentPageScaleFactor();
        let mut scaled_delta = delta;
        scaled_delta.InvScale(scale_factor);
        let inner = InputScrollNodeId {
            tree: c.runtime.ActiveTree().id,
            index: Self::InnerScrollNode(c).unwrap().id,
        };
        let inner_delta = c.runtime.ComputeScrollDelta(inner, delta, self);
        let mut pending_delta = scaled_delta - inner_delta;
        pending_delta.Scale(scale_factor);
        let outer_delta = c.runtime.ComputeScrollDelta(outer, pending_delta, self);
        if inner_delta.IsZero() && outer_delta.IsZero() {
            return ScrollResult::default();
        }
        if Self::ShouldAnimateViewport(inner_delta, outer_delta) {
            c.runtime.ScrollNodeBy(outer.index, &outer_delta, self);
            c.runtime
                .ScrollAnimationCreate(inner, inner_delta, delayed_by, self);
        } else {
            c.runtime.ScrollNodeBy(inner.index, &inner_delta, self);
            c.runtime
                .ScrollAnimationCreate(outer, outer_delta, delayed_by, self);
        }
        let mut result = ScrollResult::default();
        pending_delta = scaled_delta - inner_delta - outer_delta;
        pending_delta.Scale(scale_factor);
        result.consumed_delta = delta - pending_delta;
        result.outer_viewport_scrolled_delta = outer_delta;
        result.inner_viewport_scrolled_delta = inner_delta;
        result
    }
    // Source: cc/layers/viewport.cc:241-253.
    fn SnapPinchAnchorIfWithinMargin<B: LayerTreeTypes>(
        &mut self,
        anchor: Point,
        c: &ViewportContext<'_, B>,
    ) {
        let bounds = Self::InnerScrollNode(c).unwrap().container_bounds;
        let viewport_size = SizeF::new(bounds.width() as f32, bounds.height() as f32);
        if anchor.x() < kPinchZoomSnapMarginDips {
            self.pinch_anchor_adjustment_.x = -anchor.x();
        } else if anchor.x() as f32 > viewport_size.width() - kPinchZoomSnapMarginDips as f32 {
            self.pinch_anchor_adjustment_.x = (viewport_size.width() - anchor.x() as f32) as i32;
        }
        if anchor.y() < kPinchZoomSnapMarginDips {
            self.pinch_anchor_adjustment_.y = -anchor.y();
        } else if anchor.y() as f32 > viewport_size.height() - kPinchZoomSnapMarginDips as f32 {
            self.pinch_anchor_adjustment_.y = (viewport_size.height() - anchor.y() as f32) as i32;
        }
    }
    // Source: cc/layers/viewport.cc:255-291.
    pub fn PinchUpdate<B: LayerTreeTypes>(
        &mut self,
        magnify_delta: f32,
        anchor: Point,
        c: &mut ViewportContext<'_, B>,
    ) {
        debug_assert!(Self::InnerScrollNode(c).is_some());
        if !self.pinch_zoom_active_ {
            self.SnapPinchAnchorIfWithinMargin(anchor, c);
            self.pinch_zoom_active_ = true;
        }
        let adjusted_anchor = Point::new(
            anchor.x() + self.pinch_anchor_adjustment_.x,
            anchor.y() + self.pinch_anchor_adjustment_.y,
        );
        let mut page_scale = c.runtime.CurrentPageScaleFactor();
        let previous_scale_anchor = PointF::new(
            adjusted_anchor.x() as f32 * (1.0f32 / page_scale),
            adjusted_anchor.y() as f32 * (1.0f32 / page_scale),
        );
        c.runtime
            .SetPageScaleOnActiveTree(page_scale * magnify_delta, self);
        page_scale = c.runtime.CurrentPageScaleFactor();
        let new_scale_anchor = PointF::new(
            adjusted_anchor.x() as f32 * (1.0f32 / page_scale),
            adjusted_anchor.y() as f32 * (1.0f32 / page_scale),
        );
        let mut movement = previous_scale_anchor - new_scale_anchor;
        movement.Scale(page_scale);
        let inner = Self::InnerScrollNode(c).unwrap().id;
        movement -= c.runtime.ClampScrollNodeToMaxScrollOffset(inner, self);
        self.Pan(movement, c);
    }
    // Source: cc/layers/viewport.cc:293-319.
    pub fn PinchEnd<B: LayerTreeTypes>(
        &mut self,
        anchor: Point,
        snap_to_min: bool,
        c: &mut ViewportContext<'_, B>,
    ) {
        if snap_to_min {
            debug_assert!(c.runtime.ActiveTree().InnerViewportScrollNode().is_some());
            const kMaxZoomForSnapToMin: f32 = 1.05;
            const kSnapToMinZoomAnimationDuration: TimeDelta = TimeDelta::Milliseconds(200);
            let page_scale = c.runtime.CurrentPageScaleFactor();
            let min_scale = c.runtime.ActiveTree().min_page_scale_factor();
            if page_scale < min_scale * kMaxZoomForSnapToMin && page_scale != min_scale {
                let mut adjusted_anchor = PointF::new(
                    (anchor.x() + self.pinch_anchor_adjustment_.x) as f32,
                    (anchor.y() + self.pinch_anchor_adjustment_.y) as f32,
                );
                adjusted_anchor.Scale(min_scale / page_scale);
                adjusted_anchor += self.TotalScrollOffset(c).OffsetFromOrigin();
                // gfx::ToRoundedPoint uses round-to-nearest with halfway away from zero.
                c.runtime.StartPageScaleAnimation(
                    Point::new(
                        adjusted_anchor.x().round() as i32,
                        adjusted_anchor.y().round() as i32,
                    ),
                    true,
                    min_scale,
                    kSnapToMinZoomAnimationDuration,
                );
            }
        }
        self.pinch_anchor_adjustment_ = Vector2d::default();
        self.pinch_zoom_active_ = false;
    }
    // Source: cc/layers/viewport.cc:321-333.
    pub fn ShouldScroll<B: LayerTreeTypes>(
        &self,
        node: &InputScrollNodeId,
        c: &ViewportContext<'_, B>,
    ) -> bool {
        if Self::InnerScrollNode(c).is_none() || Self::OuterScrollNode(c).is_none() {
            debug_assert!(Self::InnerScrollNode(c).is_none());
            debug_assert!(Self::OuterScrollNode(c).is_none());
            debug_assert!(!c.runtime.ScrollNode(node).scrolls_inner_viewport);
            debug_assert!(!c.runtime.ScrollNode(node).scrolls_outer_viewport);
            return false;
        }
        c.runtime.ScrollNode(node).scrolls_inner_viewport
            || c.runtime.ScrollNode(node).scrolls_outer_viewport
    }
    // Source: cc/layers/viewport.cc:335-341.
    fn ScrollBrowserControls<B: LayerTreeTypes>(
        &mut self,
        delta: Vector2dF,
        is_inertial: bool,
        c: &mut ViewportContext<'_, B>,
    ) -> Vector2dF {
        let excess = c.runtime.ScrollBrowserControls(&delta, is_inertial);
        delta - excess
    }
    // Source: cc/layers/viewport.cc:343-354.
    fn ShouldBrowserControlsConsumeScroll<B: LayerTreeTypes>(
        &self,
        scroll_delta: Vector2dF,
        c: &ViewportContext<'_, B>,
    ) -> bool {
        if scroll_delta.y() < 0. {
            return true;
        }
        const kEpsilon: f32 = 0.1;
        if self.TotalScrollOffset(c).y() + kEpsilon < self.MaxUserReachableTotalScrollOffsetY(c) {
            return true;
        }
        false
    }
    // Source: cc/layers/viewport.cc:356-368.
    fn AdjustOverscroll(delta: Vector2dF) -> Vector2dF {
        const kEpsilon: f32 = 0.1;
        let mut adjusted = delta;
        if adjusted.x().abs() < kEpsilon {
            adjusted.set_x(0.);
        }
        if adjusted.y().abs() < kEpsilon {
            adjusted.set_y(0.);
        }
        adjusted
    }
    // Source: cc/layers/viewport.cc:370-381.
    pub fn MaxUserReachableTotalScrollOffsetY<B: LayerTreeTypes>(
        &self,
        c: &ViewportContext<'_, B>,
    ) -> f32 {
        Self::MaxUserReachableTotalScrollOffsetYForTree(c.runtime.ActiveTree())
    }
    // Source: cc/layers/viewport.cc:370-381.
    // Rust borrow split: this source getter only reads its actual resident tree.
    pub(crate) fn MaxUserReachableTotalScrollOffsetYForTree<B: LayerTreeTypes>(
        tree: &crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    ) -> f32 {
        let (scroll, transforms, state) = tree.property_trees().scroll_tree_read_context();
        let mut y = scroll
            .MaxScrollOffset(
                tree.InnerViewportScrollNode().unwrap().id,
                transforms,
                state,
            )
            .y();
        if let Some(outer) = tree.OuterViewportScrollNode() {
            if outer.user_scrollable_vertical {
                y += scroll.MaxScrollOffset(outer.id, transforms, state).y();
            } else {
                y += scroll
                    .current_scroll_offset(outer.element_id, tree.property_trees().state())
                    .y();
            }
        }
        y
    }

    // Source: cc/layers/viewport.cc:383-399.
    pub fn TotalScrollOffset<B: LayerTreeTypes>(&self, c: &ViewportContext<'_, B>) -> PointF {
        Self::TotalScrollOffsetForTree(c.runtime.ActiveTree())
    }
    // Source: cc/layers/viewport.cc:383-399.
    // Rust borrow split: this source getter only reads its actual resident tree.
    pub(crate) fn TotalScrollOffsetForTree<B: LayerTreeTypes>(
        tree: &crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    ) -> PointF {
        let Some(inner) = tree.InnerViewportScrollNode() else {
            return PointF::default();
        };
        let scroll = tree.property_trees().scroll_tree();
        let mut offset = scroll
            .current_scroll_offset(inner.element_id, tree.property_trees().state())
            .OffsetFromOrigin();
        if let Some(outer) = tree.OuterViewportScrollNode() {
            offset += scroll
                .current_scroll_offset(outer.element_id, tree.property_trees().state())
                .OffsetFromOrigin();
        }
        PointF::new(offset.x(), offset.y())
    }

    // Source: cc/layers/viewport.cc:401-411.
    fn InnerScrollNode<'a, B: LayerTreeTypes>(
        c: &'a ViewportContext<'_, B>,
    ) -> Option<&'a ScrollNode> {
        c.runtime.ActiveTree().InnerViewportScrollNode()
    }
    fn OuterScrollNode<'a, B: LayerTreeTypes>(
        c: &'a ViewportContext<'_, B>,
    ) -> Option<&'a ScrollNode> {
        c.runtime.ActiveTree().OuterViewportScrollNode()
    }
    fn scroll_tree<'a, B: LayerTreeTypes>(c: &'a ViewportContext<'_, B>) -> &'a ScrollTree {
        c.runtime.ActiveTree().property_trees().scroll_tree()
    }
}

/// Synchronous source calls used by Viewport::ComputeClampedDelta. Each call
/// resolves the actual resident node at that source access point.
pub(crate) trait ClampedDeltaContext {
    fn InnerDelta(&mut self, delta: Vector2dF) -> Vector2dF;
    fn PageScaleFactorForScroll(&self) -> f32;
    fn OuterDelta(&mut self, delta: Vector2dF) -> Vector2dF;
}
impl Viewport {
    // Source: cc/layers/viewport.cc:129-154.
    pub(crate) fn ComputeClampedDeltaWith(
        source: &mut impl ClampedDeltaContext,
        scroll_delta: Vector2dF,
    ) -> Vector2dF {
        let inner_delta = source.InnerDelta(scroll_delta);
        let page_scale = source.PageScaleFactorForScroll();
        let mut unscaled_delta = scroll_delta;
        unscaled_delta.InvScale(page_scale);
        let mut remaining_delta = unscaled_delta - inner_delta;
        remaining_delta.Scale(page_scale);
        let outer_delta = source.OuterDelta(remaining_delta);
        let mut combined_delta = inner_delta + outer_delta;
        combined_delta.Scale(page_scale);
        combined_delta
    }
}
struct RuntimeClampedDelta<'a, 'c, B: LayerTreeTypes> {
    viewport: &'a mut Viewport,
    context: &'a mut ViewportContext<'c, B>,
}
impl<B: LayerTreeTypes> ClampedDeltaContext for RuntimeClampedDelta<'_, '_, B> {
    fn InnerDelta(&mut self, delta: Vector2dF) -> Vector2dF {
        let node = Viewport::InnerScrollNode(self.context).unwrap();
        let id = InputScrollNodeId {
            tree: self.context.runtime.ActiveTree().id,
            index: node.id,
        };
        self.context
            .runtime
            .ComputeScrollDelta(id, delta, self.viewport)
    }
    fn PageScaleFactorForScroll(&self) -> f32 {
        self.context.runtime.PageScaleFactorForScroll()
    }
    fn OuterDelta(&mut self, delta: Vector2dF) -> Vector2dF {
        let node = Viewport::OuterScrollNode(self.context).unwrap();
        let id = InputScrollNodeId {
            tree: self.context.runtime.ActiveTree().id,
            index: node.id,
        };
        self.context
            .runtime
            .ComputeScrollDelta(id, delta, self.viewport)
    }
}
