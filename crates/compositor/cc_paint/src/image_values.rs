//! Actual immutable Paint image catalog lease. Stable identity and facts
//! are read from the same allocation used by playback; no identity registry or
//! animation/completion metadata projection is constructed.
use super::{CpuPaintImage, CpuPaintImageValue, CpuPaintRecordImage, CpuWorkletInput, PaintRecord};
use super::image_metadata::PaintImageId;
use compositor_values::surface_id::ContentColorUsage;
use std::sync::Arc;
#[derive(Debug)]
pub enum CpuPaintImageInputError {
    MissingResource(u64),
    Metadata {
        resource: u64,
        blocker: image_resource::PaintImageMetadataUnavailable,
    },
    DocumentColorSpaceNotPublished(u64),
    AnimationFramesNotPublished(u64),
}
impl std::fmt::Display for CpuPaintImageInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnimationFramesNotPublished(id) => write!(
                f,
                "Paint image {id} has animation facts but no frame timeline owner"
            ),
            Self::MissingResource(id) => write!(
                f,
                "Paint image resource {id} is absent from its original catalog"
            ),
            Self::Metadata { resource, blocker } => {
                write!(f, "Paint image resource {resource}: {blocker}")
            }
            Self::DocumentColorSpaceNotPublished(id) => {
                write!(f, "Paint document image {id} lacks content color usage")
            }
        }
    }
}
impl std::error::Error for CpuPaintImageInputError {}
struct ImageLease {
    record: PaintRecord,
    catalog_index: usize,
    document_record: Option<CpuPaintRecordImage>,
    software_images: Vec<std::sync::OnceLock<Arc<Vec<u8>>>>,
    animation_metadata: Option<super::image_metadata::AnimatedImageMetadata>,
}
impl ImageLease {
    fn image(&self) -> &paint_input_types::layout_input::PaintImage {
        &self
            .record
            .resources()
            .expect("validated native image catalog")
            .images[self.catalog_index]
    }
}
impl CpuPaintImage {
    pub fn WithDecodingMode(
        mut self,
        mode: compositor_values::color_space::HostImageDecodingMode,
    ) -> Self {
        self.1 = mode;
        self
    }

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
        container: Option<&image_resource::ContainerKey>,
    ) -> Result<Self, CpuPaintImageInputError> {
        let (catalog_index, image) = record
            .resources()
            .and_then(|r| {
                r.images
                    .iter()
                    .enumerate()
                    .find(|(_, image)| image.id == resource)
            })
            .ok_or(CpuPaintImageInputError::MissingResource(resource))?;
        let metadata =
            |e: &image_resource::PaintImageMetadataUnavailable| CpuPaintImageInputError::Metadata {
                resource,
                blocker: e.clone(),
            };
        let should_animate = image.ShouldAnimate().map_err(metadata)?;
        let is_complete = image.IsComplete().map_err(metadata)?;
        image.IsMultipart().map_err(metadata)?;
        let animation_metadata = if should_animate {
            let frames = image
                .BitmapAnimationFrames()
                .filter(|frames| {
                    image
                        .CompositorMetadata()
                        .ok()
                        .and_then(|metadata| match &metadata.animation {
                            image_resource::PaintImageAnimationMetadata::Known(info) => {
                                Some(info.frame_count == frames.len())
                            }
                            image_resource::PaintImageAnimationMetadata::Unavailable(_) => {
                                None
                            }
                        })
                        .unwrap_or(false)
                })
                .ok_or(CpuPaintImageInputError::AnimationFramesNotPublished(
                    resource,
                ))?;
            let repetition_count = match &image
                .CompositorMetadata()
                .expect("animation facts validated above")
                .animation
            {
                image_resource::PaintImageAnimationMetadata::Known(info) => {
                    info.repetition_count
                }
                image_resource::PaintImageAnimationMetadata::Unavailable(_) => unreachable!(),
            };
            Some(super::image_metadata::AnimatedImageMetadata {
                paint_image_id: image.paint_image_id,
                sync_animation_target_id: super::image_metadata::kInvalidId,
                sync_animation_sequence_id: 0,
                completion_state: if is_complete {
                    super::image_metadata::CompletionState::kDone
                } else {
                    super::image_metadata::CompletionState::kPartiallyDone
                },
                frames: frames
                    .iter()
                    .map(|frame| super::image_metadata::FrameMetadata {
                        complete: true,
                        duration: scroll_animation::TimeDelta::Microseconds(
                            frame.duration.as_micros().min(i64::MAX as u128) as i64,
                        ),
                    })
                    .collect(),
                repetition_count,
                reset_animation_sequence_id: 0,
            })
        } else {
            None
        };
        let document_record = match &image.content {
            image_resource::PaintImageContent::Bitmap(_) => None,
            image_resource::PaintImageContent::Document(content) => {
                let key = container.or(image.document_container_key.as_ref()).ok_or(
                    CpuPaintImageInputError::DocumentColorSpaceNotPublished(resource),
                )?;
                let erased = content.exact_record_for(key).ok_or(
                    CpuPaintImageInputError::DocumentColorSpaceNotPublished(resource),
                )?;
                let record = image_resource::downcast_document_paint_record::<
                    super::DocumentImagePaintRecord,
                >(erased)
                .ok_or(CpuPaintImageInputError::DocumentColorSpaceNotPublished(
                    resource,
                ))?;
                Some(CpuPaintRecordImage {
                    record: record.record.clone(),
                    intrinsic_size: crate::geometry::SizeF::new(
                        record.intrinsic_size.width as f32,
                        record.intrinsic_size.height as f32,
                    ),
                    record_size: crate::geometry::SizeF::new(
                        record.record_size.width as f32,
                        record.record_size.height as f32,
                    ),
                })
            }
        };
        Ok(Self(
            Arc::new(ImageLease {
                record: record.clone(),
                catalog_index,
                document_record,
                software_images: (0..image
                    .BitmapAnimationFrames()
                    .map_or(1, |frames| frames.len()))
                    .map(|_| std::sync::OnceLock::new())
                    .collect(),
                animation_metadata,
            }),
            compositor_values::color_space::HostImageDecodingMode::kUnspecified,
        ))
    }
}
impl CpuPaintImageValue for ImageLease {
    // Source boundary: cc/paint/paint_image.cc GetSwSkImage. Upload converts
    // straight RGBA once into Skia's native premultiplied N32. A decoder that
    // already published premultiplied pixels only needs native channel order.
    // OnceLock retains that SkImage-equivalent allocation.
    fn GetSwSkImage(&self, frame_index: usize) -> std::io::Result<Arc<Vec<u8>>> {
        let software_image = self.software_images.get(frame_index).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "software image frame index is outside the producer frame set",
            )
        })?;
        if let Some(image) = software_image.get() {
            return Ok(image.clone());
        }
        let input = self.image();
        let pixels = input.BitmapFramePixels(frame_index).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "software image requires an actual bitmap allocation",
            )
        })?;
        let converted = match input.alpha_type {
            image_resource::PaintImageAlphaType::Unpremultiplied => {
                skia::raster_target::premultiply_n32_rgba8888(input.width, input.height, pixels)
            }
            image_resource::PaintImageAlphaType::Premultiplied => {
                skia::raster_target::native_n32_from_premultiplied_rgba8888(
                    input.width,
                    input.height,
                    pixels,
                )
            }
        }
        .ok_or_else(|| std::io::Error::other("invalid software image bitmap allocation"))?;
        let _ = software_image.set(Arc::new(converted));
        Ok(software_image
            .get()
            .expect("same immutable image backing")
            .clone())
    }

    fn GetEncodedData(&self) -> Option<Arc<[u8]>> {
        self.image().encoded_data.clone()
    }

    fn GetPaintRecord(&self) -> Option<CpuPaintRecordImage> {
        self.document_record.clone()
    }

    fn size(&self) -> crate::geometry::Size {
        crate::geometry::Size::new(self.image().width as i32, self.image().height as i32)
    }
    fn animation_metadata(
        &self,
    ) -> Option<&super::image_metadata::AnimatedImageMetadata> {
        self.animation_metadata.as_ref()
    }
    fn stable_id(&self) -> PaintImageId {
        self.image().paint_image_id
    }
    fn ShouldAnimate(&self) -> bool {
        self.image()
            .ShouldAnimate()
            .expect("validated immutable animation facts")
    }
    fn IsLazyGenerated(&self) -> bool {
        self.image().IsLazyGenerated()
    }
    fn IsComplete(&self) -> bool {
        self.image()
            .IsComplete()
            .expect("validated immutable completion facts")
    }
    fn IsMultipart(&self) -> bool {
        self.image()
            .IsMultipart()
            .expect("validated immutable multipart facts")
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
