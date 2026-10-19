//! Selected CPU SkPictureShader tile geometry. The actual PaintRecord owner
//! remains at the caller; this file only supplies native raster size/matrices.
use crate::raster::Transform;
#[derive(Clone, Copy, Debug)]
pub struct RasterTile {
    pub width: u32,
    pub height: u32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub record_to_tile: Transform,
    pub image_to_record: Transform,
}
/// Source: third_party/skia/src/shaders/SkPictureShader.cpp:180-241;
/// src/core/SkMatrix.cpp:1479-1495 (affine decomposeScale).
/// CPU has no maxTextureSize limit and its N32 surface is RGBA8888 premul.
pub fn raster_tile(bounds: [f32; 4], total: Transform) -> Option<RasterTile> {
    use crate::include::core::SkPoint::SkPoint;
    let [left, top, right, bottom] = bounds;
    let w = right - left;
    let h = bottom - top;
    if !(w > 0. && h > 0.) {
        return None;
    }
    let sx = SkPoint::native_length(total.sx, total.ky);
    let sy = SkPoint::native_length(total.kx, total.sy);
    let (sx, sy) = if sx.is_finite() && sy.is_finite() && sx > 1. / 4096. && sy > 1. / 4096. {
        (sx, sy)
    } else {
        // Affine specialization of DifferentialAreaScale at bounds.center().
        let area =
            ((total.sx as f64 * total.sy as f64 - total.kx as f64 * total.ky as f64) as f32).abs();
        if !area.is_finite() || area <= 1. / 4096. {
            (1., 1.)
        } else {
            let s = area.sqrt();
            (s, s)
        }
    };
    let (mut rw, mut rh) = (sx * w, sy * h);
    let area = rw * rh;
    if area > 2048. * 2048. {
        let scale = (2048. * 2048. / area).sqrt();
        rw *= scale;
        rh *= scale;
    }
    let (width, height) = (rw.ceil() as u32, rh.ceil() as u32);
    if width == 0 || height == 0 {
        return None;
    }
    let (sx, sy) = (width as f32 / w, height as f32 / h);
    Some(RasterTile {
        width,
        height,
        scale_x: sx,
        scale_y: sy,
        record_to_tile: Transform::from_row(sx, 0., 0., sy, -left * sx, -top * sy),
        image_to_record: Transform::from_row(1. / sx, 0., 0., 1. / sy, left, top),
    })
}
