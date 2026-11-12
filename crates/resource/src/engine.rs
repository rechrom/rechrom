#![allow(non_snake_case)]

use crate::ResourceLoader;
use decode::{DecodeEffect, DecodeEngine, DecodedImageResource};
use image_resource::{ContainerKey, DocumentImageMutation, ImageId};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    rc::Rc,
    sync::Arc,
    time::Instant,
};
use url_loader::{ReferrerPolicy, URLLoader, URLRequest, URLStreamOperation, URLWakeCallback};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ResourceRequestKey {
    url: String,
    referrer: String,
    referrer_policy: Option<ReferrerPolicy>,
    destination: u8,
    method: String,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    fetch_identity: Option<(
        url_loader::FetchMode,
        url_loader::FetchCredentialsMode,
        url_loader::FetchOrigin,
    )>,
}

impl From<&URLRequest> for ResourceRequestKey {
    fn from(request: &URLRequest) -> Self {
        Self {
            url: request.url.clone(),
            referrer: request.referrer.clone(),
            referrer_policy: request.referrer_policy,
            destination: request.destination as u8,
            method: request.method.clone(),
            body: request.body.clone(),
            headers: request.headers.clone(),
            fetch_identity: request.fetch.as_ref().map(|fetch| {
                (
                    fetch.mode,
                    fetch.credentials,
                    fetch.requestor_origin.clone(),
                )
            }),
        }
    }
}

/// One document/open operation's resource loader. The transport and decoders
/// may be shared services, but URL/referrer, cancellation and resource identity
/// belong to this engine instance. The engine intentionally has no DOM pointer:
/// discovery and application belong to DocumentEngine and Page.
pub struct ResourceEngine {
    pub(crate) loader: Rc<RefCell<dyn URLLoader>>,
    decode: Rc<DecodeEngine>,
    document_url: RefCell<String>,
    document_referrer_policy: Rc<Cell<ReferrerPolicy>>,
    pub(crate) active: Cell<bool>,
    pub(crate) context_detached: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) fetcher_loaders: Rc<crate::fetcher_lifecycle::FetcherLoaderSets>,
    pub(crate) fetcher_task_runner: RefCell<Option<Rc<dyn crate::FetcherTaskRunner>>>,
    pub(crate) is_in_request_resource: Rc<Cell<bool>>,
    pub(crate) xmlhttp_requests: Rc<crate::XMLHttpRequestRegistry>,
    pub(crate) wake_callback: RefCell<Option<URLWakeCallback>>,
    next_image_id: Rc<Cell<ImageId>>,
    platform_images: Rc<
        RefCell<
            HashMap<
                image_resource::platform_resources::PlatformImageVariant,
                image_resource::platform_resources::PlatformDecodedImageHandle,
            >,
        >,
    >,
    pub(crate) raw_loads: RefCell<HashMap<u64, crate::raw_resource::RawLoad>>,
    pub(crate) text_track_resources:
        RefCell<HashMap<u64, Rc<crate::text_track_resource::TextTrackResource>>>,
    pub(crate) raw_inflight_bytes: Cell<u32>,
    pub(crate) raw_metrics: Arc<url_loader::FetchKeepaliveMetrics>,
    pub(crate) raw_observer: RefCell<Option<Rc<dyn crate::raw_resource::RawResourceObserver>>>,
    loads: RefCell<
        HashMap<ResourceRequestKey, Rc<RefCell<crate::resource_loader::SharedResourceLoad>>>,
    >,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ResourceMemoryDiagnostics {
    pub cached_loads: usize,
    pub loading: usize,
    pub encoded_response_capacity: usize,
    pub raw_loading: usize,
    pub keepalive_inflight_bytes: u32,
}
impl ResourceEngine {
    pub fn MemoryDiagnostics(&self) -> ResourceMemoryDiagnostics {
        let loads = self.loads.borrow();
        let mut usage = ResourceMemoryDiagnostics {
            cached_loads: loads.len(),
            raw_loading: self.raw_loads.borrow().len(),
            keepalive_inflight_bytes: self.raw_inflight_bytes.get(),
            ..Default::default()
        };
        for load in loads.values() {
            let (loading, capacity) = load.borrow().MemoryUsage();
            usage.loading += usize::from(loading);
            usage.encoded_response_capacity =
                usage.encoded_response_capacity.saturating_add(capacity);
        }
        usage
    }
    pub fn new(loader: Rc<RefCell<dyn URLLoader>>, document_url: String) -> Self {
        let in_request_resource = Rc::new(Cell::new(false));
        Self {
            loader,
            decode: Rc::new(DecodeEngine::new()),
            document_url: RefCell::new(document_url),
            document_referrer_policy: Rc::new(Cell::new(ReferrerPolicy::default())),
            active: Cell::new(true),
            context_detached: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            fetcher_loaders: Rc::new(Default::default()),
            fetcher_task_runner: RefCell::new(None),
            is_in_request_resource: in_request_resource.clone(),
            xmlhttp_requests: Rc::new(crate::XMLHttpRequestRegistry::new(in_request_resource)),
            wake_callback: RefCell::new(None),
            next_image_id: Rc::new(Cell::new(1)),
            platform_images: Rc::new(RefCell::new(HashMap::new())),
            raw_loads: RefCell::new(HashMap::new()),
            text_track_resources: RefCell::new(HashMap::new()),
            raw_inflight_bytes: Cell::new(0),
            raw_metrics: Arc::new(Default::default()),
            raw_observer: RefCell::new(None),
            loads: RefCell::new(HashMap::new()),
        }
    }

