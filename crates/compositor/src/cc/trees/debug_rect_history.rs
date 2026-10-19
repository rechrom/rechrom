// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/debug_rect_history.h:1-132; cc/trees/debug_rect_history.cc:1-22.
use super::property_ids::kInvalidPropertyNodeId;
// Source dependency: cc/debug/debug_colors.h:64.
const kFadeSteps: i32 = 50;
use crate::adapter::{debug_rect::*, draw_property::DrawSurface, geometry::Rect};
use crate::cc::{
    base::math_util::MathUtil,
    debug::layer_tree_debug_state::LayerTreeDebugState,
    input::{main_thread_scrolling_reason::MainThreadRepaintReasons, touch_action::TouchAction},
};
// Source: cc/trees/debug_rect_history.h:43-58.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugRectType {
    kPaint,
    kPropertyChanged,
    kSurfaceDamage,
    kScreenSpace,
    kTouchEventHandler,
    kWheelEventHandler,
    kScrollEventHandler,
    kMainThreadScrollHitTest,
    kMainThreadScrollRepaint,
    kRasterInducingScroll,
    kAnimationBounds,
    kLayoutShift,
    kInteractionContentfulPaint,
    kNavigationContentfulPaint,
}
// Source: cc/trees/debug_rect_history.h:60-84.
#[derive(Clone, Debug)]
pub struct DebugRect {
    pub r#type: DebugRectType,
    pub rect: Rect,
    pub touch_action: TouchAction,
    pub main_thread_scroll_repaint_reasons: MainThreadRepaintReasons,
    pub fade_step: i32,
}
impl DebugRect {
    // Source: cc/trees/debug_rect_history.h:61-75.
    pub fn new(
        r#type: DebugRectType,
        rect: Rect,
        touch_action: TouchAction,
        reasons: MainThreadRepaintReasons,
        fade_step: i32,
    ) -> Self {
        debug_assert!(
            r#type == DebugRectType::kTouchEventHandler || touch_action == TouchAction::kNone
        );
        // This assertion is intentionally the exact source condition (not reversed).
        debug_assert!(r#type != DebugRectType::kMainThreadScrollRepaint || reasons.empty());
        Self {
            r#type,
            rect,
            touch_action,
            main_thread_scroll_repaint_reasons: reasons,
            fade_step,
        }
    }
    // Source: cc/trees/debug_rect_history.h:61-65 (default arguments).
    pub fn FromRect(r#type: DebugRectType, rect: Rect) -> Self {
        Self::new(
            r#type,
            rect,
            TouchAction::kNone,
            MainThreadRepaintReasons::default(),
            1,
        )
    }
}
// Source: cc/trees/debug_rect_history.h:89-128; cc/trees/debug_rect_history.cc:29-31.
pub struct DebugRectHistory {
    debug_rects_: Vec<DebugRect>,
}
impl DebugRectHistory {
    // Source: cc/trees/debug_rect_history.cc:25-27.
    pub fn Create() -> Box<Self> {
        Box::new(Self {
            debug_rects_: Vec::new(),
        })
    }
    // Source: cc/trees/debug_rect_history.h:106-107.
    pub fn debug_rects(&mut self) -> &mut Vec<DebugRect> {
        &mut self.debug_rects_
    }
    pub fn clear_rects(&mut self) {
        self.debug_rects_.clear();
    }
    // Source: cc/trees/debug_rect_history.cc:33-77.
    pub fn SaveDebugRectsForCurrentFrame<C: DebugRectContext>(
        &mut self,
        context: &mut C,
        p: &DebugPropertyTrees<C>,
        hud: Option<C::Layer>,
        surfaces: &[i32],
        state: &LayerTreeDebugState,
    ) {
        self.debug_rects_.retain(|rect| rect.fade_step > 0);
        if state.show_touch_event_handler_rects {
            self.SaveTouchEventHandlerRects(context, p);
        }
        if state.show_wheel_event_handler_rects {
            self.SaveWheelEventHandlerRects(context, p);
        }
        if state.show_scroll_event_handler_rects {
            self.SaveScrollEventHandlerRects(context, p);
        }
        if state.show_main_thread_scroll_hit_test_rects {
            self.SaveMainThreadScrollHitTestRects(context, p);
        }
        if state.show_main_thread_scroll_repaint_rects {
            self.SaveMainThreadScrollRepaintOrRasterInducingScrollRects::<C>(
                p,
                DebugRectType::kMainThreadScrollRepaint,
            );
        }
        if state.show_raster_inducing_scroll_rects {
            self.SaveMainThreadScrollRepaintOrRasterInducingScrollRects::<C>(
                p,
                DebugRectType::kRasterInducingScroll,
            );
        }
        if state.show_contentful_paint_rects || state.show_layout_shift_regions {
            self.SaveWebVitalsDebugRects(context, p, hud);
        }
        if state.show_paint_rects {
            self.SavePaintRects(context, p);
        }
        if state.show_property_changed_rects {
            self.SavePropertyChangedRects(context, p, hud);
        }
        if state.show_surface_damage_rects {
            self.SaveSurfaceDamageRects(context, p, surfaces);
        }
        if state.show_screen_space_rects {
            self.SaveScreenSpaceRects::<C>(p, surfaces);
        }
    }
    // Source: cc/trees/debug_rect_history.cc:79-106.
    fn SaveWebVitalsDebugRects<C: DebugRectContext>(
        &mut self,
        context: &mut C,
        p: &DebugPropertyTrees<C>,
        hud: Option<C::Layer>,
    ) {
        let Some(hud) = hud else {
            return;
        };
        let hud = context.hud_mut(hud).expect("source actual HUD subtype");
        for rect in hud.WebVitalsDebugRects() {
            let r#type = match rect.r#type {
                WebVitalMetricType::kLayoutShift => DebugRectType::kLayoutShift,
                WebVitalMetricType::kInteractionContentfulPaint => {
                    DebugRectType::kInteractionContentfulPaint
                }
                WebVitalMetricType::kNavigationContentfulPaint => {
                    DebugRectType::kNavigationContentfulPaint
                }
            };
            self.debug_rects_.push(DebugRect::new(
                r#type,
                MathUtil::MapEnclosingClippedRect(&hud.ScreenSpaceTransform(p), rect.rect),
                TouchAction::kNone,
                MainThreadRepaintReasons::default(),
                kFadeSteps,
            ));
        }
        hud.ClearWebVitalsDebugRects();
    }
    // Source: cc/trees/debug_rect_history.cc:108-126.
    fn SavePaintRects<C: DebugRectContext>(&mut self, context: &C, p: &DebugPropertyTrees<C>) {
        for id in context.layers() {
            let layer = context.debug_layer(id);
            let invalidation = layer.GetInvalidationRegionForDebugging();
            if invalidation.IsEmpty() || !layer.draws_content() {
                continue;
            }
            for rect in &invalidation {
                self.debug_rects_.push(DebugRect::new(
                    DebugRectType::kPaint,
                    MathUtil::MapEnclosingClippedRect(&layer.ScreenSpaceTransform(p), rect),
                    TouchAction::kNone,
                    MainThreadRepaintReasons::default(),
                    kFadeSteps,
                ));
            }
        }
    }
    // Source: cc/trees/debug_rect_history.cc:128-142.
    fn SavePropertyChangedRects<C: DebugRectContext>(
        &mut self,
        context: &C,
        p: &DebugPropertyTrees<C>,
        hud: Option<C::Layer>,
    ) {
        for id in context.layers() {
            if Some(id) == hud {
                continue;
            }
            let layer = context.debug_layer(id);
            if !layer.LayerPropertyChanged(p) {
                continue;
            }
            let size = layer.bounds(p);
            self.debug_rects_.push(DebugRect::FromRect(
                DebugRectType::kPropertyChanged,
                MathUtil::MapEnclosingClippedRect(
                    &layer.ScreenSpaceTransform(p),
                    Rect::new(0, 0, size.width(), size.height()),
                ),
            ));
        }
    }
    // Source: cc/trees/debug_rect_history.cc:144-158.
    fn SaveSurfaceDamageRects<C: DebugRectContext>(
        &mut self,
        context: &C,
        p: &DebugPropertyTrees<C>,
        surfaces: &[i32],
    ) {
        for i in 0..surfaces.len() {
            let index = surfaces.len() - 1 - i;
            let surface = p
                .effect_tree()
                .GetRenderSurface(surfaces[index])
                .expect("render surface");
            self.debug_rects_.push(DebugRect::FromRect(
                DebugRectType::kSurfaceDamage,
                MathUtil::MapEnclosingClippedRect(
                    &surface.screen_space_transform(),
                    context.GetDamageRect(surface),
                ),
            ));
        }
    }
    // Source: cc/trees/debug_rect_history.cc:160-174.
    fn SaveScreenSpaceRects<C: DebugRectContext>(
        &mut self,
        p: &DebugPropertyTrees<C>,
        surfaces: &[i32],
    ) {
        for i in 0..surfaces.len() {
            let index = surfaces.len() - 1 - i;
            let surface = p
                .effect_tree()
                .GetRenderSurface(surfaces[index])
                .expect("render surface");
            self.debug_rects_.push(DebugRect::FromRect(
                DebugRectType::kScreenSpace,
                MathUtil::MapEnclosingClippedRect(
                    &surface.screen_space_transform(),
                    surface.content_rect(),
                ),
            ));
        }
    }
    // Source: cc/trees/debug_rect_history.cc:176-188.
    fn SaveTouchEventHandlerRects<C: DebugRectContext>(
        &mut self,
        context: &C,
        p: &DebugPropertyTrees<C>,
    ) {
        for id in context.layers() {
            let layer = context.debug_layer(id);
            for (action, region) in layer.touch_action_region() {
                for rect in region {
                    self.debug_rects_.push(DebugRect::new(
                        DebugRectType::kTouchEventHandler,
                        MathUtil::MapEnclosingClippedRect(&layer.ScreenSpaceTransform(p), rect),
                        action,
                        MainThreadRepaintReasons::default(),
                        1,
                    ));
                }
            }
        }
    }
    // Source: cc/trees/debug_rect_history.cc:190-202.
    fn SaveWheelEventHandlerRects<C: DebugRectContext>(
        &mut self,
        context: &C,
        p: &DebugPropertyTrees<C>,
    ) {
        for id in context.layers() {
            let layer = context.debug_layer(id);
            for rect in layer.wheel_event_handler_region() {
                self.debug_rects_.push(DebugRect::FromRect(
                    DebugRectType::kWheelEventHandler,
                    MathUtil::MapEnclosingClippedRect(&layer.ScreenSpaceTransform(p), rect),
                ));
            }
        }
    }
    // Source: cc/trees/debug_rect_history.cc:204-214.
    fn SaveScrollEventHandlerRects<C: DebugRectContext>(
        &mut self,
        context: &C,
        p: &DebugPropertyTrees<C>,
    ) {
        for id in context.layers() {
            let layer = context.debug_layer(id);
            if !context.have_scroll_event_handlers() {
                continue;
            }
            let size = layer.bounds(p);
            self.debug_rects_.push(DebugRect::FromRect(
                DebugRectType::kScrollEventHandler,
                MathUtil::MapEnclosingClippedRect(
                    &layer.ScreenSpaceTransform(p),
                    Rect::new(0, 0, size.width(), size.height()),
                ),
            ));
        }
    }
    // Source: cc/trees/debug_rect_history.cc:216-225.
    fn SaveMainThreadScrollHitTestRects<C: DebugRectContext>(
        &mut self,
        context: &C,
        p: &DebugPropertyTrees<C>,
    ) {
        for id in context.layers() {
            let layer = context.debug_layer(id);
            for rect in layer.main_thread_scroll_hit_test_region() {
                self.debug_rects_.push(DebugRect::FromRect(
                    DebugRectType::kMainThreadScrollHitTest,
                    MathUtil::MapEnclosingClippedRect(&layer.ScreenSpaceTransform(p), rect),
                ));
            }
        }
    }
    // Source: cc/trees/debug_rect_history.cc:227-250.
    fn SaveMainThreadScrollRepaintOrRasterInducingScrollRects<C: DebugRectContext>(
        &mut self,
        p: &DebugPropertyTrees<C>,
        r#type: DebugRectType,
    ) {
        let scroll = p.scroll_tree();
        let transforms = p.transform_tree();
        for node in scroll.nodes() {
            if if r#type == DebugRectType::kMainThreadScrollRepaint {
                scroll.ShouldRealizeScrollsOnMain(node)
            } else {
                scroll.CanRealizeScrollsOnPendingTree(node)
            } {
                if node.transform_id != kInvalidPropertyNodeId {
                    let transform = transforms.Node(node.transform_id);
                    if transform.parent_id != kInvalidPropertyNodeId {
                        let bounds = scroll.container_bounds(node.id, p.state());
                        self.debug_rects_.push(DebugRect::FromRect(
                            r#type,
                            MathUtil::MapEnclosingClippedRect(
                                transforms.ToScreen(transform.parent_id),
                                Rect::new(
                                    node.container_origin.x(),
                                    node.container_origin.y(),
                                    bounds.width(),
                                    bounds.height(),
                                ),
                            ),
                        ));
                    }
                }
            }
        }
    }
}
