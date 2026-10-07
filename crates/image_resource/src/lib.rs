//! Renderer-neutral image identities and immutable content snapshots.
//! This crate owns no DOM, layout, paint engine, raster backend or scheduler.

use std::{any::Any, fmt, io, sync::Arc, time::Duration};

pub type ImageId = u64;

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
pub enum PaintImageContent {
    Bitmap(Arc<Vec<u8>>),
    Document(Arc<dyn DocumentPaintRecord>),
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
            Self::Document(record) => formatter
                .debug_tuple("Document")
                .field(&format_args!("{:p}", Arc::as_ptr(record)))
                .finish(),
        }
    }
}

impl PartialEq for PaintImageContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bitmap(a), Self::Bitmap(b)) => Arc::ptr_eq(a, b) || a == b,
            (Self::Document(a), Self::Document(b)) => Arc::ptr_eq(a, b),
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
