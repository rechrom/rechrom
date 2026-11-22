//! Immutable path coverage service injected once when LayoutEngine is built.
//! Layout owns RasterShapeIntervals; the adapter only performs the source
//! antialiased path draw into a zero-cleared coverage surface.
use crate::shape_path::ShapePath;
use foundation::gfx;
use std::{cell::Cell, sync::Arc};
pub struct ShapeCoverage {
    pub size: gfx::Size,
    pub alpha: Arc<[u8]>,
}
pub trait ShapeCoverageRasterizer: Send + Sync {
    fn Rasterize(&self, path: &ShapePath, size: gfx::Size) -> Option<ShapeCoverage>;
}
thread_local! { static CURRENT:Cell<Option<&'static dyn ShapeCoverageRasterizer>>=const{Cell::new(None)}; }
pub struct ShapeCoverageScope(
    Option<&'static dyn ShapeCoverageRasterizer>,
    std::marker::PhantomData<std::rc::Rc<()>>,
);
impl ShapeCoverageScope {
    pub fn new(service: Option<&'static dyn ShapeCoverageRasterizer>) -> Self {
        Self(
            CURRENT.with(|current| current.replace(service)),
            std::marker::PhantomData,
        )
    }
}
impl Drop for ShapeCoverageScope {
    fn drop(&mut self) {
        CURRENT.with(|current| current.set(self.0));
    }
}
pub fn CurrentShapeCoverageRasterizer() -> Option<&'static dyn ShapeCoverageRasterizer> {
    CURRENT.with(Cell::get)
}
