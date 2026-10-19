// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/solid_color_scrollbar_layer.h:1-17,33-34,67-69;
//         cc/layers/solid_color_scrollbar_layer.cc:1-14,128.
use super::{
    layer::{Layer, LayerBorrow, LayerObject, LayerRef},
    scrollbar_layer_base::{ScrollbarLayerBase, ScrollbarLayerObject, ScrollbarLayerType},
};
use crate::{
    adapter::{
        geometry::Rect,
        layer_tree_impl::Color4f,
        main_layer::{LayerContext, MainLayerHostId, MainLayerImplTree},
        scrollbar_layer_base::SolidColorScrollbarLayerObject,
        solid_color_scrollbar_layer::*,
    },
    cc::{
        base::protected_sequence_synchronizer::ProtectedSequenceReadable,
        input::scrollbar::{Scrollbar, ScrollbarOrientation},
        trees::{
            commit_state::CommitState,
            layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
        },
    },
};
use std::sync::Arc;
// Source: cc/layers/solid_color_scrollbar_layer.h:18-65. Native resident derived owner.
pub struct SolidColorScrollbarLayer<B: SolidColorScrollbarLayerTypes> {
    base_: ScrollbarLayerBase<B>,
    thumb_thickness_: i32,
    track_start_: i32,
    color_: ProtectedSequenceReadable<Color4f>,
}
impl<B: SolidColorScrollbarLayerTypes> SolidColorScrollbarLayer<B> {
    // Source: cc/layers/solid_color_scrollbar_layer.h:23-25; cc/layers/solid_color_scrollbar_layer.cc:23-51.
    pub fn CreateOrReuse(
        scrollbar: Arc<B::Scrollbar>,
        existing: Option<LayerBorrow<B>>,
        c: &mut LayerContext<'_, B>,
    ) -> LayerRef<B> {
        debug_assert!(scrollbar.IsOverlay());
        let horizontal = scrollbar.Orientation() == ScrollbarOrientation::kHorizontal;
        let thumb = scrollbar.ThumbRect();
        let thickness = if horizontal {
            thumb.height
        } else {
            thumb.width
        };
        let track = scrollbar.TrackRect();
        let start = if horizontal { track.x } else { track.y };
        let reuse = existing.as_ref().is_some_and(|layer| {
            layer.WithSourceRef(c, |object, _| {
                let scrollbar = object
                    .AsScrollbarLayer()
                    .expect("native existing scrollbar pointer");
                let solid = scrollbar
                    .AsSolidColorScrollbarLayer()
                    .expect("native solid scrollbar static_cast");
                solid.thumb_thickness() == thickness && solid.track_start() == start
            })
        });
        let result = if reuse {
            let existing = existing.as_ref().unwrap();
            existing.WithSourceRef(c, |object, _| {
                let layer = object.AsScrollbarLayer().unwrap();
                debug_assert_eq!(
                    scrollbar.Orientation(),
                    layer.scrollbar_base().orientation()
                );
                debug_assert_eq!(
                    scrollbar.IsLeftSideVerticalScrollbar(),
                    layer.scrollbar_base().is_left_side_vertical_scrollbar()
                );
            });
            existing.ToSourceRef(c)
        } else {
            Self::Create(
                scrollbar.Orientation(),
                thickness,
                start,
                scrollbar.IsLeftSideVerticalScrollbar(),
                c,
            )
        };
        result.borrow().WithSource(c, |object, c| {
            object
                .AsScrollbarLayer()
                .unwrap()
                .AsSolidColorScrollbarLayer()
                .unwrap()
                .SetColor(scrollbar.ThumbColor(), c)
        });
        result
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:27-31; cc/layers/solid_color_scrollbar_layer.cc:53-61.
    pub fn Create(
        orientation: ScrollbarOrientation,
        thumb_thickness: i32,
        track_start: i32,
        left: bool,
        c: &mut LayerContext<'_, B>,
    ) -> LayerRef<B> {
        LayerRef::FromObject(Self::new(
            orientation,
            thumb_thickness,
            track_start,
            left,
            c,
        ))
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:56-59; cc/layers/solid_color_scrollbar_layer.cc:63-73.
    pub fn new(
        orientation: ScrollbarOrientation,
        thumb_thickness: i32,
        track_start: i32,
        left: bool,
        c: &mut LayerContext<'_, B>,
    ) -> Self {
        let result = Self {
            base_: ScrollbarLayerBase::new(orientation, left, c.Services()),
            thumb_thickness_: thumb_thickness,
            track_start_: track_start,
            color_: ProtectedSequenceReadable::new(Color4f::default()),
        };
        result.base_.base().SetOpacity(0., c);
        result
    }
    pub fn scrollbar_base(&self) -> &ScrollbarLayerBase<B> {
        &self.base_
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:42.
    pub fn thumb_thickness(&self) -> i32 {
        self.thumb_thickness_
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:43.
    pub fn track_start(&self) -> i32 {
        self.track_start_
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:46.
    pub fn color(&self, c: &LayerContext<'_, B>) -> Color4f {
        *self.color_.Read(&self.base_.base().Synchronization(c))
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:45; cc/layers/solid_color_scrollbar_layer.cc:112-126.
    pub fn SetColor(&self, mut color: Color4f, c: &mut LayerContext<'_, B>) {
        let base = self.base_.base();
        if base.host_id().is_some_and(|host| {
            c.Host(host)
                .GetSettings()
                .using_synchronous_renderer_compositor
        }) {
            color = Color4f::default();
        }
        if color != *self.color_.Read(&base.Synchronization(c)) {
            *self.color_.Write(&base.Synchronization(c)) = color;
            let size = base.bounds(c);
            base.SetNeedsDisplayRect(self, &Rect::new(0, 0, size.width(), size.height()), c);
            base.SetNeedsCommit(c);
        }
    }
}
impl<B: SolidColorScrollbarLayerTypes> LayerObject<B> for SolidColorScrollbarLayer<B> {
    fn base(&self) -> &Layer<B> {
        self.base_.base()
    }
    fn AsScrollbarLayer(&self) -> Option<&dyn ScrollbarLayerObject<B>> {
        Some(self)
    }
    // Source: cc/layers/scrollbar_layer_base.cc:98-100. Inherited final slot.
    fn IsScrollbarLayerForTesting(&self) -> bool {
        self.base_.IsScrollbarLayerForTesting()
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:20-21; cc/layers/solid_color_scrollbar_layer.cc:16-21.
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        context: &mut LayerTreeContext<'_, B>,
        _c: &mut LayerContext<'_, B>,
    ) -> Box<B::Layer> {
        B::Layer::CreateSolidColorScrollbar(
            tree,
            self.base().id(),
            self.base_.orientation(),
            self.thumb_thickness_,
            self.track_start_,
            self.base_.is_left_side_vertical_scrollbar(),
            context,
        )
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:38; cc/layers/solid_color_scrollbar_layer.cc:77-81.
    fn SetOpacity(&self, opacity: f32, c: &mut LayerContext<'_, B>) {
        debug_assert_eq!(opacity, 0.);
        self.base().SetOpacity(opacity, c);
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:39; cc/layers/solid_color_scrollbar_layer.cc:83.
    fn SetNeedsDisplayRect(&self, _rect: &Rect, _c: &mut LayerContext<'_, B>) {}
    // Source: cc/layers/solid_color_scrollbar_layer.h:37; cc/layers/solid_color_scrollbar_layer.cc:85-87.
    fn OpacityCanAnimateOnImplThread(&self) -> bool {
        true
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:51-53; cc/layers/solid_color_scrollbar_layer.cc:94-103.
    fn PushDirtyPropertiesTo(
        &self,
        target: &mut B::Layer,
        dirty: u8,
        commit: &mut CommitState<B>,
        c: &mut LayerContext<'_, B>,
        tree: &mut B::ImplContext<'_>,
    ) {
        self.base_
            .PushDirtyPropertiesTo(target, dirty, commit, c, tree);
        if dirty & Layer::<B>::kChangedGeneralProperty != 0 {
            target.scrollbar_set_color(self.color(c), tree);
        }
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:40; cc/layers/solid_color_scrollbar_layer.cc:105-110.
    fn SetLayerTreeHost(
        &self,
        allocation: &crate::cc::layers::layer::LayerBorrow<B>,
        host: Option<MainLayerHostId>,
        c: &mut LayerContext<'_, B>,
    ) {
        if host != self.base().host_id() {
            LayerObject::SetLayerTreeHost(self.base(), allocation, host, c);
            self.SetColor(self.color(c), c);
        }
    }
}
impl<B: SolidColorScrollbarLayerTypes> ScrollbarLayerObject<B> for SolidColorScrollbarLayer<B> {
    fn scrollbar_base(&self) -> &ScrollbarLayerBase<B> {
        &self.base_
    }
    // Source: cc/layers/solid_color_scrollbar_layer.h:48; cc/layers/solid_color_scrollbar_layer.cc:89-92.
    fn GetScrollbarLayerType(&self) -> ScrollbarLayerType {
        ScrollbarLayerType::kSolidColor
    }
    fn AsSolidColorScrollbarLayer(&self) -> Option<&dyn SolidColorScrollbarLayerObject<B>> {
        Some(self)
    }
}
impl<B: SolidColorScrollbarLayerTypes> SolidColorScrollbarLayerObject<B>
    for SolidColorScrollbarLayer<B>
{
    fn thumb_thickness(&self) -> i32 {
        self.thumb_thickness()
    }
    fn track_start(&self) -> i32 {
        self.track_start()
    }
    fn SetColor(&self, color: Color4f, c: &mut LayerContext<'_, B>) {
        self.SetColor(color, c);
    }
}
// Source: cc/layers/solid_color_scrollbar_layer.h:60; cc/layers/solid_color_scrollbar_layer.cc:75.
// Default derived destructor has only scalar members; native base shutdown is unique.
