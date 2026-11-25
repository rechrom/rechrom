//! cc UI resources the main LayerTreeHost creates and the impl
//! LayerTreeHostImpl uploads: cc/resources/ui_resource_bitmap.h (the bitmap
//! PaintedScrollbarLayer paints, cc/layers/painted_scrollbar_layer.cc) and
//! cc/resources/ui_resource_request.h (queued by
//! LayerTreeHost::CreateUIResource / DeleteUIResource,
//! cc/trees/layer_tree_host.cc, and carried by
//! CommitState::ui_resource_request_queue). Both are shared cc values; the
//! impl request facet (LayerTreeImpl::ProcessUIResourceRequestQueue) is
//! implemented impl-side.
use super::{
    geometry::Size,
    values::{UIResourceId, UIResourceRequestType},
};
use std::sync::Arc;

// Source: cc/resources/ui_resource_bitmap.h:28-32 UIResourceFormat (the
// formats this translation produces).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostUIBitmapFormat {
    RGBA8,
    ETC1,
}

// Source: cc/resources/ui_resource_bitmap.h:34-61 (the getters
// LayerTreeHostImpl::CreateUIResource reads).
pub trait HostUIResourceBitmap {
    fn GetFormat(&self) -> HostUIBitmapFormat;
    fn GetSize(&self) -> Size;
    fn GetOpaque(&self) -> bool;
    fn GetPixels(&self) -> &[u8];
    fn row_bytes(&self) -> usize;
}

/// cc::UIResourceBitmap: a shared immutable pixel allocation
/// (sk_sp<SkPixelRef> pixel_ref_, ui_resource_bitmap.h:69).
#[derive(Clone)]
pub struct UIBitmap(pub Arc<dyn HostUIResourceBitmap + Send + Sync>);
impl HostUIResourceBitmap for UIBitmap {
    fn GetFormat(&self) -> HostUIBitmapFormat {
        self.0.GetFormat()
    }
    fn GetSize(&self) -> Size {
        self.0.GetSize()
    }
    fn GetOpaque(&self) -> bool {
        self.0.GetOpaque()
    }
    fn GetPixels(&self) -> &[u8] {
        self.0.GetPixels()
    }
    fn row_bytes(&self) -> usize {
        self.0.row_bytes()
    }
}
impl UIBitmap {
    /// Source: cc/resources/ui_resource_bitmap.cc UIResourceBitmap(const
    /// SkBitmap&): the N32 pixels the main half painted.
    pub fn FromPixels(size: Size, pixels: Vec<u8>, row_bytes: usize, opaque: bool) -> Self {
        Self(Arc::new(RawUIBitmap::new(size, pixels, row_bytes, opaque)))
    }
}

/// The RGBA8 SkBitmap pixels a UIResourceBitmap is created from.
pub struct RawUIBitmap {
    size: Size,
    pixels: Vec<u8>,
    row_bytes: usize,
    opaque: bool,
}
impl RawUIBitmap {
    pub fn new(size: Size, pixels: Vec<u8>, row_bytes: usize, opaque: bool) -> Self {
        debug_assert_eq!(pixels.len(), row_bytes * size.height().max(0) as usize);
        Self {
            size,
            pixels,
            row_bytes,
            opaque,
        }
    }
}
impl HostUIResourceBitmap for RawUIBitmap {
    fn GetFormat(&self) -> HostUIBitmapFormat {
        HostUIBitmapFormat::RGBA8
    }
    fn GetSize(&self) -> Size {
        self.size
    }
    fn GetOpaque(&self) -> bool {
        self.opaque
    }
    fn GetPixels(&self) -> &[u8] {
        &self.pixels
    }
    fn row_bytes(&self) -> usize {
        self.row_bytes
    }
}

// Source: cc/resources/ui_resource_request.h:17-56. The bitmap is the shared
// immutable allocation; a DELETE request carries the empty bitmap (its
// payload is not read by LayerTreeHostImpl for that request type).
pub struct UIResourceRequest {
    pub kind: UIResourceRequestType,
    pub id: UIResourceId,
    pub bitmap: UIBitmap,
}
