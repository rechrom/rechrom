// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/picture_layer.h:1-30,36,48,60,70,73-80,91-95;
//         cc/layers/picture_layer.cc:1-26,264.
use super::{
    content_layer_client::ContentLayerClient,
    layer::{Layer, LayerObject, LayerRef, PictureLayerObject, PictureLayerRef},
    recording_source::RecordingSource,
};
use crate::{
    adapter::{
        geometry::Rect,
        layer_tree_host_impl::ContentColorUsage,
        layer_tree_impl::{Color4f, LayerTreeTypes},
        main_layer::{
            LayerContext, LiveFieldRead, LiveFieldWrite, MainLayerHostId, MainLayerImplTree,
        },
        picture_layer::{
            PictureDisplayItemList, PictureLayerImplTarget, PictureLayerMicroBenchmark,
            PictureLayerNodeInfo, PictureLayerServices, PictureLayerTypes, PictureRecorder,
        },
        property_tree::PropertyTreeTransform,
        recording_source::RecordingSourceTypes,
    },
    cc::{
        base::{
            math_util::MathUtil,
            protected_sequence_synchronizer::{
                ProtectedSequenceForbidden, ProtectedSequenceReadable, ProtectedSequenceWritable,
            },
            region::Region,
        },
        trees::{
            commit_state::CommitState,
            layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
            property_ids::kInvalidPropertyNodeId,
        },
    },
};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
type Recording<B> = RecordingSource<<B as PictureLayerTypes>::Recording>;
type RecordingRead<'a, B> = LiveFieldRead<RwLockReadGuard<'a, Option<Recording<B>>>, Recording<B>>;
type RecordingWrite<'a, B> =
    LiveFieldWrite<RwLockWriteGuard<'a, Option<Recording<B>>>, Recording<B>>;
type DisplayList<B> =
    <<B as PictureLayerTypes>::Recording as RecordingSourceTypes>::DisplayItemList;
type Raster<B> = <<B as PictureLayerTypes>::Recording as RecordingSourceTypes>::RasterSource;
/// Source cc::ContentLayerClient* is a nonowning virtual interface to the same
/// resident client allocation. Different native clients share this interface.
pub type PictureLayerClient<B> =
    dyn for<'c> ContentLayerClient<DisplayList<B>, LayerContext<'c, B>> + Send + Sync;
