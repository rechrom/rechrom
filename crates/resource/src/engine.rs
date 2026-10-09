#![allow(non_snake_case)]

use crate::ResourceLoader;
use decode::{DecodeEffect, DecodeEngine, DecodedImageResource};
use image_resource::{ContainerKey, DocumentImageMutation, ImageId};
use std::{
    cell::{Cell, RefCell},
    io,
    rc::Rc,
    sync::Arc,
    time::Instant,
};
use url_loader::{URLLoader, URLRequest, URLStreamOperation};

/// One document/open operation's resource loader. The transport and decoders
/// may be shared services, but URL/referrer, cancellation and resource identity
/// belong to this engine instance. The engine intentionally has no DOM pointer:
/// discovery and application belong to DocumentEngine and Page.
pub struct ResourceEngine {
    loader: Rc<RefCell<dyn URLLoader>>,
    decode: Rc<DecodeEngine>,
    document_url: RefCell<String>,
    active: Cell<bool>,
    next_image_id: Cell<ImageId>,
}

impl ResourceEngine {
    pub fn new(loader: Rc<RefCell<dyn URLLoader>>, document_url: String) -> Self {
        Self {
            loader,
            decode: Rc::new(DecodeEngine::new()),
            document_url: RefCell::new(document_url),
            active: Cell::new(true),
            next_image_id: Cell::new(1),
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
            active: Cell::new(true),
            next_image_id: Cell::new(1),
        }
    }

    pub fn DocumentURL(&self) -> String {
        self.document_url.borrow().clone()
    }

    pub fn SetDocumentURL(&self, url: String) {
        *self.document_url.borrow_mut() = url;
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

    pub fn Start(&self, mut request: URLRequest) -> ResourceLoader {
        if !self.active.get() {
            return ResourceLoader::Failed("resource engine is cancelled");
        }
        if request.referrer.is_empty() {
            request.referrer = self.DocumentURL();
        }
        ResourceLoader::Start(&mut *self.loader.borrow_mut(), &request)
    }

    pub fn StartStream(&self, mut request: URLRequest) -> io::Result<Box<dyn URLStreamOperation>> {
        if !self.active.get() {
            return Err(io::Error::other("resource engine is cancelled"));
        }
        if request.referrer.is_empty() {
            request.referrer = self.DocumentURL();
        }
        self.loader.borrow_mut().LoadStream(&request)
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

    struct BitmapDecoder;
    impl ImageDecoder for BitmapDecoder {
        fn Decode(&mut self, _: &ImageDecodeInput<'_>) -> io::Result<DecodedImage> {
            Ok(DecodedImage {
                width: 1,
                height: 1,
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
}
