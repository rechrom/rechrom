use crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryRead;
// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/scrollbar_layer_impl_base.h:1-19,146-162;
//         cc/layers/scrollbar_layer_impl_base.cc:1-16,396.
use super::layer_impl::LayerImpl;
use crate::{
    adapter::{
        geometry::{Point, PointF, Rect, RectF, ToEnclosingRect},
        input_types::ElementId,
        layer_impl::{LayerImplContext, LayerImplTypes},
        layer_tree_impl::LayerTreeId,
        scrollbar_layer_impl_base::*,
    },
    cc::{
        base::math_util::MathUtil,
        input::{
            scroll_utils::ScrollUtils,
            scrollbar::{ScrollbarOrientation, ScrollbarPart},
        },
        trees::{
            damage_reason::{DamageReason, DamageReasonSet},
            layer_tree_settings::ScrollbarAnimator,
            property_ids::kInvalidPropertyNodeId,
        },
    },
};
// Source: cc/layers/scrollbar_layer_impl_base.h:20-145. Native resident owner.
pub struct ScrollbarLayerImplBase<B: LayerImplTypes> {
    base_: LayerImpl<B>,
    scroll_element_id_: ElementId,
    is_overlay_scrollbar_: bool,
    is_web_test_: bool,
    opacity_changed_for_fade_out_animation_: bool,
    property_changed_for_other_reasons_: bool,
    thumb_thickness_scale_factor_: f32,
    current_pos_: f32,
    clip_layer_length_: f32,
    scroll_layer_length_: f32,
    orientation_: ScrollbarOrientation,
    is_left_side_vertical_scrollbar_: bool,
    vertical_adjust_: f32,
    has_find_in_page_tickmarks_: bool,
    // Rust destructor permission only, not a second native lifecycle/state.
    shutdown_: bool,
}
/// Actual sibling-only native scrollbar vtable. Every query gets the same live
/// resident base and current tree borrow; implementations retain neither.
pub trait ScrollbarLayerImplVirtual<B: LayerImplTypes> {
    // Source: cc/layers/scrollbar_layer_impl_base.h:68,75,77,112-115.
    fn ThumbThickness(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32;
    fn TrackLength(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> f32;
    fn MinimumThumbLength(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32;
    fn TrackStart(&self, base: &ScrollbarLayerImplBase<B>, c: &dyn ScrollbarGeometryRead<B>)
    -> i32;
    fn IsThumbResizable(&self) -> bool;
    // Source: cc/layers/scrollbar_layer_impl_base.h:73.
    fn GetScrollbarAnimator(&self, c: &dyn ScrollbarGeometryStyle) -> ScrollbarAnimator {
        c.ScrollbarAnimator()
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:59-61.
    fn ComputeThumbQuadRect(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> Rect {
        base.ComputeThumbQuadRect(self, c)
    }
    fn ComputeHitTestableThumbQuadRect(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> Rect {
        base.ComputeHitTestableThumbQuadRect(self, c)
    }
    fn ComputeHitTestableExpandedThumbQuadRect(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> Rect {
        base.ComputeHitTestableExpandedThumbQuadRect(self, c)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:78-85,98.
    fn BackButtonRect(&self, base: &ScrollbarLayerImplBase<B>) -> Rect {
        base.BackButtonRect()
    }
    fn ForwardButtonRect(&self, base: &ScrollbarLayerImplBase<B>) -> Rect {
        base.ForwardButtonRect()
    }
    fn BackTrackRect(&self, base: &ScrollbarLayerImplBase<B>) -> Rect {
        base.BackTrackRect()
    }
    fn ForwardTrackRect(&self, base: &ScrollbarLayerImplBase<B>) -> Rect {
        base.ForwardTrackRect()
    }
    fn SupportsDragSnapBack(&self, base: &ScrollbarLayerImplBase<B>) -> bool {
        base.SupportsDragSnapBack()
    }
    fn JumpOnTrackClick(&self, base: &ScrollbarLayerImplBase<B>) -> bool {
        base.JumpOnTrackClick()
    }
    fn OverlayScrollbarOpacity(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> f32 {
        base.OverlayScrollbarOpacity(c)
    }
    fn IdentifyScrollbarPart(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        position: PointF,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> ScrollbarPart {
        base.IdentifyScrollbarPart(self, position, c)
    }
}
impl<B: LayerImplTypes> ScrollbarLayerImplBase<B> {
    // Source: cc/layers/scrollbar_layer_impl_base.h:105-109; cc/layers/scrollbar_layer_impl_base.cc:18-33.
    pub fn new(
        tree: LayerTreeId,
        id: i32,
        orientation: ScrollbarOrientation,
        left: bool,
        overlay: bool,
        c: &mut LayerImplContext<'_, B>,
    ) -> Self {
        Self {
            base_: LayerImpl::new(tree, id, c),
            scroll_element_id_: Default::default(),
            is_overlay_scrollbar_: overlay,
            is_web_test_: false,
            opacity_changed_for_fade_out_animation_: false,
            property_changed_for_other_reasons_: false,
            thumb_thickness_scale_factor_: 1.,
            current_pos_: 0.,
            clip_layer_length_: 0.,
            scroll_layer_length_: 0.,
            orientation_: orientation,
            is_left_side_vertical_scrollbar_: left,
            vertical_adjust_: 0.,
            has_find_in_page_tickmarks_: false,
            shutdown_: false,
        }
    }
    pub fn base(&self) -> &LayerImpl<B> {
        &self.base_
    }
    pub fn base_mut(&mut self) -> &mut LayerImpl<B> {
        &mut self.base_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:110; cc/layers/scrollbar_layer_impl_base.cc:35-37.
    // Explicit before owned removal/drop: the source destructor requires the
    // actual live tree. It cannot be deferred until after that tree is gone.
    pub fn Shutdown(&mut self, c: &mut ScrollbarLayerImplContext<'_, B>) {
        assert!(
            !self.shutdown_,
            "native scrollbar destructor executed twice"
        );
        c.tree.UnregisterScrollbar(self);
        self.shutdown_ = true;
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:54; cc/layers/scrollbar_layer_impl_base.cc:39-47.
    pub fn CopyPropertiesTo(
        &self,
        target: &mut Self,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut ScrollbarLayerImplContext<'_, B>,
    ) {
        self.base_
            .CopyPropertiesTo(&mut target.base_, source, &mut c.Base());
        debug_assert!(target.IsScrollbarLayer());
        target.SetHasFindInPageTickmarks(self.has_find_in_page_tickmarks_, &mut c.Base());
        target.set_is_overlay_scrollbar(self.is_overlay_scrollbar_);
        target.set_is_web_test(self.is_web_test_);
        target.SetScrollElementId(self.scroll_element_id(), c);
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:55; cc/layers/scrollbar_layer_impl_base.cc:49-58.
    pub fn GetDamageReasons(&self) -> DamageReasonSet {
        let mut reasons = DamageReasonSet::default();
        if self.opacity_changed_for_fade_out_animation_ {
            reasons.Put(DamageReason::kScrollbarFadeOutAnimation);
        }
        if self.property_changed_for_other_reasons_ || !self.base_.update_rect().IsEmpty() {
            reasons.Put(DamageReason::kCompositorScroll);
        }
        reasons
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:56; cc/layers/scrollbar_layer_impl_base.cc:60-64.
    pub fn ResetChangeTracking(&mut self) {
        self.base_.ResetChangeTracking();
        self.opacity_changed_for_fade_out_animation_ = false;
        self.property_changed_for_other_reasons_ = false;
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:118; cc/layers/scrollbar_layer_impl_base.cc:66-68.
    pub fn IsScrollbarLayer(&self) -> bool {
        true
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:25.
    pub fn scroll_element_id(&self) -> ElementId {
        self.scroll_element_id_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:26; cc/layers/scrollbar_layer_impl_base.cc:70-77.
    pub fn SetScrollElementId(
        &mut self,
        value: ElementId,
        c: &mut ScrollbarLayerImplContext<'_, B>,
    ) {
        if self.scroll_element_id_ == value {
            return;
        }
        c.tree.UnregisterScrollbar(self);
        self.scroll_element_id_ = value;
        c.tree.RegisterScrollbar(self);
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:30; cc/layers/scrollbar_layer_impl_base.cc:79-86.
    pub fn SetCurrentPos(&mut self, value: f32, c: &mut LayerImplContext<'_, B>) -> bool {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetCurrentPos(base, value, c)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:52; cc/layers/scrollbar_layer_impl_base.cc:88-113.
    pub fn CanScrollOrientation(&self, c: &dyn ScrollbarGeometryRead<B>) -> bool {
        let node = c
            .PropertyTrees()
            .scroll_tree()
            .FindNodeFromElementId(self.scroll_element_id_);
        debug_assert!(node.is_some());
        let Some(node) = node else {
            return false;
        };
        if self.orientation() == ScrollbarOrientation::kHorizontal {
            if !node.user_scrollable_horizontal {
                return false;
            }
        } else if !node.user_scrollable_vertical {
            return false;
        }
        !MathUtil::IsFloatNearlyTheSame(self.clip_layer_length(), self.scroll_layer_length())
            && self.clip_layer_length() < self.scroll_layer_length()
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:33; cc/layers/scrollbar_layer_impl_base.cc:115-121.
    pub fn SetVerticalAdjust(&mut self, value: f32, c: &mut LayerImplContext<'_, B>) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetVerticalAdjust(base, value, c)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:31; cc/layers/scrollbar_layer_impl_base.cc:123-132.
    pub fn SetClipLayerLength(
        &mut self,
        delegate: &(impl ScrollbarLayerImplVirtual<B> + ?Sized),
        value: f32,
        c: &mut ScrollbarLayerImplContext<'_, B>,
    ) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetClipLayerLength(base, value, c, |c| delegate.GetScrollbarAnimator(c));
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:32; cc/layers/scrollbar_layer_impl_base.cc:134-145.
    pub fn SetScrollLayerLength(
        &mut self,
        delegate: &(impl ScrollbarLayerImplVirtual<B> + ?Sized),
        value: f32,
        c: &mut ScrollbarLayerImplContext<'_, B>,
    ) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetScrollLayerLength(base, value, c, |c| delegate.GetScrollbarAnimator(c));
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:66; cc/layers/scrollbar_layer_impl_base.cc:147-153.
    pub fn SetThumbThicknessScaleFactor(&mut self, value: f32, c: &mut LayerImplContext<'_, B>) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetThumbThicknessScaleFactor(base, value, c);
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:120-121; cc/layers/scrollbar_layer_impl_base.cc:155-257.
    // Thumb length/position preserves root-controls vertical adjustment; native
    // explanatory diagram (157-218) changes no data/control flow.
    fn ComputeThumbQuadRectWithThumbThicknessScale(
        &self,
        delegate: &(impl ScrollbarLayerImplVirtual<B> + ?Sized),
        scale: f32,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> Rect {
        let track_length = delegate.TrackLength(self, c);
        let thumb_length = self.ThumbLength(delegate, c);
        let thumb_thickness = delegate.ThumbThickness(self, c);
        let extent = self.scroll_layer_length() - self.clip_layer_length();
        let maximum = if extent < 0. { 0. } else { extent };
        let clamped_current_pos = self.current_pos().clamp(0., maximum);
        let mut thumb_offset = delegate.TrackStart(self, c);
        if maximum > 0. {
            let ratio = clamped_current_pos / maximum;
            let max_offset = track_length - thumb_length as f32;
            thumb_offset += (ratio * max_offset) as i32;
        }
        let adjustment = thumb_thickness as f32 * (1. - scale);
        let rect = if self.orientation_ == ScrollbarOrientation::kHorizontal {
            RectF::new(
                thumb_offset as f32,
                self.vertical_adjust_ + adjustment,
                thumb_length as f32,
                thumb_thickness as f32 - adjustment,
            )
        } else {
            RectF::new(
                if self.is_left_side_vertical_scrollbar_ {
                    (self
                        .base_
                        .bounds_for_property_trees(c.PropertyTrees())
                        .width()
                        - thumb_thickness) as f32
                } else {
                    adjustment
                },
                thumb_offset as f32,
                thumb_thickness as f32 - adjustment,
                thumb_length as f32,
            )
        };
        ToEnclosingRect(rect)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:61; cc/layers/scrollbar_layer_impl_base.cc:259-263.
    pub fn ComputeHitTestableExpandedThumbQuadRect(
        &self,
        delegate: &(impl ScrollbarLayerImplVirtual<B> + ?Sized),
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> Rect {
        debug_assert!(self.is_overlay_scrollbar());
        self.ComputeThumbQuadRectWithThumbThicknessScale(delegate, 1., c)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:59; cc/layers/scrollbar_layer_impl_base.cc:265-268.
    pub fn ComputeThumbQuadRect(
        &self,
        delegate: &(impl ScrollbarLayerImplVirtual<B> + ?Sized),
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> Rect {
        self.ComputeThumbQuadRectWithThumbThicknessScale(
            delegate,
            self.thumb_thickness_scale_factor_,
            c,
        )
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:60; cc/layers/scrollbar_layer_impl_base.cc:270-272.
    pub fn ComputeHitTestableThumbQuadRect(
        &self,
        delegate: &(impl ScrollbarLayerImplVirtual<B> + ?Sized),
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> Rect {
        delegate.ComputeThumbQuadRect(self, c)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:70-71; cc/layers/scrollbar_layer_impl_base.cc:274-306.
    pub fn SetOverlayScrollbarLayerOpacityAnimated(
        &mut self,
        opacity: f32,
        fade_out: bool,
        c: &mut ScrollbarLayerImplContext<'_, B>,
    ) {
        self.SetOverlayScrollbarLayerOpacityAnimatedWithContext(opacity, fade_out, c)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.cc:274-306. Same source
    // body with only live effect-tree and draw-invalidation fields lent.
    pub fn SetOverlayScrollbarLayerOpacityAnimatedWithContext(
        &mut self,
        opacity: f32,
        fade_out: bool,
        c: &mut (impl ScrollbarOpacityContext<B> + ?Sized),
    ) {
        debug_assert!(self.is_overlay_scrollbar());
        // Source null tree branch is excluded by the live nonnull context: a
        // native operation after Shutdown cannot acquire this owner borrow.
        if c.trees_in_viz_in_viz_process() && !c.TreeAnimationsInVizInVizProcess() {
            return;
        }
        let effect_id = self.base_.effect_tree_index();
        if effect_id == kInvalidPropertyNodeId {
            return;
        }
        let property_trees = c.property_trees_mut();
        let node = property_trees.effect_tree_mutable().MutableNode(effect_id);
        if node.opacity == opacity {
            return;
        }
        node.opacity = opacity;
        node.effect_changed = true;
        property_trees.set_changed(true);
        property_trees.effect_tree_mutable().set_needs_update(true);
        c.set_needs_update_draw_properties();
        self.opacity_changed_for_fade_out_animation_ |= fade_out;
        self.property_changed_for_other_reasons_ |= !fade_out;
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:73; cc/layers/scrollbar_layer_impl_base.cc:308-311.
    pub fn GetScrollbarAnimator(&self, c: &dyn ScrollbarGeometryRead<B>) -> ScrollbarAnimator {
        self.GetScrollbarAnimatorWithStyle(c)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:102; cc/layers/scrollbar_layer_impl_base.cc:313-315.
    pub fn GetIdleThicknessScale(&self, c: &dyn ScrollbarGeometryRead<B>) -> f32 {
        c.Settings().idle_thickness_scale
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:92; cc/layers/scrollbar_layer_impl_base.cc:317-325.
    pub fn SetHasFindInPageTickmarks(&mut self, value: bool, c: &mut LayerImplContext<'_, B>) {
        if self.has_find_in_page_tickmarks_ == value {
            return;
        }
        self.has_find_in_page_tickmarks_ = value;
        self.property_changed_for_other_reasons_ = true;
        self.base_.NoteLayerPropertyChanged(c);
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:98; cc/layers/scrollbar_layer_impl_base.cc:327-329.
    pub fn OverlayScrollbarOpacity(&self, c: &dyn ScrollbarGeometryRead<B>) -> f32 {
        self.base_.OpacityForPropertyTrees(c.PropertyTrees())
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:82; cc/layers/scrollbar_layer_impl_base.cc:331-333.
    pub fn SupportsDragSnapBack(&self) -> bool {
        false
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:83; cc/layers/scrollbar_layer_impl_base.cc:335-337.
    pub fn JumpOnTrackClick(&self) -> bool {
        false
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:100; cc/layers/scrollbar_layer_impl_base.cc:339-341.
    pub fn IsFluentScrollbarEnabled(&self, c: &dyn ScrollbarGeometryRead<B>) -> bool {
        c.Settings().enable_fluent_scrollbar
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:101; cc/layers/scrollbar_layer_impl_base.cc:343-345.
    pub fn IsFluentOverlayScrollbarEnabled(&self, c: &dyn ScrollbarGeometryRead<B>) -> bool {
        self.IsFluentOverlayScrollbarEnabledWithStyle(c)
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:76; cc/layers/scrollbar_layer_impl_base.cc:347-351.
    pub fn ThumbLength(
        &self,
        delegate: &(impl ScrollbarLayerImplVirtual<B> + ?Sized),
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32 {
        ScrollUtils::CalculateScrollbarThumbLength(
            self.scroll_layer_length() as i32,
            self.clip_layer_length() as i32,
            delegate.TrackLength(self, c) as i32,
            delegate.MinimumThumbLength(self, c),
        )
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:78; cc/layers/scrollbar_layer_impl_base.cc:353-355.
    pub fn BackButtonRect(&self) -> Rect {
        Rect::default()
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:79; cc/layers/scrollbar_layer_impl_base.cc:357-359.
    pub fn ForwardButtonRect(&self) -> Rect {
        Rect::default()
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:80; cc/layers/scrollbar_layer_impl_base.cc:361-363.
    pub fn BackTrackRect(&self) -> Rect {
        Rect::default()
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:81; cc/layers/scrollbar_layer_impl_base.cc:365-367.
    pub fn ForwardTrackRect(&self) -> Rect {
        Rect::default()
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:84-85; cc/layers/scrollbar_layer_impl_base.cc:369-394.
    pub fn IdentifyScrollbarPart(
        &self,
        delegate: &(impl ScrollbarLayerImplVirtual<B> + ?Sized),
        position: PointF,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> ScrollbarPart {
        let point = Point::new(position.x() as i32, position.y() as i32);
        if delegate.BackButtonRect(self).ContainsPoint(point) {
            return ScrollbarPart::kBackButton;
        }
        if delegate.ForwardButtonRect(self).ContainsPoint(point) {
            return ScrollbarPart::kForwardButton;
        }
        if delegate
            .ComputeHitTestableThumbQuadRect(self, c)
            .ContainsPoint(point)
        {
            return ScrollbarPart::kThumb;
        }
        if delegate.BackTrackRect(self).ContainsPoint(point) {
            return ScrollbarPart::kBackTrack;
        }
        if delegate.ForwardTrackRect(self).ContainsPoint(point) {
            return ScrollbarPart::kForwardTrack;
        }
        ScrollbarPart::kNoPart
    }
}
impl<B: LayerImplTypes> Drop for ScrollbarLayerImplBase<B> {
    fn drop(&mut self) {
        assert!(
            self.shutdown_,
            "owned scrollbar removal must synchronously call native Shutdown with the live tree before Drop"
        );
    }
}

impl<B: LayerImplTypes> ScrollbarLayerImplBase<B> {
    // Source: cc/layers/scrollbar_layer_impl_base.h:35-35.
    pub fn current_pos(&self) -> f32 {
        self.current_pos_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:36-36.
    pub fn clip_layer_length(&self) -> f32 {
        self.clip_layer_length_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:37-37.
    pub fn scroll_layer_length(&self) -> f32 {
        self.scroll_layer_length_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:38-38.
    pub fn vertical_adjust(&self) -> f32 {
        self.vertical_adjust_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:40-40.
    pub fn is_overlay_scrollbar(&self) -> bool {
        self.is_overlay_scrollbar_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:44-44.
    pub fn is_web_test(&self) -> bool {
        self.is_web_test_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:47-47.
    pub fn orientation(&self) -> ScrollbarOrientation {
        self.orientation_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:48-50.
    pub fn is_left_side_vertical_scrollbar(&self) -> bool {
        self.is_left_side_vertical_scrollbar_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:63-65.
    pub fn thumb_thickness_scale_factor(&self) -> f32 {
        self.thumb_thickness_scale_factor_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:89-91.
    pub fn has_find_in_page_tickmarks(&self) -> bool {
        self.has_find_in_page_tickmarks_
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:41-43.
    pub fn set_is_overlay_scrollbar(&mut self, value: bool) {
        self.is_overlay_scrollbar_ = value;
    }
    // Source: cc/layers/scrollbar_layer_impl_base.h:45.
    pub fn set_is_web_test(&mut self, value: bool) {
        self.is_web_test_ = value;
    }
}

/// Native checked base-pointer casts, resolved only on the actual resident
/// most-derived owned layer. Implementations perform field projection only.
pub trait ScrollbarLayerImplOwner<B: LayerImplTypes> {
    fn AsScrollbarBase(&self) -> Option<&ScrollbarLayerImplBase<B>>;
    fn AsScrollbarBaseMutable(&mut self) -> Option<&mut ScrollbarLayerImplBase<B>>;
}
// Source: cc/layers/scrollbar_layer_impl_base.h:148-151.
pub fn ToScrollbarLayerMutable<B: LayerImplTypes, L: ScrollbarLayerImplOwner<B> + ?Sized>(
    layer: &mut L,
) -> &mut ScrollbarLayerImplBase<B> {
    let result = layer.AsScrollbarBaseMutable();
    debug_assert!(result.is_some());
    result.expect("native scrollbar static_cast precondition")
}
// Source: cc/layers/scrollbar_layer_impl_base.h:153-156.
pub fn ToScrollbarLayer<B: LayerImplTypes, L: ScrollbarLayerImplOwner<B> + ?Sized>(
    layer: &L,
) -> &ScrollbarLayerImplBase<B> {
    let result = layer.AsScrollbarBase();
    debug_assert!(result.is_some());
    result.expect("native scrollbar static_cast precondition")
}
// Source: cc/layers/scrollbar_layer_impl_base.h:158.
pub use crate::adapter::layer_tree_impl::ScrollbarSet;

impl<B: LayerImplTypes> ScrollbarLayerImplOwner<B> for ScrollbarLayerImplBase<B> {
    fn AsScrollbarBase(&self) -> Option<&Self> {
        Some(self)
    }
    fn AsScrollbarBaseMutable(&mut self) -> Option<&mut Self> {
        Some(self)
    }
}
impl<B: LayerImplTypes> crate::adapter::layer_impl::LayerImplOwner<B>
    for ScrollbarLayerImplBase<B>
{
    fn impl_base(&self) -> &LayerImpl<B> {
        self.base()
    }
    fn impl_base_mut(&mut self) -> &mut LayerImpl<B> {
        self.base_mut()
    }
}

/// Borrow-only source geometry fields, disjoint from the same allocation's base.
/// The selected layer can lend its base into ScrollTree's synchronous callback
/// while its actual derived geometry fields remain available to that callback.
pub struct ScrollbarGeometryFields<'a> {
    overlay: &'a bool,
    orientation: &'a ScrollbarOrientation,
    scroll_element: &'a ElementId,
    current_pos: &'a mut f32,
    vertical_adjust: &'a mut f32,
    clip_length: &'a mut f32,
    scroll_length: &'a mut f32,
    property_changed: &'a mut bool,
    thumb_thickness_scale: &'a mut f32,
}
impl<B: LayerImplTypes> ScrollbarLayerImplBase<B> {
    pub fn SplitGeometryFields(&mut self) -> (&mut LayerImpl<B>, ScrollbarGeometryFields<'_>) {
        (
            &mut self.base_,
            ScrollbarGeometryFields {
                overlay: &self.is_overlay_scrollbar_,
                orientation: &self.orientation_,
                scroll_element: &self.scroll_element_id_,
                current_pos: &mut self.current_pos_,
                vertical_adjust: &mut self.vertical_adjust_,
                clip_length: &mut self.clip_layer_length_,
                scroll_length: &mut self.scroll_layer_length_,
                property_changed: &mut self.property_changed_for_other_reasons_,
                thumb_thickness_scale: &mut self.thumb_thickness_scale_factor_,
            },
        )
    }
}
impl ScrollbarGeometryFields<'_> {
    pub fn thumb_thickness_scale_factor(&self) -> f32 {
        *self.thumb_thickness_scale
    }
    // Source: cc/layers/scrollbar_layer_impl_base.cc:147-153.
    pub fn SetThumbThicknessScaleFactor<
        B: LayerImplTypes,
        C: crate::adapter::layer_impl::LayerImplPushContext<Types = B> + ?Sized,
    >(
        &mut self,
        base: &mut LayerImpl<B>,
        value: f32,
        c: &mut C,
    ) {
        if *self.thumb_thickness_scale == value {
            return;
        }
        *self.thumb_thickness_scale = value;
        *self.property_changed = true;
        base.NoteLayerPropertyChangedWithPushContext(c);
    }
    pub fn orientation(&self) -> ScrollbarOrientation {
        *self.orientation
    }
    // Source: cc/layers/scrollbar_layer_impl_base.cc:78-86.
    pub fn SetCurrentPos<
        B: LayerImplTypes,
        C: crate::adapter::layer_impl::LayerImplPushContext<Types = B> + ?Sized,
    >(
        &mut self,
        base: &mut LayerImpl<B>,
        value: f32,
        c: &mut C,
    ) -> bool {
        if *self.current_pos == value {
            return false;
        }
        *self.current_pos = value;
        *self.property_changed = true;
        base.NoteLayerPropertyChangedWithPushContext(c);
        true
    }
    // Source: cc/layers/scrollbar_layer_impl_base.cc:115-121.
    pub fn SetVerticalAdjust<
        B: LayerImplTypes,
        C: crate::adapter::layer_impl::LayerImplPushContext<Types = B> + ?Sized,
    >(
        &mut self,
        base: &mut LayerImpl<B>,
        value: f32,
        c: &mut C,
    ) {
        if *self.vertical_adjust == value {
            return;
        }
        *self.vertical_adjust = value;
        *self.property_changed = true;
        base.NoteLayerPropertyChangedWithPushContext(c);
    }
    // Source: cc/layers/scrollbar_layer_impl_base.cc:123-132.
    pub fn SetClipLayerLength<B: LayerImplTypes, C: ScrollbarGeometryContext<B> + ?Sized>(
        &mut self,
        base: &mut LayerImpl<B>,
        value: f32,
        c: &mut C,
        animator: impl FnOnce(&C) -> ScrollbarAnimator,
    ) {
        if *self.clip_length == value {
            return;
        }
        *self.clip_length = value;
        *self.property_changed = true;
        base.NoteLayerPropertyChangedWithPushContext(c);
        if animator(c) == ScrollbarAnimator::AURA_OVERLAY {
            c.RequestShowScrollbars(*self.scroll_element);
        }
    }
    // Source: cc/layers/scrollbar_layer_impl_base.cc:134-145.
    pub fn SetScrollLayerLength<B: LayerImplTypes, C: ScrollbarGeometryContext<B> + ?Sized>(
        &mut self,
        base: &mut LayerImpl<B>,
        value: f32,
        c: &mut C,
        animator: impl FnOnce(&C) -> ScrollbarAnimator,
    ) {
        if *self.scroll_length == value {
            return;
        }
        *self.scroll_length = value;
        *self.property_changed = true;
        base.NoteLayerPropertyChangedWithPushContext(c);
        if animator(c) == ScrollbarAnimator::AURA_OVERLAY && !c.IsFluentOverlayScrollbarEnabled() {
            c.RequestShowScrollbars(*self.scroll_element);
        }
    }
}

impl<B: LayerImplTypes> ScrollbarLayerImplBase<B> {
    // Source: cc/layers/scrollbar_layer_impl_base.cc:308-311. Same native base
    // query, with only its actual readonly style capability lent.
    pub fn GetScrollbarAnimatorWithStyle(
        &self,
        c: &dyn ScrollbarGeometryStyle,
    ) -> ScrollbarAnimator {
        c.ScrollbarAnimator()
    }
    // Source: cc/layers/scrollbar_layer_impl_base.cc:343-345.
    pub fn IsFluentOverlayScrollbarEnabledWithStyle(&self, c: &dyn ScrollbarGeometryStyle) -> bool {
        c.IsFluentOverlayScrollbarEnabled()
    }
}

/// Exactly the selected native scrollbar split into its two resident parts.
pub struct ScrollbarGeometryReceiver<'a, 'g, B: LayerImplTypes> {
    pub base: &'a mut LayerImpl<B>,
    pub geometry: &'a mut ScrollbarGeometryFields<'g>,
}
impl<B: LayerImplTypes> crate::adapter::layer_tree_impl::LayerTreeScrollbarGeometry<B>
    for ScrollbarGeometryReceiver<'_, '_, B>
{
    fn is_overlay_scrollbar(&self) -> bool {
        *self.geometry.overlay
    }
    fn orientation(&self) -> ScrollbarOrientation {
        self.geometry.orientation()
    }
    fn thumb_thickness_scale_factor(&self) -> f32 {
        self.geometry.thumb_thickness_scale_factor()
    }
    fn SetThumbThicknessScaleFactor(
        &mut self,
        value: f32,
        c: &mut dyn ScrollbarGeometryContext<B>,
    ) {
        self.geometry
            .SetThumbThicknessScaleFactor(self.base, value, c);
    }

    fn SetCurrentPos(&mut self, value: f32, c: &mut dyn ScrollbarGeometryContext<B>) {
        self.geometry.SetCurrentPos(self.base, value, c);
    }
    fn SetClipLayerLength(&mut self, value: f32, c: &mut dyn ScrollbarGeometryContext<B>) {
        self.geometry
            .SetClipLayerLength(self.base, value, c, |c| c.ScrollbarAnimator());
    }
    fn SetScrollLayerLength(&mut self, value: f32, c: &mut dyn ScrollbarGeometryContext<B>) {
        self.geometry
            .SetScrollLayerLength(self.base, value, c, |c| c.ScrollbarAnimator());
    }
    fn SetVerticalAdjust(&mut self, value: f32, c: &mut dyn ScrollbarGeometryContext<B>) {
        self.geometry.SetVerticalAdjust(self.base, value, c);
    }
}
