//! LayoutObject SVG algorithm dispatch and default geometry surface.
//!
//! Loaded at the reciprocal assembly boundary so LayoutObject does not become
//! a dependency of the standalone SVG package. The provider transition is an
//! out-of-line SVG-qualified sampling frame; constant/default virtual bodies
//! remain inlineable.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_core_services.cc:540-543
    #[inline(never)]
    pub fn UpdateSVGLayout(&mut self, info: &SVGLayoutInfo) -> SVGLayoutResult {
        self.CheckIsNotDestroyed();
        let algorithms = LayoutPassScope::Algorithms();
        if let Some(update) = unsafe { algorithms.as_ref() }
            .and_then(|algorithms| algorithms.svg_support.update_object)
        {
            return update(self, info);
        }
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:545-548
    #[inline]
    pub fn ObjectBoundingBox(&self) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:550-553
    #[inline]
    pub fn StrokeBoundingBox(&self) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:555-558
    #[inline]
    pub fn DecoratedBoundingBox(&self) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:560-563
    #[inline]
    pub fn VisualRectInLocalSVGCoordinates(&self) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        if self.IsSVGHiddenContainer() {
            return gfx::RectF::default();
        }
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:565-568
    #[inline]
    pub fn LocalSVGTransform(&self) -> AffineTransform {
        self.CheckIsNotDestroyed();
        AffineTransform::default()
    }
}