    pub fn WithDecodeEngine(
        loader: Rc<RefCell<dyn URLLoader>>,
        decode: Rc<DecodeEngine>,
        document_url: String,
    ) -> Self {
        let in_request_resource = Rc::new(Cell::new(false));
        Self {
            loader,
            decode,
            document_url: RefCell::new(document_url),
            document_referrer_policy: Rc::new(Cell::new(ReferrerPolicy::default())),
            active: Cell::new(true),
            context_detached: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            fetcher_loaders: Rc::new(Default::default()),
            fetcher_task_runner: RefCell::new(None),
            is_in_request_resource: in_request_resource.clone(),
            xmlhttp_requests: Rc::new(crate::XMLHttpRequestRegistry::new(in_request_resource)),
            wake_callback: RefCell::new(None),
            next_image_id: Rc::new(Cell::new(1)),
            platform_images: Rc::new(RefCell::new(HashMap::new())),
            raw_loads: RefCell::new(HashMap::new()),
            text_track_resources: RefCell::new(HashMap::new()),
            raw_inflight_bytes: Cell::new(0),
            raw_metrics: Arc::new(Default::default()),
            raw_observer: RefCell::new(None),
            loads: RefCell::new(HashMap::new()),
        }
    }

    pub fn XMLHttpRequestRegistry(&self) -> Rc<crate::XMLHttpRequestRegistry> {
        self.xmlhttp_requests.clone()
    }
    pub fn DocumentURL(&self) -> String {
        self.document_url.borrow().clone()
    }

