//! Display-list replay into owned or externally provided Skia CPU targets.
pub use paint::paint_engine::PaintArtifact;
pub use skia::compat::surface::RasterSurface;
use skia::src::core::SkCanvas::RasterImageCache;
pub use skia::PixelFormat;
use skia::RasterClipProductCache;

/// Integer device-space region whose old pixels must be reconstructed. The
/// caller must retain the previous opaque frame everywhere outside this region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RasterDamage {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Replay with an initial device-space semantic clip. Clearing and drawing use
/// the full path's white opaque backing. This is NOT an independent write
/// scissor: even native Skia can round an AA edge differently when a clip cuts
/// its geometry. Callers seeking exact full-frame pixels must close damage over
/// complete intersecting AA and alpha-layer support first, or replay fully.
/// Spatial filters likewise require full replay because they read outside it.
#[allow(non_snake_case)]
pub fn RenderDisplayItemListDamageIntoWindowBufferWithFormat(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    damage: &[RasterDamage],
) -> std::io::Result<()> {
    RenderDisplayItemListDamageIntoWindowBufferWithStride(
        list,
        width,
        height,
        scale,
        buffer,
        format,
        damage,
        width as usize,
    )
}

#[allow(non_snake_case)]
pub fn RenderDisplayItemListDamageIntoWindowBufferWithStride(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    damage: &[RasterDamage],
    row_stride: usize,
) -> std::io::Result<()> {
    render_damage_into_window_buffer(
        list, width, height, scale, buffer, format, damage, row_stride, None, None, false,
    )
}

pub(crate) fn render_damage_into_window_buffer_cached(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    damage: &[RasterDamage],
    row_stride: usize,
    image_cache: &mut RasterImageCache,
) -> std::io::Result<()> {
    render_damage_into_window_buffer(
        list,
        width,
        height,
        scale,
        buffer,
        format,
        damage,
        row_stride,
        Some(image_cache),
        None,
        false,
    )
}

// Call only with independently proved semantic-support closure.
// It is not a final-write scissor and arbitrary damage is still unsupported.
pub(crate) fn render_damage_into_window_buffer_cached_union(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    damage: &[RasterDamage],
    row_stride: usize,
    image_cache: &mut RasterImageCache,
) -> std::io::Result<()> {
    render_damage_into_window_buffer(
        list,
        width,
        height,
        scale,
        buffer,
        format,
        damage,
        row_stride,
        Some(image_cache),
        None,
        true,
    )
}