pub type PictureLayerClientWeak<B> = Weak<RwLock<PictureLayerClient<B>>>;
// Source: cc/layers/picture_layer.h:81-90.
// Base and recording are resident owning state. client_ is a direct weak Paint
// client allocation, not an ID lookup or owner/host/service backpointer. The
// per-field lock expresses safe access to native unprotected PaintContent fields.
// Optional owned values permit the source derived destructor to run at the last
// native LayerRef release even if a raw Rust allocation borrow still exists.
pub struct PictureLayer<B: PictureLayerTypes> {
    base_: Layer<B>,
    client_: RwLock<Option<PictureLayerClientWeak<B>>>,
    is_backdrop_filter_mask_: RwLock<bool>,
    recording_source_: ProtectedSequenceWritable<Option<Recording<B>>>,
    instrumentation_object_tracker_: ProtectedSequenceForbidden<Option<B::ObjectTracker>>,
    last_updated_invalidation_:
        ProtectedSequenceWritable<Option<Region<<B as LayerTreeTypes>::RegionBackend>>>,
    update_source_frame_number_: ProtectedSequenceReadable<i32>,
}
impl<B: PictureLayerTypes> PictureLayer<B> {
    // Source: cc/layers/picture_layer.h:26;
    //         cc/layers/picture_layer.cc:27-29.
    pub fn Create(
        client: Option<PictureLayerClientWeak<B>>,
        c: &mut LayerContext<'_, B>,
    ) -> PictureLayerRef<B> {
        let layer = LayerRef::FromObject(Self::new(client, c));
        match layer.IntoPicture() {
            Ok(picture) => picture,
            Err(_) => unreachable!("native PictureLayer upcast retains actual derived owner"),
        }
    }
    // Source: cc/layers/picture_layer.h:61;
    //         cc/layers/picture_layer.cc:31-34.
    fn new(client: Option<PictureLayerClientWeak<B>>, c: &mut LayerContext<'_, B>) -> Self {
        let base = Layer::new(c.Services());
        let recording = RecordingSource::new();
        let tracker = c.DerivedServices().CreateObjectTracker(base.id());
        Self {
            base_: base,
            client_: RwLock::new(client),
            is_backdrop_filter_mask_: RwLock::new(false),
            recording_source_: ProtectedSequenceWritable::new(Some(recording)),
            instrumentation_object_tracker_: ProtectedSequenceForbidden::new(Some(tracker)),
            last_updated_invalidation_: ProtectedSequenceWritable::new(Some(Region::default())),
            update_source_frame_number_: ProtectedSequenceReadable::new(-1),
        }
    }
    // Source: cc/layers/picture_layer.h:62;
    //         cc/layers/picture_layer.cc:36.
    fn DestroyMembers(&self, c: &LayerContext<'_, B>) {
        let sync = self.base_.Synchronization(c);
        // Native members destruct in reverse declaration order: Region, tracker,
        // recording. Explicit take preserves it before the canonical base dtor.
        drop(self.last_updated_invalidation_.Write(&sync).take());
        drop(self.instrumentation_object_tracker_.Write(&sync).take());
        drop(self.recording_source_.Write(&sync).take());
    }
    // Source: cc/layers/picture_layer.h:48.
    pub fn client(&self) -> Option<PictureLayerClientWeak<B>> {
        self.client_.read().unwrap().clone()
    }
    fn Client(&self) -> Arc<RwLock<PictureLayerClient<B>>> {
        self.client()
            .expect("source nonnull ContentLayerClient")
            .upgrade()
            .expect("ContentLayerClient must outlive its native weak binding")
    }
    // Source: cc/layers/picture_layer.h:53-58.
    pub fn GetRecordingSourceForTesting(&self, c: &LayerContext<'_, B>) -> RecordingRead<'_, B> {
        LiveFieldRead::new(
            self.recording_source_.Read(&self.base_.Synchronization(c)),
            |r| r.as_ref().expect("native recording owner is alive"),
        )
    }
    pub fn GetRecordingSourceForTestingMutable(
        &self,
        c: &LayerContext<'_, B>,
    ) -> RecordingWrite<'_, B> {
        LiveFieldWrite::new(
            self.recording_source_.Write(&self.base_.Synchronization(c)),
            |r| r.as_ref().expect("native recording owner is alive"),
            |r| r.as_mut().expect("native recording owner is alive"),
        )
    }
    // Source: cc/layers/picture_layer.h:71;
    //         cc/layers/picture_layer.cc:76-78.
    fn CreateRasterSource(&self, c: &mut LayerContext<'_, B>) -> Arc<Raster<B>> {
        let recording = self.GetRecordingSourceForTesting(c);
        recording.CreateRasterSource(&*c.DerivedServices())
    }
    // Source: cc/layers/picture_layer.h:50-51;
    //         cc/layers/picture_layer.cc:100-107.
    pub fn SetForceUpdateRecordingSource(&self, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.base_.IsPropertyChangeAllowed(c));
        self.GetRecordingSourceForTestingMutable(c)
            .set_force_update();
        self.base_
            .SetNeedsPushProperties(Layer::<B>::kChangedGeneralProperty, c);
        if self.base_.draws_content(c) && self.base_.IsAttached() {
            c.HostMut(self.base_.host_id().unwrap())
                .SetNeedsUpdateLayers();
        }
    }
    // Source: cc/layers/picture_layer.h:31;
    //         cc/layers/picture_layer.cc:174-177.
    pub fn ClearClient(&self, c: &mut LayerContext<'_, B>) {
        *self.client_.write().unwrap() = None;
        self.base_.UpdateDrawsContent(self, c);
    }
    // Source: cc/layers/picture_layer.h:33-34;
    //         cc/layers/picture_layer.cc:183-190.
    pub fn is_backdrop_filter_mask(&self) -> bool {
        *self.is_backdrop_filter_mask_.read().unwrap()
    }
    pub fn SetIsBackdropFilterMask(&self, value: bool, c: &mut LayerContext<'_, B>) {
        let mut mask = self.is_backdrop_filter_mask_.write().unwrap();
        if *mask == value {
            return;
        }
        *mask = value;
        drop(mask);
        self.base_.SetNeedsCommit(c);
    }
    // Source: cc/layers/picture_layer.h:74-75;
    //         cc/layers/picture_layer.cc:238-258.
    fn DropRecordingSourceContentIfInvalid(&self, frame: i32, c: &LayerContext<'_, B>) {
        let size = self.GetRecordingSourceForTesting(c).size();
        let bounds = *self.base_.bounds(c);
        debug_assert!(
            *self
                .update_source_frame_number_
                .Read(&self.base_.Synchronization(c))
                != frame
                || bounds == size
        );
        if *self
            .update_source_frame_number_
            .Read(&self.base_.Synchronization(c))
            != frame
            && size != bounds
        {
            self.GetRecordingSourceForTestingMutable(c).SetEmptyBounds();
        }
    }
    // Source: cc/layers/picture_layer.h:77;
    //         cc/layers/picture_layer.cc:260-262.
    fn GetDisplayItemList(
        &self,
        c: &LayerContext<'_, B>,
    ) -> Option<LiveFieldRead<RecordingRead<'_, B>, DisplayList<B>>> {
        let recording = self.GetRecordingSourceForTesting(c);
        if recording.display_list().is_none() {
            return None;
        }
        Some(LiveFieldRead::new(recording, |r| r.display_list().unwrap()))
    }
}
impl<B: PictureLayerTypes> LayerObject<B> for PictureLayer<B> {
    fn base(&self) -> &Layer<B> {
        &self.base_
    }
    fn AsPictureLayer(&self) -> Option<&dyn PictureLayerObject<B>> {
        Some(self)
    }
    fn NativeShutdown(&self, c: &mut LayerContext<'_, B>) {
        self.DestroyMembers(c);
        self.base_.Shutdown(c);
    }
    // Source: cc/layers/picture_layer.h:37-38;
    //         cc/layers/picture_layer.cc:38-41.
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        tree_context: &mut LayerTreeContext<'_, B>,
        _c: &mut LayerContext<'_, B>,
    ) -> Box<B::Layer> {
        B::Layer::CreatePicture(tree, self.base_.id(), tree_context)
    }
    // Source: cc/layers/picture_layer.h:64-66;
    //         cc/layers/picture_layer.cc:43-74.
    fn PushDirtyPropertiesTo(
        &self,
        target: &mut B::Layer,
        dirty: u8,
        commit: &mut CommitState<B>,
        c: &mut LayerContext<'_, B>,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.base_
            .PushDirtyPropertiesTo(target, dirty, commit, c, impl_tree);
        if dirty & Layer::<B>::kChangedGeneralProperty != 0 {
            let _trace = c.DerivedServices().BeginPushPropertiesTrace();
            self.DropRecordingSourceContentIfInvalid(impl_tree.source_frame_number(), c);
            if commit
                .layer_update_rects
                .get(&self.base_.id())
                .is_some_and(|rect| !rect.IsEmpty())
            {
                target.set_has_non_animated_image_update_rect();
            }
            target.set_gpu_raster_max_texture_size(commit.device_viewport_rect.size(), impl_tree);
            target.SetIsBackdropFilterMask(self.is_backdrop_filter_mask(), impl_tree);
            let source = self.CreateRasterSource(c);
            let invalidation = Region::MoveFrom(
                self.last_updated_invalidation_
                    .Write(&self.base_.Synchronization(c))
                    .as_mut()
                    .unwrap(),
            );
            target.StageNewRasterSourceForCommit(source, invalidation);
            commit
                .picture_layer_ids_with_new_raster_source
                .insert(self.base_.id());
            target.set_should_batch_updated_tiles();
        }
        debug_assert!(self
            .last_updated_invalidation_
            .Read(&self.base_.Synchronization(c))
            .as_ref()
            .unwrap()
            .IsEmpty());
    }
    // Source: cc/layers/picture_layer.h:39;
    //         cc/layers/picture_layer.cc:80-92.
    fn SetLayerTreeHost(
        &self,
        allocation: &crate::cc::layers::layer::LayerBorrow<B>,
        host: Option<MainLayerHostId>,
        c: &mut LayerContext<'_, B>,
    ) {
        LayerObject::SetLayerTreeHost(&self.base_, allocation, host, c);
        let Some(host) = host else {
            return;
        };
        self.GetRecordingSourceForTestingMutable(c)
            .SetSlowdownRasterScaleFactor(
                c.Host(host).GetDebugState().slow_down_raster_scale_factor,
            );
        *self
            .update_source_frame_number_
            .Write(&self.base_.Synchronization(c)) = -1;
    }
    // Source: cc/layers/picture_layer.h:40;
    //         cc/layers/picture_layer.cc:94-98.
    fn SetNeedsDisplayRect(&self, rect: &Rect, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.base_.IsPropertyChangeAllowed(c));
        self.GetRecordingSourceForTestingMutable(c)
            .SetNeedsDisplayRect(*rect);
        self.base_.SetNeedsDisplayRect(self, rect, c);
    }
    // Source: cc/layers/picture_layer.h:41;
    //         cc/layers/picture_layer.cc:109-114.
    fn RequiresSetNeedsDisplayOnHdrHeadroomChange(&self, c: &LayerContext<'_, B>) -> bool {
        if let Some(list) = self.GetDisplayItemList(c) {
            return list.content_color_usage() == ContentColorUsage::kHDR;
        }
        false
    }
    // Source: cc/layers/picture_layer.h:43;
    //         cc/layers/picture_layer.cc:116-150.
    fn Update(&self, c: &mut LayerContext<'_, B>) -> bool {
        let host = self.base_.host_id().expect("source attached host");
        *self
            .update_source_frame_number_
            .Write(&self.base_.Synchronization(c)) = c.Host(host).SourceFrameNumber();
        let mut updated = self.base_.Update(c);
        let mut recording = self.GetRecordingSourceForTestingMutable(c);
        recording.SetBackgroundColor(self.base_.SafeOpaqueBackgroundColor(c));
        recording.SetRequiresClear(
            !self.base_.contents_opaque(c)
                && !self.Client().read().unwrap().FillsBoundsCompletely(),
        );
        recording.SetCanUseRecordedBounds(c.Host(host).GetSettings().enable_hit_test_opaqueness);
        let frame = c.Host(host).SourceFrameNumber();
        let _trace = c.DerivedServices().BeginUpdateTrace(frame);
        let host_id = c.Host(host).GetId();
        let _task = c
            .DerivedServices()
            .BeginUpdateLayerTask(self.base_.id(), host_id);
        debug_assert!(self.client_.read().unwrap().is_some());
        let size = *self.base_.bounds(c);
        let scale = c.Host(host).recording_scale_factor();
        let client = self.Client();
        let mut client = client.write().unwrap();
        let mut invalidation = self
            .last_updated_invalidation_
            .Write(&self.base_.Synchronization(c));
        updated |= recording.Update(size, scale, &mut *client, invalidation.as_mut().unwrap(), c);
        if !updated {
            return false;
        }
        self.base_
            .SetNeedsPushProperties(Layer::<B>::kChangedGeneralProperty, c);
        self.base_.IncreasePaintCount(c);
        true
    }
    // Source: cc/layers/picture_layer.h:42;
    //         cc/layers/picture_layer.cc:152-172.
    fn GetPicture(&self, c: &mut LayerContext<'_, B>) -> Option<Arc<B::Picture>> {
        if !self.base_.draws_content(c) || self.base_.bounds(c).IsEmpty() {
            return None;
        }
        let list = self.Client().write().unwrap().PaintContentsToDisplayList(c);
        let mut recorder = c.DerivedServices().CreatePictureRecorder();
        let size = *self.base_.bounds(c);
        recorder.BeginRecording(size.width(), size.height());
        recorder.Clear(Color4f::default());
        let mut scrolls = crate::adapter::prioritized_tile::ScrollOffsetMap::new();
        let trees = c.Host(self.base_.host_id().unwrap()).property_trees();
        for element in list
            .as_ref()
            .expect("source display_list->raster_inducing_scrolls()")
            .raster_inducing_scrolls()
        {
            scrolls.insert(
                element,
                trees
                    .scroll_tree()
                    .current_scroll_offset(element, trees.state()),
            );
        }
        list.as_ref()
            .expect("source display_list->Raster()")
            .Raster(&mut recorder, &scrolls);
        Some(recorder.FinishRecordingAsPicture())
    }
    // Source: cc/layers/picture_layer.h:68;
    //         cc/layers/picture_layer.cc:179-181.
    fn HasDrawableContent(&self, c: &LayerContext<'_, B>) -> bool {
        self.client_.read().unwrap().is_some() && self.base_.HasDrawableContent(c)
    }
    // Source: cc/layers/picture_layer.h:44;
    //         cc/layers/picture_layer.cc:192-194.
    fn RunMicroBenchmark(&self, benchmark: &mut B::MicroBenchmark, _c: &mut LayerContext<'_, B>) {
        benchmark.RunOnPictureLayer(self);
    }
    // Source: cc/layers/picture_layer.h:45-46;
    //         cc/layers/picture_layer.cc:196-236.
    fn CaptureContent(
        &self,
        rect: &Rect,
        content: &mut Vec<B::NodeInfo>,
        c: &mut LayerContext<'_, B>,
    ) {
        if !self.base_.draws_content(c) {
            return;
        }
        let Some(list) = self.GetDisplayItemList(c) else {
            return;
        };
        if self.base_.transform_tree_index(c) == kInvalidPropertyNodeId {
            return;
        }
        let mut inverse = B::Transform::default();
        if !PropertyTreeTransform::GetInverse(&self.base_.ScreenSpaceTransform(c), &mut inverse) {
            return;
        }
        let mut transformed = MathUtil::ProjectEnclosingClippedRect(&inverse, *rect);
        let bounds = *self.base_.bounds(c);
        transformed.Intersect(Rect::new(0, 0, bounds.width(), bounds.height()));
        if transformed.IsEmpty() {
            return;
        }
        list.CaptureContent(transformed, content);
        let host = self.base_.host_id().unwrap();
        if let Some(outer) = c
            .Host(host)
            .LayerByElementId(c.Host(host).OuterViewportScrollElementId())
        {
            if self.base_.transform_tree_index(c)
                == outer.WithSourceRef(c, |object, c| object.base().transform_tree_index(c))
            {
                return;
            }
            let mut inverse_outer = B::Transform::default();
            if !PropertyTreeTransform::GetInverse(
                &outer.WithSourceRef(c, |object, c| object.base().ScreenSpaceTransform(c)),
                &mut inverse_outer,
            ) {
                return;
            }
            let mut combined = self.base_.ScreenSpaceTransform(c);
            PropertyTreeTransform::MultiplyAssign(&mut combined, &inverse_outer);
            for node in content {
                node.set_visual_rect(MathUtil::ProjectEnclosingClippedRect(
                    &combined,
                    node.visual_rect(),
                ));
            }
        }
    }
}
impl<B: PictureLayerTypes> PictureLayerObject<B> for PictureLayer<B> {
    fn picture(&self) -> &PictureLayer<B> {
        self
    }
    fn SetIsBackdropFilterMask(&self, value: bool, c: &mut LayerContext<'_, B>) {
        PictureLayer::SetIsBackdropFilterMask(self, value, c);
    }
}

impl<B: PictureLayerTypes> crate::adapter::recording_source::RecordingSourceContext<B::Recording>
    for LayerContext<'_, B>
{
    type Services = B::DerivedServices;
    fn RecordingServices(&mut self) -> &Self::Services {
        self.DerivedServices()
    }
}
