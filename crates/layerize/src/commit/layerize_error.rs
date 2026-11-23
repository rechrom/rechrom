//! The error value LayerizeEngine returns across the commit boundary.
//!
//! PaintInputError (runtime/paint/validation.rs) names Blink paint property
//! nodes, which stay on the main thread. The impl half receives only this
//! kind and the error's display text, which is all its CompositorError
//! carried before the split.

/// What blocked the main half from layerizing a PaintArtifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerizeErrorKind {
    MissingImageResource,
    ImageMetadata,
    ImagePlayback,
    MissingViewportMetadata,
    InvalidViewportMetadata,
    MissingStickyConstraint,
    InvalidCanvasChild,
    InvalidRecordRange,
    MissingScrollCompositorData,
    MissingChunkCompositorData,
    UnsupportedDisplayItem,
    EffectFilter,
    ClipPath,
    InvalidChunkRange,
    /// A canvas-child recording names a negative canvas or element identity.
    InvalidDomNodeId,
    /// PAC holds canvas children but the artifact has no canvas document.
    MissingCanvasDocument,
}

#[derive(Clone, Debug)]
pub struct LayerizeError {
    pub kind: LayerizeErrorKind,
    pub detail: String,
}
impl std::fmt::Display for LayerizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}
impl std::error::Error for LayerizeError {}