fn render_damage_into_window_buffer(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    damage: &[RasterDamage],
    row_stride: usize,
    mut image_cache: Option<&mut RasterImageCache>,
    mut clip_cache: Option<&mut RasterClipProductCache>,
    union: bool,
) -> std::io::Result<()> {
    let storage_width = u32::try_from(row_stride)
        .ok()
        .filter(|&s| s >= width && width > 0)
        .ok_or_else(|| std::io::Error::other("invalid damage target stride"))?;
    let expected = row_stride
        .checked_mul(height as usize)
        .ok_or_else(|| std::io::Error::other("damage target dimensions overflow"))?;
    if buffer.len() != expected
        || expected == 0
        || !cfg!(target_endian = "little")
        || !scale.is_finite()
        || scale <= 0.0
    {
        return Err(std::io::Error::other("unsupported damage raster target"));
    }
    if damage.is_empty() {
        return Ok(());
    }
    let bytes = expected
        .checked_mul(4)
        .filter(|&n| n <= isize::MAX as usize)
        .ok_or_else(|| std::io::Error::other("damage target bytes overflow"))?;
    let pointer = std::ptr::NonNull::new(buffer.as_mut_ptr().cast::<u8>()).unwrap();
    // SAFETY: exclusively borrowed initialized pixels; the canvas and inert
    // external owner are consumed before this function returns.
    let mut target =
        unsafe { skia::PixelStorage::from_external(pointer, bytes, Box::new(())) }.unwrap();
    let resources = crate::convert::resources(list);
    use crate::convert::ToSkia;
    use skia::compat::commands::{Color, CommandKind, DrawCommand, PaintRect};
    use skia::compat::commands::{Offset, PaintPathCommand, PaintPathVerb};
    for batch in damage.chunks(if union { damage.len() } else { 1 }) {
        let clips: Vec<_> = batch
            .iter()
            .filter_map(|r| {
                let right = r.x.saturating_add(r.width).min(width);
                let bottom = r.y.saturating_add(r.height).min(height);
                (r.x < right && r.y < bottom).then_some(PaintRect {
                    x: r.x as f64,
                    y: r.y as f64,
                    width: (right - r.x) as f64,
                    height: (bottom - r.y) as f64,
                })
            })
            .collect();
        if clips.is_empty() {
            continue;
        }
        let mut canvas = skia::cpu::canvas::Canvas::make_raster_direct_with_format_preserving(
            &resources,
            storage_width,
            height,
            row_stride * 4,
            target,
            format,
            true,
        )
        .ok_or_else(|| std::io::Error::other("invalid preserving raster target"))?;
        if let Some(cache) = image_cache.as_deref_mut() {
            canvas.install_image_cache(std::mem::take(cache));
        }
        if let Some(cache) = clip_cache.as_deref_mut() {
            canvas.install_clip_product_cache(std::mem::take(cache));
        }
        let clip = if clips.len() == 1 {
            DrawCommand {
                r#type: CommandKind::kClipRect,
                rect: clips[0],
                antialias: false,
                ..Default::default()
            }
        } else {
            // Same-direction Winding rectangles form one BW device region.
            // No scale is installed until after this integer union clip.
            let path = clips
                .iter()
                .flat_map(|r| {
                    let (left, top, right, bottom) = (r.x, r.y, r.x + r.width, r.y + r.height);
                    [
                        PaintPathCommand {
                            verb: PaintPathVerb::kMoveTo,
                            point: Offset { x: left, y: top },
                            ..Default::default()
                        },
                        PaintPathCommand {
                            verb: PaintPathVerb::kLineTo,
                            point: Offset { x: right, y: top },
                            ..Default::default()
                        },
                        PaintPathCommand {
                            verb: PaintPathVerb::kLineTo,
                            point: Offset {
                                x: right,
                                y: bottom,
                            },
                            ..Default::default()
                        },
                        PaintPathCommand {
                            verb: PaintPathVerb::kLineTo,
                            point: Offset { x: left, y: bottom },
                            ..Default::default()
                        },
                        PaintPathCommand {
                            verb: PaintPathVerb::kClose,
                            ..Default::default()
                        },
                    ]
                })
                .collect();
            DrawCommand {
                r#type: CommandKind::kClipPath,
                path,
                antialias: false,
                even_odd: false,
                ..Default::default()
            }
        };
        canvas.replay_item(&clip, &resources);
        canvas.replay_item(
            &DrawCommand {
                r#type: CommandKind::kDrawRect,
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: width as f64,
                    height: height as f64,
                },
                color: Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                antialias: false,
                ..Default::default()
            },
            &resources,
        );
        canvas.set_scale(scale);
        for item in list.items.iter() {
            canvas.replay_item(&item.to_skia(), &resources);
        }
        if let Some(cache) = image_cache.as_deref_mut() {
            *cache = canvas.take_image_cache();
        }
        if let Some(cache) = clip_cache.as_deref_mut() {
            *cache = canvas.take_clip_product_cache();
        }
        target = canvas.finish_direct();
    }
    drop(target.into_external_owner());
    Ok(())
}
/// Replay into an exclusively owned host mapping and return that same mapping.
/// Target encoding is selected at load/store boundaries; readback stays separate.
#[allow(non_snake_case)]
pub fn RenderDisplayItemListIntoTarget(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    pixels: skia::PixelStorage,
    format: skia::PixelFormat,
) -> std::io::Result<skia::PixelStorage> {
    render_into_target(
        list, width, height, width, scale, pixels, format, None, None,
    )
}

