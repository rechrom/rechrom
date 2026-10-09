#![allow(non_snake_case)]

//! Asset-byte decoding and retained document-image content lifecycles.
//!
//! DecodeEngine does not know URLs, caches, DOM clients, rendering tiles,
//! windows or threads. ResourceEngine supplies stable resource identities and
//! explicit timeline mutations, then routes the immutable effects.

use image_decoder::image_decoder::{DecodedImage, ImageDecodeInput, ImageDecoder};
use image_resource::{
    ContainerKey, DocumentImage, DocumentImageDecoder, DocumentImageEffect, DocumentImageFrame,
    DocumentImageMutation, ImageId, IntrinsicSize,
};
use std::{cell::RefCell, collections::BTreeMap, io, rc::Rc, sync::Arc, time::Instant};

pub enum DecodedImageResource {
    Bitmap {
        resource_id: ImageId,
        image: DecodedImage,
    },
    Document {
        resource_id: ImageId,
        frame: Arc<DocumentImageFrame>,
    },
}

pub enum DecodeMutation {
    DecodeImage {
        resource_id: ImageId,
        bytes: Arc<[u8]>,
        mime_type: String,
        container: ContainerKey,
    },
    MutateDocumentImage {
        resource_id: ImageId,
        mutation: DocumentImageMutation,
    },
    AdvanceTimelines {
        frame_time: Instant,
        begin_frame_sequence: u64,
    },
}

pub enum DecodeEffect {
    ImageReady(DecodedImageResource),
    DocumentImageFrameChanged {
        frame: Arc<DocumentImageFrame>,
    },
    DocumentImageIntrinsicSizeChanged {
        resource_id: ImageId,
        revision: u64,
        size: IntrinsicSize,
    },
    RequestBeginFrame {
        resource_id: ImageId,
    },
}

struct ActiveDocumentImage {
    image: Box<dyn DocumentImage>,
    first_frame_time: Option<Instant>,
    begin_frame_requested: bool,
}

pub struct DecodeEngine {
    bitmap_decoder: Option<Rc<RefCell<dyn ImageDecoder>>>,
    document_image_decoder: Option<Rc<RefCell<dyn DocumentImageDecoder>>>,
    web_font_decoder: web_font::WebFontDecoder,
    document_images: RefCell<BTreeMap<ImageId, ActiveDocumentImage>>,
}

