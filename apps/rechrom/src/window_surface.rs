//! Software window target backed by softbuffer and winit native handles.
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use renderer_skia::PixelFormat;
use std::{io, num::NonZeroU32, sync::Arc};
pub(crate) type WindowTarget =
    WindowSurface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>;

/// Host allocation/presentation choice. Both targets consume the same Viz
/// Page output through native Skia placement; offscreen owns ordinary CPU memory.
pub(crate) enum DisplayTarget {
    Native(WindowTarget),
    Offscreen,
}

pub(crate) enum PreparedDisplayTarget {
    Native(PreparedWindowFrame),
    Offscreen { pixels: Vec<u32>, row_stride: usize },
}

impl PreparedDisplayTarget {
    pub(crate) fn take_readback_rgba(
        &mut self,
        viewport: crate::engine::Viewport,
    ) -> io::Result<crate::engine::WindowFrame> {
        match self {
            Self::Native(frame) => frame.readback_rgba(viewport),
            Self::Offscreen { pixels, row_stride } => {
                readback_pixels(pixels, *row_stride, true, viewport)
            }
        }
    }
}

impl DisplayTarget {
    pub(crate) fn is_native(&self) -> bool {
        matches!(self, Self::Native(_))
    }

    pub(crate) fn render_page_surfaces(
        &mut self,
        pages: &[(
            &crate::page_presentation::PageFrame,
            crate::page_presentation::DeviceRect,
        )],
        viewport: crate::engine::Viewport,
        present_sequence: u64,
    ) -> io::Result<PreparedDisplayTarget> {
        match self {
            Self::Native(target) => target
                .render_page_surfaces(pages, viewport, present_sequence)
                .map(PreparedDisplayTarget::Native),
            Self::Offscreen => {
                let mut pixels = vec![0u32; viewport.width as usize * viewport.height as usize];
                draw_page_surfaces(
                    pages,
                    &mut pixels,
                    viewport.width as usize,
                    viewport,
                    PixelFormat::Bgra8888,
                )?;
                Ok(PreparedDisplayTarget::Offscreen {
                    pixels,
                    row_stride: viewport.width as usize,
                })
            }
        }
    }

    pub(crate) fn present_prepared(
        &mut self,
        prepared: PreparedDisplayTarget,
        sequence: u64,
        on_complete: softbuffer::PresentCallback,
    ) -> io::Result<()> {
        match (self, prepared) {
            (Self::Native(target), PreparedDisplayTarget::Native(frame)) => {
                target.present_prepared(frame, sequence, on_complete)
            }
            (Self::Offscreen, PreparedDisplayTarget::Offscreen { .. }) => {
                // A memory swap completes synchronously. The caller registers
                // the swap before presenting and posts its completion token to
                // the Display owner, just like an asynchronous native target.
                let now = std::time::Instant::now();
                browser_tracing::instant(
                    "frame",
                    "OffscreenFrameComplete",
                    &[("present_sequence", sequence as f64)],
                );
                on_complete(now, now);
                Ok(())
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "prepared frame target mismatch",
            )),
        }
    }
}