    /// A DocumentLoader replacement keeps the same transport/decoder services
    /// while creating a new Document fetch context. In-flight handles, policy,
    /// URL/referrer and cancellation remain attached to their original engine.
    /// Image ids and immutable platform images belong to the surviving Page's
    /// resource namespace, including frames retained by its compositor.
    pub fn ForDocument(&self, document_url: String) -> Rc<Self> {
        let in_request_resource = Rc::new(Cell::new(false));
        Rc::new(Self {
            loader: self.loader.clone(),
            decode: self.decode.clone(),
            document_url: RefCell::new(document_url),
            document_referrer_policy: Rc::new(Cell::new(ReferrerPolicy::default())),
            active: Cell::new(true),
            context_detached: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            fetcher_loaders: Rc::new(Default::default()),
            fetcher_task_runner: RefCell::new(self.fetcher_task_runner.borrow().clone()),
            is_in_request_resource: in_request_resource.clone(),
            xmlhttp_requests: Rc::new(crate::XMLHttpRequestRegistry::new(in_request_resource)),
            wake_callback: RefCell::new(self.wake_callback.borrow().clone()),
            next_image_id: self.next_image_id.clone(),
            platform_images: self.platform_images.clone(),
            raw_loads: RefCell::new(HashMap::new()),
            text_track_resources: RefCell::new(HashMap::new()),
            raw_inflight_bytes: Cell::new(0),
            raw_metrics: self.raw_metrics.clone(),
            raw_observer: RefCell::new(self.raw_observer.borrow().clone()),
            loads: RefCell::new(HashMap::new()),
        })
    }

    /// The resource owner supplies the document origin; URL loading owns the
    /// parsed origin comparison used by image CORS provenance.
    pub fn IsSameOriginWithDocument(&self, url: &str) -> bool {
        url_loader::IsSameOrigin(&self.document_url.borrow(), url)
    }

    pub fn SetDocumentURL(&self, url: String) {
        *self.document_url.borrow_mut() = url;
    }

    pub fn SetReferrerPolicy(&self, policy: ReferrerPolicy) {
        self.document_referrer_policy.set(policy);
    }

    pub fn ReferrerPolicyReadHandle(
        &self,
    ) -> foundation::referrer_policy::ReferrerPolicyReadHandle {
        foundation::referrer_policy::ReferrerPolicyReadHandle::FromOwner(
            &self.document_referrer_policy,
        )
    }
    pub fn ReferrerPolicy(&self) -> ReferrerPolicy {
        self.document_referrer_policy.get()
    }

    pub(crate) fn NormalizeRequest(&self, mut request: URLRequest) -> URLRequest {
        let policy = request
            .referrer_policy
            .unwrap_or_else(|| self.ReferrerPolicy());
        let source = if request.referrer.is_empty() {
            self.DocumentURL()
        } else {
            request.referrer.clone()
        };
        request.referrer = policy.GenerateReferrer(&source, &request.url);
        request.referrer_policy = Some(policy);
        // The explicit source/policy pair owns Referer. A caller-provided
        // header must not bypass NoReferrer or acquire a different cache key.
        request
            .headers
            .retain(|(name, _)| !name.eq_ignore_ascii_case("referer"));
        request
    }

    pub fn GetDecodeEngine(&self) -> &DecodeEngine {
        self.decode.as_ref()
    }

    fn AllocateImageId(&self) -> io::Result<ImageId> {
        let id = self.next_image_id.get();
        if id == 0 {
            return Err(io::Error::other("image resource id space exhausted"));
        }
        self.next_image_id.set(id.wrapping_add(1));
        Ok(id)
    }

    pub fn ObserveImageId(&self, id: ImageId) {
        let Some(next) = id.checked_add(1) else {
            self.next_image_id.set(0);
            return;
        };
        if self.next_image_id.get() != 0 {
            self.next_image_id.set(self.next_image_id.get().max(next));
        }
    }

