//! Skia execution backend for the source-mapped Renderer management core.
//! The current implementation uses host-readable surfaces. Allocation, Canvas playback
//! and output pixels stay here; traversal and damage decisions stay in Renderer.
mod native;
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
pub use native::{PixelFormat, RenderTarget};
