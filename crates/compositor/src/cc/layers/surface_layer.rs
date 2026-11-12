// Translation status: partial (resident range/property push complete;
// submission-state callback and host SurfaceRange registration remain host
// capabilities).
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/surface_layer.h:35-90,97-100,116-125;
//         cc/layers/surface_layer.cc:19-35,52-84,109-116,134-146,155-157,184-214.
use super::layer::{Layer, LayerObject, LayerRef};
use crate::{
    adapter::{
        main_layer::{LayerContext, MainLayerServices},
        surface_layer::{SurfaceLayerImplTarget, SurfaceLayerTypes},
    },
    cc::trees::layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
};
use std::sync::RwLock;

pub trait SurfaceLayerObject<B: crate::adapter::main_layer::MainLayerTypes> {
    fn SetSurfaceRange(
        &self,
        range: B::SurfaceRange,
        deadline: Option<u32>,
        c: &mut LayerContext<'_, B>,
    );
    fn SetStretchContentToFillBounds(&self, value: bool, c: &mut LayerContext<'_, B>);
}

/// Main-thread resident SurfaceLayer. It owns only the embed range and flags;
/// the child CompositorFrame remains wholly owned by Viz.
pub struct SurfaceLayer<B: SurfaceLayerTypes> {
    base_: Layer<B>,
    surface_range_: RwLock<B::SurfaceRange>,
    deadline_in_frames_: RwLock<Option<u32>>,
    stretch_content_to_fill_bounds_: RwLock<bool>,
    is_reflection_: RwLock<bool>,
}

impl<B: SurfaceLayerTypes> SurfaceLayer<B> {
    pub fn Create(services: &dyn MainLayerServices) -> LayerRef<B> {
        LayerRef::FromObject(Self::new(services))
    }
    pub fn new(services: &dyn MainLayerServices) -> Self {
        Self {
            base_: Layer::new(services),
            surface_range_: RwLock::new(B::DefaultSurfaceRange()),
            deadline_in_frames_: RwLock::new(Some(0)),
            stretch_content_to_fill_bounds_: RwLock::new(false),
            is_reflection_: RwLock::new(false),
        }
    }
    pub fn base(&self) -> &Layer<B> {
        &self.base_
    }
    pub fn SetRange(
        &self,
        range: B::SurfaceRange,
        deadline: Option<u32>,
        c: &mut LayerContext<'_, B>,
    ) {
        let mut current = self
            .surface_range_
            .write()
            .unwrap_or_else(|error| error.into_inner());
        if B::SurfaceRangesEqual(&current, &range)
            && *self
                .deadline_in_frames_
                .read()
                .unwrap_or_else(|error| error.into_inner())
                == deadline
        {
            return;
        }
        *current = range;
        *self
            .deadline_in_frames_
            .write()
            .unwrap_or_else(|error| error.into_inner()) = deadline;
        drop(current);
        self.base_.UpdateDrawsContent(self, c);
        self.base_.SetNeedsCommit(c);
    }
    pub fn SetStretch(&self, value: bool, c: &mut LayerContext<'_, B>) {
        let mut current = self
            .stretch_content_to_fill_bounds_
            .write()
            .unwrap_or_else(|error| error.into_inner());
        if *current == value {
            return;
        }
        *current = value;
        drop(current);
        self.base_
            .SetNeedsPushProperties(Layer::<B>::kChangedGeneralProperty, c);
    }
}

impl<B: SurfaceLayerTypes> SurfaceLayerObject<B> for SurfaceLayer<B> {
    fn SetSurfaceRange(
        &self,
        range: B::SurfaceRange,
        deadline: Option<u32>,
        c: &mut LayerContext<'_, B>,
    ) {
        self.SetRange(range, deadline, c)
    }
    fn SetStretchContentToFillBounds(&self, value: bool, c: &mut LayerContext<'_, B>) {
        self.SetStretch(value, c)
    }
}

impl<B: SurfaceLayerTypes> LayerObject<B> for SurfaceLayer<B> {
    fn base(&self) -> &Layer<B> {
        &self.base_
    }
    fn AsSurfaceLayer(&self) -> Option<&dyn SurfaceLayerObject<B>> {
        Some(self)
    }
    fn HasDrawableContent(&self, c: &LayerContext<'_, B>) -> bool {
        let valid = {
            let range = self
                .surface_range_
                .read()
                .unwrap_or_else(|error| error.into_inner());
            B::SurfaceRangeIsValid(&range)
        };
        valid && self.base_.HasDrawableContent(c)
    }
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        context: &mut LayerTreeContext<'_, B>,
        _c: &mut LayerContext<'_, B>,
    ) -> Box<B::Layer> {
        B::Layer::CreateSurface(tree, self.base_.id(), context)
    }
    fn PushDirtyPropertiesTo(
        &self,
        target: &mut B::Layer,
        dirty: u8,
        commit: &mut crate::cc::trees::commit_state::CommitState<B>,
        c: &mut LayerContext<'_, B>,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.base_
            .PushDirtyPropertiesTo(target, dirty, commit, c, impl_tree);
        if dirty & Layer::<B>::kChangedGeneralProperty != 0 {
            let range = self
                .surface_range_
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .clone();
            let deadline = self
                .deadline_in_frames_
                .write()
                .unwrap_or_else(|error| error.into_inner())
                .take();
            target.SurfaceSetRange(range, deadline, impl_tree);
            target.SurfaceSetStretchContentToFillBounds(
                *self
                    .stretch_content_to_fill_bounds_
                    .read()
                    .unwrap_or_else(|error| error.into_inner()),
                impl_tree,
            );
            target.SurfaceSetIsReflection(
                *self
                    .is_reflection_
                    .read()
                    .unwrap_or_else(|error| error.into_inner()),
                impl_tree,
            );
        }
    }
}