/// Page output already passed its own Viz and Renderer. Window embedding
/// performs same-scale image placement without PaintArtifact or cc owners.
fn draw_page_surfaces(
    pages: &[(
        &crate::page_presentation::PageFrame,
        crate::page_presentation::DeviceRect,
    )],
    pixels: &mut [u32],
    stride: usize,
    viewport: crate::engine::Viewport,
    format: PixelFormat,
) -> io::Result<()> {
    if stride < viewport.width as usize
        || pixels.len() < stride.saturating_mul(viewport.height as usize)
    {
        return Err(io::Error::other("invalid window target"));
    }
    // The normal browser scene covers the complete output (chrome plus page).
    // Clearing that target first writes every pixel twice and can push an
    // otherwise ready scroll frame past its display-link latch. Keep the clear
    // for sparse scenes; complete scenes overwrite every visible pixel below.
    if !placements_cover_viewport(pages, viewport) {
        for row in pixels
            .chunks_exact_mut(stride)
            .take(viewport.height as usize)
        {
            row[..viewport.width as usize].fill(0);
        }
    }
    for (frame, destination) in pages {
        // A top-level resize may embed the preceding Page surface for one or
        // more compositor frames. Keep its pixels at their native size and
        // clip them to the new layer bounds. This is SurfaceLayer's
        // stretch_content_to_fill_bounds=false behavior: expanding exposes the
        // cleared gutter, shrinking clips, and a newly laid-out surface
        // replaces the fallback as soon as it arrives.
        let left = destination.x.max(0) as usize;
        let top = destination.y.max(0) as usize;
        let drawn_width = destination.width.min(frame.viewport.width);
        let drawn_height = destination.height.min(frame.viewport.height);
        let right =
            (destination.x as i64 + drawn_width as i64).clamp(0, viewport.width as i64) as usize;
        let bottom =
            (destination.y as i64 + drawn_height as i64).clamp(0, viewport.height as i64) as usize;
        if left >= right || top >= bottom {
            continue;
        }
        let source_x = (left as i64 - destination.x as i64) as usize;
        let source_y = (top as i64 - destination.y as i64) as usize;
        for row in 0..bottom - top {
            let src = (source_y + row) * frame.viewport.width as usize + source_x;
            let dst = (top + row) * stride + left;
            let source = &frame.pixels[src..src + right - left];
            let target = &mut pixels[dst..dst + right - left];
            target.copy_from_slice(source);
            if format == PixelFormat::Bgrx8888 {
                for pixel in target {
                    *pixel &= 0x00ff_ffff;
                }
            }
        }
    }
    Ok(())
}

fn placements_cover_viewport(
    pages: &[(
        &crate::page_presentation::PageFrame,
        crate::page_presentation::DeviceRect,
    )],
    viewport: crate::engine::Viewport,
) -> bool {
    let width = viewport.width as i64;
    let height = viewport.height as i64;
    if width == 0 || height == 0 {
        return true;
    }
    let mut rectangles = Vec::with_capacity(pages.len());
    let mut y_edges = vec![0, height];
    for (frame, destination) in pages {
        let left = (destination.x as i64).clamp(0, width);
        let top = (destination.y as i64).clamp(0, height);
        let right = (destination.x as i64 + destination.width.min(frame.viewport.width) as i64)
            .clamp(0, width);
        let bottom = (destination.y as i64 + destination.height.min(frame.viewport.height) as i64)
            .clamp(0, height);
        if left < right && top < bottom {
            rectangles.push((left, top, right, bottom));
            y_edges.extend([top, bottom]);
        }
    }
    y_edges.sort_unstable();
    y_edges.dedup();
    y_edges.windows(2).all(|band| {
        let (top, bottom) = (band[0], band[1]);
        if top == bottom {
            return true;
        }
        let mut intervals: Vec<_> = rectangles
            .iter()
            .filter_map(|&(left, rect_top, right, rect_bottom)| {
                (rect_top <= top && rect_bottom >= bottom).then_some((left, right))
            })
            .collect();
        intervals.sort_unstable();
        let mut covered = 0;
        for (left, right) in intervals {
            if left > covered {
                return false;
            }
            covered = covered.max(right);
            if covered >= width {
                return true;
            }
        }
        false
    })
}

#[cfg(test)]
pub struct SurfaceLayer<'a> {
    pub surface: &'a crate::engine::CapturedSurface,
    pub x: u32,
    pub y: u32,
}

pub struct WindowSurface<D, W> {
    surface: softbuffer::Surface<D, W>,
    _context: softbuffer::Context<D>,
    width: u32,
    height: u32,
}

pub(crate) struct PreparedWindowFrame {
    #[cfg(target_os = "macos")]
    buffer: softbuffer::PreparedBuffer,
    #[cfg(not(target_os = "macos"))]
    pixels: Vec<u32>,
    #[cfg(not(target_os = "macos"))]
    row_stride: usize,
}

impl PreparedWindowFrame {
    /// Copy the completed software compositor target before native color
    /// management/presentation. This is enabled only by headless diagnostics.
    pub(crate) fn readback_rgba(
        &self,
        viewport: crate::engine::Viewport,
    ) -> io::Result<crate::engine::WindowFrame> {
        #[cfg(target_os = "macos")]
        let (pixels, row_stride, has_alpha) =
            (self.buffer.pixels(), self.buffer.row_stride(), true);
        #[cfg(not(target_os = "macos"))]
        let (pixels, row_stride, has_alpha) = (self.pixels.as_slice(), self.row_stride, false);
        readback_pixels(pixels, row_stride, has_alpha, viewport)
    }
}

