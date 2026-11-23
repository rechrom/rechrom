//! Production bindings of the main-half translations to Paint's native
//! values (PaintArtifact chunks, display items and property-node handles).
//! They are generic over the compositor family; the compositor binds the
//! concrete CPU family.

pub mod paint;
mod main_family;
/// Main paint services (PAC, chunk conversion, GeometryMapper, main Layer
/// services). Until S1a-3 B3 the compositor's EngineCore still drives them.
pub mod paint_services;
pub(crate) mod main_frame_lifecycle;
pub(crate) mod pac_scroll;

// Module paths of the shared value roots under which the moved bindings name
// them (the compositor keeps the same paths). Aliases only.
pub(crate) mod geometry {
    pub use crate::commit::transform::*;
}
pub(crate) mod metadata {
    pub use crate::commit::viz_metadata::*;
}
pub(crate) mod types {
    pub use crate::commit::paint::NodeId;
    pub use crate::commit::skia_value_backend::PaintSkiaBackend;
}
