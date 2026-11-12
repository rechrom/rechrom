//! Host-resource adapters for the source-aligned translated Skia renderer.
//!
//! Viz retains pass topology and immutable resource leases. This module only
//! converts those borrowed values into the concrete translated primitives
//! used for one synchronous draw or publication call.

use crate::render_target::{PixelFormat, PixelStorage, RenderTarget};
use compositor_core::{
    adapter::{
        geometry::{RRectF, Rect, RectF},
        layer_tree_impl::{Color4f, ResourceId},
        property_types::BlendMode,
        resource_types::{SharedImageFormat, SinglePlaneFormat},
    },
    runtime::{frame::RenderPassId, raster_execution::RasterResourceLease},
};
use raster::{MapResourceForRead, MapVideoResourceForRead};
use skia::{
    include::core::{
        SkAlphaType::SkAlphaType,
        SkClipOp::SkClipOp,
        SkColorType::SkColorType,
        SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
    },
    raster_target::{PixelFormat as TranslatedPixelFormat, RasterTarget},
    src::{
        core::{
            SkBlendMode::SkBlendMode, SkCanvas::SrcRectConstraint, SkColor::SkColor4f,
            SkData::SkData, SkImageInfo::SkImageInfo, SkMatrix::SkMatrix, SkPaint::SkPaint,
            SkPixmap::SkPixmap, SkPoint::SkPoint, SkRRect::SkRRect, SkRect::SkRect,
        },
        image::{
            SkImage::SkImage,
            SkImage_RasterFactories::{RasterFromData, RasterFromPixmapCopy},
        },
    },
};
use std::{collections::BTreeMap, io, sync::Arc};
use viz::adapter::VizOutput;

pub(crate) type PassId = RenderPassId;

pub(crate) struct PassBacking {
    pub(crate) rect: Rect,
    pub(crate) pixels: PixelStorage,
    pub(crate) drawn_rect: Rect,
}

impl PassBacking {
    pub(crate) fn new(rect: Rect) -> io::Result<Self> {
        let length = (rect.width as usize)
            .checked_mul(rect.height as usize)
            .filter(|value| *value <= isize::MAX as usize / 4)
            .ok_or_else(|| io::Error::other("translated render-pass allocation overflow"))?;
        Ok(Self {
            rect,
            pixels: PixelStorage::owned(vec![0; length]),
            drawn_rect: Rect::default(),
        })
    }

    pub(crate) fn image(&self) -> io::Result<Arc<dyn SkImage>> {
        image_from_bytes(
            bytemuck::cast_slice(&self.pixels),
            self.rect.width,
            self.rect.height,
            self.rect.width as usize * 4,
            SinglePlaneFormat::BGRA_8888,
        )
    }
}

pub(crate) fn raster_error(context: &'static str, error: impl core::fmt::Debug) -> io::Error {
    io::Error::other(format!("{context}: {error:?}"))
}

pub(crate) fn rect(value: Rect) -> SkRect {
    SkRect::MakeLTRB(
        value.x as f32,
        value.y as f32,
        value.right() as f32,
        value.bottom() as f32,
    )
}

pub(crate) fn rectf(value: RectF) -> SkRect {
    SkRect::MakeLTRB(value.x(), value.y(), value.right(), value.bottom())
}

pub(crate) fn rrect(value: &RRectF) -> SkRRect {
    let mut result = SkRRect::default();
    result.setRectRadii(
        &SkRect::MakeLTRB(
            value.rect.x(),
            value.rect.y(),
            value.rect.right(),
            value.rect.bottom(),
        ),
        &value.corner_radii.map(|radius| SkPoint {
            fX: radius.x(),
            fY: radius.y(),
        }),
    );
    result
}

pub(crate) fn color(value: Color4f, opacity: f32) -> SkColor4f {
    SkColor4f {
        fR: value.red,
        fG: value.green,
        fB: value.blue,
        fA: value.alpha * opacity,
    }
}

pub(crate) fn blend_mode(value: BlendMode) -> SkBlendMode {
    match value {
        BlendMode::kClear => SkBlendMode::kClear,
        BlendMode::kSrc => SkBlendMode::kSrc,
        BlendMode::kDst => SkBlendMode::kDst,
        BlendMode::kSrcOver => SkBlendMode::kSrcOver,
        BlendMode::kDstOver => SkBlendMode::kDstOver,
        BlendMode::kSrcIn => SkBlendMode::kSrcIn,
        BlendMode::kDstIn => SkBlendMode::kDstIn,
        BlendMode::kSrcOut => SkBlendMode::kSrcOut,
        BlendMode::kDstOut => SkBlendMode::kDstOut,
        BlendMode::kSrcATop => SkBlendMode::kSrcATop,
        BlendMode::kDstATop => SkBlendMode::kDstATop,
        BlendMode::kXor => SkBlendMode::kXor,
        BlendMode::kPlus => SkBlendMode::kPlus,
        BlendMode::kModulate => SkBlendMode::kModulate,
        BlendMode::kScreen => SkBlendMode::kScreen,
        BlendMode::kOverlay => SkBlendMode::kOverlay,
        BlendMode::kDarken => SkBlendMode::kDarken,
        BlendMode::kLighten => SkBlendMode::kLighten,
        BlendMode::kColorDodge => SkBlendMode::kColorDodge,
        BlendMode::kColorBurn => SkBlendMode::kColorBurn,
        BlendMode::kHardLight => SkBlendMode::kHardLight,
        BlendMode::kSoftLight => SkBlendMode::kSoftLight,
        BlendMode::kDifference => SkBlendMode::kDifference,
        BlendMode::kExclusion => SkBlendMode::kExclusion,
        BlendMode::kMultiply => SkBlendMode::kMultiply,
        BlendMode::kHue => SkBlendMode::kHue,
        BlendMode::kSaturation => SkBlendMode::kSaturation,
        BlendMode::kColor => SkBlendMode::kColor,
        BlendMode::kLuminosity => SkBlendMode::kLuminosity,
    }
}

