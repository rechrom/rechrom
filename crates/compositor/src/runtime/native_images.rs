//! Native image-decode owners used by the canonical TileManager.
//!
//! Source algorithms stay in the three owner types below. Decode storage and
//! worker execution are injected through `NativeImageDecodeCache`; the service
//! retains no Page, Host, Layer, TileManager, or raster-resource owner.
use crate::{
    adapter::{
        layer_tree_host_impl::HostImageDecodingMode,
        tile_manager::{
            CheckerImageTracker, CheckerImageTrackerId, DecodeQueueItem, DecodeType,
            DecodedImageTracker, DecodedImageTrackerId, ImageController, ImageControllerId,
            ImageDecodeCacheId, ImageTracingInfo, TileManagerDrawImage, TileManagerPaintImage,
            TileManagerTypes,
        },
    },
    cc::{
        raster::tile_task::TileTaskRef,
        tiles::{tile_manager::RasterCompletion, tile_priority::WhichTree},
    },
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
};

// Source: cc/tiles/image_decode_cache.h TaskResult.
pub(crate) struct NativeImageTaskResult<B: TileManagerTypes> {
    pub task: Option<TileTaskRef<RasterCompletion<B>>>,
    pub need_unref: bool,
    pub is_at_raster_decode: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeImageDecodeResult {
    Success,
    DecodeNotRequired,
    Failure,
}

/// Platform/cache half of Chromium's ImageDecodeCache boundary.
pub(crate) trait NativeImageDecodeCache<B: TileManagerTypes>: Send + Sync {
    fn GetTaskForImageAndRef(
        &self,
        image: &B::DrawImage,
        tracing: ImageTracingInfo,
    ) -> NativeImageTaskResult<B>;
    fn UnrefImage(&self, image: &B::DrawImage);
    fn ReduceCacheUsage(&self);
    fn RecordCacheStats(&self);
    fn MaximumMemoryLimitBytes(&self) -> usize;
    fn QueueImageDecode(
        &self,
        image: B::DrawImage,
        speculative: bool,
        completion: Box<dyn FnOnce(u64, NativeImageDecodeResult) + Send>,
    ) -> u64;
    fn UnlockImageDecode(&self, request: u64);
    fn ExternalDependencyCompletedForTask(&self, task: TileTaskRef<RasterCompletion<B>>);
    fn FlushDecodeTasks(&self);
    /// Source playback lookup. The cache applies the draw's actual frame,
    /// scale and target color parameters and returns a scoped lease retaining
    /// decoded pixels (or the source record) for the duration of playback.
    fn GetRasterContent(
        &self,
        image: &B::DrawImage,
        use_gpu_raster_mode: bool,
    ) -> B::ScopedRasterResult;
}

pub(crate) struct NativeImageDecodeCaches<B: TileManagerTypes> {
    entries: Arc<RwLock<HashMap<ImageDecodeCacheId, Arc<dyn NativeImageDecodeCache<B>>>>>,
}
impl<B: TileManagerTypes> Clone for NativeImageDecodeCaches<B> {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
        }
    }
}
impl<B: TileManagerTypes> Default for NativeImageDecodeCaches<B> {
    fn default() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
impl<B: TileManagerTypes> NativeImageDecodeCaches<B> {
    pub fn Register(&self, id: ImageDecodeCacheId, cache: Arc<dyn NativeImageDecodeCache<B>>) {
        let old = self.entries.write().unwrap().insert(id, cache);
        assert!(old.is_none(), "image decode cache identity must be unique");
    }
    pub fn Unregister(&self, id: ImageDecodeCacheId) {
        self.entries.write().unwrap().remove(&id);
    }
    fn Resolve(&self, id: ImageDecodeCacheId) -> Arc<dyn NativeImageDecodeCache<B>> {
        Arc::clone(
            self.entries
                .read()
                .unwrap()
                .get(&id)
                .expect("TileManager image cache identity is not registered"),
        )
    }