fn readback_pixels(
    pixels: &[u32],
    row_stride: usize,
    has_alpha: bool,
    viewport: crate::engine::Viewport,
) -> io::Result<crate::engine::WindowFrame> {
    let rows = |start_y: u32, height: u32| -> io::Result<crate::engine::CapturedSurface> {
        let mut rgba = Vec::with_capacity(viewport.width as usize * height as usize * 4);
        for y in start_y..start_y + height {
            let row =
                &pixels[y as usize * row_stride..y as usize * row_stride + viewport.width as usize];
            for &pixel in row {
                let bytes = pixel.to_ne_bytes();
                rgba.extend_from_slice(&[
                    bytes[2],
                    bytes[1],
                    bytes[0],
                    if has_alpha { bytes[3] } else { 255 },
                ]);
            }
        }
        crate::engine::CapturedSurface::from_pixels(viewport.width, height, rgba)
    };
    let toolbar_height = viewport.toolbar_pixels();
    Ok(crate::engine::WindowFrame {
        provenance: None,
        viewport,
        toolbar: rows(0, toolbar_height)?,
        content: rows(toolbar_height, viewport.height - toolbar_height)?,
    })
}

fn error(error: softbuffer::SoftBufferError) -> io::Error {
    io::Error::other(error.to_string())
}

