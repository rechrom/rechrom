//! Canonical ImageResourceContent identity spans request, decode and completion.
//! Source image_resource_content.h:72-211; .cc:90-178,299-421.
use super::html_image_loader::HTMLImageLoader;
use crate::ImageResourceMetadata;
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::{Rc, Weak},
};
pub struct DOMDecodedImage {
    resource_id: u64,
}
impl DOMDecodedImage {
    pub fn ResourceId(&self) -> u64 {
        self.resource_id
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageContentStatus {
    NotStarted,
    Pending,
    Loaded,
    Error,
}
#[derive(Clone, Copy, Default)]
struct DecodedMediaFacts {
    extension: Option<&'static str>,
    encoded_size: u64,
    animated: bool,
    first_frame_complete: bool,
}
/// ImageResourceContent::ResourceBuffer and response metadata retain the
/// original immutable encoded response, shared with its resource producer.
pub struct ImageEncodedResource {
    pub bytes: std::sync::Arc<[u8]>,
    pub content_disposition: String,
}
pub struct DOMImageResourceContent {
    resource_buffer: RefCell<Option<std::sync::Arc<ImageEncodedResource>>>,
    resource_info: RefCell<Option<Rc<dyn image_resource::media_timing::ImageResourceInfo>>>,
    decoded_media: Cell<DecodedMediaFacts>,
    request_url: OnceCell<String>,
    cors_enabled: Cell<bool>,
    is_broken: Cell<bool>,
    is_svg: Cell<bool>,
    status: Cell<ImageContentStatus>,
    image: RefCell<Option<Rc<DOMDecodedImage>>>,
    metadata: Cell<Option<ImageResourceMetadata>>,
    snapshot: RefCell<Option<std::sync::Arc<image_resource::PaintImage>>>,
    bitmap_provider: RefCell<Option<Rc<dyn image_resource::ImageBitmapProvider>>>,
    access_check_failed: Cell<bool>,
    should_collapse_initiator: Cell<bool>,
    observers: RefCell<Vec<Weak<HTMLImageLoader>>>,
}
impl DOMImageResourceContent {
    pub fn Pending() -> Rc<Self> {
        Rc::new(Self {
            resource_buffer: RefCell::new(None),
            resource_info: RefCell::new(None),
            decoded_media: Cell::new(DecodedMediaFacts::default()),
            request_url: OnceCell::new(),
            cors_enabled: Cell::new(false),
            is_broken: Cell::new(false),
            is_svg: Cell::new(false),
            status: Cell::new(ImageContentStatus::Pending),
            image: RefCell::new(None),
            metadata: Cell::new(None),
            snapshot: RefCell::new(None),
            bitmap_provider: RefCell::new(None),
            access_check_failed: Cell::new(false),
            should_collapse_initiator: Cell::new(false),
            observers: RefCell::new(vec![]),
        })
    }
    pub(super) fn Loaded(resource_id: u64) -> Rc<Self> {
        let owner = Self::Pending();
        owner.status.set(ImageContentStatus::Loaded);
        *owner.image.borrow_mut() =
            (resource_id != 0).then(|| Rc::new(DOMDecodedImage { resource_id }));
        owner
    }
    pub(super) fn Failed() -> Rc<Self> {
        let owner = Self::Pending();
        owner.status.set(ImageContentStatus::Error);
        owner
    }
    /// Source image_resource_content.cc:713-746. The Resource keeps this same
    /// live info allocation through request completion/cache reuse.
    pub fn SetResourceInfo(&self, info: Rc<dyn image_resource::media_timing::ImageResourceInfo>) {
        *self.resource_info.borrow_mut() = Some(info);
    }
    /// Image decoder facts published before synchronous client completion.
    /// Source BitmapImage::FilenameExtension/MaybeAnimated/FirstFrameIsComplete.
    pub fn SetEncodedResourceDataSize(&self, size: u64) {
        let mut facts = self.decoded_media.get();
        facts.encoded_size = size;
        self.decoded_media.set(facts);
    }
    pub fn SetEncodedResource(&self, bytes: std::sync::Arc<[u8]>, content_disposition: String) {
        self.SetEncodedResourceDataSize(bytes.len() as u64);
        self.resource_buffer
            .replace(Some(std::sync::Arc::new(ImageEncodedResource {
                bytes,
                content_disposition,
            })));
    }
    pub fn ResourceBuffer(&self) -> Option<std::sync::Arc<ImageEncodedResource>> {
        self.resource_buffer.borrow().clone()
    }
    pub fn FilenameExtension(&self) -> Option<&'static str> {
        self.decoded_media.get().extension
    }
    pub fn EncodedResourceDataSize(&self) -> u64 {
        self.decoded_media.get().encoded_size
    }
    pub fn SetDecodedMediaFacts(
        &self,
        extension: Option<&'static str>,
        encoded_size: u64,
        animated: bool,
        first_frame_complete: bool,
    ) {
        self.decoded_media.set(DecodedMediaFacts {
            extension,
            encoded_size,
            animated,
            first_frame_complete,
        });
    }
    pub fn SetIsSVGImage(&self, is_svg: bool) {
        self.is_svg.set(is_svg);
    }
    pub fn IsSVGImage(&self) -> bool {
        self.is_svg.get()
    }
    pub fn SetIsBroken(&self) {
        self.is_broken.set(true);
    }
    pub fn IsBroken(&self) -> bool {
        self.is_broken.get()
    }
    pub fn SetCORSEnabled(&self, enabled: bool) {
        self.cors_enabled.set(enabled);
    }
    pub fn CORSEnabled(&self) -> bool {
        self.cors_enabled.get()
    }
    pub fn SetRequestURL(&self, url: String) {
        if self.request_url.get().is_none() {
            let _ = self.request_url.set(url);
        }
    }
    pub fn URL(&self) -> &str {
        self.request_url.get().map_or("", String::as_str)
    }
    pub fn SetDeferred(&self) {
        if self.status.get() == ImageContentStatus::Pending {
            self.status.set(ImageContentStatus::NotStarted);
        }
    }
    pub fn StartDeferred(&self) {
        if self.status.get() == ImageContentStatus::NotStarted {
            self.status.set(ImageContentStatus::Pending);
        }
    }
    pub fn ErrorOccurred(&self) -> bool {
        self.status.get() == ImageContentStatus::Error
    }
    pub fn IsLoading(&self) -> bool {
        self.status.get() == ImageContentStatus::Pending
    }
    pub fn IsLoaded(&self) -> bool {
        matches!(
            self.status.get(),
            ImageContentStatus::Loaded | ImageContentStatus::Error
        )
    }
    pub fn HasImage(&self) -> bool {
        self.image.borrow().is_some()
    }
    pub fn Image(&self) -> Option<Rc<DOMDecodedImage>> {
        self.image.borrow().clone()
    }
    /// Retain the immutable decoder frame; no pixel copy or id lookup.
    pub fn SetImageSnapshot(&self, image: std::sync::Arc<image_resource::PaintImage>) {
        *self.snapshot.borrow_mut() = Some(image);
    }
    pub fn ImageSnapshot(&self) -> Option<std::sync::Arc<image_resource::PaintImage>> {
        self.snapshot.borrow().clone()
    }
    pub fn SetBitmapProvider(&self, provider: Option<Rc<dyn image_resource::ImageBitmapProvider>>) {
        *self.bitmap_provider.borrow_mut() = provider;
    }
    pub fn ConcreteImageSize(
        &self,
        multiplier: f32,
        default_size: (f32, f32),
    ) -> std::io::Result<Option<(f32, f32)>> {
        let image = self
            .ImageSnapshot()
            .ok_or_else(|| std::io::Error::other("image has no decoded current frame"))?;
        let provider = self.bitmap_provider.borrow().clone().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "image bitmap producer is not configured",
            )
        })?;
        provider.ConcreteImageSize(&image, multiplier, default_size)
    }
    pub fn BitmapForCurrentFrame(
        &self,
        container: Option<image_resource::DocumentImageBitmapRequest>,
    ) -> std::io::Result<image_resource::ImmutableImageBitmap> {
        let image = self
            .ImageSnapshot()
            .ok_or_else(|| std::io::Error::other("image has no decoded current frame"))?;
        let provider = self.bitmap_provider.borrow().clone().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "image bitmap producer is not configured",
            )
        })?;
        provider.BitmapForCurrentFrame(&image, container)
    }
    pub fn Metadata(&self) -> Option<ImageResourceMetadata> {
        self.metadata.get()
    }
    pub fn ShouldCollapseInitiator(&self) -> bool {
        self.should_collapse_initiator.get()
    }
    pub fn AccessCheckFailed(&self) -> bool {
        self.access_check_failed.get()
    }
    pub(super) fn Complete(
        self: &Rc<Self>,
        metadata: Option<ImageResourceMetadata>,
        access_check_failed: bool,
    ) {
        self.metadata.set(metadata);
        self.access_check_failed.set(access_check_failed);
        *self.image.borrow_mut() = metadata
            .filter(|metadata| metadata.id != 0)
            .map(|metadata| {
                Rc::new(DOMDecodedImage {
                    resource_id: metadata.id,
                })
            });
        self.status.set(if metadata.is_some() {
            ImageContentStatus::Loaded
        } else {
            ImageContentStatus::Error
        });
        let observers = self.Observers();
        for observer in observers {
            observer.ImageNotifyFinished(self);
        }
    }
    pub(super) fn AddObserver(self: &Rc<Self>, observer: &Rc<HTMLImageLoader>) {
        let mut observers = self.observers.borrow_mut();
        observers.retain(|observer| observer.strong_count() != 0);
        if !observers
            .iter()
            .any(|candidate| candidate.ptr_eq(&Rc::downgrade(observer)))
        {
            observers.push(Rc::downgrade(observer));
        }
        drop(observers);
        if self.IsLoaded() {
            observer.ImageNotifyFinished(self);
        }
    }
    pub(super) fn RemoveObserver(&self, observer: &Rc<HTMLImageLoader>) {
        self.observers.borrow_mut().retain(|candidate| {
            !candidate.ptr_eq(&Rc::downgrade(observer)) && candidate.strong_count() != 0
        });
    }
    pub fn Observers(&self) -> Vec<Rc<HTMLImageLoader>> {
        self.observers
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect()
    }
}