    fn TryResolve(&self, id: ImageDecodeCacheId) -> Option<Arc<dyn NativeImageDecodeCache<B>>> {
        self.entries.read().unwrap().get(&id).cloned()
    }
}

struct CpuNativePlaybackImageProvider<R: crate::runtime::types::CpuSkiaBackend> {
    cache: Option<Arc<dyn NativeImageDecodeCache<crate::runtime::types::CpuCompositorTypes<R>>>>,
    target: crate::runtime::paint::TargetColorParams,
    settings: crate::adapter::tile_manager::PlaybackImageProviderSettings<
        crate::runtime::types::CpuCompositorTypes<R>,
    >,
}

impl<R: crate::runtime::types::CpuSkiaBackend> crate::runtime::paint::CpuImageProvider<R>
    for CpuNativePlaybackImageProvider<R>
{
    // Source: cc/tiles/playback_image_provider.cc GetRasterContent.
    fn GetRasterContent(
        &mut self,
        image: &crate::runtime::paint::CpuDrawImage,
    ) -> crate::runtime::paint::CpuScopedRasterResult<R> {
        use crate::adapter::tile_manager::{TileManagerDrawImage, TileManagerPaintImage};

        let image_id = <crate::runtime::paint::CpuDrawImage as TileManagerDrawImage<
            crate::runtime::types::CpuCompositorTypes<R>,
        >>::paint_image(image)
        .stable_id();
        if self.settings.images_to_skip.contains(&image_id) {
            return crate::runtime::paint::CpuScopedRasterResult::Unavailable;
        }

        let frame_index = self
            .settings
            .image_to_current_frame_index
            .get(&image_id)
            .copied()
            .unwrap_or(crate::adapter::image_animation::kDefaultFrameIndex);
        let raster_image = <crate::runtime::paint::CpuDrawImage as TileManagerDrawImage<
            crate::runtime::types::CpuCompositorTypes<R>,
        >>::WithRasterParameters(image, 1.0, frame_index, &self.target);

        // A missing cache is a real missing embedding capability. Do not
        // fabricate decoded pixels or a placeholder record.
        let Some(cache) = self.cache.as_ref() else {
            return crate::runtime::paint::CpuScopedRasterResult::Unavailable;
        };
        cache.GetRasterContent(&raster_image, self.settings.use_gpu_raster_mode)
    }
}

struct CpuNativeWorkletProvider<R: crate::runtime::types::CpuSkiaBackend> {
    records: crate::adapter::picture_layer_impl::PaintWorkletRecordMap<
        crate::runtime::paint::CpuWorkletInput,
        crate::runtime::paint::CpuPaintRecord<R>,
    >,
}

impl<R: crate::runtime::types::CpuSkiaBackend> crate::runtime::paint::CpuWorkletImageProvider<R>
    for CpuNativeWorkletProvider<R>
{
    // Source: cc/tiles/paint_worklet_image_provider.cc GetPaintRecordResult.
    fn GetPaintRecordResult(
        &mut self,
        input: &crate::runtime::paint::CpuWorkletInput,
    ) -> crate::runtime::paint::CpuScopedRasterResult<R> {
        for index in 0..self.records.len() {
            let key = self.records.input_at(index);
            if Arc::ptr_eq(&key.0, &input.0) {
                return match self.records.get(key).and_then(|(_, record)| record.clone()) {
                    Some(record) => crate::runtime::paint::CpuScopedRasterResult::Record(record),
                    None => crate::runtime::paint::CpuScopedRasterResult::Unavailable,
                };
            }
        }
        crate::runtime::paint::CpuScopedRasterResult::Unavailable
    }
}

struct CpuImageControllerState<B: TileManagerTypes> {
    caches: NativeImageDecodeCaches<B>,
    cache_id: Option<ImageDecodeCacheId>,
    cache: Option<Arc<dyn NativeImageDecodeCache<B>>>,
    predecode_locked_images: Vec<B::DrawImage>,
}
impl<B: TileManagerTypes> Drop for CpuImageControllerState<B> {
    fn drop(&mut self) {
        if let Some(cache) = self.cache.as_ref() {
            for image in self.predecode_locked_images.drain(..) {
                cache.UnrefImage(&image);
            }
        }
    }
}
struct CpuImageControllerHandle<B: TileManagerTypes>(Arc<Mutex<CpuImageControllerState<B>>>);
impl<B: TileManagerTypes> Clone for CpuImageControllerHandle<B> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
pub(crate) struct CpuImageController<B: TileManagerTypes>(CpuImageControllerHandle<B>);
impl<B: TileManagerTypes> CpuImageController<B> {
    fn new(caches: NativeImageDecodeCaches<B>) -> Self {
        Self(CpuImageControllerHandle(Arc::new(Mutex::new(
            CpuImageControllerState {
                caches,
                cache_id: None,
                cache: None,
                predecode_locked_images: Vec::new(),
            },
        ))))
    }
}
impl<B: TileManagerTypes> CpuImageControllerHandle<B> {
    fn Cache(&self) -> Arc<dyn NativeImageDecodeCache<B>> {
        Arc::clone(
            self.0
                .lock()
                .unwrap()
                .cache
                .as_ref()
                .expect("ImageController requires its source ImageDecodeCache"),
        )
    }
    fn MaximumMemoryLimitBytes(&self) -> usize {
        self.Cache().MaximumMemoryLimitBytes()
    }
    fn QueueImageDecode(
        &self,
        image: B::DrawImage,
        speculative: bool,
        completion: Box<dyn FnOnce(u64, NativeImageDecodeResult) + Send>,
    ) -> u64 {
        self.Cache()
            .QueueImageDecode(image, speculative, completion)
    }
    fn UnlockImageDecode(&self, request: u64) {
        self.Cache().UnlockImageDecode(request)
    }
}
impl<B: TileManagerTypes> ImageController<B> for CpuImageController<B> {
    // Source: cc/tiles/image_controller.cc:225-249.
    fn SetImageDecodeCache(&mut self, id: Option<ImageDecodeCacheId>) {
        let mut state = self.0 .0.lock().unwrap();
        if state.cache_id == id {
            return;
        }
        if let Some(cache) = state.cache.take() {
            for image in state.predecode_locked_images.drain(..) {
                cache.UnrefImage(&image);
            }
        }
        state.cache_id = id;
        state.cache = id.map(|id| state.caches.Resolve(id));
    }
    fn ExternalDependencyCompletedForTask(&mut self, task: TileTaskRef<RasterCompletion<B>>) {
        self.0.Cache().ExternalDependencyCompletedForTask(task)
    }
    // Source: cc/tiles/image_controller.cc:302-317.
    fn SetPredecodeImages(
        &mut self,
        images: &[B::DrawImage],
        tracing: ImageTracingInfo,
    ) -> Vec<TileTaskRef<RasterCompletion<B>>> {
        let mut images = images.to_vec();
        let mut tasks = Vec::new();
        let mut has_at_raster_images = false;
        self.ConvertImagesToTasks(&mut images, &mut tasks, &mut has_at_raster_images, tracing);
        let mut state = self.0 .0.lock().unwrap();
        let cache = Arc::clone(
            state
                .cache
                .as_ref()
                .expect("ImageController requires its source ImageDecodeCache"),
        );
        for image in state.predecode_locked_images.drain(..) {
            cache.UnrefImage(&image);
        }
        state.predecode_locked_images = images;
        tasks
    }
    fn ReduceMemoryUsage(&mut self) {
        self.0.Cache().ReduceCacheUsage()
    }
    // Source: cc/tiles/image_controller.cc:251-292. The input vector is
    // intentionally mutable: entries without an image ref are erased.
    fn ConvertImagesToTasks(
        &mut self,
        images: &mut Vec<B::DrawImage>,
        tasks: &mut Vec<TileTaskRef<RasterCompletion<B>>>,
        has_at_raster_images: &mut bool,
        tracing: ImageTracingInfo,
    ) {
        let cache = self.0.Cache();
        *has_at_raster_images = false;
        let mut index = 0;
        while index < images.len() {
            assert!(!images[index].paint_image().IsPaintWorklet());
            let result = cache.GetTaskForImageAndRef(&images[index], tracing);
            *has_at_raster_images |= result.is_at_raster_decode;
            if let Some(task) = result.task {
                tasks.push(task);
            }
            if result.need_unref {
                index += 1;
            } else {
                images.remove(index);
            }
        }
    }
    // Source: cc/tiles/image_controller.cc:294-300.
    fn UnrefImages(&mut self, images: &[B::DrawImage]) {
        let cache = self.0.Cache();
        for image in images {
            cache.UnrefImage(image);
        }
    }
    fn FlushDecodeTasksForTesting(&mut self) {
        self.0.Cache().FlushDecodeTasks()
    }
    fn cache(&self) -> Option<ImageDecodeCacheId> {
        self.0 .0.lock().unwrap().cache_id
    }
    fn RecordCacheStats(&self) {
        if let Some(cache) = self.0 .0.lock().unwrap().cache.as_ref() {
            cache.RecordCacheStats();
        }
    }
}

struct DecodedImageLock {
    request: u64,
    expiration_frame: i32,
    expiration_time: crate::adapter::time::TimeTicks,
}
struct CpuDecodedImageTrackerState<B: TileManagerTypes> {
    locked: BTreeMap<B::PaintImageId, DecodedImageLock>,
    sync_tree_frame_number: i32,
    controller: CpuImageControllerHandle<B>,
}
pub(crate) struct CpuDecodedImageTracker<B: TileManagerTypes> {
    state: Arc<Mutex<CpuDecodedImageTrackerState<B>>>,
    origin: Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
    timer_generation: Arc<AtomicU64>,
    alive: Arc<AtomicBool>,
}
impl<B: TileManagerTypes> CpuDecodedImageTracker<B> {
    const LOCK_COMMITS: i32 = 3;
    const LOCK_TIMEOUT: crate::adapter::time::TimeDelta =
        crate::adapter::time::TimeDelta::Milliseconds(4000);

