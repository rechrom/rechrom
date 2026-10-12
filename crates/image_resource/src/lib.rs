//! Renderer-neutral image identities and immutable content snapshots.
//! This crate owns no DOM, layout, paint engine, raster backend or scheduler.

use std::{any::Any, fmt, io, sync::Arc, time::Duration};

pub type ImageId = u64;

/// The encoded image's EXIF orientation. Bitmap pixels remain in encoded
/// storage order so CSS `image-orientation: none` and `from-image` can select
/// independently at each use, matching Blink's ImageOrientation contract.
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageOrientation {
    #[default]
    kDefault = 1,
    kFlipHorizontal = 2,
    kRotate180 = 3,
    kFlipVertical = 4,
    kTranspose = 5,
    kRotate90 = 6,
    kTransverse = 7,
    kRotate270 = 8,
}

impl ImageOrientation {
    pub fn from_exif(value: u8) -> Self {
        match value {
            2 => Self::kFlipHorizontal,
            3 => Self::kRotate180,
            4 => Self::kFlipVertical,
            5 => Self::kTranspose,
            6 => Self::kRotate90,
            7 => Self::kTransverse,
            8 => Self::kRotate270,
            _ => Self::kDefault,
        }
    }

    pub fn exif(self) -> u8 {
        self as u8
    }

    pub fn uses_width_as_height(self) -> bool {
        matches!(
            self,
            Self::kTranspose | Self::kRotate90 | Self::kTransverse | Self::kRotate270
        )
    }

    pub fn oriented_size(self, width: f64, height: f64) -> (f64, f64) {
        if self.uses_width_as_height() {
            (height, width)
        } else {
            (width, height)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IntrinsicSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ContainerKey {
    pub width: u32,
    pub height: u32,
    pub zoom_bits: u64,
    pub device_pixel_ratio_bits: u64,
    pub view_specification: String,
    pub preferred_color_scheme: u8,
    /// `true` when the embedding element's computed `color-scheme` selected
    /// this value. When false, the document preference still has to be
    /// supplied by the Page assembly root.
    pub preferred_color_scheme_is_explicit: bool,
}

impl ContainerKey {
    pub fn new(width: u32, height: u32, zoom: f64, device_pixel_ratio: f64) -> Self {
        Self {
            width,
            height,
            zoom_bits: zoom.to_bits(),
            device_pixel_ratio_bits: device_pixel_ratio.to_bits(),
            view_specification: String::new(),
            preferred_color_scheme: 0,
            preferred_color_scheme_is_explicit: false,
        }
    }

    pub fn zoom(&self) -> f64 {
        f64::from_bits(self.zoom_bits)
    }

    pub fn device_pixel_ratio(&self) -> f64 {
        f64::from_bits(self.device_pixel_ratio_bits)
    }
}

/// Type-erased immutable recording. Implementations contain paint data only;
/// the retained document and its timeline stay in the document-image owner.
pub trait DocumentPaintRecord: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Send + Sync> DocumentPaintRecord for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct DocumentPaintVariant {
    pub container_key: ContainerKey,
    pub record: Arc<dyn DocumentPaintRecord>,
}

#[derive(Clone, Default)]
pub struct DocumentPaintContent {
    variants: Vec<DocumentPaintVariant>,
}

impl DocumentPaintContent {
    pub fn new(container_key: ContainerKey, record: Arc<dyn DocumentPaintRecord>) -> Self {
        Self {
            variants: vec![DocumentPaintVariant {
                container_key,
                record,
            }],
        }
    }

    pub fn with_variant(
        &self,
        container_key: ContainerKey,
        record: Arc<dyn DocumentPaintRecord>,
    ) -> Self {
        let mut variants = self.variants.clone();
        if let Some(variant) = variants
            .iter_mut()
            .find(|variant| variant.container_key == container_key)
        {
            variant.record = record;
        } else {
            variants.push(DocumentPaintVariant {
                container_key,
                record,
            });
        }
        Self { variants }
    }

    pub fn record_for(
        &self,
        container_key: &ContainerKey,
    ) -> Option<&Arc<dyn DocumentPaintRecord>> {
        self.exact_record_for(container_key)
    }

    pub fn exact_record_for(
        &self,
        container_key: &ContainerKey,
    ) -> Option<&Arc<dyn DocumentPaintRecord>> {
        self.variants
            .iter()
            .find(|variant| &variant.container_key == container_key)
            .map(|variant| &variant.record)
    }

    pub fn variants(&self) -> &[DocumentPaintVariant] {
        &self.variants
    }
}

impl fmt::Debug for DocumentPaintContent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DocumentPaintContent")
            .field("variant_count", &self.variants.len())
            .finish()
    }
}

impl PartialEq for DocumentPaintContent {
    fn eq(&self, other: &Self) -> bool {
        self.variants.len() == other.variants.len()
            && self.variants.iter().zip(&other.variants).all(|(a, b)| {
                a.container_key == b.container_key && (Arc::ptr_eq(&a.record, &b.record))
            })
    }
}

#[derive(Clone)]
pub enum PaintImageContent {
    Bitmap(Arc<Vec<u8>>),
    Document(Arc<DocumentPaintContent>),
}

impl Default for PaintImageContent {
    fn default() -> Self {
        Self::Bitmap(Arc::new(Vec::new()))
    }
}

impl fmt::Debug for PaintImageContent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bitmap(pixels) => formatter
                .debug_tuple("Bitmap")
                .field(&format_args!("{} bytes", pixels.len()))
                .finish(),
            Self::Document(record) => formatter.debug_tuple("Document").field(record).finish(),
        }
    }
}