impl image_resource::ImageResourceContentLease for DOMImageResourceContent {
    fn HasImage(&self) -> bool {
        self.HasImage()
    }
    fn IsAutomaticUpgrade(&self) -> bool {
        self.resource_info
            .borrow()
            .as_ref()
            .is_some_and(|info| info.IsAutomaticUpgrade())
    }
    fn Url(&self) -> String {
        self.resource_info
            .borrow()
            .as_ref()
            .map_or_else(|| self.URL().to_owned(), |info| info.Url())
    }
    fn ConcreteImageSize(
        &self,
        multiplier: f32,
        default_size: (f32, f32),
    ) -> std::io::Result<Option<(f32, f32)>> {
        self.ConcreteImageSize(multiplier, default_size)
    }
    fn BitmapForCurrentFrame(
        &self,
        container: Option<image_resource::DocumentImageBitmapRequest>,
    ) -> std::io::Result<image_resource::ImmutableImageBitmap> {
        self.BitmapForCurrentFrame(container)
    }
    fn ImageSnapshot(&self) -> Option<std::sync::Arc<image_resource::PaintImage>> {
        self.ImageSnapshot()
    }
    fn Metadata(&self) -> Option<ImageResourceMetadata> {
        self.Metadata()
    }
    fn ErrorOccurred(&self) -> bool {
        self.ErrorOccurred()
    }
    fn IsLoading(&self) -> bool {
        self.IsLoading()
    }
}

