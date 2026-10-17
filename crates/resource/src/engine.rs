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
        }
    }
}

/// One document/open operation's resource loader. The transport and decoders
/// may be shared services, but URL/referrer, cancellation and resource identity
/// belong to this engine instance. The engine intentionally has no DOM pointer:
/// discovery and application belong to DocumentEngine and Page.
pub struct ResourceEngine {
    loader: Rc<RefCell<dyn URLLoader>>,
    decode: Rc<DecodeEngine>,
    document_url: RefCell<String>,
    document_referrer_policy: Cell<ReferrerPolicy>,
    active: Cell<bool>,
    wake_callback: RefCell<Option<URLWakeCallback>>,
    next_image_id: Cell<ImageId>,
    loads: RefCell<
        HashMap<ResourceRequestKey, Rc<RefCell<crate::resource_loader::SharedResourceLoad>>>,
    >,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ResourceMemoryDiagnostics {
    pub cached_loads: usize,
    pub loading: usize,
    pub encoded_response_capacity: usize,
}
impl ResourceEngine {
    pub fn MemoryDiagnostics(&self) -> ResourceMemoryDiagnostics {
        let loads = self.loads.borrow();
        let mut usage = ResourceMemoryDiagnostics {
            cached_loads: loads.len(),
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
        Self {
            loader,
            decode: Rc::new(DecodeEngine::new()),
            document_url: RefCell::new(document_url),
            document_referrer_policy: Cell::new(ReferrerPolicy::default()),
            active: Cell::new(true),
            wake_callback: RefCell::new(None),
            next_image_id: Cell::new(1),
            loads: RefCell::new(HashMap::new()),
        }
    }

    pub fn WithDecodeEngine(
        loader: Rc<RefCell<dyn URLLoader>>,
        decode: Rc<DecodeEngine>,
        document_url: String,
    ) -> Self {
        Self {
            loader,
            decode,
            document_url: RefCell::new(document_url),
            document_referrer_policy: Cell::new(ReferrerPolicy::default()),
            active: Cell::new(true),
            wake_callback: RefCell::new(None),
            next_image_id: Cell::new(1),
            loads: RefCell::new(HashMap::new()),
        }
    }

    pub fn DocumentURL(&self) -> String {
        self.document_url.borrow().clone()
    }

    pub fn SetDocumentURL(&self, url: String) {
        *self.document_url.borrow_mut() = url;
    }

    pub fn SetReferrerPolicy(&self, policy: ReferrerPolicy) {
        self.document_referrer_policy.set(policy);
    }

    pub fn ReferrerPolicy(&self) -> ReferrerPolicy {
        self.document_referrer_policy.get()
    }

    fn NormalizeRequest(&self, mut request: URLRequest) -> URLRequest {
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
        if !self.active.get() {
            return ResourceLoader::Failed("resource engine is cancelled");
        }
        let request = self.NormalizeRequest(request);
        // Chromium's ResourceFetcher/MemoryCache coalesces identical in-flight
        // subresource loads. Keep that ownership here at the document resource
        // boundary: callers still receive independent handles, while one
        // transport operation and one immutable terminal response are shared.
        // Non-idempotent requests retain one operation per call.
        if request.method.eq_ignore_ascii_case("GET") && request.body.is_empty() {
            let key = ResourceRequestKey::from(&request);
            if let Some(shared) = self.loads.borrow().get(&key).cloned() {
                let mut resource = ResourceLoader::FromShared(shared);
                resource.SetDefer(request.defer);
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
            let mut resource = ResourceLoader::Start(&mut *self.loader.borrow_mut(), &request);
            if let Some(callback) = self.wake_callback.borrow().clone() {
                resource.SetWakeCallback(callback);
            }
            self.loads.borrow_mut().insert(key, resource.Shared());
            return resource;
        }
        let mut resource = ResourceLoader::Start(&mut *self.loader.borrow_mut(), &request);
        if let Some(callback) = self.wake_callback.borrow().clone() {
            resource.SetWakeCallback(callback);
        }
        resource
    }

    /// MemoryCache::ResourceForURL(...)->IsLoaded() equivalent for a request
    /// owned by this document resource engine. This is intentionally a URL
    /// request query rather than a DOM/source-string cache.
    pub fn HasLoaded(&self, request: URLRequest) -> bool {
        let request = self.NormalizeRequest(request);
        let key = ResourceRequestKey::from(&request);
        self.loads.borrow().get(&key).is_some_and(|shared| {
            ResourceLoader::FromShared(shared.clone()).Status()
                == crate::resource_loader::ResourceLoadStatus::kComplete
        })
    }

    pub fn StartStream(&self, request: URLRequest) -> io::Result<Box<dyn URLStreamOperation>> {
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

    pub fn HasAnimatedImages(&self) -> bool {
        self.active.get() && self.decode.HasPendingBeginFrame()
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
        self.active.set(false);
        self.decode.SetWakeCallback(None);
        for shared in self.loads.borrow().values() {
            ResourceLoader::CancelShared(shared);
        }
        self.loads.borrow_mut().clear();
        self.decode.ClearDocumentImages();
    }

    pub fn IsActive(&self) -> bool {
        self.active.get()
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
                width: 1,
                height: 1,
                orientation: Default::default(),
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
