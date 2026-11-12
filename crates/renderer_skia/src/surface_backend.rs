//! Render-surface selection boundary corresponding to Chromium Viz output
//! surface selection.
//!
//! `TranslatedSoftwareRenderer` remains the CPU Canvas playback owner. The
//! optional Metal backing is a real GPU resource and command target, but is not
//! selected for Canvas playback until the translated Ganesh/Graphite device can
//! consume it. Keeping that distinction explicit prevents a GPU allocation
//! from being mistaken for a GPU renderer.

use crate::{PixelFormat, RenderTarget};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendPreference {
    Software,
    MetalIfCanvasPlaybackAvailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendCapabilities {
    pub software_canvas_playback: bool,
    pub metal_surface_allocation: bool,
    /// The first executable Ganesh/Metal render-pass subset: premultiplied
    /// SrcOver solid rectangles through a compiled MTL pipeline.
    pub metal_solid_rect_playback: bool,
    /// Repeat/repeat, linear, non-mipmapped premultiplied texture quads with
    /// a destination-to-texture local matrix.
    pub metal_repeat_linear_texture_playback: bool,
    /// Real multisample Metal attachment plus single-sample resolve texture.
    /// This is a prerequisite for GPU Canvas playback, not a claim that the
    /// translated Canvas device is complete.
    pub metal_msaa_render_target: bool,
    pub metal_canvas_playback: bool,
}

impl BackendCapabilities {
    pub const fn current() -> Self {
        Self {
            software_canvas_playback: true,
            metal_surface_allocation: cfg!(all(target_vendor = "apple", feature = "metal_surface")),
            metal_solid_rect_playback: cfg!(all(
                target_vendor = "apple",
                feature = "metal_surface"
            )),
            metal_repeat_linear_texture_playback: cfg!(all(
                target_vendor = "apple",
                feature = "metal_surface"
            )),
            metal_msaa_render_target: cfg!(all(target_vendor = "apple", feature = "metal_surface")),
            // A Metal texture alone is not a GPU Canvas. This changes only
            // when translated Ganesh/Graphite playback is wired to it.
            metal_canvas_playback: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderSurfaceDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

impl RenderSurfaceDescriptor {
    pub fn from_target(target: &RenderTarget) -> Self {
        Self {
            width: target.width,
            height: target.height,
            format: target.format,
        }
    }
}

pub enum RenderSurface {
    Software(RenderTarget),
    #[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
    Metal(MetalSurface),
}

impl RenderSurface {
    /// Select the backend from actual capabilities. Until GPU Canvas playback
    /// exists, `MetalIfCanvasPlaybackAvailable` correctly retains the software
    /// target rather than returning a GPU-shaped stub.
    pub fn select(target: RenderTarget, preference: BackendPreference) -> Self {
        let capabilities = BackendCapabilities::current();
        if preference == BackendPreference::MetalIfCanvasPlaybackAvailable
            && capabilities.metal_canvas_playback
        {
            // The allocation arm becomes reachable together with translated
            // GPU Canvas playback. Keeping the false capability above means no
            // current call can lose the supplied CPU target.
            #[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
            if let Ok(surface) = MetalSurface::new(RenderSurfaceDescriptor::from_target(&target)) {
                return Self::Metal(surface);
            }
        }
        Self::Software(target)
    }

    pub fn descriptor(&self) -> RenderSurfaceDescriptor {
        match self {
            Self::Software(target) => RenderSurfaceDescriptor::from_target(target),
            #[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
            Self::Metal(surface) => surface.descriptor,
        }
    }
}

#[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
pub struct MetalSurface {
    descriptor: RenderSurfaceDescriptor,
    target: skia::metal_target::MetalTarget,
}

#[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
impl MetalSurface {
    pub fn new(
        descriptor: RenderSurfaceDescriptor,
    ) -> Result<Self, skia::metal_target::MetalTargetError> {
        Self::new_with_sample_count(descriptor, 1)
    }

    pub fn new_with_sample_count(
        descriptor: RenderSurfaceDescriptor,
        sample_count: usize,
    ) -> Result<Self, skia::metal_target::MetalTargetError> {
        if descriptor.format != PixelFormat::Bgra8888 {
            return Err(skia::metal_target::MetalTargetError::UnsupportedPixelFormat);
        }
        let target = skia::metal_target::MetalTarget::new_with_sample_count(
            descriptor.width as usize,
            descriptor.height as usize,
            sample_count,
        )?;
        Ok(Self { descriptor, target })
    }

    pub fn sample_count(&self) -> usize {
        self.target.sample_count()
    }

    pub fn clear(&self, rgba: [f64; 4]) -> Result<(), skia::metal_target::MetalTargetError> {
        self.target.clear(rgba)
    }

    /// Execute the currently translated Metal draw subset. Keeping this on the
    /// renderer surface boundary lets a future Ganesh-backed backend consume
    /// the same resource without exposing MTL objects across packages.
    pub fn draw_solid_rect(
        &self,
        rect: [f32; 4],
        rgba: [f32; 4],
    ) -> Result<(), skia::metal_target::MetalTargetError> {
        self.target.draw_solid_rect(rect, rgba)
    }

    pub fn draw_repeat_linear_textured_quad(
        &self,
        rect: [f32; 4],
        source_width: usize,
        source_height: usize,
        source_rgba: &[u8],
        source_row_bytes: usize,
        local_matrix: [f32; 9],
        opacity: f32,
    ) -> Result<(), skia::metal_target::MetalTargetError> {
        self.target.draw_repeat_linear_textured_quad(
            rect,
            source_width,
            source_height,
            source_rgba,
            source_row_bytes,
            local_matrix,
            opacity,
        )
    }

    pub fn read_pixels(
        &self,
        pixels: &mut [u8],
        row_bytes: usize,
    ) -> Result<(), skia::metal_target::MetalTargetError> {
        self.target.read_pixels(pixels, row_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PixelStorage;

    #[test]
    fn gpu_preference_retains_software_until_canvas_playback_exists() {
        let target = RenderTarget {
            width: 2,
            height: 3,
            row_stride: 8,
            format: PixelFormat::Bgra8888,
            pixels: PixelStorage::owned(vec![0; 6]),
            retained: false,
        };
        let surface =
            RenderSurface::select(target, BackendPreference::MetalIfCanvasPlaybackAvailable);
        assert!(matches!(&surface, RenderSurface::Software(_)));
        assert_eq!(
            surface.descriptor(),
            RenderSurfaceDescriptor {
                width: 2,
                height: 3,
                format: PixelFormat::Bgra8888,
            }
        );
    }

    #[test]
    fn reports_allocation_separately_from_gpu_canvas_playback() {
        let capabilities = BackendCapabilities::current();
        assert!(capabilities.software_canvas_playback);
        assert!(!capabilities.metal_canvas_playback);
        assert_eq!(
            capabilities.metal_solid_rect_playback,
            cfg!(all(target_vendor = "apple", feature = "metal_surface"))
        );
        assert_eq!(
            capabilities.metal_repeat_linear_texture_playback,
            cfg!(all(target_vendor = "apple", feature = "metal_surface"))
        );
        assert_eq!(
            capabilities.metal_msaa_render_target,
            cfg!(all(target_vendor = "apple", feature = "metal_surface"))
        );
        assert_eq!(
            capabilities.metal_surface_allocation,
            cfg!(all(target_vendor = "apple", feature = "metal_surface"))
        );
    }

    #[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
    #[test]
    fn renderer_surface_executes_metal_solid_draw_subset() {
        let surface = MetalSurface::new(RenderSurfaceDescriptor {
            width: 3,
            height: 2,
            format: PixelFormat::Bgra8888,
        })
        .expect("Metal surface");
        surface.clear([0.0, 0.0, 0.0, 1.0]).expect("clear");
        surface
            .draw_solid_rect([1.0, 0.0, 3.0, 2.0], [0.0, 1.0, 0.0, 1.0])
            .expect("solid draw");
        let mut pixels = vec![0; 3 * 2 * 4];
        surface.read_pixels(&mut pixels, 3 * 4).expect("readback");
        assert_eq!(&pixels[0..4], &[0, 0, 0, 255]);
        assert_eq!(&pixels[4..8], &[0, 255, 0, 255]);
        assert_eq!(&pixels[20..24], &[0, 255, 0, 255]);
    }

    #[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
    #[test]
    fn renderer_surface_executes_repeat_linear_texture_subset() {
        let surface = MetalSurface::new(RenderSurfaceDescriptor {
            width: 4,
            height: 1,
            format: PixelFormat::Bgra8888,
        })
        .expect("Metal surface");
        surface.clear([0.0, 0.0, 0.0, 1.0]).expect("clear");
        surface
            .draw_repeat_linear_textured_quad(
                [0.0, 0.0, 4.0, 1.0],
                2,
                1,
                &[255, 0, 0, 255, 0, 255, 0, 255],
                8,
                [0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 1.0],
                1.0,
            )
            .expect("texture draw");
        let mut pixels = vec![0; 4 * 4];
        surface.read_pixels(&mut pixels, 4 * 4).expect("readback");
        assert_eq!(&pixels[0..4], &[0, 0, 255, 255]);
        assert_eq!(&pixels[4..8], &[0, 255, 0, 255]);
        assert_eq!(&pixels[8..12], &[0, 0, 255, 255]);
        assert_eq!(&pixels[12..16], &[0, 255, 0, 255]);
    }

    #[cfg(all(target_vendor = "apple", feature = "metal_surface"))]
    #[test]
    fn renderer_surface_allocates_four_sample_resolve_target() {
        let surface = MetalSurface::new_with_sample_count(
            RenderSurfaceDescriptor {
                width: 4,
                height: 2,
                format: PixelFormat::Bgra8888,
            },
            4,
        )
        .expect("4x MSAA Metal surface");
        assert_eq!(surface.sample_count(), 4);
        surface.clear([0.0, 0.0, 0.0, 1.0]).expect("clear");
        surface
            .draw_solid_rect([0.5, 0.0, 3.5, 2.0], [1.0, 1.0, 1.0, 1.0])
            .expect("MSAA rect");
        let mut pixels = vec![0; 4 * 2 * 4];
        surface
            .read_pixels(&mut pixels, 4 * 4)
            .expect("resolve readback");
        assert!((32..=223).contains(&pixels[0]));
        assert_eq!(&pixels[4..8], &[255, 255, 255, 255]);
    }
}
