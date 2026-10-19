// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/solid_color_layer_impl.h:1-15,23-24,48-51;
//         cc/layers/solid_color_layer_impl.cc:1-16,87.
use super::{
    append_quads_context::AppendQuadsContext,
    append_quads_data::AppendQuadsData,
    layer_impl::{LayerImpl, LayerImplVirtual},
};
use crate::{
    adapter::{
        geometry::Rect,
        layer_impl::{LayerImplContext, LayerImplSharedQuadState, LayerType},
        layer_tree_impl::{Color4f, LayerTreeId},
        occlusion::SkBlendMode,
        solid_color_layer_impl::*,
    },
    cc::trees::occlusion::Occlusion,
};
// Source: cc/layers/solid_color_layer_impl.h:16-47. Native derived owner.
pub struct SolidColorLayerImpl<B: SolidColorLayerImplTypes> {
    base_: LayerImpl<B>,
}
impl<B: SolidColorLayerImplTypes> SolidColorLayerImpl<B> {
    // Source: cc/layers/solid_color_layer_impl.h:18-21.
    pub fn Create(tree: LayerTreeId, id: i32, c: &mut LayerImplContext<'_, B>) -> Box<B::Layer> {
        Box::new(B::WrapSolidColorLayerImpl(Self::new(tree, id, c)))
    }
    // Source: cc/layers/solid_color_layer_impl.h:46; cc/layers/solid_color_layer_impl.cc:18-20.
    pub fn new(tree: LayerTreeId, id: i32, c: &mut LayerImplContext<'_, B>) -> Self {
        Self {
            base_: LayerImpl::new(tree, id, c),
        }
    }
    pub fn base(&self) -> &LayerImpl<B> {
        &self.base_
    }
    pub fn base_mut(&mut self) -> &mut LayerImpl<B> {
        &mut self.base_
    }
    // Source: cc/layers/solid_color_layer_impl.h:38; cc/layers/solid_color_layer_impl.cc:24-26.
    pub fn GetLayerType(&self) -> LayerType {
        LayerType::kSolidColor
    }
    // Source: cc/layers/solid_color_layer_impl.h:39-40; cc/layers/solid_color_layer_impl.cc:28-31.
    pub fn CreateLayerImpl(
        &self,
        tree: LayerTreeId,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Self::Create(tree, self.base_.id(), c)
    }
    // Source: cc/layers/solid_color_layer_impl.h:26-33; cc/layers/solid_color_layer_impl.cc:33-66.
    pub fn AppendSolidQuads(
        pass: &mut B::RenderPass,
        occlusion: &Occlusion<B::Transform>,
        state: B::SharedQuadStateHandle,
        visible_layer_rect: Rect,
        color: Color4f,
        force_aa_off: bool,
        effect_blend: SkBlendMode,
        _data: &mut AppendQuadsData<B::SurfaceId>,
    ) {
        if effect_blend == SkBlendMode::kSrcOver {
            let alpha = color.alpha * pass.SharedQuadState(state).opacity();
            if alpha < f32::EPSILON {
                return;
            }
        }
        let visible = occlusion.GetUnoccludedContentRect(visible_layer_rect);
        if visible.IsEmpty() {
            return;
        }
        let quad = pass.CreateAndAppendSolidColorDrawQuad();
        pass.SolidColorDrawQuadSetNew(
            quad,
            state,
            visible_layer_rect,
            visible,
            color,
            force_aa_off,
        );
    }
    // Source: cc/layers/solid_color_layer_impl.h:41-43; cc/layers/solid_color_layer_impl.cc:68-85.
    pub fn AppendQuads(
        &mut self,
        context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        Self::AppendQuadsForBase(&mut self.base_, context, pass, data, c);
    }
    // Source: cc/layers/solid_color_layer_impl.cc:68-85.
    // Rust split entry of the exact same source body, not another quad policy.
    fn AppendQuadsForBase(
        base: &mut LayerImpl<B>,
        _context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        let state = pass.CreateAndAppendSharedQuadState();
        base.PopulateSharedQuadState(
            pass.SharedQuadStateMutable(state),
            base.contents_opaque(),
            c,
        );
        let size = base.bounds(c);
        base.AppendDebugBorderQuad(
            pass,
            Rect::new(0, 0, size.width(), size.height()),
            state,
            data,
            c,
        );
        let effect = c
            .tree
            .property_trees()
            .effect_tree()
            .Node(base.effect_tree_index());
        let size = base.bounds(c);
        Self::AppendSolidQuads(
            pass,
            &base.draw_properties().occlusion_in_content_space,
            state,
            Rect::new(0, 0, size.width(), size.height()),
            base.background_color(),
            !c.tree.settings().enable_edge_anti_aliasing,
            effect.blend_mode,
            data,
        );
    }
}
// Source: cc/layers/solid_color_layer_impl.h:35; cc/layers/solid_color_layer_impl.cc:22.
// Default derived destruction immediately enters canonical resident base Drop.
/// The derived class has no sibling payload; this is its actual vtable facade.
pub struct SolidColorLayerImplVirtual;
impl<B: SolidColorLayerImplTypes> LayerImplVirtual<B> for SolidColorLayerImplVirtual {
    fn GetLayerType(&self, _base: &LayerImpl<B>) -> LayerType {
        LayerType::kSolidColor
    }
    fn CreateLayerImpl(
        &self,
        base: &LayerImpl<B>,
        tree: LayerTreeId,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        SolidColorLayerImpl::Create(tree, base.id(), c)
    }
    fn AppendQuads(
        &mut self,
        base: &mut LayerImpl<B>,
        context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        SolidColorLayerImpl::AppendQuadsForBase(base, context, pass, data, c);
    }
}
