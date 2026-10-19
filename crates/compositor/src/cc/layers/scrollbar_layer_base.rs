// Translation status: complete
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/scrollbar_layer_base.h:1-14,56-59; cc/layers/scrollbar_layer_base.cc:1-16,102.
use super::layer::{Layer, LayerBorrow, LayerObject, LayerRef};
use crate::{
    adapter::{
        input_types::ElementId,
        main_layer::{LayerContext, MainLayerImplTree, MainLayerServices, MainLayerTypes},
        scrollbar_layer_base::*,
        solid_color_scrollbar_layer::SolidColorScrollbarLayerTypes,
    },
    cc::{
        base::protected_sequence_synchronizer::ProtectedSequenceReadable,
        input::scrollbar::{Scrollbar, ScrollbarOrientation},
        trees::commit_state::CommitState,
    },
};
use std::sync::Arc;
// Source: cc/layers/scrollbar_layer_base.h:32-37.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarLayerType {
    kSolidColor,
    kPainted,
    kNinePatchThumb,
}
// Source: cc/layers/scrollbar_layer_base.h:14-55. Source abstract derived owner.
pub struct ScrollbarLayerBase<B: MainLayerTypes> {
    base_: Layer<B>,
    orientation_: ScrollbarOrientation,
    is_left_side_vertical_scrollbar_: bool,
    scroll_element_id_: ProtectedSequenceReadable<ElementId>,
    has_find_in_page_tickmarks_: ProtectedSequenceReadable<bool>,
}
/// Native virtual scrollbar-class identity and actual resident base projection.
pub trait ScrollbarLayerObject<B: MainLayerTypes>: LayerObject<B> {
    fn scrollbar_base(&self) -> &ScrollbarLayerBase<B>;
    fn GetScrollbarLayerType(&self) -> ScrollbarLayerType;
    fn AsSolidColorScrollbarLayer(&self) -> Option<&dyn SolidColorScrollbarLayerObject<B>> {
        None
    }
}
impl<B: MainLayerTypes> ScrollbarLayerBase<B> {
    // Source: cc/layers/scrollbar_layer_base.h:40-41; cc/layers/scrollbar_layer_base.cc:18-21.
    pub fn new(
        orientation: ScrollbarOrientation,
        is_left_side_vertical_scrollbar: bool,
        services: &dyn MainLayerServices,
    ) -> Self {
        Self {
            base_: Layer::new(services),
            orientation_: orientation,
            is_left_side_vertical_scrollbar_: is_left_side_vertical_scrollbar,
            scroll_element_id_: ProtectedSequenceReadable::new(Default::default()),
            has_find_in_page_tickmarks_: ProtectedSequenceReadable::new(false),
        }
    }
    pub fn base(&self) -> &Layer<B> {
        &self.base_
    }
    // Source: cc/layers/scrollbar_layer_base.h:21.
    pub fn scroll_element_id(&self, c: &LayerContext<'_, B>) -> ElementId {
        *self.scroll_element_id_.Read(&self.base_.Synchronization(c))
    }
    // Source: cc/layers/scrollbar_layer_base.h:23.
    pub fn orientation(&self) -> ScrollbarOrientation {
        self.orientation_
    }
    // Source: cc/layers/scrollbar_layer_base.h:24-26.
    pub fn is_left_side_vertical_scrollbar(&self) -> bool {
        self.is_left_side_vertical_scrollbar_
    }
    // Source: cc/layers/scrollbar_layer_base.h:27-29.
    pub fn has_find_in_page_tickmarks(&self, c: &LayerContext<'_, B>) -> bool {
        *self
            .has_find_in_page_tickmarks_
            .Read(&self.base_.Synchronization(c))
    }
    // Source: cc/layers/scrollbar_layer_base.h:20; cc/layers/scrollbar_layer_base.cc:64-70.
    pub fn SetScrollElementId(&self, element_id: ElementId, c: &mut LayerContext<'_, B>) {
        if element_id == *self.scroll_element_id_.Read(&self.base_.Synchronization(c)) {
            return;
        }
        *self
            .scroll_element_id_
            .Write(&self.base_.Synchronization(c)) = element_id;
        self.base_.SetNeedsCommit(c);
    }
    // Source: cc/layers/scrollbar_layer_base.h:30; cc/layers/scrollbar_layer_base.cc:72-80.
    pub fn SetHasFindInPageTickmarks(&self, value: bool, c: &mut LayerContext<'_, B>) -> bool {
        if *self
            .has_find_in_page_tickmarks_
            .Read(&self.base_.Synchronization(c))
            == value
        {
            return false;
        }
        *self
            .has_find_in_page_tickmarks_
            .Write(&self.base_.Synchronization(c)) = value;
        self.base_
            .SetNeedsPushProperties(Layer::<B>::kChangedGeneralProperty, c);
        true
    }
    // Source: cc/layers/scrollbar_layer_base.h:49; cc/layers/scrollbar_layer_base.cc:98-100.
    pub fn IsScrollbarLayerForTesting(&self) -> bool {
        true
    }
}
impl<B: ScrollbarLayerTypes> ScrollbarLayerBase<B> {
    // Source: cc/layers/scrollbar_layer_base.h:44-46; cc/layers/scrollbar_layer_base.cc:82-96.
    pub fn PushDirtyPropertiesTo(
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
            debug_assert_eq!(target.scrollbar_orientation(), self.orientation_);
            debug_assert_eq!(
                target.scrollbar_is_left_side_vertical(),
                self.is_left_side_vertical_scrollbar_
            );
            target.scrollbar_SetHasFindInPageTickmarks(
                *self
                    .has_find_in_page_tickmarks_
                    .Read(&self.base_.Synchronization(c)),
                tree,
            );
            target.scrollbar_SetScrollElementId(
                *self.scroll_element_id_.Read(&self.base_.Synchronization(c)),
                tree,
            );
        }
    }
    // Source: cc/layers/scrollbar_layer_base.h:42; cc/layers/scrollbar_layer_base.cc:23.
    // Native defaulted derived destructor owns no extra payload. Protected fields
    // contain only scalar values; the canonical Layer shutdown remains unique.
}
impl<B: SolidColorScrollbarLayerTypes> ScrollbarLayerBase<B> {
    // Source: cc/layers/scrollbar_layer_base.h:16-18; cc/layers/scrollbar_layer_base.cc:25-62.
    pub fn CreateOrReuse(
        scrollbar: Arc<B::Scrollbar>,
        mut existing: Option<LayerBorrow<B>>,
        c: &mut LayerContext<'_, B>,
    ) -> Result<LayerRef<B>, ScrollbarConstructorUnavailable> {
        let mut needed = ScrollbarLayerType::kPainted;
        if scrollbar.IsSolidColor() {
            needed = ScrollbarLayerType::kSolidColor;
        } else if scrollbar.UsesNinePatchThumbResource() {
            debug_assert!(scrollbar.IsOverlay());
            needed = ScrollbarLayerType::kNinePatchThumb;
        }
        if existing.as_ref().is_some_and(|layer| {
            layer.WithSourceRef(c, |object, _| {
                let layer = object
                    .AsScrollbarLayer()
                    .expect("native scrollbar static_cast requires its actual resident subtype");
                layer.GetScrollbarLayerType() != needed
                    || layer.scrollbar_base().orientation() != scrollbar.Orientation()
                    || layer.scrollbar_base().is_left_side_vertical_scrollbar()
                        != scrollbar.IsLeftSideVerticalScrollbar()
            })
        }) {
            existing = None;
        }
        match needed {
            ScrollbarLayerType::kSolidColor => Ok(
                super::solid_color_scrollbar_layer::SolidColorScrollbarLayer::<B>::CreateOrReuse(
                    scrollbar, existing, c,
                ),
            ),
            ScrollbarLayerType::kPainted => {
                // Rust external capability error only. Native selection/reuse above
                // is unchanged; missing owners never become stand-in layers.
                let create = B::PaintedScrollbarConstructor()
                    .ok_or(ScrollbarConstructorUnavailable::Painted)?;
                Ok(create(scrollbar, existing, c))
            }
            ScrollbarLayerType::kNinePatchThumb => {
                let create = B::NinePatchScrollbarConstructor()
                    .ok_or(ScrollbarConstructorUnavailable::NinePatchThumb)?;
                Ok(create(scrollbar, existing, c))
            }
        }
        // The exhaustive native enum match makes source NOTREACHED unreachable.
    }
}