    pub fn Start(&self, request: URLRequest) -> ResourceLoader {
        self.StartWithLoadEventPolicy(request, crate::ResourceLoadEventPolicy::Regular)
    }
    pub fn StartWithLoadEventPolicy(
        &self,
        request: URLRequest,
        policy: crate::ResourceLoadEventPolicy,
    ) -> ResourceLoader {
        let _request = crate::fetcher_lifecycle::RequestResourceScope::new(
            self.is_in_request_resource.clone(),
        );

        if request.fetch.as_ref().is_some_and(|fetch| {
            fetch.keepalive
                || !matches!(
                    (request.destination, fetch.context),
                    (
                        url_loader::RequestDestination::kImage,
                        foundation::content_security_policy::CSPRequestContext::Image
                            | foundation::content_security_policy::CSPRequestContext::ImageSet
                    ) | (
                        url_loader::RequestDestination::kFont,
                        foundation::content_security_policy::CSPRequestContext::Font
                    )
                )
        }) {
            return ResourceLoader::Failed(
                "typed RawResource FetchParameters require FetchRaw admission",
            );
        }

        if !self.active.get() {
            return ResourceLoader::Failed("resource engine is cancelled");
        }
        let request = self.NormalizeRequest(request);
        // Chromium's ResourceFetcher/MemoryCache coalesces identical in-flight
        // subresource loads. Keep that ownership here at the document resource
        // boundary: callers still receive independent handles, while one
        // transport operation and one immutable terminal response are shared.
        // Non-idempotent requests retain one operation per call.
        if request.method.eq_ignore_ascii_case("GET")
            && request.body.is_empty()
            && request.encoded_body.is_none()
        {
            let key = ResourceRequestKey::from(&request);
            if let Some(shared) = self.loads.borrow().get(&key).cloned() {
                let mut resource = ResourceLoader::FromShared(shared);
                resource.SetScheduling(&request);
                if policy.BlocksLoad(&request) {
                    resource.PromoteToBlocking();
                }
                browser_tracing::instant_with_text(
                    "resource",
                    "ResourceRequestReused",
                    &request.url,
                    &[
                        ("destination", request.destination as u8 as f64),
                        ("status", resource.Status() as u8 as f64),
                    ],
                );
                if let Some(callback) = self.wake_callback.borrow().clone() {
                    resource.SetWakeCallback(callback);
                }
                return resource;
            }
            let mut resource = ResourceLoader::Prepare(&request);
            if policy == crate::ResourceLoadEventPolicy::LinkPreload {
                resource.MarkLinkPreload();
            }
            self.fetcher_loaders.Add(
                &resource,
                if policy.BlocksLoad(&request) {
                    crate::fetcher_lifecycle::LoaderSet::Blocking
                } else {
                    crate::fetcher_lifecycle::LoaderSet::NonBlocking
                },
            );
            resource.StartPrepared(&mut *self.loader.borrow_mut(), &request);
            if let Some(callback) = self.wake_callback.borrow().clone() {
                resource.SetWakeCallback(callback);
            }
            self.loads.borrow_mut().insert(key, resource.Shared());
            return resource;
        }
        let mut resource = ResourceLoader::Prepare(&request);
        if policy == crate::ResourceLoadEventPolicy::LinkPreload {
            resource.MarkLinkPreload();
        }
        self.fetcher_loaders.Add(
            &resource,
            if policy.BlocksLoad(&request) {
                crate::fetcher_lifecycle::LoaderSet::Blocking
            } else {
                crate::fetcher_lifecycle::LoaderSet::NonBlocking
            },
        );
        resource.StartPrepared(&mut *self.loader.borrow_mut(), &request);
        if let Some(callback) = self.wake_callback.borrow().clone() {
            resource.SetWakeCallback(callback);
        }
        resource
    }

    /// MemoryCache::ResourceForURL(...)->IsLoaded() equivalent for a request
    /// owned by this document resource engine. This is intentionally a URL
    /// request query rather than a DOM/source-string cache.
    /// ResourceFetcher removes an errored image from MemoryCache, retaining
    /// existing clients' resource identity while allowing a fresh src retry.
    pub fn RemoveImageFromMemoryCache(&self, resource: &ResourceLoader) {
        if let Some(request) = resource.Request() {
            let key = ResourceRequestKey::from(request.as_ref());
            let mut loads = self.loads.borrow_mut();
            if loads
                .get(&key)
                .is_some_and(|cached| Rc::ptr_eq(cached, &resource.Shared()))
            {
                loads.remove(&key);
            }
        }
    }
    pub fn ReloadImage(&self, request: URLRequest) -> ResourceLoader {
        self.ReloadImageWithLoadEventPolicy(request, crate::ResourceLoadEventPolicy::Regular)
    }
    pub fn ReloadImageWithLoadEventPolicy(
        &self,
        request: URLRequest,
        policy: crate::ResourceLoadEventPolicy,
    ) -> ResourceLoader {
        let normalized = self.NormalizeRequest(request.clone());
        self.loads
            .borrow_mut()
            .remove(&ResourceRequestKey::from(&normalized));
        self.StartWithLoadEventPolicy(request, policy)
    }
    pub fn HasLoaded(&self, request: URLRequest) -> bool {
        let request = self.NormalizeRequest(request);
        let key = ResourceRequestKey::from(&request);
        self.loads.borrow().get(&key).is_some_and(|shared| {
            ResourceLoader::FromShared(shared.clone()).Status()
                == crate::resource_loader::ResourceLoadStatus::kComplete
        })
    }