    fn new(
        controller: CpuImageControllerHandle<B>,
        origin: Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(CpuDecodedImageTrackerState {
                locked: BTreeMap::new(),
                sync_tree_frame_number: -1,
                controller,
            })),
            origin,
            timer_generation: Arc::new(AtomicU64::new(0)),
            alive: Arc::new(AtomicBool::new(true)),
        }
    }

    fn Expire(
        state: &Arc<Mutex<CpuDecodedImageTrackerState<B>>>,
        now: crate::adapter::time::TimeTicks,
    ) -> crate::adapter::time::TimeDelta {
        let (controller, expired, next) = {
            let mut state = state.lock().expect("decoded image tracker lock poisoned");
            let frame = state.sync_tree_frame_number;
            let expired_ids: Vec<_> = state
                .locked
                .iter()
                .filter_map(|(id, lock)| {
                    (lock.expiration_frame < frame || lock.expiration_time <= now)
                        .then_some(id.clone())
                })
                .collect();
            let expired = expired_ids
                .into_iter()
                .filter_map(|id| state.locked.remove(&id).map(|lock| lock.request))
                .collect::<Vec<_>>();
            let next = state
                .locked
                .values()
                .map(|lock| lock.expiration_time - now)
                .min()
                .unwrap_or(crate::adapter::time::TimeDelta::Max());
            (state.controller.clone(), expired, next)
        };
        for request in expired {
            controller.UnlockImageDecode(request);
        }
        next
    }

    fn PostTimer(
        origin: Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
        state: Arc<Mutex<CpuDecodedImageTrackerState<B>>>,
        generation: Arc<AtomicU64>,
        alive: Arc<AtomicBool>,
        delay: crate::adapter::time::TimeDelta,
        expected: u64,
    ) {
        if delay == crate::adapter::time::TimeDelta::Max() {
            return;
        }
        let closure_origin = origin.clone();
        origin.PostDelayedTask(
            crate::adapter::task_runner::TaskLocation {
                function: "DecodedImageTracker::CheckForExpiredDecodes",
                file: "cc/tiles/decoded_image_tracker.cc",
                line: 92,
            },
            Box::new(move || {
                if !alive.load(Ordering::Acquire) || generation.load(Ordering::Acquire) != expected
                {
                    return;
                }
                let next = Self::Expire(&state, closure_origin.Now());
                if next != crate::adapter::time::TimeDelta::Max() {
                    Self::PostTimer(closure_origin, state, generation, alive, next, expected);
                }
            }),
            delay,
        );
    }

    fn RestartTimer(&self, delay: crate::adapter::time::TimeDelta) {
        let generation = self.timer_generation.fetch_add(1, Ordering::AcqRel) + 1;
        Self::PostTimer(
            self.origin.clone(),
            self.state.clone(),
            self.timer_generation.clone(),
            self.alive.clone(),
            delay,
            generation,
        );
    }

    fn UnlockAll(&self) {
        self.timer_generation.fetch_add(1, Ordering::AcqRel);
        let (controller, requests) = {
            let mut state = self
                .state
                .lock()
                .expect("decoded image tracker lock poisoned");
            let requests = std::mem::take(&mut state.locked)
                .into_values()
                .map(|lock| lock.request)
                .collect::<Vec<_>>();
            (state.controller.clone(), requests)
        };
        for request in requests {
            controller.UnlockImageDecode(request);
        }
    }
}
impl<B: TileManagerTypes> DecodedImageTracker<B> for CpuDecodedImageTracker<B> {
    // Source: cc/tiles/decoded_image_tracker.cc:57-74,111-136.
    fn QueueImageDecode(
        &mut self,
        image: &B::DrawImage,
        callback: Box<dyn FnOnce(bool) + Send>,
        speculative: bool,
    ) {
        let image_id = image.paint_image().stable_id();
        let state = self.state.clone();
        let origin = self.origin.clone();
        let generation = self.timer_generation.clone();
        let alive = self.alive.clone();
        let controller = state
            .lock()
            .expect("decoded image tracker lock poisoned")
            .controller
            .clone();
        let completion_controller = controller.clone();
        controller.QueueImageDecode(
            image.clone(),
            speculative,
            Box::new(move |request, result| {
                let delivery_origin = origin.clone();
                origin.PostDelayedTask(
                    crate::adapter::task_runner::TaskLocation {
                        function: "DecodedImageTracker::ImageDecodeFinished",
                        file: "cc/tiles/decoded_image_tracker.cc",
                        line: 111,
                    },
                    Box::new(move || {
                        if !alive.load(Ordering::Acquire) {
                            if result == NativeImageDecodeResult::Success {
                                completion_controller.UnlockImageDecode(request);
                            }
                            return;
                        }
                        if result == NativeImageDecodeResult::Success {
                            let now = delivery_origin.Now();
                            let (old, next_delay) = {
                                let mut state =
                                    state.lock().expect("decoded image tracker lock poisoned");
                                let expiration_frame = state
                                    .sync_tree_frame_number
                                    .saturating_add(Self::LOCK_COMMITS);
                                let old = state.locked.insert(
                                    image_id,
                                    DecodedImageLock {
                                        request,
                                        expiration_frame,
                                        expiration_time: now + Self::LOCK_TIMEOUT,
                                    },
                                );
                                let next = state
                                    .locked
                                    .values()
                                    .map(|lock| lock.expiration_time - now)
                                    .min()
                                    .unwrap_or(crate::adapter::time::TimeDelta::Max());
                                (old, next)
                            };
                            if let Some(old) = old {
                                completion_controller.UnlockImageDecode(old.request);
                            }
                            let expected = generation.fetch_add(1, Ordering::AcqRel) + 1;
                            Self::PostTimer(
                                delivery_origin.clone(),
                                state.clone(),
                                generation.clone(),
                                alive.clone(),
                                next_delay,
                                expected,
                            );
                        }
                        callback(matches!(
                            result,
                            NativeImageDecodeResult::Success
                                | NativeImageDecodeResult::DecodeNotRequired
                        ));
                    }),
                    crate::adapter::time::TimeDelta::Microseconds(0),
                );
            }),
        );
    }