impl Default for DecodeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DecodeEngine {
    /// An unconfigured engine is sufficient for document/script-only resource
    /// users. Asset decode attempts fail explicitly.
    pub fn new() -> Self {
        Self {
            bitmap_decoder: None,
            document_image_decoder: None,
            web_font_decoder: web_font::WebFontDecoder::new(),
            document_images: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn WithDecoders(
        bitmap_decoder: Rc<RefCell<dyn ImageDecoder>>,
        document_image_decoder: Rc<RefCell<dyn DocumentImageDecoder>>,
    ) -> Self {
        Self {
            bitmap_decoder: Some(bitmap_decoder),
            document_image_decoder: Some(document_image_decoder),
            web_font_decoder: web_font::WebFontDecoder::new(),
            document_images: RefCell::new(BTreeMap::new()),
        }
    }

    /// Returns the platform web-font decoder used by resource loading.
    /// FontEngine consumes the decoded OpenType bytes; it does not own the
    /// WOFF/WOFF2 and OTS decode boundary.
    pub fn GetWebFontDecoder(&self) -> &web_font::WebFontDecoder {
        &self.web_font_decoder
    }

    /// Returns the configured bitmap decoder capability. Document images such
    /// as SVG retain their own lifecycle and do not pass through this decoder.
    pub fn GetImageDecoder(&self) -> io::Result<Rc<RefCell<dyn ImageDecoder>>> {
        self.bitmap_decoder
            .as_ref()
            .cloned()
            .ok_or_else(|| io::Error::other("bitmap decoder is not configured"))
    }

    pub fn ApplyMutation(&self, mutation: DecodeMutation) -> io::Result<Vec<DecodeEffect>> {
        match mutation {
            DecodeMutation::DecodeImage {
                resource_id,
                bytes,
                mime_type,
                container,
            } => Ok(vec![DecodeEffect::ImageReady(self.DecodeImage(
                resource_id,
                bytes,
                &mime_type,
                &container,
            )?)]),
            DecodeMutation::MutateDocumentImage {
                resource_id,
                mutation,
            } => self.MutateDocumentImage(resource_id, mutation),
            DecodeMutation::AdvanceTimelines {
                frame_time,
                begin_frame_sequence,
            } => self.AdvanceTimelines(frame_time, begin_frame_sequence),
        }
    }

    pub fn DecodeImage(
        &self,
        resource_id: ImageId,
        bytes: Arc<[u8]>,
        mime_type: &str,
        container: &ContainerKey,
    ) -> io::Result<DecodedImageResource> {
        let document_decoder = self
            .document_image_decoder
            .as_ref()
            .ok_or_else(|| io::Error::other("document image decoder is not configured"))?;
        if document_decoder.borrow().can_decode(&bytes, mime_type) {
            if self.document_images.borrow().contains_key(&resource_id) {
                return Err(io::Error::other("document image identity already exists"));
            }
            let created =
                document_decoder
                    .borrow_mut()
                    .create(resource_id, bytes, mime_type, container)?;
            if created.initial_frame.resource_id != resource_id {
                return Err(io::Error::other(
                    "document image changed resource identity during creation",
                ));
            }
            let begin_frame_requested = created
                .effects
                .iter()
                .any(|effect| matches!(effect, DocumentImageEffect::RequestBeginFrame));
            self.document_images.borrow_mut().insert(
                resource_id,
                ActiveDocumentImage {
                    image: created.image,
                    first_frame_time: None,
                    begin_frame_requested,
                },
            );
            return Ok(DecodedImageResource::Document {
                resource_id,
                frame: created.initial_frame,
            });
        }
        let bitmap_decoder = self.GetImageDecoder()?;
        let image = bitmap_decoder.borrow_mut().Decode(&ImageDecodeInput {
            bytes: &bytes,
            mime_type,
        })?;
        Ok(DecodedImageResource::Bitmap { resource_id, image })
    }

    pub fn HasPendingBeginFrame(&self) -> bool {
        self.document_images
            .borrow()
            .values()
            .any(|image| image.begin_frame_requested)
    }

    pub fn MutateDocumentImage(
        &self,
        resource_id: ImageId,
        mutation: DocumentImageMutation,
    ) -> io::Result<Vec<DecodeEffect>> {
        let mut images = self.document_images.borrow_mut();
        let image = images
            .get_mut(&resource_id)
            .ok_or_else(|| io::Error::other("document image resource is not active"))?;
        image.begin_frame_requested = false;
        let effects = image.image.apply_mutation(mutation)?;
        Self::TranslateDocumentImageEffects(resource_id, image, effects)
    }

    pub fn AdvanceTimelines(
        &self,
        frame_time: Instant,
        begin_frame_sequence: u64,
    ) -> io::Result<Vec<DecodeEffect>> {
        let mut output = Vec::new();
        for (&resource_id, image) in self.document_images.borrow_mut().iter_mut() {
            if !image.begin_frame_requested {
                continue;
            }
            image.begin_frame_requested = false;
            let origin = *image.first_frame_time.get_or_insert(frame_time);
            let elapsed = frame_time.saturating_duration_since(origin);
            let effects = image
                .image
                .apply_mutation(DocumentImageMutation::AdvanceTimeline {
                    frame_time: elapsed,
                    begin_frame_sequence,
                })?;
            output.extend(Self::TranslateDocumentImageEffects(
                resource_id,
                image,
                effects,
            )?);
        }
        Ok(output)
    }

    fn TranslateDocumentImageEffects(
        expected_id: ImageId,
        image: &mut ActiveDocumentImage,
        effects: Vec<DocumentImageEffect>,
    ) -> io::Result<Vec<DecodeEffect>> {
        let mut output = Vec::with_capacity(effects.len());
        for effect in effects {
            match effect {
                DocumentImageEffect::FrameChanged { frame, .. } => {
                    if frame.resource_id != expected_id {
                        return Err(io::Error::other("document image changed resource identity"));
                    }
                    output.push(DecodeEffect::DocumentImageFrameChanged { frame });
                }
                DocumentImageEffect::IntrinsicSizeChanged {
                    resource_id,
                    revision,
                    size,
                    ..
                } => {
                    if resource_id != expected_id {
                        return Err(io::Error::other("document image changed resource identity"));
                    }
                    output.push(DecodeEffect::DocumentImageIntrinsicSizeChanged {
                        resource_id,
                        revision,
                        size,
                    });
                }
                DocumentImageEffect::RequestBeginFrame => {
                    image.begin_frame_requested = true;
                    output.push(DecodeEffect::RequestBeginFrame {
                        resource_id: expected_id,
                    });
                }
            }
        }
        Ok(output)
    }

    pub fn ClearDocumentImages(&self) {
        self.document_images.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image_resource::{CreatedDocumentImage, MutationCause};

    struct Bitmap;
    impl ImageDecoder for Bitmap {
        fn Decode(&mut self, _: &ImageDecodeInput<'_>) -> io::Result<DecodedImage> {
            unreachable!("document image decoder accepts the fixture")
        }
    }

    struct AnimatedDocumentImage {
        id: ImageId,
    }
    impl DocumentImage for AnimatedDocumentImage {
        fn apply_mutation(
            &mut self,
            mutation: DocumentImageMutation,
        ) -> io::Result<Vec<DocumentImageEffect>> {
            let DocumentImageMutation::AdvanceTimeline {
                begin_frame_sequence,
                ..
            } = mutation
            else {
                return Ok(Vec::new());
            };
            Ok(vec![
                DocumentImageEffect::FrameChanged {
                    frame: Arc::new(DocumentImageFrame {
                        resource_id: self.id,
                        revision: 2,
                        intrinsic_size: IntrinsicSize {
                            width: 10,
                            height: 20,
                        },
                        container_key: ContainerKey::new(10, 20, 1.0, 1.0),
                        record: Arc::new(()),
                    }),
                    cause: MutationCause {
                        begin_frame_sequence,
                    },
                },
                DocumentImageEffect::RequestBeginFrame,
            ])
        }

        fn has_active_animation(&self) -> bool {
            true
        }
    }

    struct DocumentDecoder;
    impl DocumentImageDecoder for DocumentDecoder {
        fn can_decode(&self, _: &[u8], _: &str) -> bool {
            true
        }

        fn create(
            &mut self,
            resource_id: ImageId,
            _: Arc<[u8]>,
            _: &str,
            container: &ContainerKey,
        ) -> io::Result<image_resource::CreatedDocumentImage> {
            Ok(CreatedDocumentImage {
                initial_frame: Arc::new(DocumentImageFrame {
                    resource_id,
                    revision: 1,
                    intrinsic_size: IntrinsicSize {
                        width: 10,
                        height: 20,
                    },
                    container_key: container.clone(),
                    record: Arc::new(()),
                }),
                image: Box::new(AnimatedDocumentImage { id: resource_id }),
                effects: vec![DocumentImageEffect::RequestBeginFrame],
            })
        }
    }

    #[test]
    fn routes_separate_decoders_and_retains_document_timeline() {
        let engine = DecodeEngine::WithDecoders(
            Rc::new(RefCell::new(Bitmap)),
            Rc::new(RefCell::new(DocumentDecoder)),
        );
        assert!(engine
            .GetWebFontDecoder()
            .Decode(b"not a font".to_vec())
            .is_err());
        let image = engine
            .DecodeImage(
                7,
                Arc::from(&b"<svg/>"[..]),
                "image/svg+xml",
                &ContainerKey::new(10, 20, 1.0, 1.0),
            )
            .unwrap();
        assert!(matches!(
            image,
            DecodedImageResource::Document { resource_id: 7, .. }
        ));
        assert!(engine.HasPendingBeginFrame());
        let effects = engine.AdvanceTimelines(Instant::now(), 42).unwrap();
        assert!(effects.iter().any(|effect| matches!(
            effect,
            DecodeEffect::DocumentImageFrameChanged { frame }
                if frame.resource_id == 7 && frame.revision == 2
        )));
        assert!(engine.HasPendingBeginFrame());
    }
}
