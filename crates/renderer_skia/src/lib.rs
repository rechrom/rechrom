//! Skia execution backend for the source-mapped Renderer management core.
//! The current implementation uses host-readable surfaces. Allocation, Canvas playback
//! and output pixels stay here; traversal and damage decisions stay in Renderer.
mod render_target;
mod surface_backend;
mod translated_adapter;
mod components {
    pub(crate) mod viz {
        pub(crate) mod service {
            pub(crate) mod display {
                pub(crate) mod translated_software_renderer;
            }
        }
    }
}

pub use components::viz::service::display::translated_software_renderer::TranslatedSoftwareRenderer as RendererSkiaBackend;
pub use render_target::{
    swizzle_bgra_to_rgba, HostPixelStorage, PixelFormat, PixelStorage, RenderTarget,
};
#[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
pub use surface_backend::MetalSurface;
pub use surface_backend::{
    BackendCapabilities, BackendPreference, RenderSurface, RenderSurfaceDescriptor,
};