    // Source: cc/tiles/decoded_image_tracker.cc:77-79.
    fn UnlockAllImages(&mut self) {
        self.UnlockAll()
    }

    // Source: cc/tiles/decoded_image_tracker.cc:82-87.
    fn OnImagesUsedInDraw(&mut self, images: &[B::DrawImage]) {
        let (controller, requests) = {
            let mut state = self
                .state
                .lock()
                .expect("decoded image tracker lock poisoned");
            let mut requests = Vec::new();
            for image in images {
                if let Some(lock) = state.locked.remove(&image.paint_image().stable_id()) {
                    requests.push(lock.request);
                }
            }
            (state.controller.clone(), requests)
        };
        for request in requests {
            controller.UnlockImageDecode(request);
        }
    }

    // Source: cc/tiles/decoded_image_tracker.cc:89-94.
    fn SetSyncTreeFrameNumber(&mut self, frame_number: i32) {
        let changed = {
            let mut state = self
                .state
                .lock()
                .expect("decoded image tracker lock poisoned");
            if state.sync_tree_frame_number == frame_number {
                false
            } else {
                state.sync_tree_frame_number = frame_number;
                true
            }
        };
        if changed {
            let delay = Self::Expire(&self.state, self.origin.Now());
            self.RestartTimer(delay);
        }
    }
}
impl<B: TileManagerTypes> Drop for CpuDecodedImageTracker<B> {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Release);
        self.UnlockAll();
    }
}