    pub fn StartStream(&self, request: URLRequest) -> io::Result<Box<dyn URLStreamOperation>> {
        if request.fetch.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "typed RawResource FetchParameters require FetchRaw admission",
            ));
        }

        if !self.active.get() {
            return Err(io::Error::other("resource engine is cancelled"));
        }
        let request = self.NormalizeRequest(request);
        let mut stream = self.loader.borrow_mut().LoadStream(&request)?;
        if let Some(callback) = self.wake_callback.borrow().clone() {
            stream.SetWakeCallback(callback);
        }
        Ok(stream)
    }

    pub fn SetWakeCallback(&self, callback: URLWakeCallback) {
        *self.wake_callback.borrow_mut() = Some(callback.clone());
        self.decode.SetWakeCallback(Some(callback.clone()));
        let loads = self.loads.borrow().values().cloned().collect::<Vec<_>>();
        for shared in loads {
            ResourceLoader::FromShared(shared).SetWakeCallback(callback.clone());
        }
        for load in self.raw_loads.borrow_mut().values_mut() {
            load.loader.SetWakeCallback(callback.clone());
        }
        for resource in self.text_track_resources.borrow().values() {
            resource.SetWakeCallback(callback.clone());
        }
    }

    /// Queue another owner turn for already-admitted resource work.
    ///
    /// A transport normally wakes the owner when it becomes ready.  A single
    /// owner turn can deliberately consume only one such completion, though,
    /// while other completions from the same parallel fetch group are already
    /// queued.  Re-posting through the embedding callback preserves those
    /// task identities instead of relying on an unrelated frame or timer.
    pub fn RequestWake(&self) {
        let callback = self.wake_callback.borrow().clone();
        if let Some(callback) = callback {
            callback();
        }
    }

    pub fn BeginImageDecode(
        &self,
        bytes: Arc<[u8]>,
        mime_type: &str,
        container: &ContainerKey,
    ) -> io::Result<decode::ImageDecodeJob> {
        if !self.active.get() {
            return Err(io::Error::other("resource engine is cancelled"));
        }
        Ok(self
            .decode
            .BeginImageDecode(self.AllocateImageId()?, bytes, mime_type, container))
    }

    pub fn BeginVideoFrameDecode(
        &self,
        bytes: Arc<[u8]>,
        mime_type: &str,
    ) -> io::Result<decode::VideoDecodeJob> {
        if !self.active.get() {
            return Err(io::Error::other("resource engine is cancelled"));
        }
        Ok(self
            .decode
            .BeginVideoFrameDecode(self.AllocateImageId()?, bytes, mime_type))
    }

    pub fn DecodeImage(
        &self,
        bytes: Arc<[u8]>,
        mime_type: &str,
        container: &ContainerKey,
    ) -> io::Result<DecodedImageResource> {
        if !self.active.get() {
            return Err(io::Error::other("resource engine is cancelled"));
        }
        let resource_id = self.AllocateImageId()?;
        self.decode
            .DecodeImage(resource_id, bytes, mime_type, container)
    }

    /// Image::LoadPlatformResource, using the same concrete bitmap decoder and
    /// immutable PaintImage representation as network image resources.
    pub fn LoadPlatformImage(
        &self,
        variant: image_resource::platform_resources::PlatformImageVariant,
    ) -> io::Result<image_resource::platform_resources::PlatformDecodedImageHandle> {
        if let Some(image) = self.platform_images.borrow().get(&variant) {
            return Ok(image.clone());
        }
        let bytes: Arc<[u8]> = Arc::from(variant.EncodedBytes());
        let decoded = self.DecodeImage(
            bytes.clone(),
            "image/png",
            &ContainerKey::new(0, 0, 1.0, 1.0),
        )?;
        let DecodedImageResource::Bitmap {
            resource_id,
            image,
            encoded_data,
        } = decoded
        else {
            return Err(io::Error::other(
                "Blink platform PNG produced a document image",
            ));
        };
        if !image.animation_frames.is_empty() {
            return Err(io::Error::other("Blink broken-image PNG must be static"));
        }
        if let image_resource::PaintImageAnimationMetadata::Unavailable(error) = &image.animation {
            return Err(io::Error::new(io::ErrorKind::Unsupported, error.clone()));
        }

        let image = image_resource::PaintImage {
            id: resource_id,
            paint_image_id: image_resource::PaintImageId::allocate(),
            revision: 1,
            compositor_metadata: Ok(image_resource::PaintImageMetadata {
                animation: image.animation,
                completion_state: image_resource::PaintImageCompletionState::Done,
                is_multipart: false,
            }),
            width: image.width,
            height: image.height,
            orientation: image.orientation,
            hot_spot: image.hot_spot,
            alpha_type: image.alpha_type,
            is_cors_same_origin: true,
            is_lazy_generated: true,
            encoded_data: Some(encoded_data),
            content: image_resource::PaintImageContent::Bitmap(Arc::new(image.rgba8)),
            ..Default::default()
        };
        let handle =
            Arc::new(image_resource::platform_resources::PlatformDecodedImage { variant, image });
        self.platform_images
            .borrow_mut()
            .insert(variant, handle.clone());
        Ok(handle)
    }

    pub fn HasAnimatedImages(&self) -> bool {
        self.active.get() && self.decode.HasPendingBeginFrame()
    }
    pub fn ImageBitmapProvider(
        &self,
        resource_id: ImageId,
    ) -> Option<Rc<dyn image_resource::ImageBitmapProvider>> {
        self.decode.ImageBitmapProvider(resource_id)
    }

    pub fn AdvanceDocumentImages(
        &self,
        frame_time: Instant,
        begin_frame_sequence: u64,
    ) -> io::Result<Vec<DecodeEffect>> {
        if !self.active.get() {
            return Err(io::Error::other("resource engine is cancelled"));
        }
        self.decode
            .AdvanceTimelines(frame_time, begin_frame_sequence)
    }

    pub fn MutateDocumentImage(
        &self,
        resource_id: ImageId,
        mutation: DocumentImageMutation,
    ) -> io::Result<Vec<DecodeEffect>> {
        if !self.active.get() {
            return Err(io::Error::other("resource engine is cancelled"));
        }
        self.decode.MutateDocumentImage(resource_id, mutation)
    }

    pub fn Cancel(&self) {
        self.context_detached
            .store(true, std::sync::atomic::Ordering::Release);
        self.active.set(false);
        self.decode.SetWakeCallback(None);
        for shared in self.loads.borrow().values() {
            ResourceLoader::CancelShared(shared);
        }
        self.loads.borrow_mut().clear();
        self.DetachRawResources();
        // This abrupt context cancellation is distinct from StopFetching's
        // synchronous client-delivery path. Include uncached POST/other
        // loaders in the original native membership sets as well as cache.
        for set in [
            crate::fetcher_lifecycle::LoaderSet::NonBlocking,
            crate::fetcher_lifecycle::LoaderSet::Blocking,
        ] {
            for shared in self.fetcher_loaders.Snapshot(set) {
                ResourceLoader::CancelShared(&shared);
            }
        }
        self.CancelTextTrackResources();
        self.decode.ClearDocumentImages();
    }

    fn DetachRawResources(&self) {
        self.raw_observer.borrow_mut().take();
        for load in self.raw_loads.borrow_mut().values_mut() {
            if load.fetch.keepalive {
                let was_loading = load.loader.Status() == crate::ResourceLoadStatus::kLoading;
                // The native network owner detaches even when a caller still
                // holds a Resource handle. Timeout begins at disconnection.
                if let Err(error) = load.loader.DetachForKeepalive() {
                    browser_tracing::instant_with_text(
                        "resource",
                        "RawResourceDetachFailed",
                        &error.to_string(),
                        &[],
                    );
                    ResourceLoader::CancelShared(&load.loader.Shared());
                }
                if was_loading
                    && !load
                        .fetch
                        .failed
                        .swap(true, std::sync::atomic::Ordering::AcqRel)
                {
                    load.fetch.metrics.Record(
                        load.fetch.context,
                        url_loader::FetchKeepaliveRequestState::Failed,
                        false,
                    );
                }
                load.fetch
                    .context_detached
                    .store(true, std::sync::atomic::Ordering::Release);
            } else {
                ResourceLoader::CancelShared(&load.loader.Shared());
            }
        }
        self.raw_loads.borrow_mut().clear();
        self.raw_inflight_bytes.set(0);
    }
    pub fn IsActive(&self) -> bool {
        self.active.get()
    }
}