// Skia's restricted pixmap stores tightly packed rows. Wrap an aligned host
// row as a wider raster, then clip replay to the actual visible width. Padding
// stays in the original mapping; there is no readback or row-copy pass.
fn render_into_target(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    storage_width: u32,
    scale: f64,
    pixels: skia::PixelStorage,
    format: skia::PixelFormat,
    mut image_cache: Option<&mut RasterImageCache>,
    mut clip_cache: Option<&mut RasterClipProductCache>,
) -> std::io::Result<skia::PixelStorage> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(std::io::Error::other("invalid raster target scale"));
    }
    let row_bytes = (storage_width as usize)
        .checked_mul(4)
        .ok_or_else(|| std::io::Error::other("raster target stride overflows"))?;
    let resources = crate::convert::resources(list);
    let mut canvas = skia::cpu::canvas::Canvas::make_raster_direct_with_format(
        &resources,
        storage_width,
        height,
        row_bytes,
        pixels,
        format,
    )
    .ok_or_else(|| std::io::Error::other("invalid direct raster target"))?;
    if let Some(cache) = image_cache.as_deref_mut() {
        canvas.install_image_cache(std::mem::take(cache));
    }
    if let Some(cache) = clip_cache.as_deref_mut() {
        canvas.install_clip_product_cache(std::mem::take(cache));
    }
    use crate::convert::ToSkia;
    if storage_width != width {
        canvas.replay_item(
            &paint::paint_engine::DisplayItem {
                r#type: paint::paint_engine::DisplayItemType::kClipRect,
                rect: paint::paint_engine::PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: f64::from(width),
                    height: f64::from(height),
                },
                ..Default::default()
            }
            .to_skia(),
            &resources,
        );
    }
    canvas.set_scale(scale);
    #[cfg(feature = "profiling")]
    let trace = std::env::var_os("RENDERER_TRACE_ITEMS").is_some();
    #[cfg(feature = "profiling")]
    let totals = std::env::var_os("RENDERER_TRACE_TOTALS").is_some();
    #[cfg(feature = "profiling")]
    let trace_threshold = std::time::Duration::from_micros(
        std::env::var("RENDERER_TRACE_ITEM_THRESHOLD_US")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(5_000),
    );
    #[cfg(feature = "profiling")]
    let mut kinds = totals.then(|| {
        vec![
            None::<(
                paint::paint_engine::DisplayItemType,
                std::time::Duration,
                usize
            )>;
            list.items
                .iter()
                .map(|item| item.r#type as usize)
                .max()
                .unwrap_or(0)
                + 1
        ]
    });
    for (_index, item) in list.items.iter().enumerate() {
        #[cfg(feature = "profiling")]
        let started = (trace || totals).then(std::time::Instant::now);
        canvas.replay_item(&item.to_skia(), &resources);
        #[cfg(feature = "profiling")]
        if let Some(started) = started {
            let elapsed = started.elapsed();
            if let Some(kinds) = kinds.as_mut() {
                let entry = kinds[item.r#type as usize].get_or_insert((
                    item.r#type,
                    std::time::Duration::ZERO,
                    0,
                ));
                entry.1 += elapsed;
                entry.2 += 1;
            }
            if trace && elapsed > trace_threshold {
                eprintln!(
                    "raster-item index={_index} kind={:?} ms={:.3} rect={:?}",
                    item.r#type,
                    elapsed.as_secs_f64() * 1000.0,
                    item.rect
                );
                if item.r#type == paint::paint_engine::DisplayItemType::kDrawBoxShadow {
                    eprintln!(
                        "raster-shadow blur={} spread={} inset={} radii={:?} offset={:?}",
                        item.blur_radius,
                        item.spread,
                        item.inset,
                        item.corner_radii,
                        item.shadow_offset
                    );
                }
            }
        }
    }
    #[cfg(feature = "profiling")]
    if let Some(kinds) = kinds {
        let mut kinds: Vec<_> = kinds.into_iter().flatten().collect();
        kinds.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        let sum: std::time::Duration = kinds.iter().map(|entry| entry.1).sum();
        eprintln!(
            "raster-summary width={width} height={height} scale={scale} items={} replay_ms={:.3}",
            list.items.len(),
            sum.as_secs_f64() * 1000.0
        );
        for (kind, elapsed, count) in kinds {
            eprintln!(
                "raster-kind kind={kind:?} count={count} ms={:.3}",
                elapsed.as_secs_f64() * 1000.0
            );
        }
    }
    if let Some(cache) = image_cache {
        *cache = canvas.take_image_cache();
    }
    if let Some(cache) = clip_cache {
        *cache = canvas.take_clip_product_cache();
    }
    Ok(canvas.finish_direct())
}