pub(crate) fn matrix(value: &foundation::gfx::Transform, origin: Rect) -> io::Result<SkMatrix> {
    if value.rc(3, 0) != 0. || value.rc(3, 1) != 0. || value.rc(3, 3) != 1. {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated software renderer requires an affine quad transform",
        ));
    }
    let fields = [
        value.rc(0, 0),
        value.rc(0, 1),
        value.rc(0, 3),
        value.rc(1, 0),
        value.rc(1, 1),
        value.rc(1, 3),
    ];
    if !fields.iter().all(|field| field.is_finite()) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated software renderer received a non-finite quad transform",
        ));
    }
    Ok(SkMatrix::MakeAll(
        fields[0] as f32,
        fields[1] as f32,
        fields[2] as f32 - origin.x as f32,
        fields[3] as f32,
        fields[4] as f32,
        fields[5] as f32 - origin.y as f32,
        0.,
        0.,
        1.,
    ))
}

pub(crate) fn is_scale_and_integer_translate(matrix: &SkMatrix) -> bool {
    let nearly_integer = |value: f32| (value - value.round()).abs() <= 1.0 / 4096.0;
    nearly_integer(matrix.getTranslateX())
        && nearly_integer(matrix.getTranslateY())
        && matrix.getSkewX().abs() <= 1.0 / 4096.0
        && matrix.getSkewY().abs() <= 1.0 / 4096.0
}

pub(crate) fn resource(
    input: &VizOutput,
    id: ResourceId,
) -> io::Result<Arc<dyn RasterResourceLease>> {
    input
        .Resource(id)
        .ok_or_else(|| io::Error::other("translated frame resource has no raster lease"))
}

pub(crate) fn image_info(
    width: i32,
    height: i32,
    format: SharedImageFormat,
) -> io::Result<SkImageInfo> {
    let (color_type, alpha_type) = if format == SinglePlaneFormat::RGBA_8888 {
        (
            SkColorType::kRGBA_8888_SkColorType,
            SkAlphaType::kPremul_SkAlphaType,
        )
    } else if format == SinglePlaneFormat::BGRA_8888 {
        (
            SkColorType::kBGRA_8888_SkColorType,
            SkAlphaType::kPremul_SkAlphaType,
        )
    } else if format == SinglePlaneFormat::BGRX_8888 {
        (
            SkColorType::kBGRA_8888_SkColorType,
            SkAlphaType::kOpaque_SkAlphaType,
        )
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("translated software renderer cannot sample {format:?}"),
        ));
    };
    Ok(SkImageInfo::MakeWithColorSpace(
        width, height, color_type, alpha_type, None,
    ))
}

fn image_from_bytes(
    pixels: &[u8],
    width: i32,
    height: i32,
    row_bytes: usize,
    format: SharedImageFormat,
) -> io::Result<Arc<dyn SkImage>> {
    if width <= 0 || height <= 0 {
        return Err(io::Error::other("translated image has empty dimensions"));
    }
    let required = row_bytes
        .checked_mul(height as usize)
        .ok_or_else(|| io::Error::other("translated image byte count overflow"))?;
    if row_bytes < width as usize * 4 || pixels.len() < required {
        return Err(io::Error::other(
            "translated image pixels do not match their dimensions",
        ));
    }
    let info = image_info(width, height, format)?;
    let pixmap = SkPixmap::new(&info, pixels.as_ptr(), row_bytes);
    unsafe { RasterFromPixmapCopy(&pixmap) }
        .ok_or_else(|| io::Error::other("translated immutable image allocation failed"))
}

unsafe extern "C" fn release_raster_lease(_pixels: *const u8, context: *mut core::ffi::c_void) {
    // RasterFromData may release the immutable storage on any thread. The
    // lease contract is Send + Sync and the boxed Arc is the one retained
    // owner corresponding to this release callback.
    unsafe {
        drop(Box::from_raw(
            context.cast::<Arc<dyn RasterResourceLease>>(),
        ));
    }
}

