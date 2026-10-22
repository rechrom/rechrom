//! Host-resource adapters for the source-aligned translated Skia renderer.
//!
//! Viz retains pass topology and immutable resource leases. This module only
//! converts those borrowed values into the concrete translated primitives
//! used for one synchronous draw or publication call.

use crate::native::{PixelFormat, RenderTarget};
use compositor_core::{
    adapter::{
        geometry::{RRectF, Rect, RectF},
        layer_tree_impl::{Color4f, ResourceId},
        property_types::BlendMode,
        resource_types::{SharedImageFormat, SinglePlaneFormat},
    },
    runtime::{frame::RenderPassId, raster_execution::RasterResourceLease},
};
use raster::MapResourceForRead;
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
use viz::native::VizOutput;

pub(crate) type PassId = RenderPassId;

pub(crate) struct PassBacking {
    pub(crate) rect: Rect,
    pub(crate) pixels: Vec<u8>,
    pub(crate) drawn_rect: Rect,
}

impl PassBacking {
    pub(crate) fn new(rect: Rect) -> io::Result<Self> {
        let length = (rect.width as usize)
            .checked_mul(rect.height as usize)
            .and_then(|value| value.checked_mul(4))
            .filter(|value| *value <= isize::MAX as usize)
            .ok_or_else(|| io::Error::other("translated render-pass allocation overflow"))?;
        Ok(Self {
            rect,
            pixels: vec![0; length],
            drawn_rect: Rect::default(),
        })
    }

    pub(crate) fn image(&self) -> io::Result<Arc<dyn SkImage>> {
        image_from_bytes(
            &self.pixels,
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

fn target_format(format: PixelFormat) -> TranslatedPixelFormat {
    match format {
        PixelFormat::Rgba8888 => TranslatedPixelFormat::Rgba8888,
        PixelFormat::Bgra8888 => TranslatedPixelFormat::Bgra8888,
        PixelFormat::Bgrx8888 => TranslatedPixelFormat::Bgrx8888,
    }
}

pub(crate) fn validate_target(input: &VizOutput, target: &RenderTarget<'_>) -> io::Result<()> {
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
    target: RenderTarget<'_>,
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
        if destination == full_bounds
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
                let source_stride = backing.rect.width as usize * 4;
                let target_stride = target.row_stride * 4;
                let row_bytes = damage.width as usize * 4;
                let target_bytes: &mut [u8] = bytemuck::cast_slice_mut(target.pixels);
                for y in damage.y as usize..damage.bottom() as usize {
                    let source_start = y * source_stride + damage.x as usize * 4;
                    let target_start = y * target_stride + damage.x as usize * 4;
                    target_bytes[target_start..target_start + row_bytes].copy_from_slice(
                        &backing.pixels[source_start..source_start + row_bytes],
                    );
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
    let pixels = bytemuck::cast_slice_mut(target.pixels);
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
        paint.setBlendMode(SkBlendMode::kSrc);
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
