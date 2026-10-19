// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/solid_color_layer.h:1-17,22-23,38-40;
//         cc/layers/solid_color_layer.cc:1-12,48.
use super::layer::{Layer, LayerObject, LayerRef};
use crate::{
    adapter::{
        layer_tree_impl::Color4f,
        main_layer::{LayerContext, MainLayerServices},
        solid_color_layer::*,
    },
    cc::trees::layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
};
use std::sync::Arc;
// Source: cc/layers/solid_color_layer.h:18-37. Native empty derived resident owner.
pub struct SolidColorLayer<B: SolidColorLayerTypes> {
    base_: Layer<B>,
}
impl<B: SolidColorLayerTypes> SolidColorLayer<B> {
    // Source: cc/layers/solid_color_layer.h:20; cc/layers/solid_color_layer.cc:19-21.
    pub fn Create(services: &dyn MainLayerServices) -> LayerRef<B> {
        LayerRef::FromObject(Self::new(services))
    }
    // Source: cc/layers/solid_color_layer.h:33; cc/layers/solid_color_layer.cc:23.
    pub fn new(services: &dyn MainLayerServices) -> Self {
        Self {
            base_: Layer::new(services),
        }
    }
    pub fn base(&self) -> &Layer<B> {
        &self.base_
    }
}
impl<B: SolidColorLayerTypes> LayerObject<B> for SolidColorLayer<B> {
    fn base(&self) -> &Layer<B> {
        &self.base_
    }
    // Source: cc/layers/solid_color_layer.h:25-26; cc/layers/solid_color_layer.cc:14-17.
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        context: &mut LayerTreeContext<'_, B>,
        _c: &mut LayerContext<'_, B>,
    ) -> Box<B::Layer> {
        B::Layer::CreateSolidColor(tree, self.base_.id(), context)
    }
    // Source: cc/layers/solid_color_layer.h:28; cc/layers/solid_color_layer.cc:27-31.
    fn SetBackgroundColor(&self, color: Color4f, c: &mut LayerContext<'_, B>) {
        self.base_.SetContentsOpaque(color.alpha == 1., c);
        self.base_.SetBackgroundColor(color, c);
        self.base_.UpdateDrawsContent(self, c);
    }
    // Source: cc/layers/solid_color_layer.h:29; cc/layers/solid_color_layer.cc:33-42.
    fn GetPicture(&self, c: &mut LayerContext<'_, B>) -> Option<Arc<B::Picture>> {
        if !self.base_.draws_content(c) || self.base_.bounds(c).IsEmpty() {
            return None;
        }
        let mut recorder = c.DerivedServices().CreateSolidPictureRecorder();
        let bounds = self.base_.bounds(c);
        recorder.BeginRecording(bounds.width(), bounds.height());
        recorder.DrawColor(self.base_.background_color(c));
        Some(recorder.FinishRecordingAsPicture())
    }
    // Source: cc/layers/solid_color_layer.h:30; cc/layers/solid_color_layer.cc:44-46.
    fn IsSolidColorLayerForTesting(&self) -> bool {
        true
    }
}
// Source: cc/layers/solid_color_layer.h:36; cc/layers/solid_color_layer.cc:25.
// Native defaulted derived destructor has no payload; LayerObject's default
// NativeShutdown invokes the unique canonical base destructor synchronously.