pub(crate) fn lease_image(lease: Arc<dyn RasterResourceLease>) -> io::Result<Arc<dyn SkImage>> {
    let mapping = MapResourceForRead(lease.as_ref())?;
    let size = mapping.size();
    if size.width() <= 0 || size.height() <= 0 {
        return Err(io::Error::other("translated image has empty dimensions"));
    }
    let row_bytes = mapping.row_bytes();
    let required = row_bytes
        .checked_mul(size.height() as usize)
        .ok_or_else(|| io::Error::other("translated image byte count overflow"))?;
    let pixels = mapping.bytes();
    if row_bytes < size.width() as usize * 4 || pixels.len() < required {
        return Err(io::Error::other(
            "translated image pixels do not match their dimensions",
        ));
    }
    let info = image_info(size.width(), size.height(), mapping.format())?;
    // Preserve the resource's actual opacity in the SkImage metadata. Skia's
    // source sprite path uses this to select memcpy for SrcOver N32 tiles;
    // treating every RGBA/BGRA tile as translucent forces per-pixel blending.
    let info = if lease.is_opaque() {
        info.makeAlphaType(SkAlphaType::kOpaque_SkAlphaType)
    } else {
        info
    };
    let pixels = pixels.as_ptr();
    let length = mapping.bytes().len();
    drop(mapping);
    let context = Box::into_raw(Box::new(lease)).cast();
    let data = unsafe { SkData::MakeWithProc(pixels, length, Some(release_raster_lease), context) };
    unsafe { RasterFromData(&info, Some(data), row_bytes) }
        .ok_or_else(|| io::Error::other("translated immutable image allocation failed"))
}

#[derive(Clone, Copy)]
struct Nv12Planes<'a> {
    width: usize,
    height: usize,
    y_stride: usize,
    uv_stride: usize,
    y_plane: &'a [u8],
    uv_plane: &'a [u8],
}

#[derive(Clone, Copy)]
struct BilinearAxis {
    lower: usize,
    upper: usize,
    fraction: f32,
}

#[inline]
fn bilinear_axis(coordinate: f32, length: usize) -> BilinearAxis {
    let floor = coordinate.floor();
    let lower = (floor as isize).clamp(0, length.saturating_sub(1) as isize) as usize;
    let upper = ((floor as isize) + 1).clamp(0, length.saturating_sub(1) as isize) as usize;
    BilinearAxis {
        lower,
        upper,
        fraction: coordinate - floor,
    }
}

#[inline]
fn bilinear_plane_sample_precomputed(
    bytes: &[u8],
    stride: usize,
    x: BilinearAxis,
    y: BilinearAxis,
    pixel_stride: usize,
    channel: usize,
) -> f32 {
    let top_offset = y.lower * stride + channel;
    let bottom_offset = y.upper * stride + channel;
    let value = |offset| bytes[offset] as f32 / 255.0;
    let top = value(top_offset + x.lower * pixel_stride) * (1.0 - x.fraction)
        + value(top_offset + x.upper * pixel_stride) * x.fraction;
    let bottom = value(bottom_offset + x.lower * pixel_stride) * (1.0 - x.fraction)
        + value(bottom_offset + x.upper * pixel_stride) * x.fraction;
    top * (1.0 - y.fraction) + bottom * y.fraction
}

#[inline]
fn yuv_row(matrix: [[f32; 3]; 3], translate: [f32; 3], yuv: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|row| {
        (matrix[row][0] * yuv[0]
            + matrix[row][1] * yuv[1]
            + matrix[row][2] * yuv[2]
            + translate[row])
            .clamp(0.0, 1.0)
    })
}

fn bt709_apple_to_srgb_byte_slow(encoded: f32) -> u8 {
    // macOS accelerated decode tags this transfer as a pure 1.961 gamma.
    // Chrome's normal-GPU foreign-layer golden follows fp32 through the YUV
    // and transfer stages, then quantizes the output UNORM with nearest-even.
    let linear = encoded.clamp(0.0, 1.0).powf(1.961);
    let srgb = if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (srgb.clamp(0.0, 1.0) * 255.0)
        .round_ties_even()
        .clamp(0.0, 255.0) as u8
}

