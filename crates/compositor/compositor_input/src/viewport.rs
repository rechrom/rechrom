// Translation status: complete
// Source: cc/layers/viewport.h:1-142; cc/layers/viewport.cc:1-413.
//! Compositor-input viewport policy. The facade supplies live tree access
//! through `ViewportRuntime`; this package owns every scroll, pinch, snap and
//! browser-controls decision made by Chromium's cc::Viewport.

use crate::{
    input_types::{ElementId, InputLayerId, InputScrollNodeId},
    scroll_snap_data::{SnapPositionData, SnapPositionType},
    scroll_state::ScrollState,
    scrollbar::ScrollbarOrientation,
};
use compositor_scheduler::TimeDelta;
use compositor_tree::{
    geometry::{cpp_max, Vector2d},
    input_geometry::{Point, PointF, Size, SizeF, Vector2dF},
    LayerTreeId,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct ViewportScrollResult {
    pub consumed_delta: Vector2dF,
    pub content_scrolled_delta: Vector2dF,
    pub outer_viewport_scrolled_delta: Vector2dF,
    pub inner_viewport_scrolled_delta: Vector2dF,
}

/// Copy-only projection of the fields viewport policy reads from ScrollNode.
/// It prevents the input package from depending on the facade's resident tree.
#[derive(Clone, Copy, Debug)]
pub struct ViewportScrollNode {
    pub id: i32,
    pub element_id: ElementId,
    pub container_bounds: Size,
    pub scrolls_inner_viewport: bool,
    pub scrolls_outer_viewport: bool,
    pub user_scrollable_vertical: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewportMetrics {
    pub scroll_operations: u64,
    pub inner_scrolls: u64,
    pub outer_scrolls: u64,
    pub browser_controls_scrolls: u64,
    pub animated_scrolls: u64,
    pub pinch_updates: u64,
    pub snap_attempts: u64,
    pub snap_scrolls: u64,
}

/// Live operations supplied by the compositor facade. Values are deliberately
/// limited to what viewport policy consumes; no LayerTreeImpl or ScrollTree
/// owner crosses the package boundary.
pub trait ViewportRuntime {
    fn ActiveTreeId(&self) -> LayerTreeId;
    fn ScrollNode(&self, id: &InputScrollNodeId) -> ViewportScrollNode;
    fn InnerScrollNode(&self) -> Option<ViewportScrollNode>;
    fn OuterScrollNode(&self) -> Option<ViewportScrollNode>;
    fn CurrentPageScaleFactor(&self) -> f32;
    fn PageScaleFactorForScroll(&self) -> f32;
    fn MinPageScaleFactor(&self) -> f32;
    fn CurrentScrollOffset(&self, element: ElementId) -> PointF;
    fn MaxScrollOffset(&self, node_id: i32) -> PointF;
    fn CurrentlyScrollingNodeId(&self) -> Option<i32>;
    fn FindSnapPosition(&self, node_id: i32, current_position: PointF) -> Option<SnapPositionData>;
    fn ScrollNodeBy(&mut self, id: i32, delta: &Vector2dF, viewport: &mut Viewport) -> Vector2dF;
    fn ClampScrollNodeToMaxScrollOffset(&mut self, id: i32, viewport: &mut Viewport) -> Vector2dF;
    fn SetPageScaleOnActiveTree(&mut self, scale: f32, viewport: &mut Viewport);
    fn ScrollSingleNode(
        &mut self,
        node: InputScrollNodeId,
        delta: Vector2dF,
        point: Point,
        direct: bool,
        viewport: &mut Viewport,
    ) -> Vector2dF;
    fn CanConsumeDelta(
        &mut self,
        state: &ScrollState,
        node: InputScrollNodeId,
        viewport: &mut Viewport,
    ) -> bool;
    fn ComputeScrollDelta(
        &mut self,
        node: InputScrollNodeId,
        delta: Vector2dF,
        viewport: &mut Viewport,
    ) -> Vector2dF;
    fn ScrollAnimationCreate(
        &mut self,
        node: InputScrollNodeId,
        delta: Vector2dF,
        delay: TimeDelta,
        viewport: &mut Viewport,
    );
    fn ScrollBrowserControls(&mut self, delta: &Vector2dF, inertial: bool) -> Vector2dF;
    fn ScrollbarsFor(&mut self, id: ElementId) -> Vec<InputLayerId>;
    fn ScrollbarOrientation(&mut self, layer: &InputLayerId) -> ScrollbarOrientation;
    fn ScrollbarBounds(&mut self, layer: &InputLayerId) -> Size;
    fn StartPageScaleAnimation(
        &mut self,
        target: Point,
        use_anchor: bool,
        scale: f32,
        duration: TimeDelta,
    );
}

pub struct ViewportContext<'a> {
    pub runtime: &'a mut dyn ViewportRuntime,
}

pub type ScrollResult = ViewportScrollResult;
pub const kPinchZoomSnapMarginDips: i32 = 100;

// No Clone: the source deletes copy construction and assignment.
pub struct Viewport {
    pinch_zoom_active_: bool,
    pinch_anchor_adjustment_: Vector2d,
    metrics_: ViewportMetrics,
}

impl Viewport {
    pub fn Create<Owner>(_owner: Owner) -> Box<Self> {
        Box::new(Self::new())
    }

    fn new() -> Self {
        Self {
            pinch_zoom_active_: false,
            pinch_anchor_adjustment_: Vector2d::default(),
            metrics_: ViewportMetrics::default(),
        }
    }

    pub const fn metrics(&self) -> &ViewportMetrics {
        &self.metrics_
    }

    pub fn Pan(&mut self, delta: Vector2dF, c: &mut ViewportContext<'_>) {
        debug_assert!(c.runtime.InnerScrollNode().is_some());
        let mut pending_delta = delta;
        pending_delta.InvScale(c.runtime.CurrentPageScaleFactor());
        let id = c.runtime.InnerScrollNode().unwrap().id;
        c.runtime.ScrollNodeBy(id, &pending_delta, self);
    }

    pub fn ScrollBy(
        &mut self,
        physical_delta: Vector2dF,
        viewport_point: Point,
        is_direct_manipulation: bool,
        affect_browser_controls: bool,
        scroll_outer_viewport: bool,
        is_inertial: bool,
        c: &mut ViewportContext<'_>,
    ) -> ScrollResult {
        self.metrics_.scroll_operations += 1;
        if c.runtime.OuterScrollNode().is_none() {
            return ScrollResult::default();
        }
        let mut scroll_node_delta = physical_delta;
        if affect_browser_controls && self.ShouldBrowserControlsConsumeScroll(physical_delta, c) {
            scroll_node_delta -= self.ScrollBrowserControls(physical_delta, is_inertial, c);
            self.metrics_.browser_controls_scrolls += 1;
        }
        let mut pending_scroll_node_delta = scroll_node_delta;
        let inner = InputScrollNodeId {
            tree: c.runtime.ActiveTreeId(),
            index: c.runtime.InnerScrollNode().unwrap().id,
        };
        let inner_delta = c.runtime.ScrollSingleNode(
            inner,
            pending_scroll_node_delta,
            viewport_point,
            is_direct_manipulation,
            self,
        );
        if !inner_delta.IsZero() {
            self.metrics_.inner_scrolls += 1;
        }
        pending_scroll_node_delta -= inner_delta;
        let mut outer_delta = Vector2dF::default();
        if scroll_outer_viewport {
            let outer = InputScrollNodeId {
                tree: c.runtime.ActiveTreeId(),
                index: c.runtime.OuterScrollNode().unwrap().id,
            };
            outer_delta = c.runtime.ScrollSingleNode(
                outer,
                pending_scroll_node_delta,
                viewport_point,
                is_direct_manipulation,
                self,
            );
            if !outer_delta.IsZero() {
                self.metrics_.outer_scrolls += 1;
            }
            pending_scroll_node_delta -= outer_delta;
        }
        ScrollResult {
            outer_viewport_scrolled_delta: outer_delta,
            inner_viewport_scrolled_delta: inner_delta,
            consumed_delta: physical_delta - Self::AdjustOverscroll(pending_scroll_node_delta),
            content_scrolled_delta: scroll_node_delta - pending_scroll_node_delta,
        }
    }

    pub fn CanScroll(
        &mut self,
        node: &InputScrollNodeId,
        scroll_state: &ScrollState,
        c: &mut ViewportContext<'_>,
    ) -> bool {
        debug_assert!(self.ShouldScroll(node, c));
        let mut result = c.runtime.CanConsumeDelta(
            scroll_state,
            InputScrollNodeId {
                tree: c.runtime.ActiveTreeId(),
                index: c.runtime.InnerScrollNode().unwrap().id,
            },
            self,
        );
        if c.runtime.ScrollNode(node).scrolls_inner_viewport {
            return result;
        }
        result |= c.runtime.CanConsumeDelta(
            scroll_state,
            InputScrollNodeId {
                tree: c.runtime.ActiveTreeId(),
                index: c.runtime.OuterScrollNode().unwrap().id,
            },
            self,
        );
        result
    }

    pub fn SnapIfNeeded(&mut self, c: &mut ViewportContext<'_>) {
        self.metrics_.snap_attempts += 1;
        let Some(node) = c.runtime.OuterScrollNode() else {
            return;
        };
        if c.runtime.CurrentlyScrollingNodeId() == Some(node.id) {
            return;
        }
        let current_position = self.TotalScrollOffset(c);
        let Some(snap) = c.runtime.FindSnapPosition(node.id, current_position) else {
            return;
        };
        if snap.r#type == SnapPositionType::kNone {
            return;
        }
        self.metrics_.snap_scrolls += 1;
        let mut delta = snap.position - current_position;
        delta.Scale(c.runtime.PageScaleFactorForScroll());
        self.ScrollBy(delta, Point::default(), false, false, true, false, c);
    }

    pub fn ComputeClampedDelta(
        &mut self,
        scroll_delta: Vector2dF,
        c: &mut ViewportContext<'_>,
    ) -> Vector2dF {
        let mut source = RuntimeClampedDelta {
            viewport: self,
            context: c,
        };
        Self::ComputeClampedDeltaWith(&mut source, scroll_delta)
    }

    pub fn GetInnerViewportSizeExcludingScrollbars(&self, c: &mut ViewportContext<'_>) -> SizeF {
        let inner = c.runtime.InnerScrollNode().expect("inner viewport node");
        let mut inner_bounds = SizeF::new(
            inner.container_bounds.width() as f32,
            inner.container_bounds.height() as f32,
        );
        let outer = c.runtime.OuterScrollNode().expect("outer viewport node");
        let mut scrollbars_size = SizeF::default();
        for handle in c.runtime.ScrollbarsFor(outer.element_id) {
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

    pub fn ScrollByInnerFirst(&mut self, delta: Vector2dF, c: &mut ViewportContext<'_>) {
        let inner = c.runtime.InnerScrollNode().expect("inner viewport node").id;
        let unused = c.runtime.ScrollNodeBy(inner, &delta, self);
        if let Some(outer) = c.runtime.OuterScrollNode().map(|node| node.id) {
            if !unused.IsZero() {
                c.runtime.ScrollNodeBy(outer, &unused, self);
            }
        }
    }

    fn ShouldAnimateViewport(viewport_delta: Vector2dF, pending_delta: Vector2dF) -> bool {
        cpp_max(viewport_delta.x().abs(), viewport_delta.y().abs())
            > cpp_max(pending_delta.x().abs(), pending_delta.y().abs())
    }

    pub fn ScrollAnimated(
        &mut self,
        delta: Vector2dF,
        delayed_by: TimeDelta,
        c: &mut ViewportContext<'_>,
    ) -> ScrollResult {
        self.metrics_.animated_scrolls += 1;
        let Some(outer_id) = c.runtime.OuterScrollNode().map(|node| node.id) else {
            return ScrollResult::default();
        };
        let outer = InputScrollNodeId {
            tree: c.runtime.ActiveTreeId(),
            index: outer_id,
        };
        let scale_factor = c.runtime.CurrentPageScaleFactor();
        let mut scaled_delta = delta;
        scaled_delta.InvScale(scale_factor);
        let inner = InputScrollNodeId {
            tree: c.runtime.ActiveTreeId(),
            index: c.runtime.InnerScrollNode().unwrap().id,
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
        pending_delta = scaled_delta - inner_delta - outer_delta;
        pending_delta.Scale(scale_factor);
        ScrollResult {
            consumed_delta: delta - pending_delta,
            outer_viewport_scrolled_delta: outer_delta,
            inner_viewport_scrolled_delta: inner_delta,
            ..ScrollResult::default()
        }
    }

    fn SnapPinchAnchorIfWithinMargin(&mut self, anchor: Point, c: &ViewportContext<'_>) {
        let bounds = c.runtime.InnerScrollNode().unwrap().container_bounds;
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

    pub fn PinchUpdate(&mut self, magnify_delta: f32, anchor: Point, c: &mut ViewportContext<'_>) {
        self.metrics_.pinch_updates += 1;
        debug_assert!(c.runtime.InnerScrollNode().is_some());
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
            adjusted_anchor.x() as f32 / page_scale,
            adjusted_anchor.y() as f32 / page_scale,
        );
        c.runtime
            .SetPageScaleOnActiveTree(page_scale * magnify_delta, self);
        page_scale = c.runtime.CurrentPageScaleFactor();
        let new_scale_anchor = PointF::new(
            adjusted_anchor.x() as f32 / page_scale,
            adjusted_anchor.y() as f32 / page_scale,
        );
        let mut movement = previous_scale_anchor - new_scale_anchor;
        movement.Scale(page_scale);
        let inner = c.runtime.InnerScrollNode().unwrap().id;
        movement -= c.runtime.ClampScrollNodeToMaxScrollOffset(inner, self);
        self.Pan(movement, c);
    }

    pub fn PinchEnd(&mut self, anchor: Point, snap_to_min: bool, c: &mut ViewportContext<'_>) {
        if snap_to_min {
            debug_assert!(c.runtime.InnerScrollNode().is_some());
            const kMaxZoomForSnapToMin: f32 = 1.05;
            const kSnapToMinZoomAnimationDuration: TimeDelta = TimeDelta::Milliseconds(200);
            let page_scale = c.runtime.CurrentPageScaleFactor();
            let min_scale = c.runtime.MinPageScaleFactor();
            if page_scale < min_scale * kMaxZoomForSnapToMin && page_scale != min_scale {
                let mut adjusted_anchor = PointF::new(
                    (anchor.x() + self.pinch_anchor_adjustment_.x) as f32,
                    (anchor.y() + self.pinch_anchor_adjustment_.y) as f32,
                );
                adjusted_anchor.Scale(min_scale / page_scale);
                adjusted_anchor += self.TotalScrollOffset(c).OffsetFromOrigin();
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

    pub fn ShouldScroll(&self, node: &InputScrollNodeId, c: &ViewportContext<'_>) -> bool {
        if c.runtime.InnerScrollNode().is_none() || c.runtime.OuterScrollNode().is_none() {
            debug_assert!(c.runtime.InnerScrollNode().is_none());
            debug_assert!(c.runtime.OuterScrollNode().is_none());
            let node = c.runtime.ScrollNode(node);
            debug_assert!(!node.scrolls_inner_viewport);
            debug_assert!(!node.scrolls_outer_viewport);
            return false;
        }
        let node = c.runtime.ScrollNode(node);
        node.scrolls_inner_viewport || node.scrolls_outer_viewport
    }

    fn ScrollBrowserControls(
        &mut self,
        delta: Vector2dF,
        is_inertial: bool,
        c: &mut ViewportContext<'_>,
    ) -> Vector2dF {
        delta - c.runtime.ScrollBrowserControls(&delta, is_inertial)
    }

    fn ShouldBrowserControlsConsumeScroll(
        &self,
        scroll_delta: Vector2dF,
        c: &ViewportContext<'_>,
    ) -> bool {
        if scroll_delta.y() < 0. {
            return true;
        }
        const kEpsilon: f32 = 0.1;
        self.TotalScrollOffset(c).y() + kEpsilon < self.MaxUserReachableTotalScrollOffsetY(c)
    }

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

    pub fn MaxUserReachableTotalScrollOffsetY(&self, c: &ViewportContext<'_>) -> f32 {
        let inner = c.runtime.InnerScrollNode().expect("inner viewport node");
        let outer = c.runtime.OuterScrollNode().map(|node| {
            (
                node.user_scrollable_vertical,
                c.runtime.MaxScrollOffset(node.id).y(),
                c.runtime.CurrentScrollOffset(node.element_id).y(),
            )
        });
        Self::MaxUserReachableTotalScrollOffsetYFromOffsets(
            c.runtime.MaxScrollOffset(inner.id).y(),
            outer,
        )
    }

    pub fn TotalScrollOffset(&self, c: &ViewportContext<'_>) -> PointF {
        let inner = c
            .runtime
            .InnerScrollNode()
            .map(|node| c.runtime.CurrentScrollOffset(node.element_id));
        let outer = c
            .runtime
            .OuterScrollNode()
            .map(|node| c.runtime.CurrentScrollOffset(node.element_id));
        Self::TotalScrollOffsetFromOffsets(inner, outer)
    }

    /// Pure package-owned reduction used by facade clients that already hold a
    /// read-only tree borrow and therefore cannot construct a live runtime.
    pub fn TotalScrollOffsetFromOffsets(inner: Option<PointF>, outer: Option<PointF>) -> PointF {
        let Some(inner) = inner else {
            return PointF::default();
        };
        let mut offset = inner.OffsetFromOrigin();
        if let Some(outer) = outer {
            offset += outer.OffsetFromOrigin();
        }
        PointF::new(offset.x(), offset.y())
    }

    /// `outer` is `(user_scrollable, max_y, current_y)`.
    pub fn MaxUserReachableTotalScrollOffsetYFromOffsets(
        inner_max_y: f32,
        outer: Option<(bool, f32, f32)>,
    ) -> f32 {
        inner_max_y
            + outer
                .map(
                    |(user_scrollable, max_y, current_y)| {
                        if user_scrollable {
                            max_y
                        } else {
                            current_y
                        }
                    },
                )
                .unwrap_or(0.0)
    }

    pub fn ComputeClampedDeltaWith(
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

pub trait ClampedDeltaContext {
    fn InnerDelta(&mut self, delta: Vector2dF) -> Vector2dF;
    fn PageScaleFactorForScroll(&self) -> f32;
    fn OuterDelta(&mut self, delta: Vector2dF) -> Vector2dF;
}

struct RuntimeClampedDelta<'a, 'c> {
    viewport: &'a mut Viewport,
    context: &'a mut ViewportContext<'c>,
}

impl ClampedDeltaContext for RuntimeClampedDelta<'_, '_> {
    fn InnerDelta(&mut self, delta: Vector2dF) -> Vector2dF {
        let id = InputScrollNodeId {
            tree: self.context.runtime.ActiveTreeId(),
            index: self.context.runtime.InnerScrollNode().unwrap().id,
        };
        self.context
            .runtime
            .ComputeScrollDelta(id, delta, self.viewport)
    }

    fn PageScaleFactorForScroll(&self) -> f32 {
        self.context.runtime.PageScaleFactorForScroll()
    }

    fn OuterDelta(&mut self, delta: Vector2dF) -> Vector2dF {
        let id = InputScrollNodeId {
            tree: self.context.runtime.ActiveTreeId(),
            index: self.context.runtime.OuterScrollNode().unwrap().id,
        };
        self.context
            .runtime
            .ComputeScrollDelta(id, delta, self.viewport)
    }
}