// Chromium image_resource_content.cc:592-603,677-710,713-748. No pixel-size,
// URL suffix or document delivery-time inference occurs at this boundary.
impl image_resource::media_timing::MediaTiming for DOMImageResourceContent {
    fn Url(&self) -> String {
        self.resource_info
            .borrow()
            .as_ref()
            .map_or_else(|| self.URL().to_owned(), |info| info.Url())
    }
    fn MediaType(&self) -> String {
        self.decoded_media.get().extension.unwrap_or("").to_owned()
    }
    fn RequestPriority(&self) -> Option<image_resource::media_timing::RequestPriority> {
        self.resource_info
            .borrow()
            .as_ref()
            .and_then(|info| info.RequestPriority())
    }
    fn IsDataUrl(&self) -> bool {
        self.URL()
            .get(..5)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
    }
    fn IsPaintedFirstFrame(&self) -> bool {
        let image = self.decoded_media.get();
        image.animated && image.first_frame_complete
    }
    fn IsSufficientContentLoadedForPaint(&self) -> bool {
        self.IsLoaded()
    }
    fn GetFirstVideoFrameTime(&self) -> Option<std::time::Instant> {
        None
    }
    fn ContentSizeForEntropy(&self) -> u64 {
        self.resource_info
            .borrow()
            .as_ref()
            .and_then(|info| info.ExpectedContentLength())
            .filter(|length| *length > 0)
            .unwrap_or_else(|| {
                if self.HasImage() {
                    self.decoded_media.get().encoded_size
                } else {
                    0
                }
            })
    }
    fn ResourceLoadTimings(&self) -> image_resource::media_timing::ResourceLoadTimings {
        self.resource_info
            .borrow()
            .as_ref()
            .map_or_else(Default::default, |info| info.ResourceLoadTimings())
    }
}