// First fp32 encoded value that quantizes to each output byte 1..=255. The
// positive-float bit pattern has the same ordering as the value, so the hot
// conversion loop needs only an integer binary search and no powf. These are
// generated from bt709_apple_to_srgb_byte_slow and checked exhaustively below.
const BT709_APPLE_TO_SRGB_THRESHOLD_BITS: [u32; 255] = [
    0x3c38f07e, 0x3ca1ebe8, 0x3cd21ab5, 0x3cf96e93, 0x3d0dc4bb, 0x3d1d0b43, 0x3d2b0238, 0x3d37f450,
    0x3d441409, 0x3d4f8561, 0x3d5a6e75, 0x3d65618c, 0x3d706d56, 0x3d7b9119, 0x3d836613, 0x3d890eee,
    0x3d8ec2d1, 0x3d948171, 0x3d9a4a90, 0x3da01de6, 0x3da5fb3e, 0x3dabe258, 0x3db1d301, 0x3db7cd02,
    0x3dbdd02f, 0x3dc3dc50, 0x3dc9f13f, 0x3dd00ecc, 0x3dd634d1, 0x3ddc6324, 0x3de299a2, 0x3de8d823,
    0x3def1e86, 0x3df56ca9, 0x3dfbc26a, 0x3e010fd6, 0x3e044227, 0x3e07781b, 0x3e0ab1a4, 0x3e0deeb2,
    0x3e112f3b, 0x3e147330, 0x3e17ba86, 0x3e1b0530, 0x3e1e5324, 0x3e21a457, 0x3e24f8bb, 0x3e285049,
    0x3e2baaf5, 0x3e2f08b5, 0x3e326981, 0x3e35cd4e, 0x3e393413, 0x3e3c9dc7, 0x3e400a62, 0x3e4379db,
    0x3e46ec2d, 0x3e4a614b, 0x3e4dd92f, 0x3e5153d2, 0x3e54d12c, 0x3e585137, 0x3e5bd3ea, 0x3e5f593f,
    0x3e62e131, 0x3e666bb4, 0x3e69f8ca, 0x3e6d8863, 0x3e711a82, 0x3e74af1b, 0x3e78462b, 0x3e7bdfa8,
    0x3e7f7b93, 0x3e818cef, 0x3e835d47, 0x3e852ec9, 0x3e870178, 0x3e88d54d, 0x3e8aaa48, 0x3e8c8065,
    0x3e8e57a4, 0x3e902fff, 0x3e920978, 0x3e93e40a, 0x3e95bfb3, 0x3e979c70, 0x3e997a41, 0x3e9b5921,
    0x3e9d3912, 0x3e9f1a0e, 0x3ea0fc15, 0x3ea2df24, 0x3ea4c33b, 0x3ea6a855, 0x3ea88e73, 0x3eaa7591,
    0x3eac5db0, 0x3eae46cb, 0x3eb030e2, 0x3eb21bf2, 0x3eb407fc, 0x3eb5f4fa, 0x3eb7e2ef, 0x3eb9d1d5,
    0x3ebbc1af, 0x3ebdb276, 0x3ebfa42e, 0x3ec196d1, 0x3ec38a62, 0x3ec57eda, 0x3ec7743d, 0x3ec96a85,
    0x3ecb61b5, 0x3ecd59c5, 0x3ecf52bc, 0x3ed14c90, 0x3ed34749, 0x3ed542dd, 0x3ed73f50, 0x3ed93c9e,
    0x3edb3ac8, 0x3edd39c9, 0x3edf39a6, 0x3ee13a56, 0x3ee33be0, 0x3ee53e3a, 0x3ee7416d, 0x3ee9456d,
    0x3eeb4a42, 0x3eed4fe3, 0x3eef5658, 0x3ef15d98, 0x3ef365a6, 0x3ef56e7c, 0x3ef77821, 0x3ef9828b,
    0x3efb8dc2, 0x3efd99bb, 0x3effa680, 0x3f00da03, 0x3f01e129, 0x3f02e8b0, 0x3f03f09a, 0x3f04f8e2,
    0x3f06018d, 0x3f070a95, 0x3f0813ff, 0x3f091dc5, 0x3f0a27ec, 0x3f0b326f, 0x3f0c3d51, 0x3f0d488f,
    0x3f0e542b, 0x3f0f6020, 0x3f106c74, 0x3f117921, 0x3f12862b, 0x3f13938e, 0x3f14a14c, 0x3f15af63,
    0x3f16bdd5, 0x3f17cc9d, 0x3f18dbc0, 0x3f19eb39, 0x3f1afb0b, 0x3f1c0b33, 0x3f1d1bb4, 0x3f1e2c89,
    0x3f1f3db7, 0x3f204f38, 0x3f216111, 0x3f22733d, 0x3f2385c0, 0x3f249894, 0x3f25abc0, 0x3f26bf3c,
    0x3f27d30f, 0x3f28e732, 0x3f29fba9, 0x3f2b1072, 0x3f2c258e, 0x3f2d3afa, 0x3f2e50b9, 0x3f2f66c7,
    0x3f307d28, 0x3f3193d8, 0x3f32aad9, 0x3f33c229, 0x3f34d9ca, 0x3f35f1b8, 0x3f3709f7, 0x3f382282,
    0x3f393b5e, 0x3f3a5486, 0x3f3b6dfd, 0x3f3c87bf, 0x3f3da1d2, 0x3f3ebc2e, 0x3f3fd6d9, 0x3f40f1cf,
    0x3f420d12, 0x3f43289f, 0x3f44447a, 0x3f45609e, 0x3f467d0e, 0x3f4799c8, 0x3f48b6ce, 0x3f49d41b,
    0x3f4af1b5, 0x3f4c0f96, 0x3f4d2dc3, 0x3f4e4c36, 0x3f4f6af5, 0x3f5089fa, 0x3f51a949, 0x3f52c8df,
    0x3f53e8bf, 0x3f5508e4, 0x3f562952, 0x3f574a05, 0x3f586b02, 0x3f598c42, 0x3f5aadcc, 0x3f5bcf99,
    0x3f5cf1af, 0x3f5e1408, 0x3f5f36a9, 0x3f60598d, 0x3f617cb8, 0x3f62a025, 0x3f63c3da, 0x3f64e7d1,
    0x3f660c0e, 0x3f67308d, 0x3f685552, 0x3f697a58, 0x3f6a9fa4, 0x3f6bc530, 0x3f6ceb02, 0x3f6e1115,
    0x3f6f376c, 0x3f705e04, 0x3f7184df, 0x3f72abfb, 0x3f73d359, 0x3f74faf8, 0x3f7622da, 0x3f774afb,
    0x3f78735f, 0x3f799c01, 0x3f7ac4e6, 0x3f7bee09, 0x3f7d176f, 0x3f7e4112, 0x3f7f6af8,
];

const BT709_APPLE_TO_SRGB_BUCKET_COUNT: usize = (1.0_f32.to_bits() >> 16) as usize + 1;

