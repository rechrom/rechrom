//! Shared immutable image metadata and the narrow live content lease.
//! The resource/DOM producer owns decoding and completion. Layout retains the
//! same lease identity; it never re-resolves an element's current URL.
// cpp: dom/document.h:158-163
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageResourceMetadata {
    pub id: u64,
    pub natural_width: f64,
    pub natural_height: f64,
    pub resolution_scale: f64,
    pub orientation: crate::ImageOrientation,
    /// Image::IsOpaque published by the decoded resource owner.
    pub known_to_be_opaque: bool,
    /// ImageResourceContent::IsCorsSameOrigin(), retained for SVG feImage's
    /// FilterEffect origin-taint decision.
    pub is_cors_same_origin: bool,
}

impl Default for ImageResourceMetadata {
    fn default() -> Self {
        Self {
            id: 0,
            natural_width: 0.0,
            natural_height: 0.0,
            resolution_scale: 1.0,
            orientation: Default::default(),
            known_to_be_opaque: false,
            is_cors_same_origin: false,
        }
    }
}

impl ImageResourceMetadata {
    pub fn NaturalSize(&self, respect_orientation: bool) -> (f64, f64) {
        if respect_orientation {
            self.orientation
                .oriented_size(self.natural_width, self.natural_height)
        } else {
            (self.natural_width, self.natural_height)
        }
    }

    /// CSS intrinsic size for one HTML responsive-image use. Candidate
    /// density is element-owned; resource resolution remains the fallback for
    /// ordinary `src` and CSS image consumers.
    pub fn NaturalSizeForCandidate(
        &self,
        respect_orientation: bool,
        candidate_density: Option<f64>,
    ) -> (f64, f64) {
        let size = self.NaturalSize(respect_orientation);
        let density = candidate_density
            .filter(|density| density.is_finite() && *density > 0.0)
            .unwrap_or(self.resolution_scale);
        let scale = self.resolution_scale / density;
        (size.0 * scale, size.1 * scale)
    }
}

/// Source ImageResourceContent reads required by LayoutImageResource.
/// Rc<dyn ImageResourceContentLease> retains the producer's original owner.
pub trait ImageResourceContentLease {
    // Source ImageResourceContent::HasImage/IsAutomaticUpgrade/Url. The
    // response/request identity remains with the original resource producer.
    fn HasImage(&self) -> bool;
    fn IsAutomaticUpgrade(&self) -> bool;
    fn Url(&self) -> String;
    fn Metadata(&self) -> Option<ImageResourceMetadata>;
    fn ErrorOccurred(&self) -> bool;
    fn IsLoading(&self) -> bool;
    /// Immutable current-frame value published by the decoded resource owner.
    /// Metadata-only producers cannot manufacture drawable content.
    fn ImageSnapshot(&self) -> Option<std::sync::Arc<crate::PaintImage>> {
        None
    }
    /// Image::AsSkBitmapForCurrentFrame and SVGImageForContainer use the
    /// decoded image's existing producer; the consumer receives pixels only.
    fn BitmapForCurrentFrame(
        &self,
        _container: Option<crate::DocumentImageBitmapRequest>,
    ) -> std::io::Result<crate::ImmutableImageBitmap> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "image bitmap producer is not configured",
        ))
    }
    fn ConcreteImageSize(
        &self,
        _multiplier: f32,
        _default_size: (f32, f32),
    ) -> std::io::Result<Option<(f32, f32)>> {
        Ok(None)
    }
}
/// ImageLoader::UpdateLayoutObject passes current content and completion.
/// Layout owns whether to keep its old cached image until this one completes.
#[derive(Clone)]
pub struct ImageResourceContentUpdate {
    pub content: Option<std::rc::Rc<dyn ImageResourceContentLease>>,
    pub complete: bool,
}
