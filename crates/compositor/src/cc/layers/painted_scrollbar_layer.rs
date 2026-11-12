// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Source: cc/layers/painted_scrollbar_layer.{h,cc}.
use super::{
    layer::{Layer, LayerBorrow, LayerObject, LayerRef},
    scrollbar_layer_base::{ScrollbarLayerBase, ScrollbarLayerObject, ScrollbarLayerType},
};
use crate::{
    adapter::{
        geometry::Size,
        layer_tree_impl::UIResourceId,
        main_layer::{LayerContext, MainLayerHostId, MainLayerImplTree},
        painted_scrollbar_layer::{PaintedScrollbarLayerImplTarget, PaintedScrollbarLayerTypes},
    },
    cc::{
        input::scrollbar::Scrollbar,
        layers::layer::LayerRef as MainLayerRef,
        trees::{
            commit_state::CommitState,
            layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
        },
    },
};
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy)]
struct PaintedState {
    internal_contents_scale: f32,
    internal_content_bounds: Size,
    thumb_resource: UIResourceId,
    painted_opacity: f32,
    minimum_thumb_length: i32,
    thumb_thickness: i32,
    track_rect: crate::adapter::geometry::Rect,
    is_overlay: bool,
}

pub struct PaintedScrollbarLayer<B: PaintedScrollbarLayerTypes> {
    base_: ScrollbarLayerBase<B>,
    scrollbar_: RwLock<Arc<B::Scrollbar>>,
    state_: RwLock<PaintedState>,
}

impl<B: PaintedScrollbarLayerTypes> PaintedScrollbarLayer<B> {
    pub fn Constructor(
        scrollbar: Arc<B::Scrollbar>,
        existing: Option<LayerBorrow<B>>,
        c: &mut LayerContext<'_, B>,
    ) -> LayerRef<B> {
        Self::CreateOrReuse(scrollbar, existing, c)
    }

    // Source: painted_scrollbar_layer.cc:31-38. Chromium reuses the resident
    // layer when the ScrollbarDisplayItem still names the same WebScrollbar.
    // Rechrom receives a fresh immutable snapshot for that same logical item,
    // so replace the snapshot inside the already selected existing layer while
    // preserving its Layer id and impl-side animation controller.
    pub fn CreateOrReuse(
        scrollbar: Arc<B::Scrollbar>,
        existing: Option<LayerBorrow<B>>,
        c: &mut LayerContext<'_, B>,
    ) -> LayerRef<B> {
        if let Some(existing) = existing {
            if existing.WithSourceRef(c, |object, _| {
                object.ReusePaintedScrollbarSnapshot(scrollbar.clone())
            }) {
                return existing.ToSourceRef(c);
            }
        }
        LayerRef::FromObject(Self::new(scrollbar, c))
    }

    fn new(scrollbar: Arc<B::Scrollbar>, c: &mut LayerContext<'_, B>) -> Self {
        let thumb = scrollbar.ThumbRect();
        let thumb_thickness = if scrollbar.Orientation()
            == crate::cc::input::scrollbar::ScrollbarOrientation::kHorizontal
        {
            thumb.height
        } else {
            thumb.width
        };
        Self {
            base_: ScrollbarLayerBase::new(
                scrollbar.Orientation(),
                scrollbar.IsLeftSideVerticalScrollbar(),
                c.Services(),
            ),
            state_: RwLock::new(PaintedState {
                internal_contents_scale: 1.0,
                internal_content_bounds: Size::default(),
                thumb_resource: UIResourceId::default(),
                painted_opacity: scrollbar.Opacity(),
                minimum_thumb_length: scrollbar.MinimumThumbLength(),
                thumb_thickness,
                track_rect: scrollbar.TrackRect(),
                is_overlay: scrollbar.IsOverlay(),
            }),
            scrollbar_: RwLock::new(scrollbar),
        }
    }

    pub fn scrollbar_base(&self) -> &ScrollbarLayerBase<B> {
        &self.base_
    }