const fn bt709_apple_to_srgb_bucket_base() -> [u8; BT709_APPLE_TO_SRGB_BUCKET_COUNT] {
    let mut buckets = [0_u8; BT709_APPLE_TO_SRGB_BUCKET_COUNT];
    let mut bucket = 0;
    let mut value = 0;
    while bucket < buckets.len() {
        let first_bits = (bucket as u32) << 16;
        while value < BT709_APPLE_TO_SRGB_THRESHOLD_BITS.len()
            && first_bits >= BT709_APPLE_TO_SRGB_THRESHOLD_BITS[value]
        {
            value += 1;
        }
        buckets[bucket] = value as u8;
        bucket += 1;
    }
    buckets
}

const BT709_APPLE_TO_SRGB_BUCKET_BASE: [u8; BT709_APPLE_TO_SRGB_BUCKET_COUNT] =
    bt709_apple_to_srgb_bucket_base();

#[inline]
fn bt709_apple_to_srgb_byte(encoded: f32) -> u8 {
    let encoded_bits = encoded.clamp(0.0, 1.0).to_bits();
    let mut value = BT709_APPLE_TO_SRGB_BUCKET_BASE[(encoded_bits >> 16) as usize];
    while (value as usize) < BT709_APPLE_TO_SRGB_THRESHOLD_BITS.len()
        && encoded_bits >= BT709_APPLE_TO_SRGB_THRESHOLD_BITS[value as usize]
    {
        value += 1;
    }
    value
}

#[cfg(test)]
fn convert_nv12_bt709_apple_to_srgb(
    video: Nv12Planes<'_>,
    matrix_id: compositor_core::runtime::resources::ColorMatrixId,
) -> io::Result<Vec<u8>> {
    convert_nv12_bt709_apple_to_srgb_scaled(video, matrix_id, video.width, video.height)
}

fn convert_nv12_bt709_apple_to_srgb_scaled(
    video: Nv12Planes<'_>,
    matrix_id: compositor_core::runtime::resources::ColorMatrixId,
    output_width: usize,
    output_height: usize,
) -> io::Result<Vec<u8>> {
    use compositor_core::runtime::resources::ColorMatrixId;

    let chroma_width = video.width.div_ceil(2);
    let chroma_height = video.height.div_ceil(2);
    let y_bytes = video
        .y_stride
        .checked_mul(video.height)
        .ok_or_else(|| io::Error::other("translated video luma byte count overflow"))?;
    let uv_bytes = video
        .uv_stride
        .checked_mul(chroma_height)
        .ok_or_else(|| io::Error::other("translated video chroma byte count overflow"))?;
    if video.width == 0
        || video.height == 0
        || video.y_stride < video.width
        || video.uv_stride < chroma_width * 2
        || video.y_plane.len() < y_bytes
        || video.uv_plane.len() < uv_bytes
    {
        return Err(io::Error::other(
            "translated video planes do not match their dimensions",
        ));
    }

    // Source: third_party/skia/src/core/SkYUVMath.cpp with
    // SK_YUV_COLOR_SPACE_HIGH_PRECISION. from_bits preserves Chromium's
    // hexadecimal f32 constants exactly.
    let (matrix, translate) = match matrix_id {
        ColorMatrixId::Bt709 => (
            [
                [
                    f32::from_bits(0x3f95_0a86),
                    f32::from_bits(0xb1a9_95ba),
                    f32::from_bits(0x3fe5_788b),
                ],
                [
                    f32::from_bits(0x3f95_0a85),
                    f32::from_bits(0xbe5a_5dd7),
                    f32::from_bits(0xbf08_6cbf),
                ],
                [
                    f32::from_bits(0x3f95_0a85),
                    f32::from_bits(0x4007_3197),
                    f32::from_bits(0xb2b2_aaa5),
                ],
            ],
            [
                f32::from_bits(0xbf79_12ef),
                f32::from_bits(0x3e9a_5bf0),
                f32::from_bits(0xbf91_1353),
            ],
        ),
        ColorMatrixId::Smpte170M => (
            [
                [
                    f32::from_bits(0x3f95_0a86),
                    f32::from_bits(0xb077_764c),
                    f32::from_bits(0x3fcc_4a9b),
                ],
                [
                    f32::from_bits(0x3f95_0a86),
                    f32::from_bits(0xbec8_9512),
                    f32::from_bits(0xbf50_1ea7),
                ],
                [
                    f32::from_bits(0x3f95_0a86),
                    f32::from_bits(0x4001_1a55),
                    f32::from_bits(0x3296_f05b),
                ],
            ],
            [
                f32::from_bits(0xbf5f_cbb8),
                f32::from_bits(0x3f08_1b63),
                f32::from_bits(0xbf8a_f5f4),
            ],
        ),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "translated video renderer received an unsupported YUV matrix",
            ));
        }
    };
    let pixels = output_width
        .checked_mul(output_height)
        .and_then(|count| count.checked_mul(4))
        .ok_or_else(|| io::Error::other("translated video output byte count overflow"))?;
    let mut rgba = vec![0_u8; pixels];
    let source_x: Vec<f32> = (0..output_width)
        .map(|x| (x as f32 + 0.5) * video.width as f32 / output_width as f32 - 0.5)
        .collect();
    let luma_x: Vec<BilinearAxis> = source_x
        .iter()
        .map(|x| bilinear_axis(*x, video.width))
        .collect();
    // KeyHelpers.cpp configures centered siting and the Graphite shader keeps
    // UV coordinates in Y texel space. At a Y texel center that becomes
    // x/2-.25, y/2-.25 in UV texel space. Clamp mirrors the texture sampler.
    let chroma_x: Vec<BilinearAxis> = source_x
        .iter()
        .map(|x| bilinear_axis(*x * 0.5 - 0.25, chroma_width))
        .collect();
    let fill_rows = |start_y: usize, rows: &mut [u8]| {
        for (local_y, row) in rows.chunks_exact_mut(output_width * 4).enumerate() {
            let y = start_y + local_y;
            let source_y = (y as f32 + 0.5) * video.height as f32 / output_height as f32 - 0.5;
            let luma_y = bilinear_axis(source_y, video.height);
            let chroma_y = bilinear_axis(source_y * 0.5 - 0.25, chroma_height);
            for x in 0..output_width {
                let yuv = [
                    bilinear_plane_sample_precomputed(
                        video.y_plane,
                        video.y_stride,
                        luma_x[x],
                        luma_y,
                        1,
                        0,
                    ),
                    bilinear_plane_sample_precomputed(
                        video.uv_plane,
                        video.uv_stride,
                        chroma_x[x],
                        chroma_y,
                        2,
                        0,
                    ),
                    bilinear_plane_sample_precomputed(
                        video.uv_plane,
                        video.uv_stride,
                        chroma_x[x],
                        chroma_y,
                        2,
                        1,
                    ),
                ];
                let encoded = yuv_row(matrix, translate, yuv);
                let offset = x * 4;
                row[offset] = bt709_apple_to_srgb_byte(encoded[0]);
                row[offset + 1] = bt709_apple_to_srgb_byte(encoded[1]);
                row[offset + 2] = bt709_apple_to_srgb_byte(encoded[2]);
                row[offset + 3] = 255;
            }
        }
    };
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(8)
        .min(output_height.max(1));
    if workers > 1 && output_width.saturating_mul(output_height) >= 256 * 1024 {
        let rows_per_worker = output_height.div_ceil(workers);
        let bytes_per_worker = rows_per_worker * output_width * 4;
        std::thread::scope(|scope| {
            for (worker, rows) in rgba.chunks_mut(bytes_per_worker).enumerate() {
                scope.spawn(move || fill_rows(worker * rows_per_worker, rows));
            }
        });
    } else {
        fill_rows(0, &mut rgba);
    }
    Ok(rgba)
}

