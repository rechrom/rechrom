// Copyright 2014 The Chromium Authors
// Source: cc/layers/painted_scrollbar_layer_impl.{h,cc}.
use super::{
    append_quads_context::AppendQuadsContext,
    append_quads_data::AppendQuadsData,
    layer_impl::LayerImpl,
    scrollbar_layer_impl_base::{ScrollbarLayerImplBase, ScrollbarLayerImplVirtual},
};
use crate::adapter::{
    geometry::{Rect, RectF, Size},
    layer_impl::{LayerImplContext, LayerType},
    layer_tree_impl::{Color4f, LayerTreeId, UIResourceId},
    painted_scrollbar_layer_impl::PaintedScrollbarLayerImplTypes,
    scrollbar_layer_impl_base::{ScrollbarGeometryRead, ScrollbarGeometryStyle},
    solid_color_layer_impl::SolidColorRenderPass,
};
use crate::cc::{
    input::scrollbar::ScrollbarOrientation, trees::layer_tree_settings::ScrollbarAnimator,
};

pub struct PaintedScrollbarLayerImpl<B: PaintedScrollbarLayerImplTypes> {
    base_: ScrollbarLayerImplBase<B>,
    internal_contents_scale_: f32,
    internal_content_bounds_: Size,
    thumb_ui_resource_id_: UIResourceId,
    painted_opacity_: f32,
    track_rect_: Rect,
    minimum_thumb_length_: i32,
    thumb_thickness_: i32,
}

