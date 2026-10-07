//! Software window target backed by softbuffer and winit native handles.
use layer_tile::FramePlan;
use paint::paint_engine::PaintArtifact;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use renderer::layer_tile_renderer::LayerTileRenderer;
#[cfg(test)]
use skia::compat::surface::RasterSurface;
use std::{io, num::NonZeroU32, sync::Arc};
pub(crate) type WindowTarget =
    WindowSurface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>;

#[cfg(test)]
pub struct SurfaceLayer<'a> {
    pub surface: &'a RasterSurface,
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

    /// Produce a complete output surface without publishing it to the native
    /// compositor. The display scheduler may submit it at a later deadline.
    pub(crate) fn prepare(
        &mut self,
        toolbar: &rechrom::page::Page,
        content: Option<&rechrom::page::Page>,
        overlay: Option<&Arc<PaintArtifact>>,
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
        let profile = std::env::var_os("BROWSER_APP_PROFILE_WINDOW")
            .is_some()
            .then(std::time::Instant::now);
        let content_height = self.height - viewport.toolbar_pixels();
        let acquire_trace = browser_tracing::span("present", "AcquirePreparedBuffer");
        #[cfg(target_os = "macos")]
        let mut buffer = self.surface.prepared_buffer().map_err(error)?;
        #[cfg(not(target_os = "macos"))]
        let mut buffer = vec![0u32; self.width as usize * self.height as usize];
        #[cfg(target_os = "macos")]
        let row_stride = buffer.row_stride();
        #[cfg(not(target_os = "macos"))]
        let row_stride = self.width as usize;
        let acquired = profile.map(|_| std::time::Instant::now());
        drop(acquire_trace);
        #[cfg(target_os = "macos")]
        let format = match buffer.pixel_format() {
            softbuffer::BufferFormat::Xrgb8888 => skia::PixelFormat::Bgrx8888,
            softbuffer::BufferFormat::Argb8888 => skia::PixelFormat::Bgra8888,
        };
        #[cfg(not(target_os = "macos"))]
        let format = skia::PixelFormat::Bgrx8888;
        #[cfg(target_os = "macos")]
        let pixels = buffer.pixels_mut();
        #[cfg(not(target_os = "macos"))]
        let pixels = buffer.as_mut_slice();
        let toolbar_len = row_stride * viewport.toolbar_pixels() as usize;
        let (top, bottom) = pixels.split_at_mut(toolbar_len);
        let toolbar_update = {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 2,
                ..Default::default()
            });
            let mut trace = browser_tracing::span("render", "Toolbar");
            let update = toolbar.PaintInto(
                self.width,
                viewport.toolbar_pixels(),
                viewport.scale,
                top,
                format,
                row_stride,
            )?;
            trace.set("damage_pixels", update.damage_pixels as f64);
            update
        };
        let toolbar_done = profile.map(|_| std::time::Instant::now());
        let content_update = {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            let mut trace = browser_tracing::span("render", "Content");
            let update = if let Some(page) = content {
                if let Some(artifact) = overlay {
                    page.PaintArtifactInto(
                        artifact,
                        self.width,
                        content_height,
                        viewport.scale,
                        bottom,
                        format,
                        row_stride,
                    )?
                } else {
                    page.PaintInto(
                        self.width,
                        content_height,
                        viewport.scale,
                        bottom,
                        format,
                        row_stride,
                    )?
                }
            } else {
                bottom.fill(0x00ff_ffff);
                renderer::layer_tile_renderer::RasterUpdate {
                    mode: "empty",
                    reason: "no-page",
                    damage_pixels: self.width as usize * content_height as usize,
                    copied_bytes: 0,
                }
            };
            trace.set("damage_pixels", update.damage_pixels as f64);
            update
        };
        let content_done = profile.map(|_| std::time::Instant::now());
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
            #[cfg(target_os = "macos")]
            let (marker_time, host_time_ns) = crate::begin_frame_source::sample_host_clock();
            #[cfg(not(target_os = "macos"))]
            let (marker_time, host_time_ns) = (std::time::Instant::now(), f64::NAN);
            browser_tracing::instant_at(
                "present",
                "OpticalFrameMarker",
                marker_time,
                &[
                    ("present_sequence", present_sequence as f64),
                    ("encoded_sequence", (present_sequence & 0xfff) as f64),
                    ("host_time_ns", host_time_ns),
                ],
            );
        }
        frame_trace.set("damage_pixels", content_update.damage_pixels as f64);
        frame_trace.set(
            "copied_bytes",
            (toolbar_update.copied_bytes + content_update.copied_bytes) as f64,
        );
        frame_trace.set("succeeded", 1.0);
        if let Some(start) = profile {
            let (acquired, toolbar_done, content_done) = (
                acquired.unwrap(),
                toolbar_done.unwrap(),
                content_done.unwrap(),
            );
            let tile_stats = content
                .map(rechrom::page::Page::LayerTileStats)
                .unwrap_or_default();
            println!("window-prepare-frame acquire_ms={:.3} toolbar_ms={:.3} content_ms={:.3} total_ms={:.3} app_copied_bytes={} app_format_passes=0 toolbar_mode={} content_mode={} content_reason={} damage_pixels={} layers={} tiles={} raster_tasks={} reused_tiles={}",(acquired-start).as_secs_f64()*1000.0,(toolbar_done-acquired).as_secs_f64()*1000.0,(content_done-toolbar_done).as_secs_f64()*1000.0,start.elapsed().as_secs_f64()*1000.0,toolbar_update.copied_bytes+content_update.copied_bytes,toolbar_update.mode,content_update.mode,content_update.reason,content_update.damage_pixels,tile_stats.layers,tile_stats.tiles,tile_stats.raster_tasks,tile_stats.reused_tiles);
        }
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

    /// Viz-side equivalent of the Page-based path above.  The Page publishes
    /// immutable paint artifacts; layer/tile planning and resident pixel
    /// resources stay on the display owner so main-thread JavaScript cannot
    /// stop an already committed scroll tree from drawing.
    pub(crate) fn compose_prepared_plans(
        &mut self,
        toolbar: &FramePlan,
        toolbar_renderer: &mut LayerTileRenderer,
        content: Option<&FramePlan>,
        content_renderer: &mut LayerTileRenderer,
        viewport: crate::engine::Viewport,
        present_sequence: u64,
        overlay_scrollbar: Option<crate::compositor::OverlayScrollbar>,
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
        let content_height = self.height - viewport.toolbar_pixels();
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
            softbuffer::BufferFormat::Xrgb8888 => skia::PixelFormat::Bgrx8888,
            softbuffer::BufferFormat::Argb8888 => skia::PixelFormat::Bgra8888,
        };
        #[cfg(not(target_os = "macos"))]
        let format = skia::PixelFormat::Bgrx8888;
        #[cfg(target_os = "macos")]
        let pixels = buffer.pixels_mut();
        #[cfg(not(target_os = "macos"))]
        let pixels = buffer.as_mut_slice();
        let toolbar_len = row_stride * viewport.toolbar_pixels() as usize;
        let (top, bottom) = pixels.split_at_mut(toolbar_len);
        let toolbar_update = {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 2,
                ..Default::default()
            });
            toolbar_renderer.compose_with_stride(
                toolbar,
                self.width,
                viewport.toolbar_pixels(),
                top,
                format,
                row_stride,
            )?
        };
        let content_update = if let Some(content) = content {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            content_renderer.compose_with_stride(
                content,
                self.width,
                content_height,
                bottom,
                format,
                row_stride,
            )?
        } else {
            bottom.fill(0x00ff_ffff);
            renderer::layer_tile_renderer::RasterUpdate {
                mode: "empty",
                reason: "no-page",
                damage_pixels: self.width as usize * content_height as usize,
                copied_bytes: 0,
            }
        };
        if let Some(scrollbar) = overlay_scrollbar {
            paint_overlay_scrollbar(
                bottom,
                row_stride,
                self.width,
                content_height,
                viewport.scale,
                format,
                scrollbar,
            );
        }
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
        frame_trace.set("damage_pixels", content_update.damage_pixels as f64);
        frame_trace.set(
            "copied_bytes",
            (toolbar_update.copied_bytes + content_update.copied_bytes) as f64,
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
        on_present: softbuffer::PresentCallback,
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
            on_present(started, finished);
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

/// Viz/cc-style solid-color overlay scrollbar. Its geometry comes from the
/// compositor scroll tree and is drawn into the final output after page layer
/// composition, so moving the thumb never invalidates or rerasterizes page
/// tiles. The shape follows NativeThemeMac's overlay thumb: transparent track,
/// black at 50% opacity, and a rounded capsule inset from the viewport edge.
fn paint_overlay_scrollbar(
    pixels: &mut [u32],
    stride: usize,
    width: u32,
    height: u32,
    scale: f64,
    _format: skia::PixelFormat,
    scrollbar: crate::compositor::OverlayScrollbar,
) {
    if width == 0 || height == 0 || scrollbar.maximum <= 0.0 {
        return;
    }
    let scale = scale.max(1.0);
    let viewport = scrollbar.viewport_length.max(1.0);
    let contents = viewport + scrollbar.maximum;
    let edge_inset = 3.0 * scale;
    let length_inset = 2.0 * scale;
    let thumb_width = 10.0 * scale;
    let inner_length = (height as f64 - length_inset * 2.0).max(0.0);
    let thumb_length = (inner_length * viewport / contents)
        .max(24.0 * scale)
        .min(inner_length);
    if thumb_width <= 0.0 || thumb_length <= 0.0 {
        return;
    }
    let travel = (inner_length - thumb_length).max(0.0);
    let progress = (scrollbar.offset / scrollbar.maximum).clamp(0.0, 1.0);
    let top = length_inset + travel * progress;
    let bottom = top + thumb_length;
    let right = width as f64 - edge_inset;
    let left = (right - thumb_width).max(0.0);
    let radius = thumb_width * 0.5;
    let center_x = (left + right) * 0.5;
    let cap_top = top + radius;
    let cap_bottom = bottom - radius;
    let y_start = top.floor().max(0.0) as usize;
    let y_end = bottom.ceil().min(height as f64) as usize;
    let x_start = left.floor().max(0.0) as usize;
    let x_end = right.ceil().min(width as f64) as usize;
    for y in y_start..y_end {
        let py = y as f64 + 0.5;
        let nearest_y = py.clamp(cap_top, cap_bottom.max(cap_top));
        for x in x_start..x_end {
            let px = x as f64 + 0.5;
            let distance = ((px - center_x).powi(2) + (py - nearest_y).powi(2)).sqrt();
            let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
            if coverage == 0.0 {
                continue;
            }
            let alpha = (128.0 * coverage).round() as u32;
            let inverse = 255 - alpha;
            let pixel = &mut pixels[y * stride + x];
            let high = *pixel & 0xff00_0000;
            let red_blue = (((*pixel & 0x00ff_00ff) * inverse) >> 8) & 0x00ff_00ff;
            let green = (((*pixel & 0x0000_ff00) * inverse) >> 8) & 0x0000_ff00;
            *pixel = high | red_blue | green;
        }
    }
}

/// Eight 3×4 CSS cells at (8,48): magic 5; four low-to-high 3-bit
/// sequence digits; two checksum digits; magic 3. RGB channels encode bits.
/// Public capture sees only these 24×4 CSS pixels; no page pixel readback is
/// required. The sequence wraps at 4096; PresentReturn retains its full value.
fn paint_optical_sequence(
    pixels: &mut [u32],
    stride: usize,
    width: u32,
    height: u32,
    scale: f64,
    format: skia::PixelFormat,
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
            skia::PixelFormat::Bgra8888 => 0xff00_0000 | red << 16 | green << 8 | blue,
            skia::PixelFormat::Bgrx8888 => red << 16 | green << 8 | blue,
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
                    let surface = RasterSurface::from_pixels(11, 4, src.clone()).unwrap();
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