/// Software-output counterpart of Chromium's multiplanar SkYUVA image. Plane
/// sampling stays in Renderer (not Decode/Paint), uses centered 4:2:0 linear
/// chroma coordinates, then applies the resource's transfer into the sRGB
/// output surface. The Metal backend can consume the same lease without this
/// temporary materialization.
pub(crate) fn video_lease_image(
    lease: Arc<dyn RasterResourceLease>,
) -> io::Result<Arc<dyn SkImage>> {
    let size = lease.size();
    video_lease_image_scaled(lease, size.width(), size.height())
}

pub(crate) fn video_lease_image_scaled(
    lease: Arc<dyn RasterResourceLease>,
    output_width: i32,
    output_height: i32,
) -> io::Result<Arc<dyn SkImage>> {
    use compositor_core::runtime::resources::{ColorPrimaryId, ColorRangeId, ColorTransferId};
    let video = MapVideoResourceForRead(lease.as_ref())?;
    let width = video.size.width();
    let height = video.size.height();
    if width <= 0
        || height <= 0
        || video.color_space.range != ColorRangeId::Limited
        || video.color_space.transfer != ColorTransferId::Bt709Apple
        || video.color_space.primaries != ColorPrimaryId::Bt709
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated video renderer received an unsupported color space",
        ));
    }
    let width_usize = width as usize;
    let height_usize = height as usize;
    if output_width <= 0 || output_height <= 0 {
        return Err(io::Error::other("translated video output size is empty"));
    }
    let rgba = convert_nv12_bt709_apple_to_srgb_scaled(
        Nv12Planes {
            width: width_usize,
            height: height_usize,
            y_stride: video.y_stride,
            uv_stride: video.uv_stride,
            y_plane: video.y_plane,
            uv_plane: video.uv_plane,
        },
        video.color_space.matrix,
        output_width as usize,
        output_height as usize,
    )?;
    image_from_bytes(
        &rgba,
        output_width,
        output_height,
        output_width as usize * 4,
        SinglePlaneFormat::RGBA_8888,
    )
}

#[cfg(test)]
mod video_tests {
    use super::*;
    use compositor_core::runtime::resources::ColorMatrixId;

    #[test]
    fn bt709_apple_transfer_thresholds_preserve_exact_quantization() {
        for value in 0..=u16::MAX {
            let encoded = value as f32 / u16::MAX as f32;
            assert_eq!(
                bt709_apple_to_srgb_byte(encoded),
                bt709_apple_to_srgb_byte_slow(encoded),
                "encoded={encoded}"
            );
        }
        for threshold in BT709_APPLE_TO_SRGB_THRESHOLD_BITS {
            for bits in [threshold - 1, threshold, threshold + 1] {
                let encoded = f32::from_bits(bits);
                assert_eq!(
                    bt709_apple_to_srgb_byte(encoded),
                    bt709_apple_to_srgb_byte_slow(encoded),
                    "encoded_bits={bits:#010x}"
                );
            }
        }
    }

