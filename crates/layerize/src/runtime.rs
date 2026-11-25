//! Production bindings of the main-half translations to Paint's native
//! values (PaintArtifact chunks, display items and property-node handles).
//! They are generic over the compositor family; the compositor binds the
//! concrete CPU family.

pub mod paint;
mod main_family;
/// Main paint services (PAC, chunk conversion, GeometryMapper, main Layer
/// services), owned by LayerizeEngine.
pub mod paint_services;
pub(crate) mod main_endpoint;
pub(crate) mod main_frame_lifecycle;
pub(crate) mod main_host;
/// The main half's construction values (Blink LayerTreeView::Initialize inputs).
pub mod main_host_params;
pub(crate) mod pac_scroll;
/// ProxyMain (cc/trees/proxy_main.cc), the main half of the commit protocol.
pub mod proxy_main;

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