    fn UpdatePainted(&self, c: &mut LayerContext<'_, B>) -> bool {
        let mut updated = self.base_.base().Update(c);
        let Some(host_id) = self.base_.base().host_id() else {
            return updated;
        };
        let scrollbar = self
            .scrollbar_
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let scale = c.Host(host_id).device_scale_factor().max(1.0);
        let thumb = scrollbar.ThumbRect();
        let logical = Size::new(thumb.width.max(0), thumb.height.max(0));
        let pixel = Size::new(
            (logical.width() as f32 * scale).ceil() as i32,
            (logical.height() as f32 * scale).ceil() as i32,
        );
        let mut state = self.state_.write().unwrap_or_else(|e| e.into_inner());
        let repaint = state.thumb_resource == UIResourceId::default()
            || state.internal_content_bounds != pixel
            || scrollbar.ThumbNeedsRepaint();
        if repaint && !pixel.IsEmpty() {
            let bitmap = B::RasterizeScrollbarThumb(&scrollbar, logical, pixel);
            if state.thumb_resource != UIResourceId::default() {
                c.HostMut(host_id).DeleteUIResource(state.thumb_resource);
            }
            state.thumb_resource = c.HostMut(host_id).CreateUIResource(bitmap);
            state.internal_contents_scale = scale;
            state.internal_content_bounds = pixel;
            updated = true;
        }
        let thumb_thickness = if scrollbar.Orientation()
            == crate::cc::input::scrollbar::ScrollbarOrientation::kHorizontal
        {
            thumb.height
        } else {
            thumb.width
        };
        let track_rect = scrollbar.TrackRect();
        let minimum_thumb_length = scrollbar.MinimumThumbLength();
        if state.thumb_thickness != thumb_thickness {
            state.thumb_thickness = thumb_thickness;
            updated = true;
        }
        if state.track_rect != track_rect {
            state.track_rect = track_rect;
            updated = true;
        }
        if state.minimum_thumb_length != minimum_thumb_length {
            state.minimum_thumb_length = minimum_thumb_length;
            updated = true;
        }
        let opacity = scrollbar.Opacity();
        if state.painted_opacity != opacity {
            state.painted_opacity = opacity;
            updated = true;
        }
        if updated {
            self.base_
                .base()
                .SetNeedsPushProperties(Layer::<B>::kChangedGeneralProperty, c);
        }
        updated
    }
}

impl<B: PaintedScrollbarLayerTypes> LayerObject<B> for PaintedScrollbarLayer<B> {
    fn base(&self) -> &Layer<B> {
        self.base_.base()
    }
    fn ReusePaintedScrollbarSnapshot(&self, scrollbar: Arc<B::Scrollbar>) -> bool {
        let current = self
            .scrollbar_
            .read()
            .unwrap_or_else(|e| e.into_inner());
        // These are constructor-time invariants in Chromium's
        // PaintedScrollbarLayer::UpdateGeometry. A changed invariant requires
        // a new most-derived layer; ordinary geometry/paint changes reuse it.
        if current.Orientation() != scrollbar.Orientation()
            || current.IsLeftSideVerticalScrollbar()
                != scrollbar.IsLeftSideVerticalScrollbar()
            || current.IsOverlay() != scrollbar.IsOverlay()
            || current.SupportsDragSnapBack() != scrollbar.SupportsDragSnapBack()
        {
            return false;
        }
        drop(current);
        *self
            .scrollbar_
            .write()
            .unwrap_or_else(|e| e.into_inner()) = scrollbar;
        true
    }
    fn AsScrollbarLayer(&self) -> Option<&dyn ScrollbarLayerObject<B>> {
        Some(self)
    }
    fn IsScrollbarLayerForTesting(&self) -> bool {
        true
    }
    fn Update(&self, c: &mut LayerContext<'_, B>) -> bool {
        self.UpdatePainted(c)
    }
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        context: &mut LayerTreeContext<'_, B>,
        _c: &mut LayerContext<'_, B>,
    ) -> Box<B::Layer> {
        B::Layer::CreatePaintedScrollbar(
            tree,
            self.base().id(),
            self.base_.orientation(),
            self.base_.is_left_side_vertical_scrollbar(),
            context,
        )
    }
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
        if dirty & Layer::<B>::kChangedGeneralProperty == 0 {
            return;
        }
        let state = *self.state_.read().unwrap_or_else(|e| e.into_inner());
        let target = target
            .painted_scrollbar()
            .expect("native PaintedScrollbarLayerImpl target subtype");
        target.SetInternalContentsScaleAndBounds(
            state.internal_contents_scale,
            state.internal_content_bounds,
        );
        target.SetThumbUIResourceId(state.thumb_resource);
        target.SetScrollbarPaintedOpacity(state.painted_opacity);
        target.SetTrackRect(state.track_rect);
        target.SetMinimumThumbLength(state.minimum_thumb_length);
        target.SetThumbThickness(state.thumb_thickness);
        target
            .scrollbar_base_mut()
            .set_is_overlay_scrollbar(state.is_overlay);
    }
    fn SetLayerTreeHost(
        &self,
        allocation: &LayerBorrow<B>,
        host: Option<MainLayerHostId>,
        c: &mut LayerContext<'_, B>,
    ) {
        LayerObject::SetLayerTreeHost(self.base(), allocation, host, c);
    }
}

impl<B: PaintedScrollbarLayerTypes> ScrollbarLayerObject<B> for PaintedScrollbarLayer<B> {
    fn scrollbar_base(&self) -> &ScrollbarLayerBase<B> {
        &self.base_
    }
    fn GetScrollbarLayerType(&self) -> ScrollbarLayerType {
        ScrollbarLayerType::kPainted
    }
}
