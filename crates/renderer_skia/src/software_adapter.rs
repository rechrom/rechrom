//! Local resource/target and Skia capability adapters for translated Renderer.
//! These adapters are not counted as translated source. Native engine input is
//! borrowed; completed pass pixels are owned once by the software backend.
use compositor_core::{
    adapter::{
        geometry::{Rect, RectF},
        layer_tree_impl::{Color4f, ResourceId},
        property_types::BlendMode,
    },
    runtime::cpu_raster::CpuRasterResourceLease,
};
use skia::{
    compat::commands::{self as command, CommandKind, DrawCommand, PaintRect, ResourceContext},
    cpu::canvas::Canvas,
    raster::{FilterQuality, Paint, Pattern, PixmapMut, PixmapRef, Transform as SkTransform},
};
use std::{collections::BTreeMap, io};
use viz::native::VizOutput;
pub(crate) type PassId = compositor_core::runtime::frame::RenderPassId;

use crate::native::RenderTarget;
pub(crate) struct PassBacking {
    pub(crate) rect: Rect,
    pub(crate) pixels: Vec<u8>,
    pub(crate) drawn_rect: Rect,
}
impl PassBacking {
    pub(crate) fn new(rect: Rect) -> io::Result<Self> {
        let len = (rect.width as usize)
            .checked_mul(rect.height as usize)
            .and_then(|v| v.checked_mul(4))
            .filter(|v| *v <= isize::MAX as usize)
            .ok_or_else(|| io::Error::other("native pass allocation overflow"))?;
        Ok(Self {
            rect,
            pixels: vec![0; len],
            drawn_rect: Rect::default(),
        })
    }
    pub(crate) fn pixmap(&self) -> PixmapRef<'_> {
        PixmapRef::from_bytes_with_format(
            &self.pixels,
            self.rect.width as u32,
            self.rect.height as u32,
            skia::PixelFormat::Bgra8888,
        )
        .expect("actual N32 pass backing")
    }
}

pub(crate) fn publish(
    input: &VizOutput,
    target: RenderTarget<'_>,
    backings: &BTreeMap<PassId, PassBacking>,
    drawn_damage: &BTreeMap<PassId, Rect>,
) -> io::Result<()> {
    let frame = input.Frame();
    let pixels = bytemuck::cast_slice_mut(target.pixels);
    let mut output = PixmapMut::from_bytes_with_format(
        pixels,
        target.row_stride as u32,
        target.height,
        target.format,
    )
    .ok_or_else(|| io::Error::other("invalid borrowed host N32 target"))?;
    let full_bounds = Rect::new(0, 0, target.width as i32, target.height as i32);
    if !target.retained {
        let mut background = Paint::default();
        background.set_color_rgba8(255, 255, 255, 255);
        background.blend_mode = skia::BlendMode::kSrc;
        background.anti_alias = false;
        let bounds =
            skia::raster::Rect::from_xywh(0., 0., target.width as f32, target.height as f32)
                .expect("validated output bounds");
        output.fill_rect(bounds, &background, SkTransform::identity(), None);
    }
    for (pass, d) in frame.RenderPasses().iter().filter_map(|pass| {
        input
            .RootDestination(pass.RenderPassId())
            .map(|destination| (pass, destination))
    }) {
        let backing = backings
            .get(&pass.RenderPassId())
            .ok_or_else(|| io::Error::other("completed root pass is unavailable"))?;
        let mut damage = if target.retained {
            let source = pass.OutputRect();
            let damage = drawn_damage
                .get(&pass.RenderPassId())
                .copied()
                .unwrap_or_default();
            if source.width == d.width && source.height == d.height {
                Rect::new(
                    d.x.saturating_add(damage.x.saturating_sub(source.x)),
                    d.y.saturating_add(damage.y.saturating_sub(source.y)),
                    damage.width,
                    damage.height,
                )
            } else {
                d
            }
        } else {
            d
        };
        damage.Intersect(full_bounds);
        if damage.IsEmpty() {
            continue;
        }
        if target.format == skia::PixelFormat::Bgra8888
            && d.width == backing.drawn_rect.width
            && d.height == backing.drawn_rect.height
            && d.x >= 0
            && d.y >= 0
        {
            // SoftwareRenderer's completed root device is already the
            // final surface. Skia's matching-info path is rect_memcpy;
            // replaying it through an image shader needlessly resamples
            // every output pixel and can blend retained transparency twice.
            let source_x = damage.x - d.x;
            let source_y = damage.y - d.y;
            let bytes = output.data_mut();
            let row_bytes = damage.width as usize * 4;
            for row in 0..damage.height as usize {
                let src = ((source_y as usize + row) * backing.rect.width as usize
                    + source_x as usize)
                    * 4;
                let dst = ((damage.y as usize + row) * target.row_stride + damage.x as usize) * 4;
                bytes[dst..dst + row_bytes].copy_from_slice(&backing.pixels[src..src + row_bytes]);
            }
            continue;
        }
        let sx = d.width as f32 / backing.drawn_rect.width as f32;
        let sy = d.height as f32 / backing.drawn_rect.height as f32;
        let paint = Paint {
            // A retained backing may be larger than the completed root after
            // a shrink. Sample only the logical image, not its stale padding.
            shader: Pattern::new_native_subset(
                backing.pixmap(),
                [
                    0.,
                    0.,
                    backing.drawn_rect.width as f32,
                    backing.drawn_rect.height as f32,
                ],
                FilterQuality::Bilinear,
                1.,
                SkTransform::from_row(sx, 0., 0., sy, d.x as f32, d.y as f32),
            ),
            blend_mode: skia::BlendMode::kSrc,
            anti_alias: false,
            ..Paint::default()
        };
        if let Some(bounds) = skia::raster::Rect::from_xywh(
            damage.x as f32,
            damage.y as f32,
            damage.width as f32,
            damage.height as f32,
        ) {
            output.fill_rect(bounds, &paint, SkTransform::identity(), None);
        }
    }

    Ok(())
}