impl<B: PaintedScrollbarLayerImplTypes> PaintedScrollbarLayerImpl<B> {
    pub fn Create(
        tree: LayerTreeId,
        id: i32,
        orientation: ScrollbarOrientation,
        left: bool,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Box::new(B::WrapPaintedScrollbarLayerImpl(Self::new(
            tree,
            id,
            orientation,
            left,
            c,
        )))
    }
    pub fn new(
        tree: LayerTreeId,
        id: i32,
        orientation: ScrollbarOrientation,
        left: bool,
        c: &mut LayerImplContext<'_, B>,
    ) -> Self {
        Self {
            base_: ScrollbarLayerImplBase::new(tree, id, orientation, left, true, c),
            internal_contents_scale_: 1.0,
            internal_content_bounds_: Size::default(),
            thumb_ui_resource_id_: UIResourceId::default(),
            painted_opacity_: 0.0,
            track_rect_: Rect::default(),
            minimum_thumb_length_: 0,
            thumb_thickness_: 0,
        }
    }
    pub fn base(&self) -> &LayerImpl<B> {
        self.base_.base()
    }
    pub fn base_mut(&mut self) -> &mut LayerImpl<B> {
        self.base_.base_mut()
    }
    pub fn scrollbar_base(&self) -> &ScrollbarLayerImplBase<B> {
        &self.base_
    }
    pub fn scrollbar_base_mut(&mut self) -> &mut ScrollbarLayerImplBase<B> {
        &mut self.base_
    }
    pub fn GetLayerType(&self) -> LayerType {
        LayerType::kPaintedScrollbar
    }
    pub fn SetInternalContentsScaleAndBounds(&mut self, scale: f32, bounds: Size) {
        self.internal_contents_scale_ = scale;
        self.internal_content_bounds_ = bounds;
    }
    pub fn SetThumbUIResourceId(&mut self, id: UIResourceId) {
        self.thumb_ui_resource_id_ = id;
    }
    pub fn SetScrollbarPaintedOpacity(&mut self, opacity: f32) {
        self.painted_opacity_ = opacity;
    }
    pub fn SetTrackRect(&mut self, rect: Rect) {
        self.track_rect_ = rect;
    }
    pub fn SetMinimumThumbLength(&mut self, value: i32) {
        self.minimum_thumb_length_ = value;
    }
    pub fn SetThumbThickness(&mut self, value: i32) {
        self.thumb_thickness_ = value;
    }
    pub fn CreateLayerImpl(
        &self,
        tree: LayerTreeId,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Self::Create(
            tree,
            self.base().id(),
            self.base_.orientation(),
            self.base_.is_left_side_vertical_scrollbar(),
            c,
        )
    }
    pub fn CopyPropertiesTo(
        &self,
        target: &mut Self,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut crate::adapter::scrollbar_layer_impl_base::ScrollbarLayerImplContext<'_, B>,
    ) {
        self.base_.CopyPropertiesTo(&mut target.base_, source, c);
        target.internal_contents_scale_ = self.internal_contents_scale_;
        target.internal_content_bounds_ = self.internal_content_bounds_;
        target.thumb_ui_resource_id_ = self.thumb_ui_resource_id_;
        target.painted_opacity_ = self.painted_opacity_;
        target.track_rect_ = self.track_rect_;
        target.minimum_thumb_length_ = self.minimum_thumb_length_;
        target.thumb_thickness_ = self.thumb_thickness_;
    }
    pub fn AppendQuads(
        &mut self,
        _append: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        let thumb = self.base_.ComputeThumbQuadRect(self, c);
        let visible = self
            .base()
            .draw_properties()
            .occlusion_in_content_space
            .GetUnoccludedContentRect(thumb);
        if visible.IsEmpty() {
            return;
        }
        let resource = c.tree.ResourceIdForUIResource(self.thumb_ui_resource_id_);
        if resource == Default::default() {
            return;
        }
        let scaled =
            crate::adapter::geometry::ScaleToEnclosingRect(thumb, self.internal_contents_scale_);
        let scaled_visible =
            crate::adapter::geometry::ScaleToEnclosingRect(visible, self.internal_contents_scale_);
        let state = pass.CreateAndAppendSharedQuadState();
        self.base().PopulateScaledSharedQuadState(
            pass.SharedQuadStateMutable(state),
            self.internal_contents_scale_,
            false,
            c,
        );
        use crate::adapter::layer_impl::LayerImplSharedQuadState as _;
        pass.SharedQuadStateMutable(state)
            .MultiplyOpacity(self.painted_opacity_);
        let resource_size = c.tree.GetUIResourceSize(self.thumb_ui_resource_id_);
        pass.AppendScrollbarTextureQuad(
            state,
            scaled,
            scaled_visible,
            resource,
            RectF::new(
                0.0,
                0.0,
                resource_size.width() as f32,
                resource_size.height() as f32,
            ),
            Color4f::default(),
        );
        let _ = data;
    }
}

impl<B: PaintedScrollbarLayerImplTypes> ScrollbarLayerImplVirtual<B>
    for PaintedScrollbarLayerImpl<B>
{
    fn ThumbThickness(
        &self,
        _base: &ScrollbarLayerImplBase<B>,
        _c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32 {
        self.thumb_thickness_
    }
    fn TrackLength(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        _c: &dyn ScrollbarGeometryRead<B>,
    ) -> f32 {
        if base.orientation() == ScrollbarOrientation::kVertical {
            self.track_rect_.height as f32 + base.vertical_adjust()
        } else {
            self.track_rect_.width as f32
        }
    }
    fn MinimumThumbLength(
        &self,
        _base: &ScrollbarLayerImplBase<B>,
        _c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32 {
        self.minimum_thumb_length_
    }
    fn TrackStart(
        &self,
        _base: &ScrollbarLayerImplBase<B>,
        _c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32 {
        if self.base_.orientation() == ScrollbarOrientation::kVertical {
            self.track_rect_.y
        } else {
            self.track_rect_.x
        }
    }
    fn IsThumbResizable(&self) -> bool {
        false
    }
    fn GetScrollbarAnimator(&self, _c: &dyn ScrollbarGeometryStyle) -> ScrollbarAnimator {
        ScrollbarAnimator::NO_ANIMATOR
    }
}
