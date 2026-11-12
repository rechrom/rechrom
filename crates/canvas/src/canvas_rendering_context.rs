//! Canvas rendering-context boundary corresponding to Blink's
//! `core/html/canvas/canvas_rendering_context.{h,cc}`.

#![allow(non_snake_case)]

use crate::CanvasContextCreationAttributesCore;
use std::sync::Arc;

/// Context families accepted by `HTMLCanvasElement::getContext`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanvasContextType {
    Canvas2D,
    WebGL1,
    WebGL2,
    BitmapRenderer,
    /// Source CanvasRenderingAPI::kWebgpu; no backend is implied by this value.
    WebGPU,
}

impl CanvasContextType {
    /// Blink compares the WebIDL context identifier before asking a provider
    /// to create a context. Unsupported identifiers never claim the canvas.
    pub fn FromContextId(context_id: &str) -> Option<Self> {
        match context_id {
            "2d" => Some(Self::Canvas2D),
            "webgl" | "experimental-webgl" => Some(Self::WebGL1),
            "webgl2" => Some(Self::WebGL2),
            "bitmaprenderer" => Some(Self::BitmapRenderer),
            "webgpu" => Some(Self::WebGPU),
            _ => None,
        }
    }
}

/// Immutable premultiplied canvas output consumed by paint/raster. A context
/// may keep its GPU resource internally; this snapshot is the explicit CPU
/// crossing used until compositor shared-image support is installed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanvasPixelSnapshot {
    pub width: u32,
    pub height: u32,
    pub row_bytes: usize,
    pub pixels: Vec<u8>,
}

impl CanvasPixelSnapshot {
    /// Validate and pack a backend row-strided RGBA8 snapshot into the tight
    /// storage consumed by PaintImage/SkImage. Chromium's StaticBitmapImage
    /// boundary similarly owns a complete immutable frame rather than a live
    /// rendering-context pointer.
    pub fn IntoTightlyPackedPixels(self) -> Option<Arc<Vec<u8>>> {
        let tight_row_bytes = usize::try_from(self.width).ok()?.checked_mul(4)?;
        let rows = usize::try_from(self.height).ok()?;
        let required = self.row_bytes.checked_mul(rows)?;
        if self.row_bytes < tight_row_bytes || self.pixels.len() < required {
            return None;
        }
        if self.row_bytes == tight_row_bytes && self.pixels.len() == required {
            return Some(Arc::new(self.pixels));
        }
        let mut packed = Vec::with_capacity(tight_row_bytes.checked_mul(rows)?);
        for row in self.pixels[..required].chunks_exact(self.row_bytes) {
            packed.extend_from_slice(&row[..tight_row_bytes]);
        }
        Some(Arc::new(packed))
    }
}

/// Immutable canvas frame crossing the Canvas -> Layout/Paint boundary.
/// `dom_node_id` selects the owning LayoutHTMLCanvas while `resource_id` is
/// the stable image identity used by Paint/Raster caching.
#[derive(Clone, Debug)]
pub struct CanvasPaintResource {
    pub dom_node_id: u64,
    pub resource_id: u64,
    pub revision: u64,
    pub width: u32,
    pub height: u32,
    pub origin_clean: bool,
    pub is_clear: bool,
    pub pixels: Arc<Vec<u8>>,
}

impl PartialEq for CanvasPaintResource {
    fn eq(&self, other: &Self) -> bool {
        self.dom_node_id == other.dom_node_id
            && self.resource_id == other.resource_id
            && self.revision == other.revision
            && self.width == other.width
            && self.height == other.height
            && self.origin_clean == other.origin_clean
            && self.is_clear == other.is_clear
    }
}

impl Eq for CanvasPaintResource {}

/// Rendering contexts own their API state and backend resources. DOM bindings
/// hold only this interface, preserving the Blink package boundary between
/// `HTMLCanvasElement` and modules/webgl.
pub trait CanvasRenderingContext {
    fn GetContextType(&self) -> CanvasContextType;
    /// Source canvas_rendering_context.h:330-332. This is the context's
    /// accepted creation dictionary, after provider capability admission.
    fn CreationAttributes(&self) -> CanvasContextCreationAttributesCore;
    fn Reshape(&mut self, width: u32, height: u32);
    fn IsContextLost(&self) -> bool;
    /// Monotonic content identity. Pixel storage is immutable for one value;
    /// consumers compare this revision instead of rescanning the full buffer.
    fn ContentRevision(&self) -> u64 {
        0
    }
    fn PaintRenderingResultsToCanvas(&mut self) -> Option<CanvasPixelSnapshot>;
}

/// Injection point implemented by Canvas2D/WebGL packages. Returning `None`
/// is a genuine unsupported/backend-failure result and does not claim the
/// element, so a later creation attempt remains valid.
pub trait CanvasContextFactory {
    fn Create(
        &mut self,
        context_type: CanvasContextType,
        attributes: CanvasContextCreationAttributesCore,
        width: u32,
        height: u32,
    ) -> Option<Box<dyn CanvasRenderingContext>>;
}