impl PartialEq for PaintImageContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bitmap(a), Self::Bitmap(b)) => Arc::ptr_eq(a, b) || a == b,
            (Self::Document(a), Self::Document(b)) => Arc::ptr_eq(a, b) || a == b,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct DocumentImageFrame {
    pub resource_id: ImageId,
    pub revision: u64,
    pub intrinsic_size: IntrinsicSize,
    pub container_key: ContainerKey,
    pub record: Arc<dyn DocumentPaintRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageResourceChange {
    Content {
        id: ImageId,
        revision: u64,
    },
    IntrinsicSize {
        id: ImageId,
        revision: u64,
        size: IntrinsicSize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MutationCause {
    pub begin_frame_sequence: u64,
}

#[derive(Clone)]
pub enum DocumentImageMutation {
    SetContainer {
        container: ContainerKey,
    },
    AdvanceTimeline {
        frame_time: Duration,
        begin_frame_sequence: u64,
    },
    SetPreferredColorScheme {
        scheme: u8,
    },
}

#[derive(Clone)]
pub enum DocumentImageEffect {
    FrameChanged {
        frame: Arc<DocumentImageFrame>,
        cause: MutationCause,
    },
    IntrinsicSizeChanged {
        resource_id: ImageId,
        revision: u64,
        size: IntrinsicSize,
        cause: MutationCause,
    },
    RequestBeginFrame,
}

pub trait DocumentImage {
    fn apply_mutation(
        &mut self,
        mutation: DocumentImageMutation,
    ) -> io::Result<Vec<DocumentImageEffect>>;
    fn has_active_animation(&self) -> bool;
}

pub struct CreatedDocumentImage {
    pub initial_frame: Arc<DocumentImageFrame>,
    pub image: Box<dyn DocumentImage>,
    pub effects: Vec<DocumentImageEffect>,
}

pub trait DocumentImageDecoder {
    fn can_decode(&self, bytes: &[u8], mime_type: &str) -> bool;
    fn create(
        &mut self,
        resource_id: ImageId,
        bytes: Arc<[u8]>,
        mime_type: &str,
        container: &ContainerKey,
    ) -> io::Result<CreatedDocumentImage>;
}