    #[test]
    fn bt709_apple_conversion_uses_normal_gpu_fp32_stages() {
        let rgba = convert_nv12_bt709_apple_to_srgb(
            Nv12Planes {
                width: 1,
                height: 1,
                y_stride: 1,
                uv_stride: 2,
                y_plane: &[16],
                uv_plane: &[16, 114],
            },
            ColorMatrixId::Bt709,
        )
        .unwrap();

        assert_eq!(rgba, [0, 34, 0, 255]);
    }

    #[test]
    fn centered_bilinear_chroma_matches_normal_gpu_pixel_fixture() {
        // Padding bytes make the fixture also verify that plane strides, not
        // logical widths, locate the following row.
        let y = [
            16, 64, 128, 235, 0xee, 0xee, 32, 80, 144, 220, 0xee, 0xee, 48, 96, 160, 200, 0xee,
            0xee, 64, 112, 176, 180, 0xee, 0xee,
        ];
        let uv = [16, 240, 128, 128, 0xdd, 0xdd, 240, 16, 90, 200, 0xdd, 0xdd];
        let rgba = convert_nv12_bt709_apple_to_srgb(
            Nv12Planes {
                width: 4,
                height: 4,
                y_stride: 6,
                uv_stride: 6,
                y_plane: &y,
                uv_plane: &uv,
            },
            ColorMatrixId::Bt709,
        )
        .unwrap();

        assert_eq!(
            rgba,
            [
                207, 0, 0, 255, 212, 32, 0, 255, 189, 133, 81, 255, 255, 255, 255, 255, 130, 0, 0,
                255, 168, 68, 0, 255, 205, 150, 116, 255, 255, 233, 222, 255, 0, 63, 166, 255, 48,
                112, 176, 255, 220, 165, 162, 255, 255, 199, 164, 255, 0, 103, 255, 255, 0, 142,
                255, 255, 236, 181, 193, 255, 255, 170, 122, 255,
            ]
        );
    }
}

pub(crate) fn target_format(format: PixelFormat) -> TranslatedPixelFormat {
    match format {
        PixelFormat::Rgba8888 => TranslatedPixelFormat::Rgba8888,
        PixelFormat::Bgra8888 => TranslatedPixelFormat::Bgra8888,
        PixelFormat::Bgrx8888 => TranslatedPixelFormat::Bgrx8888,
    }
}

pub(crate) fn validate_target(input: &VizOutput, target: &RenderTarget) -> io::Result<()> {
    let output = input.OutputRect();
    let length = target
        .row_stride
        .checked_mul(target.height as usize)
        .filter(|length| {
            output == Rect::new(0, 0, target.width as i32, target.height as i32)
                && target.row_stride >= target.width as usize
                && target.pixels.len() == *length
        })
        .ok_or_else(|| io::Error::other("aggregated frame does not match render target"))?;
    debug_assert_eq!(length, target.pixels.len());
    Ok(())
}