pub(crate) fn simple(canvas: &mut Canvas, kind: CommandKind) {
    canvas.replay_item(
        &DrawCommand {
            r#type: kind,
            ..Default::default()
        },
        &ResourceContext::default(),
    );
}
pub(crate) fn rect(r: Rect) -> PaintRect {
    PaintRect {
        x: r.x as f64,
        y: r.y as f64,
        width: r.width as f64,
        height: r.height as f64,
    }
}
pub(crate) fn rectf(r: RectF) -> PaintRect {
    PaintRect {
        x: r.x() as f64,
        y: r.y() as f64,
        width: r.width() as f64,
        height: r.height() as f64,
    }
}
pub(crate) fn color(c: Color4f, opacity: f32) -> command::Color {
    command::Color {
        red: c.red,
        green: c.green,
        blue: c.blue,
        alpha: c.alpha * opacity,
    }
}
pub(crate) fn clip_rect(canvas: &mut Canvas, r: Rect) {
    canvas.replay_item(
        &DrawCommand {
            r#type: CommandKind::kClipRect,
            rect: rect(r),
            antialias: false,
            ..Default::default()
        },
        &ResourceContext::default(),
    );
}
pub(crate) fn native_transform(
    transform: &foundation::gfx::Transform,
    origin: Rect,
) -> io::Result<SkTransform> {
    // The current CPU Skia matrix backend is affine. Do not silently discard a
    // legal perspective matrix while that native primitive is unavailable.
    if transform.rc(3, 0) != 0. || transform.rc(3, 1) != 0. || transform.rc(3, 3) != 1. {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Skia CPU perspective matrix primitive is not exposed",
        ));
    }
    Ok(SkTransform::from_row(
        transform.rc(0, 0) as f32,
        transform.rc(1, 0) as f32,
        transform.rc(0, 1) as f32,
        transform.rc(1, 1) as f32,
        transform.rc(0, 3) as f32 - origin.x as f32,
        transform.rc(1, 3) as f32 - origin.y as f32,
    ))
}
pub(crate) fn resource<'a>(
    input: &'a VizOutput,
    id: ResourceId,
) -> io::Result<std::sync::Arc<dyn CpuRasterResourceLease<PixelFormat = skia::PixelFormat>>> {
    input
        .Resource(id)
        .ok_or_else(|| io::Error::other("native frame resource has no CPU lease"))
}
pub(crate) fn image_view(
    lease: &dyn CpuRasterResourceLease<PixelFormat = skia::PixelFormat>,
) -> io::Result<PixmapRef<'_>> {
    let size = lease.size();
    if lease.row_bytes() != size.width() as usize * 4 {
        return Err(io::Error::other(
            "native CPU image row bytes are not tightly packed",
        ));
    }
    PixmapRef::from_bytes_with_format(
        lease.pixels(),
        size.width() as u32,
        size.height() as u32,
        lease.format(),
    )
    .ok_or_else(|| io::Error::other("invalid actual native image pixels"))
}

pub(crate) fn skia_blend_mode(mode: BlendMode) -> skia::BlendMode {
    raster::native_filters::NativeBlendMode(mode)
}
pub(crate) fn validate_target(input: &VizOutput, target: &RenderTarget<'_>) -> io::Result<()> {
    let output = input.OutputRect();
    let length = target
        .row_stride
        .checked_mul(target.height as usize)
        .filter(|&length| {
            output == Rect::new(0, 0, target.width as i32, target.height as i32)
                && target.row_stride >= target.width as usize
                && target.pixels.len() == length
        })
        .ok_or_else(|| io::Error::other("aggregated frame does not match render target"))?;
    debug_assert_eq!(length, target.pixels.len());
    Ok(())
}