#[allow(non_snake_case)]
pub fn RenderDisplayItemListIntoWindowBuffer(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
) -> std::io::Result<()> {
    RenderDisplayItemListIntoWindowBufferWithFormat(
        list,
        width,
        height,
        scale,
        buffer,
        skia::PixelFormat::Bgrx8888,
    )
}

#[allow(non_snake_case)]
pub fn RenderDisplayItemListIntoWindowBufferWithFormat(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
) -> std::io::Result<()> {
    RenderDisplayItemListIntoWindowBufferWithStride(
        list,
        width,
        height,
        scale,
        buffer,
        format,
        width as usize,
    )
}

/// Replay directly into rows with platform padding, measured in u32 pixels.
#[allow(non_snake_case)]
pub fn RenderDisplayItemListIntoWindowBufferWithStride(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    row_stride: usize,
) -> std::io::Result<()> {
    render_into_window_buffer(
        list, width, height, scale, buffer, format, row_stride, None, None,
    )
}

pub(crate) fn render_into_window_buffer_cached(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    row_stride: usize,
    image_cache: &mut RasterImageCache,
) -> std::io::Result<()> {
    render_into_window_buffer(
        list,
        width,
        height,
        scale,
        buffer,
        format,
        row_stride,
        Some(image_cache),
        None,
    )
}

fn render_into_window_buffer(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    row_stride: usize,
    image_cache: Option<&mut RasterImageCache>,
    clip_cache: Option<&mut RasterClipProductCache>,
) -> std::io::Result<()> {
    let storage_width = u32::try_from(row_stride)
        .ok()
        .filter(|&stride| width > 0 && stride >= width)
        .ok_or_else(|| std::io::Error::other("invalid window target stride"))?;
    let expected = row_stride
        .checked_mul(height as usize)
        .ok_or_else(|| std::io::Error::other("window target dimensions overflow"))?;
    if buffer.len() != expected || expected == 0 || !cfg!(target_endian = "little") {
        return Err(std::io::Error::other(
            "unsupported window buffer dimensions or byte order",
        ));
    }
    let bytes = expected
        .checked_mul(4)
        .filter(|&n| n <= isize::MAX as usize)
        .ok_or_else(|| std::io::Error::other("window target bytes overflow"))?;
    let ptr = std::ptr::NonNull::new(buffer.as_mut_ptr().cast::<u8>()).unwrap();
    // SAFETY: the caller's exclusive slice owns initialized u32 pixels for the
    // whole call. Storage/canvas cannot escape; they are consumed before this
    // function returns or before the slice borrow ends during unwinding. The
    // inert owner token has no allocator/deallocator responsibilities here.
    let target = unsafe { skia::PixelStorage::from_external(ptr, bytes, Box::new(())) }.unwrap();
    let result = render_into_target(
        list,
        width,
        height,
        storage_width,
        scale,
        target,
        format,
        image_cache,
        clip_cache,
    )?;
    drop(result.into_external_owner());
    Ok(())
}

