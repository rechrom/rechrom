//! Actual immutable Paint image catalog lease. Native stable identity and facts
//! are read from the same allocation used by playback; no identity registry or
//! animation/completion metadata projection is constructed.
use super::{CpuPaintImage, CpuPaintImageValue, CpuPaintRecordImage, CpuWorkletInput, PaintRecord};
use crate::adapter::{image_animation::PaintImageId, layer_tree_host_impl::ContentColorUsage};
use std::sync::Arc;
#[derive(Debug)]
pub enum CpuPaintImageInputError {
    MissingResource(u64),
    Metadata {
        resource: u64,
        blocker: paint::paint_engine::ImageMetadataUnavailable,
    },
    DocumentColorSpaceNotPublished(u64),
    AnimationFramesNotPublished(u64),
}
impl std::fmt::Display for CpuPaintImageInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnimationFramesNotPublished(id) => write!(
                f,
                "Paint image {id} has animation facts but no native frame timeline owner"
            ),
            Self::MissingResource(id) => write!(
                f,
                "Paint image resource {id} is absent from its original catalog"
            ),
            Self::Metadata { resource, blocker } => {
                write!(f, "Paint image resource {resource}: {blocker}")
            }
            Self::DocumentColorSpaceNotPublished(id) => write!(
                f,
                "Paint document image {id} lacks native content color usage"
            ),
        }
    }
}
impl std::error::Error for CpuPaintImageInputError {}
struct NativeImageLease {
    record: PaintRecord,
    catalog_index: usize,
    document_record: Option<CpuPaintRecordImage>,
    software_image: std::sync::OnceLock<Arc<Vec<u8>>>,
}
impl NativeImageLease {
    fn image(&self) -> &paint::paint_engine::NativePaintImage {
        &self
            .record
            .artifact()
            .resources
            .as_ref()
            .expect("validated native image catalog")
            .images[self.catalog_index]
    }
}
impl CpuPaintImage {
    /// Image resources are immutable for the life of this original record.
    /// Facts are validated here once, before the infallible cc value interface
    /// can be acquired. Unknown animation/document facts remain typed errors.
    pub fn FromPaintResource(
        record: &PaintRecord,
        resource: u64,
    ) -> Result<Self, CpuPaintImageInputError> {
        Self::FromPaintResourceForContainer(record, resource, None)
    }

    /// Resolves the exact document-image recording selected by this paint use.
    /// A single SVG resource may be embedded at several container sizes in the
    /// same artifact, so the use-site key is authoritative when one is present.
    pub fn FromPaintResourceForContainer(
        record: &PaintRecord,
        resource: u64,
        container: Option<&paint::paint_engine::DocumentImageContainerKey>,
    ) -> Result<Self, CpuPaintImageInputError> {
        let (catalog_index, image) = record
            .artifact()
            .resources
            .as_ref()
            .and_then(|r| {
                r.images
                    .iter()
                    .enumerate()
                    .find(|(_, image)| image.id == resource)
            })
            .ok_or(CpuPaintImageInputError::MissingResource(resource))?;
        let metadata =
            |e: &paint::paint_engine::ImageMetadataUnavailable| CpuPaintImageInputError::Metadata {
                resource,
                blocker: e.clone(),
            };
        if image.ShouldAnimate().map_err(metadata)? {
            return Err(CpuPaintImageInputError::AnimationFramesNotPublished(
                resource,
            ));
        }
        image.IsComplete().map_err(metadata)?;
        image.IsMultipart().map_err(metadata)?;
        let document_record = match &image.content {
            paint::paint_engine::PaintImageContent::Bitmap(_) => None,
            paint::paint_engine::PaintImageContent::Document(content) => {
                let key = container.or(image.document_container_key.as_ref()).ok_or(
                    CpuPaintImageInputError::DocumentColorSpaceNotPublished(resource),
                )?;
                let record = content
                    .exact_record_for(key)
                    .and_then(|record| {
                        record
                            .as_any()
                            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
                    })
                    .ok_or(CpuPaintImageInputError::DocumentColorSpaceNotPublished(
                        resource,
                    ))?;
                Some(CpuPaintRecordImage {
                    record: PaintRecord::from_artifact(record.artifact.clone()),
                    intrinsic_size: crate::adapter::geometry::SizeF::new(
                        record.intrinsic_size.width as f32,
                        record.intrinsic_size.height as f32,
                    ),
                    record_size: crate::adapter::geometry::SizeF::new(
                        record.record_size.width as f32,
                        record.record_size.height as f32,
                    ),
                })
            }
        };
        Ok(Self(Arc::new(NativeImageLease {
            record: record.clone(),
            catalog_index,
            document_record,
            software_image: std::sync::OnceLock::new(),
        })))
    }
}
impl CpuPaintImageValue for NativeImageLease {
    // Source boundary: cc/paint/paint_image.cc GetSwSkImage. The immutable
    // producer owns straight RGBA. Skia performs the actual upload conversion;
    // OnceLock retains that one resulting SkImage-equivalent allocation.
    fn GetSwSkImage(&self) -> std::io::Result<Arc<Vec<u8>>> {
        if let Some(image) = self.software_image.get() {
            return Ok(image.clone());
        }
        let input = self.image();
        let pixels = input.BitmapPixels().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "software image requires an actual bitmap allocation",
            )
        })?;
        let converted = skia::raster_target::premultiply_rgba8888(
            input.width,
            input.height,
            pixels,
        )
        .ok_or_else(|| std::io::Error::other("invalid software image bitmap allocation"))?;
        let _ = self.software_image.set(Arc::new(converted));
        Ok(self
            .software_image
            .get()
            .expect("same immutable image backing")
            .clone())
    }

    fn GetPaintRecord(&self) -> Option<CpuPaintRecordImage> {
        self.document_record.clone()
    }

    fn size(&self) -> crate::adapter::geometry::Size {
        crate::adapter::geometry::Size::new(self.image().width as i32, self.image().height as i32)
    }
    fn animation_metadata(
        &self,
    ) -> Option<&crate::adapter::image_animation::AnimatedImageMetadata> {
        None
    } // The constructor proves the actual producer is static.
    fn stable_id(&self) -> PaintImageId {
        self.image().paint_image_id
    }
    fn ShouldAnimate(&self) -> bool {
        self.image()
            .ShouldAnimate()
            .expect("validated immutable native animation facts")
    }
    fn IsLazyGenerated(&self) -> bool {
        self.image().IsLazyGenerated()
    }
    fn IsComplete(&self) -> bool {
        self.image()
            .IsComplete()
            .expect("validated immutable native completion facts")
    }
    fn IsMultipart(&self) -> bool {
        self.image()
            .IsMultipart()
            .expect("validated immutable native multipart facts")
    }
    fn ContentColorUsage(&self) -> ContentColorUsage {
        ContentColorUsage::kSRGB
    }
    fn WorkletInput(&self) -> Option<&CpuWorkletInput> {
        None
    } // Actual producer variant is a bitmap, not a worklet.
    fn GetReinterpretAsSRGB(&self) -> bool {
        false
    } // Actual RGBA8 bitmap is already source sRGB.
}
