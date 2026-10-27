//! Stable package attribution for compositor performance traces.
//!
//! Every expensive production operation must be attributed to exactly one of
//! these responsibility packages. The facade may nest package spans to show
//! orchestration, but it must not collapse work into an undifferentiated
//! `compositor` duration. Package internals add work-count fields at the point
//! where those counts are authoritative.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompositorPackage {
    Layerize,
    Tree,
    Tile,
    Raster,
    Draw,
    Frame,
    Resource,
    Scheduler,
    Input,
    Animation,
}

impl CompositorPackage {
    pub const fn TraceCategory(self) -> &'static str {
        match self {
            Self::Layerize => "compositor.layerize",
            Self::Tree => "compositor.tree",
            Self::Tile => "compositor.tile",
            Self::Raster => "compositor.raster",
            Self::Draw => "compositor.draw",
            Self::Frame => "compositor.frame",
            Self::Resource => "compositor.resource",
            Self::Scheduler => "compositor.scheduler",
            Self::Input => "compositor.input",
            Self::Animation => "compositor.animation",
        }
    }
}

pub(crate) fn Span(package: CompositorPackage, operation: &'static str) -> browser_tracing::Span {
    browser_tracing::span(package.TraceCategory(), operation)
}