#[allow(non_snake_case)]
pub fn ClearWindowBuffer(width: u32, height: u32, buffer: &mut [u32]) -> std::io::Result<()> {
    RenderDisplayItemListIntoWindowBuffer(&PaintArtifact::default(), width, height, 1.0, buffer)
}
#[allow(non_snake_case)]
pub fn RenderDisplayItemList(surface: &mut RasterSurface, list: &PaintArtifact, scale: f64) {
    let (width, height) = surface.size();
    surface.replace_pixels(crate::pure_replay::RasterizeSourceDisplayItemListWithScale(
        list, width, height, scale,
    ));
}
/// Render a complete new frame and transfer its CPU readback buffer.
#[allow(non_snake_case)]
pub fn RenderDisplayItemListToSurface(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
) -> std::io::Result<RasterSurface> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|count| count.checked_mul(4))
        .filter(|_| width > 0 && height > 0)
        .ok_or_else(|| std::io::Error::other("invalid raster surface dimensions"))?;
    RasterSurface::from_pixels(
        width,
        height,
        crate::pure_replay::RasterizeSourceDisplayItemListWithScale(list, width, height, scale),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::internal::layout_input_types::Color;
    use paint::paint_engine::{DisplayItem, DisplayItemType, PaintRect};

    #[test]
    fn aligned_window_rows_match_packed_replay_and_keep_guards() {
        for width in [1u32, 7, 63, 65, 319, 321, 827, 1061] {
            for scale in [1.0, 1.5, 2.0] {
                let height = 19;
                let stride = (width as usize + 15) & !15;
                let list = PaintArtifact {
                    items: vec![DisplayItem {
                        r#type: DisplayItemType::kDrawRoundedRect,
                        rect: PaintRect {
                            x: 0.25,
                            y: 1.25,
                            width: f64::from(width),
                            height: 6.75,
                        },
                        color: Color {
                            red: 0.13,
                            green: 0.62,
                            blue: 0.89,
                            alpha: 0.71,
                        },
                        ..Default::default()
                    }]
                    .into(),
                    ..Default::default()
                };
                for format in [skia::PixelFormat::Bgrx8888, skia::PixelFormat::Bgra8888] {
                    let mut packed = vec![0; width as usize * height as usize];
                    RenderDisplayItemListIntoWindowBufferWithFormat(
                        &list,
                        width,
                        height,
                        scale,
                        &mut packed,
                        format,
                    )
                    .unwrap();
                    let len = stride * height as usize;
                    let mut guarded = vec![0x81b793c5; len + 6];
                    let ptr = guarded[3..].as_ptr();
                    RenderDisplayItemListIntoWindowBufferWithStride(
                        &list,
                        width,
                        height,
                        scale,
                        &mut guarded[3..3 + len],
                        format,
                        stride,
                    )
                    .unwrap();
                    assert_eq!(ptr, guarded[3..].as_ptr());
                    assert!(guarded[..3]
                        .iter()
                        .chain(&guarded[3 + len..])
                        .all(|&p| p == 0x81b793c5));
                    for (actual, expected) in guarded[3..3 + len]
                        .chunks_exact(stride)
                        .zip(packed.chunks_exact(width as usize))
                    {
                        assert_eq!(
                            &actual[..width as usize],
                            expected,
                            "width={width} scale={scale} format={format:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn invalid_window_stride_is_rejected() {
        for stride in [0, 2, usize::MAX] {
            assert!(RenderDisplayItemListIntoWindowBufferWithStride(
                &PaintArtifact::default(),
                3,
                2,
                1.0,
                &mut [0; 6],
                skia::PixelFormat::Bgra8888,
                stride
            )
            .is_err());
        }
    }

    #[test]
    fn borrowed_window_target_preserves_pointer_guards_and_rgba_colors() {
        if !cfg!(target_endian = "little") {
            return;
        }
        for width in [1u32, 7, 8, 9, 63, 64, 65] {
            let mut list = PaintArtifact::default();
            for (x, color) in [
                (
                    0.0,
                    Color {
                        red: 0.83,
                        green: 0.17,
                        blue: 0.39,
                        alpha: 0.43,
                    },
                ),
                (
                    3.25,
                    Color {
                        red: 0.09,
                        green: 0.61,
                        blue: 0.91,
                        alpha: 0.71,
                    },
                ),
            ] {
                std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                    r#type: DisplayItemType::kDrawRoundedRect,
                    rect: PaintRect {
                        x,
                        y: 1.25,
                        width: f64::from(width),
                        height: 9.75,
                    },
                    color,
                    ..Default::default()
                });
            }
            let reference = crate::pure_replay::RasterizeSourceDisplayItemList(&list, width, 17);
            let count = width as usize * 17;
            for format in [skia::PixelFormat::Bgrx8888, skia::PixelFormat::Bgra8888] {
                let mut guarded = vec![0x81b7_93c5; count + 8];
                let ptr = guarded[3..3 + count].as_ptr();
                RenderDisplayItemListIntoWindowBufferWithFormat(
                    &list,
                    width,
                    17,
                    1.0,
                    &mut guarded[3..3 + count],
                    format,
                )
                .unwrap();
                assert_eq!(ptr, guarded[3..3 + count].as_ptr());
                assert!(guarded[..3]
                    .iter()
                    .chain(&guarded[3 + count..])
                    .all(|&v| v == 0x81b7_93c5));
                for (&actual, rgba) in guarded[3..3 + count].iter().zip(reference.chunks_exact(4)) {
                    let expected = (if format == skia::PixelFormat::Bgra8888 {
                        u32::from(rgba[3]) << 24
                    } else {
                        0
                    }) | (u32::from(rgba[0]) << 16)
                        | (u32::from(rgba[1]) << 8)
                        | u32::from(rgba[2]);
                    assert_eq!(actual, expected, "width={width}");
                }
            }
        }
        assert!(RenderDisplayItemListIntoWindowBuffer(
            &PaintArtifact::default(),
            7,
            3,
            1.0,
            &mut [0; 20]
        )
        .is_err());
    }

    #[test]
    fn device_scale_changes_geometry_without_changing_logical_commands() {
        let mut list = PaintArtifact::default();
        std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect: PaintRect {
                x: 1.0,
                y: 1.0,
                width: 2.0,
                height: 2.0,
            },
            color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            ..Default::default()
        });
        let mut surface = RasterSurface::new(8, 8).unwrap();
        RenderDisplayItemList(&mut surface, &list, 1.0);
        assert_eq!(
            surface.pixels(),
            crate::pure_replay::RasterizeSourceDisplayItemList(&list, 8, 8)
        );
        RenderDisplayItemList(&mut surface, &list, 2.0);
        let transferred = RenderDisplayItemListToSurface(&list, 8, 8, 2.0).unwrap();
        assert_eq!(surface.pixels(), transferred.pixels());
        assert!(RenderDisplayItemListToSurface(&list, 0, 8, 2.0).is_err());
        let pixel = |x: usize, y: usize| &surface.pixels()[(y * 8 + x) * 4..(y * 8 + x + 1) * 4];
        assert_eq!(pixel(1, 1), &[255, 255, 255, 255]);
        assert_eq!(pixel(4, 4), &[255, 0, 0, 255]);
    }
}

pub(crate) fn render_damage_into_window_buffer_cached_products(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    damage: &[RasterDamage],
    row_stride: usize,
    image_cache: &mut RasterImageCache,
    clip_cache: &mut RasterClipProductCache,
) -> std::io::Result<()> {
    render_damage_into_window_buffer(
        list,
        width,
        height,
        scale,
        buffer,
        format,
        damage,
        row_stride,
        Some(image_cache),
        Some(clip_cache),
        false,
    )
}
pub(crate) fn render_into_window_buffer_cached_products(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    buffer: &mut [u32],
    format: skia::PixelFormat,
    row_stride: usize,
    image_cache: &mut RasterImageCache,
    clip_cache: &mut RasterClipProductCache,
) -> std::io::Result<()> {
    render_into_window_buffer(
        list,
        width,
        height,
        scale,
        buffer,
        format,
        row_stride,
        Some(image_cache),
        Some(clip_cache),
    )
}