impl Drop for ResourceEngine {
    fn drop(&mut self) {
        self.DetachRawResources();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image_decoder::image_decoder::{DecodedImage, ImageDecodeInput, ImageDecoder};
    use image_resource::{CreatedDocumentImage, DocumentImageDecoder};
    use url_loader::{URLLoadOperation, URLResponse};

    struct Pending;
    impl URLLoadOperation for Pending {
        fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
            Ok(None)
        }
    }

    struct Loader(Rc<RefCell<Vec<URLRequest>>>);
    impl URLLoader for Loader {
        fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
            self.0.borrow_mut().push(request.clone());
            Ok(Box::new(Pending))
        }
    }

    struct Ready(Option<URLResponse>);
    impl URLLoadOperation for Ready {
        fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
            Ok(self.0.take())
        }
    }

    struct ReadyLoader(Rc<RefCell<Vec<URLRequest>>>);
    impl URLLoader for ReadyLoader {
        fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
            self.0.borrow_mut().push(request.clone());
            Ok(Box::new(Ready(Some(URLResponse {
                final_url: request.url.clone(),
                status_code: 200,
                body: b"shared".to_vec(),
                ..Default::default()
            }))))
        }
    }

    struct BitmapDecoder;
    impl ImageDecoder for BitmapDecoder {
        fn Decode(&mut self, _: &ImageDecodeInput<'_>) -> io::Result<DecodedImage> {
            Ok(DecodedImage {
                filename_extension: None,
                alpha_type: Default::default(),
                animation: Default::default(),
                width: 1,
                height: 1,
                orientation: Default::default(),
                hot_spot: None,
                animation_frames: Vec::new(),
                rgba8: vec![1, 2, 3, 4],
            })
        }
    }

    struct DocumentDecoder;
    impl DocumentImageDecoder for DocumentDecoder {
        fn can_decode(&self, _: &[u8], _: &str) -> bool {
            false
        }

        fn create(
            &mut self,
            _: ImageId,
            _: Arc<[u8]>,
            _: &str,
            _: &ContainerKey,
        ) -> io::Result<CreatedDocumentImage> {
            unreachable!("can_decode is false")
        }
    }

    #[test]
    fn document_engine_supplies_referrer_and_owns_image_identity() {
        let requests = Rc::new(RefCell::new(Vec::new()));
        let decode = Rc::new(DecodeEngine::WithDecoders(
            Rc::new(RefCell::new(BitmapDecoder)),
            Rc::new(RefCell::new(DocumentDecoder)),
        ));
        let engine = Rc::new(ResourceEngine::WithDecodeEngine(
            Rc::new(RefCell::new(Loader(requests.clone()))),
            decode,
            "https://example.test/page".into(),
        ));

        let _pending = engine.Start(URLRequest {
            url: "https://example.test/image.png".into(),
            ..Default::default()
        });
        assert_eq!(requests.borrow()[0].referrer, "https://example.test/page");

        let container = ContainerKey::new(1, 1, 1.0, 1.0);
        let first = engine
            .DecodeImage(Arc::from(&b"a"[..]), "image/png", &container)
            .unwrap();
        assert!(matches!(
            first,
            DecodedImageResource::Bitmap { resource_id: 1, .. }
        ));
        engine.ObserveImageId(8);
        let next = engine
            .DecodeImage(Arc::from(&b"b"[..]), "image/png", &container)
            .unwrap();
        assert!(matches!(
            next,
            DecodedImageResource::Bitmap { resource_id: 9, .. }
        ));

        engine.Cancel();
        assert!(!engine.IsActive());
        assert!(engine
            .DecodeImage(Arc::from(&b"c"[..]), "image/png", &container)
            .is_err());
    }

    #[test]
    fn identical_get_subresources_share_one_transport_operation() {
        let requests = Rc::new(RefCell::new(Vec::new()));
        let engine = ResourceEngine::new(
            Rc::new(RefCell::new(ReadyLoader(requests.clone()))),
            "https://example.test/page".into(),
        );
        let request = URLRequest {
            url: "https://example.test/shared.css".into(),
            destination: url_loader::RequestDestination::kStyleSheet,
            ..Default::default()
        };
        let mut first = engine.Start(request.clone());
        let mut second = engine.Start(request);

        assert_eq!(requests.borrow().len(), 1);
        assert!(first.Poll());
        assert!(second.Poll());
        assert_eq!(first.TakeResult().response.unwrap().body, b"shared");
        assert_eq!(second.TakeResult().response.unwrap().body, b"shared");
    }

    #[test]
    fn policy_normalization_is_shared_by_requests_cache_queries_and_streams() {
        let requests = Rc::new(RefCell::new(Vec::new()));
        let engine = ResourceEngine::new(
            Rc::new(RefCell::new(ReadyLoader(requests.clone()))),
            "https://user:secret@example.test/private?q=1#fragment".into(),
        );
        let request = URLRequest {
            url: "https://other.test/image.png".into(),
            destination: url_loader::RequestDestination::kImage,
            ..Default::default()
        };
        let mut first = engine.Start(request.clone());
        assert_eq!(requests.borrow()[0].referrer, "https://example.test/");
        assert!(first.Poll());
        assert!(engine.HasLoaded(request.clone()));
        engine.SetReferrerPolicy(ReferrerPolicy::NoReferrer);
        assert!(!engine.HasLoaded(request.clone()));
        let mut omitted = engine.Start(request.clone());
        assert!(omitted.Poll());
        assert_eq!(requests.borrow()[1].referrer, "");
        assert!(engine.HasLoaded(request.clone()));
        let explicit_omit = URLRequest {
            referrer_policy: Some(ReferrerPolicy::NoReferrer),
            headers: vec![("Referer".into(), "https://must-not-leak.test/".into())],
            ..request.clone()
        };
        engine.SetReferrerPolicy(ReferrerPolicy::UnsafeUrl);
        assert!(engine.HasLoaded(explicit_omit.clone()));
        let _same = engine.Start(explicit_omit.clone());
        assert_eq!(requests.borrow().len(), 2);
        let _stream = engine.StartStream(explicit_omit).unwrap();
        assert_eq!(requests.borrow().last().unwrap().referrer, "");
        assert!(requests.borrow().last().unwrap().headers.is_empty());
    }
}
