//! Skia execution backend for the source-mapped Renderer management core.
//! The current implementation uses CPU surfaces. Allocation, Canvas playback
//! and output pixels stay here; traversal and damage decisions stay in Renderer.
mod native;
mod software_adapter;
mod components {
    pub(crate) mod viz {
        pub(crate) mod service {
            pub(crate) mod display {
                pub(crate) mod renderer_utils;
                pub(crate) mod software_renderer;
            }
        }
    }
}

pub use components::viz::service::display::software_renderer::SoftwareRenderer as RendererSkiaBackend;
pub use native::{PictureDisplayItemList, PictureRasterRequest, PictureRasterizer, RenderTarget};
pub use skia::PixelFormat;