pub(crate) trait NativeCheckerImageClient: Send + Sync {
    fn NeedsInvalidationForCheckerImagedTiles(&self);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DecodePolicy {
    Async,
    Sync,
}
struct CheckerDecodeState<D> {
    policy: DecodePolicy,
    draw: Option<D>,
}
struct CheckerCompletion<I> {
    image: I,
    request: u64,
    result: NativeImageDecodeResult,
}
pub(crate) struct CpuCheckerImageTracker<B: TileManagerTypes> {
    controller: CpuImageControllerHandle<B>,
    client: Arc<dyn NativeCheckerImageClient>,
    enable_checker_imaging: bool,
    min_image_bytes_to_checker: usize,
    force_disabled: bool,
    images_pending_invalidation: BTreeSet<B::PaintImageId>,
    invalidated_images_on_current_sync_tree: BTreeSet<B::PaintImageId>,
    image_decode_queue: Vec<DecodeQueueItem<B::PaintImage>>,
    decode_priority_allowed: Option<DecodeType>,
    outstanding_image_decode: Option<B::PaintImageId>,
    image_async_decode_state: BTreeMap<B::PaintImageId, CheckerDecodeState<B::DrawImage>>,
    image_id_to_decode: BTreeMap<B::PaintImageId, u64>,
    decoding_mode_map: BTreeMap<B::PaintImageId, HostImageDecodingMode>,
    completions: Arc<Mutex<Vec<CheckerCompletion<B::PaintImageId>>>>,
}
impl<B: TileManagerTypes> CpuCheckerImageTracker<B> {
    pub fn UpdateImageDecodingHints(
        &mut self,
        modes: BTreeMap<B::PaintImageId, HostImageDecodingMode>,
    ) {
        // Source: cc/tiles/checker_image_tracker.cc:446-484.
        if !self.enable_checker_imaging {
            return;
        }
        for (id, mode) in modes {
            if mode == HostImageDecodingMode::kSync {
                if let Some(state) = self.image_async_decode_state.get_mut(&id) {
                    if state.policy == DecodePolicy::Async {
                        state.policy = DecodePolicy::Sync;
                        self.images_pending_invalidation.insert(id.clone());
                    }
                }
            }
            self.decoding_mode_map
                .entry(id)
                .and_modify(|old| {
                    *old = if *old == mode {
                        mode
                    } else if *old == HostImageDecodingMode::kSync
                        || mode == HostImageDecodingMode::kSync
                    {
                        HostImageDecodingMode::kSync
                    } else if *old == HostImageDecodingMode::kUnspecified
                        || mode == HostImageDecodingMode::kUnspecified
                    {
                        HostImageDecodingMode::kUnspecified
                    } else {
                        HostImageDecodingMode::kAsync
                    };
                })
                .or_insert(mode);
        }
    }
    fn DrainCompletions(&mut self) {
        let completions = std::mem::take(&mut *self.completions.lock().unwrap());
        for completion in completions {
            if self.outstanding_image_decode.as_ref() != Some(&completion.image) {
                continue;
            }
            self.outstanding_image_decode = None;
            let Some(state) = self.image_async_decode_state.get_mut(&completion.image) else {
                self.controller.UnlockImageDecode(completion.request);
                continue;
            };
            if state.policy == DecodePolicy::Sync {
                self.controller.UnlockImageDecode(completion.request);
                continue;
            }
            if completion.result == NativeImageDecodeResult::Success {
                state.policy = DecodePolicy::Sync;
                self.image_id_to_decode
                    .insert(completion.image.clone(), completion.request);
                self.images_pending_invalidation.insert(completion.image);
                self.client.NeedsInvalidationForCheckerImagedTiles();
            } else {
                self.controller.UnlockImageDecode(completion.request);
            }
        }
        self.ScheduleNextImageDecode();
    }
    fn ScheduleNextImageDecode(&mut self) {
        if self.outstanding_image_decode.is_some() {
            return;
        }
        let Some(allowed) = self.decode_priority_allowed else {
            return;
        };
        while self
            .image_decode_queue
            .first()
            .is_some_and(|request| request.decode_type as u8 <= allowed as u8)
        {
            let request = self.image_decode_queue.remove(0);
            let id = request.image.stable_id();
            let Some(state) = self.image_async_decode_state.get(&id) else {
                continue;
            };
            if state.policy != DecodePolicy::Async {
                continue;
            }
            let draw = state
                .draw
                .clone()
                .expect("async checker state retains its source draw image");
            self.outstanding_image_decode = Some(id.clone());
            let completions = Arc::clone(&self.completions);
            let callback_id = id.clone();
            self.controller.QueueImageDecode(
                draw,
                false,
                Box::new(move |request, result| {
                    completions.lock().unwrap().push(CheckerCompletion {
                        image: callback_id,
                        request,
                        result,
                    });
                }),
            );
            break;
        }
    }
}
impl<B: TileManagerTypes> Drop for CpuCheckerImageTracker<B> {
    fn drop(&mut self) {
        for (_, request) in std::mem::take(&mut self.image_id_to_decode) {
            self.controller.UnlockImageDecode(request);
        }
    }
}
impl<B: TileManagerTypes> CheckerImageTracker<B> for CpuCheckerImageTracker<B> {
    // Source: cc/tiles/checker_image_tracker.cc:161-194.
    fn ClearTracker(&mut self, clear_policy: bool) {
        self.DrainCompletions();
        for (_, request) in std::mem::take(&mut self.image_id_to_decode) {
            self.controller.UnlockImageDecode(request);
        }
        if clear_policy {
            self.decoding_mode_map.clear();
            self.image_async_decode_state.clear();
            self.image_decode_queue.clear();
        } else {
            for id in &self.images_pending_invalidation {
                if let Some(state) = self.image_async_decode_state.get_mut(id) {
                    state.policy = DecodePolicy::Async;
                }
            }
        }
        self.images_pending_invalidation.clear();
    }
    fn SetNoDecodesAllowed(&mut self) {
        self.decode_priority_allowed = None;
    }
    // Source: cc/tiles/checker_image_tracker.cc:235-305.
    fn ShouldCheckerImage(&mut self, image: &B::DrawImage, tree: WhichTree) -> bool {
        self.DrainCompletions();
        let paint = image.paint_image();
        let id = paint.stable_id();
        if !self.enable_checker_imaging || !paint.IsLazyGenerated() {
            return false;
        }
        if tree == WhichTree::ACTIVE_TREE
            && self.invalidated_images_on_current_sync_tree.contains(&id)
        {
            return true;
        }
        if self.images_pending_invalidation.contains(&id) {
            return true;
        }
        if self.decoding_mode_map.get(&id) != Some(&HostImageDecodingMode::kAsync) {
            return false;
        }
        let max_bytes = self.controller.MaximumMemoryLimitBytes();
        let can_checker = !paint.ShouldAnimate()
            && !paint.IsMultipart()
            && paint.IsComplete()
            && image.EstimatedDecodedBytes() >= self.min_image_bytes_to_checker
            && image.EstimatedDecodedBytes() <= max_bytes
            && !self.force_disabled;
        let state = self
            .image_async_decode_state
            .entry(id)
            .or_insert_with(|| CheckerDecodeState {
                policy: if can_checker {
                    DecodePolicy::Async
                } else {
                    DecodePolicy::Sync
                },
                draw: Some(image.clone()),
            });
        state.draw = Some(image.clone());
        state.policy == DecodePolicy::Async
    }
    fn ScheduleImageDecodeQueue(&mut self, queue: Vec<DecodeQueueItem<B::PaintImage>>) {
        self.DrainCompletions();
        debug_assert!(queue
            .windows(2)
            .all(|pair| { pair[0].decode_type as u8 <= pair[1].decode_type as u8 }));
        self.image_decode_queue = queue;
        self.ScheduleNextImageDecode();
    }
    fn SetMaxDecodePriorityAllowed(&mut self, decode_type: DecodeType) {
        self.DrainCompletions();
        debug_assert!(self
            .decode_priority_allowed
            .is_none_or(|old| old as u8 <= decode_type as u8));
        self.decode_priority_allowed = Some(decode_type);
        self.ScheduleNextImageDecode();
    }
    fn TakeImagesToInvalidateOnSyncTree(&mut self) -> &BTreeSet<B::PaintImageId> {
        self.DrainCompletions();
        assert!(self.invalidated_images_on_current_sync_tree.is_empty());
        std::mem::swap(
            &mut self.invalidated_images_on_current_sync_tree,
            &mut self.images_pending_invalidation,
        );
        &self.invalidated_images_on_current_sync_tree
    }
    fn DidActivateSyncTree(&mut self) {
        self.DrainCompletions();
        for id in std::mem::take(&mut self.invalidated_images_on_current_sync_tree) {
            if let Some(request) = self.image_id_to_decode.remove(&id) {
                self.controller.UnlockImageDecode(request);
            }
            self.image_async_decode_state.remove(&id);
        }
    }
    // Source: cc/tiles/checker_image_tracker.cc:224-227. Inserting a
    // synchronous DecodeState before raster prevents the later source draw
    // from being admitted to checker imaging.
    fn DisallowCheckeringForImage(&mut self, image: &B::PaintImage) {
        self.image_async_decode_state
            .entry(image.stable_id())
            .or_insert(CheckerDecodeState {
                policy: DecodePolicy::Sync,
                draw: None,
            });
    }
    fn set_force_disabled(&mut self, value: bool) {
        self.force_disabled = value;
    }
}

pub(crate) struct CpuNativeImageServices<B: TileManagerTypes> {
    caches: NativeImageDecodeCaches<B>,
    client: Arc<dyn NativeCheckerImageClient>,
    next: u64,
    controllers: HashMap<ImageControllerId, CpuImageController<B>>,
    decoded: HashMap<DecodedImageTrackerId, CpuDecodedImageTracker<B>>,
    checkers: HashMap<CheckerImageTrackerId, CpuCheckerImageTracker<B>>,
}
impl<B: TileManagerTypes> CpuNativeImageServices<B> {
    pub fn new(
        caches: NativeImageDecodeCaches<B>,
        client: Arc<dyn NativeCheckerImageClient>,
    ) -> Self {
        Self {
            caches,
            client,
            next: 1,
            controllers: HashMap::new(),
            decoded: HashMap::new(),
            checkers: HashMap::new(),
        }
    }
    fn Next(&mut self) -> u64 {
        let id = self.next;
        self.next = self
            .next
            .checked_add(1)
            .expect("native image owner id overflow");
        id
    }
    pub fn CreateImageController(&mut self) -> ImageControllerId {
        let id = ImageControllerId(self.Next());
        self.controllers
            .insert(id, CpuImageController::new(self.caches.clone()));
        id
    }
    pub fn CreateDecodedImageTracker(
        &mut self,
        controller: ImageControllerId,
        origin: Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
    ) -> DecodedImageTrackerId {
        let controller = self.controllers[&controller].0.clone();
        let id = DecodedImageTrackerId(self.Next());
        self.decoded
            .insert(id, CpuDecodedImageTracker::new(controller, origin));
        id
    }
    pub fn CreateCheckerImageTracker(
        &mut self,
        controller: ImageControllerId,
        enable: bool,
        min_bytes: usize,
    ) -> CheckerImageTrackerId {
        let controller = self.controllers[&controller].0.clone();
        let id = CheckerImageTrackerId(self.Next());
        self.checkers.insert(
            id,
            CpuCheckerImageTracker {
                controller,
                client: Arc::clone(&self.client),
                enable_checker_imaging: enable,
                min_image_bytes_to_checker: min_bytes,
                force_disabled: false,
                images_pending_invalidation: BTreeSet::new(),
                invalidated_images_on_current_sync_tree: BTreeSet::new(),
                image_decode_queue: Vec::new(),
                decode_priority_allowed: None,
                outstanding_image_decode: None,
                image_async_decode_state: BTreeMap::new(),
                image_id_to_decode: BTreeMap::new(),
                decoding_mode_map: BTreeMap::new(),
                completions: Arc::new(Mutex::new(Vec::new())),
            },
        );
        id
    }
    pub fn ImageController(&mut self, id: ImageControllerId) -> &mut dyn ImageController<B> {
        self.controllers
            .get_mut(&id)
            .expect("live ImageController identity")
    }
    pub fn DecodedImageTracker(
        &mut self,
        id: DecodedImageTrackerId,
    ) -> &mut dyn DecodedImageTracker<B> {
        self.decoded
            .get_mut(&id)
            .expect("live DecodedImageTracker identity")
    }
    pub fn CheckerImageTracker(
        &mut self,
        id: CheckerImageTrackerId,
    ) -> &mut dyn CheckerImageTracker<B> {
        self.checkers
            .get_mut(&id)
            .expect("live CheckerImageTracker identity")
    }
    pub fn UpdateCheckerImageDecodingHints(
        &mut self,
        id: CheckerImageTrackerId,
        modes: BTreeMap<B::PaintImageId, HostImageDecodingMode>,
    ) {
        self.checkers
            .get_mut(&id)
            .expect("live CheckerImageTracker identity")
            .UpdateImageDecodingHints(modes);
    }
    pub fn DestroyCheckerImageTracker(&mut self, id: CheckerImageTrackerId) {
        self.checkers.remove(&id);
    }
    pub fn DestroyDecodedImageTracker(&mut self, id: DecodedImageTrackerId) {
        self.decoded.remove(&id);
    }
    pub fn DestroyImageController(&mut self, id: ImageControllerId) {
        self.controllers.remove(&id);
    }
}

impl<R: crate::runtime::types::CpuSkiaBackend>
    CpuNativeImageServices<crate::runtime::types::CpuCompositorTypes<R>>
{
    pub(crate) fn CreatePlaybackImageProvider(
        &self,
        cache: Option<ImageDecodeCacheId>,
        target: &crate::runtime::paint::TargetColorParams,
        settings: crate::adapter::tile_manager::PlaybackImageProviderSettings<
            crate::runtime::types::CpuCompositorTypes<R>,
        >,
    ) -> crate::runtime::paint::CpuPlaybackImageProvider<R> {
        let cache = cache.and_then(|id| self.caches.TryResolve(id));
        crate::runtime::paint::CpuPlaybackImageProvider(Box::new(CpuNativePlaybackImageProvider {
            cache,
            target: target.clone(),
            settings,
        }))
    }

    pub(crate) fn CreatePaintWorkletImageProvider(
        &self,
        records: crate::adapter::picture_layer_impl::PaintWorkletRecordMap<
            crate::runtime::paint::CpuWorkletInput,
            crate::runtime::paint::CpuPaintRecord<R>,
        >,
    ) -> crate::runtime::paint::CpuWorkletProvider<R> {
        crate::runtime::paint::CpuWorkletProvider(Box::new(CpuNativeWorkletProvider { records }))
    }
}
