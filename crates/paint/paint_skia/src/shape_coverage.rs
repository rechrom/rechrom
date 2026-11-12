//! Source Shape::ExtractPathData, core/layout/shapes/shape.cc:265-301.
//! Independent AA coverage service: immutable neutral path in, alpha bytes out.
use foundation::gfx;
use layoutng_geometry::{
    shape_coverage::{ShapeCoverage, ShapeCoverageRasterizer},
    shape_path::{PathVerb, ShapePath},
};
use skia::{
    include::core::{SkPathTypes::SkPathFillType, SkSurfaceProps::SkSurfaceProps},
    raster_target::{PixelFormat, RasterTarget},
    src::core::{SkColorSpace::SkColorSpace, SkPaint::SkPaint, SkPathBuilder::SkPathBuilder},
};
pub struct SkiaShapeCoverageRasterizer;
impl ShapeCoverageRasterizer for SkiaShapeCoverageRasterizer {
    fn Rasterize(&self, path: &ShapePath, size: gfx::Size) -> Option<ShapeCoverage> {
        let count = (size.width() as usize).checked_mul(size.height() as usize)?;
        if size.width() <= 0 || size.height() <= 0 || count > i32::MAX as usize / 4 {
            return None;
        }
        let mut pixels = Vec::new();
        pixels.try_reserve_exact(count * 4).ok()?;
        pixels.resize(count * 4, 0);
        let mut builder = SkPathBuilder::with_fill_type(if path.even_odd {
            SkPathFillType::kEvenOdd
        } else {
            SkPathFillType::kWinding
        });
        for c in path.commands.iter() {
            match c.verb {
                PathVerb::kMoveTo => {
                    builder.moveToXY(c.point.x as f32, c.point.y as f32);
                }
                PathVerb::kLineTo => {
                    builder.lineToXY(c.point.x as f32, c.point.y as f32);
                }
                PathVerb::kQuadraticTo => {
                    builder.quadToXY(
                        c.control1.x as f32,
                        c.control1.y as f32,
                        c.point.x as f32,
                        c.point.y as f32,
                    );
                }
                PathVerb::kConicTo => {
                    builder.conicToXY(
                        c.control1.x as f32,
                        c.control1.y as f32,
                        c.point.x as f32,
                        c.point.y as f32,
                        c.conic_weight as f32,
                    );
                }
                PathVerb::kCubicTo => {
                    builder.cubicToXY(
                        c.control1.x as f32,
                        c.control1.y as f32,
                        c.control2.x as f32,
                        c.control2.y as f32,
                        c.point.x as f32,
                        c.point.y as f32,
                    );
                }
                PathVerb::kClose => {
                    builder.close();
                }
            }
        }
        let native = builder.detach(None);
        let props = SkSurfaceProps::default();
        let mut target = RasterTarget::new_with_color_space(
            &mut pixels,
            size.width(),
            size.height(),
            size.width() as usize * 4,
            PixelFormat::Bgra8888,
            Some(SkColorSpace::MakeSRGB()),
            Some(&props),
        )
        .ok()?;
        let mut paint = SkPaint::default();
        paint.setColor(0xff000000);
        paint.setAntiAlias(true);
        unsafe {
            target.draw_path(&native, &paint).ok()?;
        }
        target.finish().ok()?;
        let alpha = pixels
            .chunks_exact(4)
            .map(|pixel| pixel[3])
            .collect::<Vec<_>>()
            .into();
        Some(ShapeCoverage { size, alpha })
    }
}
