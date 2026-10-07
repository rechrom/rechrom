/*!
`tiny-skia` is a tiny [Skia](https://skia.org/) subset ported to Rust.

`tiny-skia` API is a bit unconventional.
It doesn't look like cairo, QPainter (Qt), HTML Canvas or even Skia itself.
Instead, `tiny-skia` provides a set of low-level drawing APIs
and a user should manage the world transform, clipping mask and style manually.

See the `examples/` directory for usage examples.
*/

#![allow(clippy::approx_constant)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::identity_op)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::wrong_self_convention)]

#[cfg(not(any(feature = "std", feature = "no-std-float")))]
compile_error!("You have to activate either the `std` or the `no-std-float` feature.");

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub(crate) use crate::compat::color;
pub(crate) use crate::compat::raster_bounds as geom;
pub(crate) use crate::include::core::SkShader as shaders;
pub(crate) use crate::include::private::SkFixed as fixed_point;
pub(crate) use crate::include::private::SkMath as math;
pub(crate) use crate::src::core::SkAlphaRuns as alpha_runs;
pub(crate) use crate::src::core::SkBlendMode as blend_mode;
pub(crate) use crate::src::core::SkBlitter as blitter;
pub(crate) use crate::src::core::SkEdge as edge;
pub(crate) use crate::src::core::SkEdgeBuilder as edge_builder;
pub(crate) use crate::src::core::SkEdgeClipper as edge_clipper;
pub(crate) use crate::src::core::SkGeometry as path_geometry;
pub(crate) use crate::src::core::SkLineClipper as line_clipper;
pub(crate) use crate::src::core::SkMaskBuilder as mask;
pub(crate) use crate::src::core::SkPixmap as pixmap;
pub(crate) use crate::src::core::SkRasterPipeline as pipeline;
pub(crate) use crate::src::core::SkScan as scan;
pub(crate) use crate::src::core::SkVx::skvx as wide;
pub(crate) use crate::src::pathops::SkPathOpsTypes as path64;

pub(crate) use crate::compat::pixmap_draw as painter;

pub use crate::include::core::SkColorType::PixelFormat;
pub use crate::include::core::SkPaint::Paint;
pub use crate::include::core::SkPathTypes::FillRule;
pub use crate::src::core::SkBitmap::Pixmap;
pub use crate::src::core::SkPixelRef::PixelStorage;
pub use blend_mode::BlendMode;
pub use color::{Color, ColorSpace, ColorU8, PremultipliedColor, PremultipliedColorU8};
pub use color::{ALPHA_OPAQUE, ALPHA_TRANSPARENT, ALPHA_U8_OPAQUE, ALPHA_U8_TRANSPARENT};
pub use mask::{Mask, MaskType};
pub use pixmap::{PixmapMut, PixmapRef, BYTES_PER_PIXEL};
pub use shaders::{FilterQuality, GradientStop, PixmapPaint, SpreadMode};
pub use shaders::{LinearGradient, Pattern, RadialGradient, Shader, SweepGradient};

pub use crate::path::{IntRect, IntSize, NonZeroRect, Point, Rect, Size, Transform};
pub use crate::path::{LineCap, LineJoin, Stroke, StrokeDash};
pub use crate::path::{Path, PathBuilder, PathSegment, PathSegmentsIter, PathStroker};

/// Blend a premultiplied RGBA8888 source row into the selected N32 storage.
/// This adapter shares SkOpts::blit_row_s32a_opaque with CPU Canvas; it uses
/// the migrated SkPMSrcOver_neon8 kernel on ARM64 and its scalar counterpart.
/// Both slices must contain the same number of complete four-byte pixels.
pub fn blend_premultiplied_rgba_row(dst: &mut [u8], src: &[u8], format: PixelFormat) {
    crate::src::opts::SkBlitRow_opts::blit_row_s32a_opaque(dst, src, format);
}

/// An integer length that is guarantee to be > 0
pub(crate) type LengthU32 = core::num::NonZeroU32;
