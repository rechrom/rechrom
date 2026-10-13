//! The admitted 8-bit Blink/CrabbyAvif libyuv path, translated to safe Rust.
//!
//! Source: libyuv a0fa6329391922faca64955c1a7bf635ab87d833,
//! row_common.cc (YuvPixel, YPixel and I601/JPEG/H709/F709 constants),
//! scale_common.cc (ScaleRowUp2_{Linear,Bilinear}_C), scale_any.cc
//! (SUH2LANY/SU2BLANY edges), convert_argb.cc (I420ToARGBMatrixBilinear).
//! Copyright 2011 The LibYuv Project Authors. All rights reserved.
//! BSD-3-Clause license: third_party/LICENSE.libyuv-avif-rgb8.
//!
//! This is straight RGBA. The existing raster boundary owns premultiplication.
//! Chroma is rounded once after the 2D 9:3:3:1 filter, never in two passes.

use super::avif_backend::{codec_error, invalid, unsupported};
use crabby_avif::{
    image::{Image, Plane, YuvRange},
    MatrixCoefficients, PixelFormat,
};
use std::io;

#[derive(Clone, Copy)]
struct Constants {
    yg: i32,
    yb: i32,
    ub: i32,
    ug: i32,
    vg: i32,
    vr: i32,
}

fn constants(image: &Image) -> io::Result<Constants> {
    let matrix = if image.yuv_format == PixelFormat::Yuv400
        && image.matrix_coefficients == MatrixCoefficients::Identity
    {
        MatrixCoefficients::Bt601
    } else {
        image.matrix_coefficients
    };
    let (yg, yb, ub, ug, vg, vr) = match (matrix, image.yuv_range) {
        (MatrixCoefficients::Bt709, YuvRange::Full) => (16320, 32, 119, 12, 30, 101),
        (MatrixCoefficients::Bt709, YuvRange::Limited) => (18997, -1160, 128, 14, 34, 115),
        (
            MatrixCoefficients::Bt470bg
            | MatrixCoefficients::Bt601
            | MatrixCoefficients::Unspecified,
            YuvRange::Full,
        ) => (16320, 32, 113, 22, 46, 90),
        (
            MatrixCoefficients::Bt470bg
            | MatrixCoefficients::Bt601
            | MatrixCoefficients::Unspecified,
            YuvRange::Limited,
        ) => (18997, -1160, 128, 25, 52, 102),
        _ => return Err(unsupported("unimplemented 8-bit RGB conversion matrix")),
    };
    Ok(Constants {
        yg,
        yb,
        ub,
        ug,
        vg,
        vr,
    })
}

fn convert_pixel(y: u8, u: u8, v: u8, c: Constants) -> [u8; 3] {
    let y = ((i32::from(y) * 257 * c.yg) >> 16) + c.yb;
    let u = i32::from(u) - 128;
    let v = i32::from(v) - 128;
    let clamp = |x: i32| (x >> 6).clamp(0, 255) as u8;
    [
        clamp(y + v * c.vr),
        clamp(y - u * c.ug - v * c.vg),
        clamp(y + u * c.ub),
    ]
}

// SUH2LANY/SU2BLANY explicitly replicate both horizontal endpoints, including
// the last pixel of an ODD-width image. This differs from generic resampling.
fn horizontal(x: usize, width: usize) -> (usize, usize, u32, u32) {
    if x == 0 || x == width - 1 {
        (x / 2, x / 2, 4, 0)
    } else {
        let i = (x - 1) / 2;
        if x & 1 == 1 {
            (i, i + 1, 3, 1)
        } else {
            (i, i + 1, 1, 3)
        }
    }
}

fn expanded_row(image: &Image, plane: Plane, y: u32, out: &mut [u8]) -> io::Result<()> {
    let (top, bottom, wt, wb) = if image.yuv_format == PixelFormat::Yuv422 {
        (y, y, 4, 0)
    } else if y == 0 || (image.height & 1 == 0 && y == image.height - 1) {
        (y / 2, y / 2, 4, 0)
    } else {
        let top = (y - 1) / 2;
        if y & 1 == 1 {
            (top, top + 1, 3, 1)
        } else {
            (top, top + 1, 1, 3)
        }
    };
    let a = image.row(plane, top).map_err(codec_error)?;
    let b = image.row(plane, bottom).map_err(codec_error)?;
    let width = out.len();
    for (x, value) in out.iter_mut().enumerate() {
        let (left, right, wl, wr) = horizontal(x, width);
        *value = ((u32::from(a[left]) * wl * wt
            + u32::from(a[right]) * wr * wt
            + u32::from(b[left]) * wl * wb
            + u32::from(b[right]) * wr * wb
            + 8)
            >> 4) as u8;
    }
    Ok(())
}

pub(super) fn convert(image: &Image) -> io::Result<Vec<u8>> {
    if image.depth != 8 || image.width == 0 || image.height == 0 {
        return Err(invalid("RGB8 conversion requires nonempty 8-bit planes"));
    }
    let identity = image.matrix_coefficients == MatrixCoefficients::Identity
        && image.yuv_format == PixelFormat::Yuv444
        && image.yuv_range == YuvRange::Full;
    let c = if identity {
        None
    } else {
        Some(constants(image)?)
    };
    let width = image.width as usize;
    let len = width
        .checked_mul(image.height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| invalid("RGBA size overflow"))?;
    let mut out = Vec::new();
    out.try_reserve_exact(len).map_err(invalid)?;
    let mut u = Vec::new();
    u.try_reserve_exact(width).map_err(invalid)?;
    u.resize(width, 128);
    let mut v = Vec::new();
    v.try_reserve_exact(width).map_err(invalid)?;
    v.resize(width, 128);
    for y in 0..image.height {
        let y_row = image.row(Plane::Y, y).map_err(codec_error)?;
        let alpha = if image.has_alpha() {
            Some(image.row(Plane::A, y).map_err(codec_error)?)
        } else {
            None
        };
        match image.yuv_format {
            PixelFormat::Yuv400 => {}
            PixelFormat::Yuv444 => {
                u.copy_from_slice(&image.row(Plane::U, y).map_err(codec_error)?[..width]);
                v.copy_from_slice(&image.row(Plane::V, y).map_err(codec_error)?[..width]);
            }
            PixelFormat::Yuv420 | PixelFormat::Yuv422 => {
                expanded_row(image, Plane::U, y, &mut u)?;
                expanded_row(image, Plane::V, y, &mut v)?;
            }
            _ => return Err(unsupported("unimplemented 8-bit chroma layout")),
        }
        for x in 0..width {
            let rgb = if identity {
                [v[x], y_row[x], u[x]]
            } else {
                convert_pixel(y_row[x], u[x], v[x], c.unwrap())
            };
            out.extend_from_slice(&rgb);
            out.push(alpha.map_or(255, |a| a[x]));
        }
    }
    Ok(out)
}
