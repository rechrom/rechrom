//! Manage VizOutput through Chromium's translated DirectRenderer core.
//! The embedding injects drawing and output capabilities through RendererBackend.
pub mod native;
pub use components::viz::{
    common::display::renderer_settings::RendererSettings,
    service::display::{
        direct_renderer::{
            DirectRendererBackend as RendererBackend, RenderPassRequirements,
            ShouldApplyRoundedCorner,
        },
        render_pass_alpha_type::RenderPassAlphaType,
    },
};
pub use native::{RenderUpdate, Renderer};
mod components {
    pub(crate) mod viz {
        pub(crate) mod common {
            pub(crate) mod viz_utils;
            pub(crate) mod display {
                pub(crate) mod renderer_settings;
            }
        }
        pub(crate) mod service {
            pub(crate) mod display {
                pub(crate) mod bsp_tree;
                pub(crate) mod bsp_walk_action;
                pub(crate) mod direct_renderer;
                pub(crate) mod draw_polygon;
                pub(crate) mod occlusion_culler;
                pub(crate) mod render_pass_alpha_type;
            }
        }
    }
}