pub(crate) fn publish(
    input: &VizOutput,
    target: &mut RenderTarget,
    backings: &BTreeMap<PassId, PassBacking>,
    drawn_damage: &BTreeMap<PassId, Rect>,
) -> io::Result<()> {
    let frame = input.Frame();
    let full_bounds = Rect::new(0, 0, target.width as i32, target.height as i32);
    let root_passes: Vec<_> = frame
        .RenderPasses()
        .iter()
        .filter_map(|pass| {
            input
                .RootDestination(pass.RenderPassId())
                .map(|destination| (pass, destination))
        })
        .collect();
    // A software Display commonly publishes several already-rasterized root
    // planes into one host target (for example toolbar and document
    // surfaces). They are BGRA bitmaps in device pixels, so routing every
    // unscaled plane back through SkImage/Canvas turns a row copy into a full
    // image draw. Opaque planes can keep the same kSrc ordering and damage
    // clipping while copying directly into the borrowed output buffer.
    // Transparent roots must go through Skia so that their clear pixels do
    // not erase the roots below them.
    let direct_roots = target.format == PixelFormat::Bgra8888
        && root_passes.iter().all(|(pass, destination)| {
            let source = pass.OutputRect();
            let Some(backing) = backings.get(&pass.RenderPassId()) else {
                return false;
            };
            !pass.has_transparent_background
                && source.width == destination.width
                && source.height == destination.height
                && backing.rect.width >= source.width
                && backing.rect.height >= source.height
                && backing.drawn_rect.width == source.width
                && backing.drawn_rect.height == source.height
        });
    if direct_roots {
        if !target.retained {
            target.pixels.fill(u32::MAX);
        }
        for (pass, destination) in &root_passes {
            let source = pass.OutputRect();
            let backing = backings
                .get(&pass.RenderPassId())
                .ok_or_else(|| io::Error::other("completed translated root pass is unavailable"))?;
            let mut damage = if target.retained {
                let damage = drawn_damage
                    .get(&pass.RenderPassId())
                    .copied()
                    .unwrap_or_default();
                Rect::new(
                    destination
                        .x
                        .saturating_add(damage.x.saturating_sub(source.x)),
                    destination
                        .y
                        .saturating_add(damage.y.saturating_sub(source.y)),
                    damage.width,
                    damage.height,
                )
            } else {
                *destination
            };
            damage.Intersect(full_bounds);
            if damage.IsEmpty() {
                continue;
            }
            let source_x = damage.x.saturating_sub(destination.x) as usize;
            let source_y = damage.y.saturating_sub(destination.y) as usize;
            let row_pixels = damage.width as usize;
            for row in 0..damage.height as usize {
                let source_start = (source_y + row) * backing.rect.width as usize + source_x;
                let target_start =
                    (damage.y as usize + row) * target.row_stride + damage.x as usize;
                target.pixels[target_start..target_start + row_pixels]
                    .copy_from_slice(&backing.pixels[source_start..source_start + row_pixels]);
            }
        }
        return Ok(());
    }
    // SoftwareRenderer's usual Page output is one unscaled BGRA root pass.
    // Publishing it through a copied SkImage performs two full-frame memory
    // passes before the embedding even reaches its host target. The backing is
    // already complete immutable N32 output, so copy its valid damage rows
    // directly into the borrowed target, matching SoftwareOutputDevice's
    // same-format bitmap handoff.
    if target.format == PixelFormat::Bgra8888 && root_passes.len() == 1 {
        let (pass, destination) = root_passes[0];
        let backing = backings
            .get(&pass.RenderPassId())
            .ok_or_else(|| io::Error::other("completed translated root pass is unavailable"))?;
        if !pass.has_transparent_background
            && destination == full_bounds
            && backing.drawn_rect.width == destination.width
            && backing.drawn_rect.height == destination.height
        {
            let source = pass.OutputRect();
            let mut damage = if target.retained {
                let damage = drawn_damage
                    .get(&pass.RenderPassId())
                    .copied()
                    .unwrap_or_default();
                Rect::new(
                    damage.x.saturating_sub(source.x),
                    damage.y.saturating_sub(source.y),
                    damage.width,
                    damage.height,
                )
            } else {
                full_bounds
            };
            damage.Intersect(full_bounds);
            if !damage.IsEmpty() {
                let source_stride = backing.rect.width as usize;
                let target_stride = target.row_stride;
                let row_pixels = damage.width as usize;
                for y in damage.y as usize..damage.bottom() as usize {
                    let source_start = y * source_stride + damage.x as usize;
                    let target_start = y * target_stride + damage.x as usize;
                    target.pixels[target_start..target_start + row_pixels]
                        .copy_from_slice(&backing.pixels[source_start..source_start + row_pixels]);
                }
            }
            return Ok(());
        }
    }
    let format = target_format(target.format);
    let row_bytes = target
        .row_stride
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("translated target row byte overflow"))?;
    let pixels = bytemuck::cast_slice_mut(target.pixels.as_mut_slice());
    let mut output = RasterTarget::new(
        pixels,
        target.width as i32,
        target.height as i32,
        row_bytes,
        format,
        None,
    )
    .map_err(|error| raster_error("translated host target creation failed", error))?;
    if !target.retained {
        output
            .draw_color(
                &SkColor4f {
                    fR: 1.,
                    fG: 1.,
                    fB: 1.,
                    fA: 1.,
                },
                SkBlendMode::kSrc,
            )
            .map_err(|error| raster_error("translated host clear failed", error))?;
    }
    for (pass, destination) in root_passes {
        let backing = backings
            .get(&pass.RenderPassId())
            .ok_or_else(|| io::Error::other("completed translated root pass is unavailable"))?;
        let mut damage = if target.retained {
            let source = pass.OutputRect();
            let damage = drawn_damage
                .get(&pass.RenderPassId())
                .copied()
                .unwrap_or_default();
            if source.width == destination.width && source.height == destination.height {
                Rect::new(
                    destination
                        .x
                        .saturating_add(damage.x.saturating_sub(source.x)),
                    destination
                        .y
                        .saturating_add(damage.y.saturating_sub(source.y)),
                    damage.width,
                    damage.height,
                )
            } else {
                destination
            }
        } else {
            destination
        };
        damage.Intersect(full_bounds);
        if damage.IsEmpty() {
            continue;
        }
        if backing.drawn_rect.IsEmpty() {
            return Err(io::Error::other(
                "translated root pass has no completed drawn rectangle",
            ));
        }
        let image = backing.image()?;
        let source = SkRect::MakeLTRB(
            0.,
            0.,
            backing.drawn_rect.width as f32,
            backing.drawn_rect.height as f32,
        );
        output.save();
        output.clip_rect(&rect(damage), SkClipOp::kIntersect, false);
        let mut paint = SkPaint::default();
        paint.setBlendMode(if pass.has_transparent_background {
            SkBlendMode::kSrcOver
        } else {
            SkBlendMode::kSrc
        });
        paint.setAntiAlias(false);
        let sampling = if destination.width == backing.drawn_rect.width
            && destination.height == backing.drawn_rect.height
        {
            SkSamplingOptions::with_filter(SkFilterMode::kNearest)
        } else {
            SkSamplingOptions::with_filter(SkFilterMode::kLinear)
        };
        unsafe {
            output.draw_image_rect(
                &image,
                Some(&source),
                &rect(destination),
                &sampling,
                Some(&paint),
                SrcRectConstraint::kStrict_SrcRectConstraint,
            )
        }
        .map_err(|error| raster_error("translated root publication failed", error))?;
        output
            .restore()
            .map_err(|error| raster_error("translated root publication restore failed", error))?;
    }
    output
        .finish()
        .map_err(|error| raster_error("translated host target finish failed", error))
}
