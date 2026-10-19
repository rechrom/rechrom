use crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryRead;
// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/solid_color_scrollbar_layer_impl.h:1-14,64-66;
//         cc/layers/solid_color_scrollbar_layer_impl.cc:1-15,114.
use super::{
    append_quads_context::AppendQuadsContext,
    append_quads_data::AppendQuadsData,
    layer_impl::LayerImpl,
    scrollbar_layer_impl_base::{ScrollbarLayerImplBase, ScrollbarLayerImplVirtual},
};
use crate::adapter::{
    geometry::Rect,
    layer_impl::{LayerImplContext, LayerType},
    layer_tree_impl::{Color4f, LayerTreeId},
    scrollbar_layer_impl_base::ScrollbarLayerImplContext,
    solid_color_layer_impl::SolidColorRenderPass,
    solid_color_scrollbar_layer_impl::*,
};
use crate::cc::input::scrollbar::ScrollbarOrientation;
// Source: cc/layers/solid_color_scrollbar_layer_impl.h:15-62. Native composed resident owner.
pub struct SolidColorScrollbarLayerImpl<B: SolidColorScrollbarLayerImplTypes> {
    base_: ScrollbarLayerImplBase<B>,
    thumb_thickness_: i32,
    track_start_: i32,
    color_: Color4f,
}
impl<B: SolidColorScrollbarLayerImplTypes> SolidColorScrollbarLayerImpl<B> {
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:17-23; cc/layers/solid_color_scrollbar_layer_impl.cc:17-27.
    pub fn Create(
        tree: LayerTreeId,
        id: i32,
        orientation: ScrollbarOrientation,
        thumb_thickness: i32,
        track_start: i32,
        left: bool,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Box::new(B::WrapSolidColorScrollbarLayerImpl(Self::new(
            tree,
            id,
            orientation,
            thumb_thickness,
            track_start,
            left,
            c,
        )))
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:45-50; cc/layers/solid_color_scrollbar_layer_impl.cc:42-55.
    pub fn new(
        tree: LayerTreeId,
        id: i32,
        orientation: ScrollbarOrientation,
        thumb_thickness: i32,
        track_start: i32,
        left: bool,
        c: &mut LayerImplContext<'_, B>,
    ) -> Self {
        Self {
            base_: ScrollbarLayerImplBase::new(tree, id, orientation, left, true, c),
            thumb_thickness_: thumb_thickness,
            track_start_: track_start,
            color_: Color4f::default(),
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
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:24; cc/layers/solid_color_scrollbar_layer_impl.cc:29.
    pub fn Shutdown(&mut self, c: &mut ScrollbarLayerImplContext<'_, B>) {
        self.base_.Shutdown(c);
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:27; cc/layers/solid_color_scrollbar_layer_impl.cc:31-33.
    pub fn GetLayerType(&self) -> LayerType {
        LayerType::kSolidColorScrollbar
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:28-29; cc/layers/solid_color_scrollbar_layer_impl.cc:35-40.
    pub fn CreateLayerImpl(
        &self,
        tree: LayerTreeId,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Self::Create(
            tree,
            self.base().id(),
            self.base_.orientation(),
            self.thumb_thickness_,
            self.track_start_,
            self.base_.is_left_side_vertical_scrollbar(),
            c,
        )
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:30; cc/layers/solid_color_scrollbar_layer_impl.cc:57-60.
    pub fn CopyPropertiesTo(
        &self,
        target: &mut Self,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut ScrollbarLayerImplContext<'_, B>,
    ) {
        self.base_.CopyPropertiesTo(&mut target.base_, source, c);
        target.set_color(self.color_);
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:36; cc/layers/solid_color_scrollbar_layer_impl.cc:62-71.
    pub fn ThumbThickness(&self, c: &dyn ScrollbarGeometryRead<B>) -> i32 {
        self.Geometry().ThumbThickness(&self.base_, c)
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:53; cc/layers/solid_color_scrollbar_layer_impl.cc:73-75.
    pub fn MinimumThumbLength(&self, c: &dyn ScrollbarGeometryRead<B>) -> i32 {
        self.Geometry().MinimumThumbLength(&self.base_, c)
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:54; cc/layers/solid_color_scrollbar_layer_impl.cc:77-83.
    pub fn TrackLength(&self, c: &dyn ScrollbarGeometryRead<B>) -> f32 {
        self.Geometry().TrackLength(&self.base_, c)
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:55; cc/layers/solid_color_scrollbar_layer_impl.cc:85.
    pub fn TrackStart(&self) -> i32 {
        self.track_start_
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:56; cc/layers/solid_color_scrollbar_layer_impl.cc:87-89.
    pub fn IsThumbResizable(&self) -> bool {
        true
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:38.
    pub fn set_color(&mut self, color: Color4f) {
        self.color_ = color;
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:40.
    pub fn thumb_thickness(&self) -> i32 {
        self.thumb_thickness_
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:41.
    pub fn track_start(&self) -> i32 {
        self.track_start_
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:42.
    pub fn color(&self) -> Color4f {
        self.color_
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.h:32-34; cc/layers/solid_color_scrollbar_layer_impl.cc:91-112.
    pub fn AppendQuads(
        &mut self,
        _context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        let state = pass.CreateAndAppendSharedQuadState();
        self.base().PopulateSharedQuadState(
            pass.SharedQuadStateMutable(state),
            self.base().contents_opaque(),
            c,
        );
        let size = self.base().bounds(c);
        self.base().AppendDebugBorderQuad(
            pass,
            Rect::new(0, 0, size.width(), size.height()),
            state,
            data,
            c,
        );
        let thumb = self.base_.ComputeThumbQuadRect(&self.Geometry(), c);
        let visible = self
            .base()
            .draw_properties()
            .occlusion_in_content_space
            .GetUnoccludedContentRect(thumb);
        if visible.IsEmpty() {
            return;
        }
        let quad = pass.CreateAndAppendSolidColorDrawQuad();
        pass.SolidColorDrawQuadSetNew(quad, state, thumb, visible, self.color_, false);
    }
    /// Short sibling borrow used by the exact canonical scrollbar geometry body.
    pub fn Geometry(&self) -> SolidColorScrollbarGeometry<'_> {
        SolidColorScrollbarGeometry {
            thumb_thickness: &self.thumb_thickness_,
            track_start: &self.track_start_,
        }
    }
}
/// Source raw virtual receiver expressed as a borrow of only its two actual
/// derived scalar fields; no base/tree/context is stored in this facade.
pub struct SolidColorScrollbarGeometry<'a> {
    thumb_thickness: &'a i32,
    track_start: &'a i32,
}
impl<B: SolidColorScrollbarLayerImplTypes> ScrollbarLayerImplVirtual<B>
    for SolidColorScrollbarGeometry<'_>
{
    // Source: cc/layers/solid_color_scrollbar_layer_impl.cc:62-71.
    fn ThumbThickness(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32 {
        if *self.thumb_thickness != -1 {
            return *self.thumb_thickness;
        }
        if base.orientation() == ScrollbarOrientation::kHorizontal {
            base.base()
                .bounds_for_property_trees(c.PropertyTrees())
                .height()
        } else {
            base.base()
                .bounds_for_property_trees(c.PropertyTrees())
                .width()
        }
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.cc:73-75.
    fn MinimumThumbLength(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32 {
        self.ThumbThickness(base, c)
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.cc:77-83.
    fn TrackLength(
        &self,
        base: &ScrollbarLayerImplBase<B>,
        c: &dyn ScrollbarGeometryRead<B>,
    ) -> f32 {
        if base.orientation() == ScrollbarOrientation::kHorizontal {
            (base
                .base()
                .bounds_for_property_trees(c.PropertyTrees())
                .width()
                - self.TrackStart(base, c) * 2) as f32
        } else {
            base.base()
                .bounds_for_property_trees(c.PropertyTrees())
                .height() as f32
                + base.vertical_adjust()
                - (self.TrackStart(base, c) * 2) as f32
        }
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.cc:85.
    fn TrackStart(
        &self,
        _base: &ScrollbarLayerImplBase<B>,
        _c: &dyn ScrollbarGeometryRead<B>,
    ) -> i32 {
        *self.track_start
    }
    // Source: cc/layers/solid_color_scrollbar_layer_impl.cc:87-89.
    fn IsThumbResizable(&self) -> bool {
        true
    }
}

impl<B: SolidColorScrollbarLayerImplTypes>
    super::scrollbar_layer_impl_base::ScrollbarLayerImplOwner<B>
    for SolidColorScrollbarLayerImpl<B>
{
    fn AsScrollbarBase(&self) -> Option<&ScrollbarLayerImplBase<B>> {
        Some(self.scrollbar_base())
    }
    fn AsScrollbarBaseMutable(&mut self) -> Option<&mut ScrollbarLayerImplBase<B>> {
        Some(self.scrollbar_base_mut())
    }
}
impl<B: SolidColorScrollbarLayerImplTypes> crate::adapter::layer_impl::LayerImplOwner<B>
    for SolidColorScrollbarLayerImpl<B>
{
    fn impl_base(&self) -> &LayerImpl<B> {
        self.base()
    }
    fn impl_base_mut(&mut self) -> &mut LayerImpl<B> {
        self.base_mut()
    }
}