impl<D: HasDisplayHandle, W: HasWindowHandle> WindowSurface<D, W> {
    pub fn new(display: D, window: W) -> io::Result<Self> {
        let context = softbuffer::Context::new(display).map_err(error)?;
        let surface = softbuffer::Surface::new(&context, window).map_err(error)?;
        Ok(Self {
            surface,
            _context: context,
            width: 0,
            height: 0,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) -> io::Result<()> {
        self.width = width;
        self.height = height;
        if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
            self.surface.resize(w, h).map_err(error)?;
        }
        Ok(())
    }

    /// Place completed Page surfaces into a prepared native buffer.
    pub(crate) fn render_page_surfaces(
        &mut self,
        pages: &[(
            &crate::page_presentation::PageFrame,
            crate::page_presentation::DeviceRect,
        )],
        viewport: crate::engine::Viewport,
        present_sequence: u64,
    ) -> io::Result<PreparedWindowFrame> {
        let mut frame_trace = browser_tracing::span("present", "PrepareWindowFrame");
        frame_trace.set("present_sequence", present_sequence as f64);
        frame_trace.set("width", viewport.width as f64);
        frame_trace.set("height", viewport.height as f64);
        frame_trace.set("device_scale", viewport.scale);
        frame_trace.set("succeeded", 0.0);
        if self.width != viewport.width || self.height != viewport.height {
            self.resize(viewport.width, viewport.height)?;
        }
        let acquire_trace = browser_tracing::span("present", "AcquirePreparedBuffer");
        #[cfg(target_os = "macos")]
        let mut buffer = self.surface.prepared_buffer().map_err(error)?;
        #[cfg(not(target_os = "macos"))]
        let mut buffer = vec![0u32; self.width as usize * self.height as usize];
        #[cfg(target_os = "macos")]
        let row_stride = buffer.row_stride();
        #[cfg(not(target_os = "macos"))]
        let row_stride = self.width as usize;
        drop(acquire_trace);
        #[cfg(target_os = "macos")]
        let format = match buffer.pixel_format() {
            softbuffer::BufferFormat::Xrgb8888 => PixelFormat::Bgrx8888,
            softbuffer::BufferFormat::Argb8888 => PixelFormat::Bgra8888,
        };
        #[cfg(not(target_os = "macos"))]
        let format = PixelFormat::Bgrx8888;
        #[cfg(target_os = "macos")]
        let pixels = buffer.pixels_mut();
        #[cfg(not(target_os = "macos"))]
        let pixels = buffer.as_mut_slice();
        draw_page_surfaces(pages, pixels, row_stride, viewport, format)?;
        static OPTICAL_PROBE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let optical_probe = *OPTICAL_PROBE.get_or_init(|| {
            std::env::var_os("BROWSER_PRESENTATION_PROBE").is_some_and(|v| v == "1")
        });
        if optical_probe {
            paint_optical_sequence(
                pixels,
                row_stride,
                viewport.width,
                viewport.height,
                viewport.scale,
                format,
                present_sequence,
            );
        }
        frame_trace.set(
            "damage_pixels",
            viewport.width as f64 * viewport.height as f64,
        );

        frame_trace.set("succeeded", 1.0);
        #[cfg(target_os = "macos")]
        {
            Ok(PreparedWindowFrame { buffer })
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(PreparedWindowFrame {
                pixels: buffer,
                row_stride,
            })
        }
    }

    /// Submit a complete frame. No page, tile, raster or composition work is
    /// performed here; macOS transfers the prepared IOSurface without a copy.
    pub(crate) fn present_prepared(
        &mut self,
        prepared: PreparedWindowFrame,
        present_sequence: u64,
        on_swap_ack: softbuffer::PresentCallback,
    ) -> io::Result<()> {
        let context = browser_tracing::context();
        let ready_at = std::time::Instant::now();
        let mut trace = browser_tracing::span("present", "PresentPreparedFrame");
        trace.set("present_sequence", present_sequence as f64);
        trace.set("succeeded", 0.0);
        let callback: softbuffer::PresentCallback = Box::new(move |started, finished| {
            let _context = browser_tracing::scope(context);
            browser_tracing::interval("present", "NativePresentQueue", ready_at, started, &[]);
            browser_tracing::interval(
                "present",
                "PresentReturn",
                started,
                finished,
                &[
                    ("succeeded", 1.0),
                    ("window_thread", cfg!(target_os = "macos") as u8 as f64),
                    ("present_sequence", present_sequence as f64),
                ],
            );
            on_swap_ack(started, finished);
        });
        static OPTICAL_PROBE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let optical_probe = *OPTICAL_PROBE.get_or_init(|| {
            std::env::var_os("BROWSER_PRESENTATION_PROBE").is_some_and(|v| v == "1")
        });
        #[cfg(target_os = "macos")]
        if optical_probe {
            self.surface
                .present_prepared_with_transaction_observer(
                    prepared.buffer,
                    callback,
                    native_transaction_observer(context, present_sequence),
                )
                .map_err(error)?;
        } else {
            self.surface
                .present_prepared_with_callback(prepared.buffer, callback)
                .map_err(error)?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut buffer = self.surface.buffer_mut().map_err(error)?;
            let target_stride = buffer.row_stride();
            if target_stride == prepared.row_stride {
                buffer.copy_from_slice(&prepared.pixels);
            } else {
                for (source, target) in prepared
                    .pixels
                    .chunks_exact(prepared.row_stride)
                    .zip(buffer.chunks_exact_mut(target_stride))
                {
                    target[..self.width as usize].copy_from_slice(&source[..self.width as usize]);
                }
            }
            buffer.present_with_callback(callback).map_err(error)?;
        }
        trace.set("succeeded", 1.0);
        Ok(())
    }
}

fn native_transaction_observer(
    context: browser_tracing::Context,
    present_sequence: u64,
) -> softbuffer::NativeTransactionObserver {
    Arc::new(move |phase, surface_id, at| {
        // CA handlers can run after another BeginFrame has started.
        // They observe a transaction phase, not WindowServer presentation.
        // Associate the registration with its immutable originating update.
        let _context = browser_tracing::scope(context);
        #[cfg(target_os = "macos")]
        let host_time_ns = {
            let (sample_at, host_ns) = crate::begin_frame_source::sample_host_clock();
            if at >= sample_at {
                host_ns + at.duration_since(sample_at).as_nanos() as f64
            } else {
                host_ns - sample_at.duration_since(at).as_nanos() as f64
            }
        };
        #[cfg(not(target_os = "macos"))]
        let host_time_ns = f64::NAN;
        let name = match phase {
            softbuffer::NativeTransactionPhase::PreCommit => "CAPreCommitHandler",
            softbuffer::NativeTransactionPhase::PostCommit => "CAPostCommitHandler",
        };
        browser_tracing::instant_at(
            "present",
            name,
            at,
            &[
                ("present_sequence", present_sequence as f64),
                ("iosurface_id", surface_id as f64),
                ("host_time_ns", host_time_ns),
            ],
        );
    })
}

fn paint_optical_sequence(
    pixels: &mut [u32],
    stride: usize,
    width: u32,
    height: u32,
    scale: f64,
    format: PixelFormat,
    present_sequence: u64,
) {
    let sequence = (present_sequence & 0xfff) as u16;
    let checksum = (sequence ^ (sequence >> 6) ^ 0x2d) & 0x3f;
    let cells = [
        5,
        sequence & 7,
        (sequence >> 3) & 7,
        (sequence >> 6) & 7,
        (sequence >> 9) & 7,
        checksum & 7,
        (checksum >> 3) & 7,
        3,
    ];
    let edge = |css: usize, bound: u32| ((css as f64 * scale).round() as usize).min(bound as usize);
    // Keep the diagnostic marker below native titlebar and recording controls.
    let top = edge(48, height);
    let bottom = edge(52, height);
    for (index, bits) in cells.iter().enumerate() {
        let red = if bits & 1 != 0 { 255u32 } else { 0 };
        let green = if bits & 2 != 0 { 255u32 } else { 0 };
        let blue = if bits & 4 != 0 { 255u32 } else { 0 };
        let packed = match format {
            PixelFormat::Bgra8888 => 0xff00_0000 | red << 16 | green << 8 | blue,
            PixelFormat::Bgrx8888 => red << 16 | green << 8 | blue,
            _ => 0xff00_0000 | blue << 16 | green << 8 | red,
        };
        let left = edge(8 + index * 3, width);
        let right = edge(11 + index * 3, width);
        for y in top..bottom {
            pixels[y * stride + left..y * stride + right].fill(packed);
        }
    }
}

#[cfg(test)]
pub(crate) fn compose_layers(
    buffer: &mut [u32],
    width: u32,
    height: u32,
    layers: &[SurfaceLayer<'_>],
) {
    buffer.fill(0x00ff_ffff);
    for layer in layers {
        copy_layer(buffer, width, height, layer);
    }
}

#[cfg(test)]
fn copy_layer(buffer: &mut [u32], width: u32, height: u32, layer: &SurfaceLayer<'_>) {
    let (source_width, source_height) = layer.surface.size();
    let rows = source_height.min(height.saturating_sub(layer.y));
    let columns = source_width.min(width.saturating_sub(layer.x));
    if columns == 0 || rows == 0 {
        return;
    }
    let pixels = layer.surface.pixels();
    for y in 0..rows as usize {
        let src = y * source_width as usize * 4;
        let dst = (y + layer.y as usize) * width as usize + layer.x as usize;
        rgba_to_window_rgb(
            &mut buffer[dst..dst + columns as usize],
            &pixels[src..src + columns as usize * 4],
        );
    }
}

// softbuffer accepts numeric 0x00RRGGBB words, whereas our CPU readback is
// RGBA bytes. This is a window-format adapter, not Skia alpha conversion.
// Convert complete groups without per-pixel slice/trait overhead in DEBUG.
#[cfg(test)]
fn rgba_to_window_rgb(dst: &mut [u32], src: &[u8]) {
    assert_eq!(src.len(), dst.len() * 4);
    let mut offset = 0;
    #[cfg(target_endian = "little")]
    {
        #[repr(C, packed)]
        struct Block {
            w0: u64,
            w1: u64,
            w2: u64,
            w3: u64,
            w4: u64,
            w5: u64,
            w6: u64,
            w7: u64,
            w8: u64,
            w9: u64,
            w10: u64,
            w11: u64,
            w12: u64,
            w13: u64,
            w14: u64,
            w15: u64,
            w16: u64,
            w17: u64,
            w18: u64,
            w19: u64,
            w20: u64,
            w21: u64,
            w22: u64,
            w23: u64,
            w24: u64,
            w25: u64,
            w26: u64,
            w27: u64,
            w28: u64,
            w29: u64,
            w30: u64,
            w31: u64,
        }
        #[inline(always)]
        fn swizzle(v: u64) -> u64 {
            ((v & 0x0000_00ff_0000_00ff) << 16)
                | (v & 0x0000_ff00_0000_ff00)
                | ((v & 0x00ff_0000_00ff_0000) >> 16)
        }
        while offset + 64 <= dst.len() {
            // SAFETY: Both blocks cover 64 complete pixels within checked
            // slices; packed fields allow either source/destination alignment.
            // Shared source and exclusive destination slices cannot overlap.
            unsafe {
                let input = src.as_ptr().add(offset * 4).cast::<Block>();
                let output = dst.as_mut_ptr().add(offset).cast::<Block>();
                (*output).w0 = swizzle((*input).w0);
                (*output).w1 = swizzle((*input).w1);
                (*output).w2 = swizzle((*input).w2);
                (*output).w3 = swizzle((*input).w3);
                (*output).w4 = swizzle((*input).w4);
                (*output).w5 = swizzle((*input).w5);
                (*output).w6 = swizzle((*input).w6);
                (*output).w7 = swizzle((*input).w7);
                (*output).w8 = swizzle((*input).w8);
                (*output).w9 = swizzle((*input).w9);
                (*output).w10 = swizzle((*input).w10);
                (*output).w11 = swizzle((*input).w11);
                (*output).w12 = swizzle((*input).w12);
                (*output).w13 = swizzle((*input).w13);
                (*output).w14 = swizzle((*input).w14);
                (*output).w15 = swizzle((*input).w15);
                (*output).w16 = swizzle((*input).w16);
                (*output).w17 = swizzle((*input).w17);
                (*output).w18 = swizzle((*input).w18);
                (*output).w19 = swizzle((*input).w19);
                (*output).w20 = swizzle((*input).w20);
                (*output).w21 = swizzle((*input).w21);
                (*output).w22 = swizzle((*input).w22);
                (*output).w23 = swizzle((*input).w23);
                (*output).w24 = swizzle((*input).w24);
                (*output).w25 = swizzle((*input).w25);
                (*output).w26 = swizzle((*input).w26);
                (*output).w27 = swizzle((*input).w27);
                (*output).w28 = swizzle((*input).w28);
                (*output).w29 = swizzle((*input).w29);
                (*output).w30 = swizzle((*input).w30);
                (*output).w31 = swizzle((*input).w31);
            }
            offset += 64;
        }
    }
    for x in offset..dst.len() {
        let pixel = &src[x * 4..x * 4 + 4];
        dst[x] = (u32::from(pixel[0]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_frame(
        width: u32,
        height: u32,
        pixels: Vec<u32>,
    ) -> crate::page_presentation::PageFrame {
        crate::page_presentation::PageFrame {
            provenance: crate::page_presentation::FrameProvenance {
                document: (1, 1),
                content: None,
            },
            viewport: crate::engine::Viewport {
                width,
                height,
                scale: 1.0,
            },
            is_handling_interaction: false,
            pixels: Arc::new(pixels),
            recycler: None,
        }
    }

    #[test]
    fn stale_resize_surface_keeps_native_size_and_exposes_gutter() {
        let frame = page_frame(2, 2, vec![1, 2, 3, 4]);
        let placement = crate::page_presentation::DeviceRect::new(0, 0, 4, 2);
        let viewport = crate::engine::Viewport {
            width: 4,
            height: 2,
            scale: 1.0,
        };
        let mut pixels = vec![99; 8];
        draw_page_surfaces(
            &[(&frame, placement)],
            &mut pixels,
            4,
            viewport,
            PixelFormat::Bgra8888,
        )
        .unwrap();
        assert_eq!(pixels, vec![1, 2, 0, 0, 3, 4, 0, 0]);
    }

    #[test]
    fn offscreen_readback_excludes_toolbar_and_keeps_rgba_channels() {
        for scale in [1.0, 2.0] {
            let viewport = crate::engine::Viewport {
                width: 4,
                height: (crate::chrome::HEIGHT * scale).round() as u32 + 3,
                scale,
            };
            let toolbar_pixels = viewport.toolbar_pixels() as usize * viewport.width as usize;
            let mut pixels = vec![0xff33_2211; viewport.width as usize * viewport.height as usize];
            pixels[toolbar_pixels..].fill(0xff91_3a7f);
            let mut prepared = PreparedDisplayTarget::Offscreen {
                pixels,
                row_stride: 4,
            };
            let frame = prepared.take_readback_rgba(viewport).unwrap();
            assert_eq!(frame.content.size(), (4, 3));
            assert_eq!(frame.toolbar.size(), (4, viewport.toolbar_pixels()));
            assert!(frame
                .content
                .pixels()
                .chunks_exact(4)
                .all(|pixel| pixel == [0x7f, 0x3a, 0x91, 255]));
            assert!(frame
                .toolbar
                .pixels()
                .chunks_exact(4)
                .all(|pixel| pixel == [0x11, 0x22, 0x33, 255]));
        }
    }

    #[test]
    fn offscreen_completion_acknowledges_once_without_native_target() {
        let mut target = DisplayTarget::Offscreen;
        assert!(!target.is_native());
        let completions = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = completions.clone();
        target
            .present_prepared(
                PreparedDisplayTarget::Offscreen {
                    pixels: vec![0],
                    row_stride: 1,
                },
                42,
                Box::new(move |started, finished| {
                    assert_eq!(started, finished);
                    observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }),
            )
            .unwrap();
        assert_eq!(completions.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
    #[test]
    fn window_format_swizzle_handles_every_channel_alignment_tail_and_guard() {
        for count in 0..=65 {
            for misalign in 0..8 {
                for dst_offset in 0..3 {
                    let mut input = vec![0xc3; misalign + count * 4 + 9];
                    for i in 0..count * 4 {
                        input[misalign + i] = (i.wrapping_mul(71) + count * 13 + misalign) as u8;
                    }
                    let src = &input[misalign..misalign + count * 4];
                    let mut dst = vec![0x9e51_a2c7; dst_offset + count + 5];
                    rgba_to_window_rgb(&mut dst[dst_offset..dst_offset + count], src);
                    for i in 0..count {
                        assert_eq!(
                            dst[dst_offset + i],
                            (u32::from(src[4 * i]) << 16)
                                | (u32::from(src[4 * i + 1]) << 8)
                                | u32::from(src[4 * i + 2])
                        );
                    }
                    assert!(dst[..dst_offset]
                        .iter()
                        .chain(&dst[dst_offset + count..])
                        .all(|&v| v == 0x9e51_a2c7));
                }
            }
        }
        let input: Vec<u8> = (0..=255u8)
            .flat_map(|v| [v, v.wrapping_mul(37), 255 - v, v.wrapping_mul(13)])
            .collect();
        let mut dst = vec![0; 256];
        rgba_to_window_rgb(&mut dst, &input);
        for (pixel, &output) in input.chunks_exact(4).zip(&dst) {
            assert_eq!(
                output,
                (u32::from(pixel[0]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[2])
            );
        }
    }

    #[test]
    fn clipped_layers_keep_white_outside_and_preserve_source_stride() {
        for width in [1, 7, 8, 9, 17] {
            for x in 0..=width + 1 {
                for y in 0..=6 {
                    let src: Vec<u8> = (0..11 * 4 * 4).map(|i| (i * 17) as u8).collect();
                    let surface =
                        crate::engine::CapturedSurface::from_pixels(11, 4, src.clone()).unwrap();
                    let mut actual = vec![0; width as usize * 5];
                    compose_layers(
                        &mut actual,
                        width,
                        5,
                        &[SurfaceLayer {
                            surface: &surface,
                            x,
                            y,
                        }],
                    );
                    let mut expected = vec![0x00ff_ffff; width as usize * 5];
                    for sy in 0..4u32 {
                        for sx in 0..11u32 {
                            if x + sx < width && y + sy < 5 {
                                let i = (sy * 11 + sx) as usize * 4;
                                expected[((y + sy) * width + x + sx) as usize] =
                                    (u32::from(src[i]) << 16)
                                        | (u32::from(src[i + 1]) << 8)
                                        | u32::from(src[i + 2]);
                            }
                        }
                    }
                    assert_eq!(actual, expected);
                }
            }
        }
    }
}
