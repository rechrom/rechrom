//! Resident glyph scan resources adapted from SkStrike::prepareForImage,
//! SkGlyph::setImage and SkScalerContext::getImage. This local representation
//! retains ordered SkBlitter operations instead of flattening them to A8: that
//! preserves the existing fill/stroke overlap and SrcOver coverage rounding.
//! Every fixed edge coefficient is part of the key modulo integer placement.
//! Clipped glyphs additionally include their normalized viewport clip. Thus
//! f32 path chopping differences cannot accidentally share a glyph resource.
use super::glyph_paths::GlyphPaths;
use crate::raster::geom::{IntRectExt, IntSizeExt};
use crate::raster::{FillRule, Mask, Paint, Pixmap, Transform};
use crate::src::core::SkBlitter::Blitter;
use crate::src::core::SkEdgeBuilder::{ShiftedIntRect, SkBasicEdgeBuilder};
use crate::src::core::SkRasterPipelineBlitter::RasterPipelineBlitter;
use crate::src::core::SkStrikeCache::SkStrikeCache;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone, Hash, PartialEq, Eq)]
struct Descriptor {
    owner: usize,
    stroke: bool,
}
#[derive(Clone, Hash, PartialEq, Eq)]
enum Key {
    Exact {
        matrix: [u32; 6],
        width: u32,
        height: u32,
    },
    Normalized(Vec<i32>),
}
struct Row {
    x: i32,
    y: i32,
    runs: Vec<(u16, u8)>,
    solid: bool,
}
struct Resource {
    // Prevent address reuse and keep the exact source outline resident.
    owner: Arc<GlyphPaths>,
    rows: Vec<Row>,
    bounds: Option<(i32, i32, i32, i32)>,
}
impl Resource {
    fn heap_bytes(&self) -> usize {
        self.owner.heap_bytes()
            + std::mem::size_of::<GlyphPaths>()
            + 2 * std::mem::size_of::<usize>()
            + self.rows.capacity() * std::mem::size_of::<Row>()
            + self
                .rows
                .iter()
                .map(|r| r.runs.capacity() * std::mem::size_of::<(u16, u8)>())
                .sum::<usize>()
    }
}
struct Entry {
    resource: Arc<Resource>,
    origin: (i32, i32),
    key_bytes: usize,
    owns_payload: bool,
}
impl Entry {
    fn heap_bytes(&self) -> usize {
        // Normalized records own the resource charge; exact descriptor aliases
        // charge only their inline object/key/Arc metadata (SkStrike does that).
        // Both records live in ONE strike and are purged atomically together.
        self.key_bytes
            + if self.owns_payload {
                self.resource.heap_bytes()
                    + std::mem::size_of::<Resource>()
                    + 2 * std::mem::size_of::<usize>()
            } else {
                0
            }
    }
}
thread_local! {
    static CACHE:RefCell<SkStrikeCache<Descriptor,Key,Entry>> = RefCell::new(SkStrikeCache::default());
    #[cfg(test)] static PREPARATIONS:std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    #[cfg(test)] static GENERATIONS:std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    #[cfg(test)] static DISABLED:std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Profiling-only control for byte-for-byte comparison against the legacy
/// generator. Evaluate once per text run; normal builds have no env lookup.
pub(crate) fn enabled() -> bool {
    #[cfg(feature = "profiling")]
    {
        std::env::var_os("SKIA_DISABLE_OUTLINE_SCAN_CACHE").is_none()
    }
    #[cfg(not(feature = "profiling"))]
    {
        true
    }
}

struct Recorder {
    origin: (i32, i32),
    rows: Vec<Row>,
}
impl Blitter for Recorder {
    fn blit_h(&mut self, x: u32, y: u32, width: crate::raster::LengthU32) {
        self.rows.push(Row {
            x: x as i32 - self.origin.0,
            y: y as i32 - self.origin.1,
            runs: vec![(width.get() as u16, 255)],
            solid: true,
        });
    }
    fn blit_anti_h(
        &mut self,
        x: u32,
        y: u32,
        alpha: &mut [u8],
        runs: &mut [crate::raster::alpha_runs::AlphaRun],
    ) {
        let mut spans = Vec::new();
        let mut at = 0;
        while let Some(width) = runs[at] {
            spans.push((width.get(), alpha[at]));
            at += width.get() as usize;
        }
        self.rows.push(Row {
            x: x as i32 - self.origin.0,
            y: y as i32 - self.origin.1,
            runs: spans,
            solid: false,
        });
    }
}

/// Returns false for scan cases that retain the legacy path (large tiles,
/// non-AA or unusually large glyph coordinates), true after an exact replay.
#[cfg(test)]
pub(crate) fn draw(
    pixmap: &mut Pixmap,
    owner: &Arc<GlyphPaths>,
    stroke: bool,
    paint: &Paint,
    transform: Transform,
    mask: Option<&Mask>,
) -> bool {
    draw_with_bw_clip(pixmap, owner, stroke, paint, transform, mask, None)
}

pub(crate) fn draw_with_bw_clip(
    pixmap: &mut Pixmap,
    owner: &Arc<GlyphPaths>,
    stroke: bool,
    paint: &Paint,
    transform: Transform,
    mask: Option<&Mask>,
    bw_clip: Option<crate::raster::IntRect>,
) -> bool {
    #[cfg(test)]
    if DISABLED.with(|v| v.get()) {
        return false;
    }
    if !paint.anti_alias || pixmap.width() > 8191 || pixmap.height() > 8191 {
        return false;
    }
    let source = if stroke {
        match owner.stroke.as_ref() {
            Some(p) => p,
            None => return false,
        }
    } else {
        &owner.fill
    };
    let descriptor = Descriptor {
        owner: Arc::as_ptr(owner) as usize,
        stroke,
    };
    let exact = Key::Exact {
        matrix: [
            transform.sx.to_bits(),
            transform.kx.to_bits(),
            transform.ky.to_bits(),
            transform.sy.to_bits(),
            transform.tx.to_bits(),
            transform.ty.to_bits(),
        ],
        width: pixmap.width(),
        height: pixmap.height(),
    };
    let image = CACHE.with(|cache| {
        cache.borrow_mut().with_strike(descriptor, |strike| {
            // Official strike descriptors + packed glyph identity are looked up
            // BEFORE generating metrics/image. Exact transform/viewport keys are a
            // local descriptor extension preserving this fallback's unquantized f32.
            if let Some(Some(image)) = strike.find_image(&exact) {
                return Ok(image);
            }
            #[cfg(test)]
            PREPARATIONS.with(|v| v.set(v.get() + 1));
            let Some(path) = source.clone().transform(transform) else {
                return Err(false);
            };
            let bounds = path.bounds();
            if bounds.width() < 0.0001
                || bounds.height() < 0.0001
                || bounds.width() > 1024.
                || bounds.height() > 1024.
                || path
                    .points
                    .iter()
                    .any(|p| !p.is_finite() || p.x.abs() > 8190. || p.y.abs() > 8190.)
            {
                return Err(false);
            }
            let origin = (bounds.left().floor() as i32, bounds.top().floor() as i32);
            let Some(ir) = crate::raster::IntRect::from_ltrb(
                origin.0,
                origin.1,
                bounds.right().ceil() as i32,
                bounds.bottom().ceil() as i32,
            ) else {
                return Err(false);
            };
            let viewport = pixmap.size().to_screen_int_rect(0, 0);
            if ir.intersect(&viewport.to_int_rect()).is_none() {
                return Err(true);
            }
            let contained = ir
                .to_screen_int_rect()
                .is_some_and(|r| viewport.contains(&r));
            let shifted = ShiftedIntRect::new(&viewport, 2).unwrap();
            let Some(edges) = SkBasicEdgeBuilder::build_edges(
                &path,
                if contained { None } else { Some(&shifted) },
                2,
            ) else {
                return Err(true);
            };
            let mut key = Vec::with_capacity(edges.len() * 19 + 8);
            key.extend_from_slice(&[ir.width() as i32, ir.height() as i32, i32::from(contained)]);
            if !contained {
                key.extend_from_slice(&[
                    -origin.0,
                    -origin.1,
                    pixmap.width() as i32 - origin.0,
                    pixmap.height() as i32 - origin.1,
                ]);
            }
            for edge in &edges {
                edge.append_glyph_signature(origin.0, origin.1, 2, &mut key);
            }
            let key_bytes = key.capacity() * std::mem::size_of::<i32>();

            let normalized = strike
                .prepare_image(
                    Key::Normalized(key),
                    || {
                        #[cfg(test)]
                        GENERATIONS.with(|v| v.set(v.get() + 1));
                        let mut recorder = Recorder {
                            origin,
                            rows: vec![],
                        };
                        crate::src::core::SkScan_AntiPath::fill_path(
                            &path,
                            FillRule::Winding,
                            &viewport,
                            &mut recorder,
                        );
                        Some(Entry {
                            resource: Arc::new(Resource {
                                owner: owner.clone(),
                                bounds: recorder.rows.iter().fold(None, |bounds, row| {
                                    let right = row.x
                                        + row.runs.iter().map(|&(n, _)| i32::from(n)).sum::<i32>();
                                    Some(match bounds {
                                        None => (row.x, row.y, right, row.y + 1),
                                        Some((l, t, r, b)) => (
                                            l.min(row.x),
                                            t.min(row.y),
                                            r.max(right),
                                            b.max(row.y + 1),
                                        ),
                                    })
                                }),
                                rows: recorder.rows,
                            }),
                            origin,
                            key_bytes,
                            owns_payload: true,
                        })
                    },
                    Entry::heap_bytes,
                )
                .unwrap();
            // Same with_strike borrow prevents an intervening purge from orphaning
            // an uncharged alias. The normalized owner and ALL exact aliases share
            // whole-strike lifetime; returned Arc handles preserve transient draws.
            Ok(strike
                .prepare_image(
                    exact,
                    || {
                        Some(Entry {
                            resource: normalized.resource.clone(),
                            origin,
                            key_bytes: 0,
                            owns_payload: false,
                        })
                    },
                    Entry::heap_bytes,
                )
                .unwrap())
        })
    });
    let image = match image {
        Ok(image) => image,
        Err(done) => return done,
    };
    let origin = image.origin;
    // SkScanClipper need not wrap the blitter when a BW rectangle contains
    // all emitted scan bounds. Use exact recorded event bounds (including
    // any AA edge pixels), not an approximate source-path bounding box.
    // Complex/AA clips and partially contained glyphs retain the old mask.
    let contained = bw_clip
        .zip(image.resource.bounds)
        .is_some_and(|(clip, (l, t, r, b))| {
            l + origin.0 >= clip.left()
                && t + origin.1 >= clip.top()
                && r + origin.0 <= clip.right()
                && b + origin.1 <= clip.bottom()
        });
    let mask = if contained { None } else { mask };
    let mut subpix = pixmap.as_mut();
    let mut subpix = subpix.as_subpixmap();
    let submask = mask.map(|m| m.as_submask());
    let Some(mut blitter) = RasterPipelineBlitter::new(paint, submask, &mut subpix) else {
        return true;
    };
    // The admitted glyph width is <=1024. Original sparse run partition and
    // alpha values are reconstructed in bounded draw-local scratch, retaining
    // original blitter arithmetic rather than using a different A8 blitter.
    let mut aa = [0u8; 1026];
    let mut runs = [None; 1026];
    for row in &image.resource.rows {
        if row.solid {
            blitter.blit_h(
                (row.x + origin.0) as u32,
                (row.y + origin.1) as u32,
                crate::raster::LengthU32::new(row.runs[0].0 as u32).unwrap(),
            );
            continue;
        }
        let mut at = 0;
        for &(width, alpha) in &row.runs {
            runs[at] = std::num::NonZeroU16::new(width);
            aa[at] = alpha;
            at += width as usize;
        }
        runs[at] = None;
        blitter.blit_anti_h(
            (row.x + origin.0) as u32,
            (row.y + origin.1) as u32,
            &mut aa[..=at],
            &mut runs[..=at],
        );
    }
    true
}

#[cfg(test)]
pub(crate) fn without_cache<R>(f: impl FnOnce() -> R) -> R {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            DISABLED.with(|v| v.set(self.0));
        }
    }
    let old = DISABLED.with(|v| v.replace(true));
    let _reset = Reset(old);
    f()
}
#[cfg(test)]
pub(crate) fn purge() {
    CACHE.with(|c| c.borrow_mut().purge_all());
}
#[cfg(test)]
pub(crate) fn stats() -> (u64, u64) {
    CACHE.with(|c| {
        let c = c.borrow();
        (c.hits(), GENERATIONS.with(|v| v.get()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::commands::*;
    use crate::src::core::SkCanvas::{GlyphRasterMode, SkCanvas};
    use crate::{PixelFormat, PixelStorage};
    use ttf_parser::Face;
    fn render(
        list: &ResourceContext,
        command: &DrawCommand,
        format: PixelFormat,
        clip: u8,
        scale: f32,
        cached: bool,
    ) -> Vec<u8> {
        let f = || {
            let mut c = SkCanvas::make_raster_direct_with_format(
                list,
                320,
                128,
                1280,
                PixelStorage::owned(vec![0; 320 * 128 * 4]),
                format,
            )
            .unwrap();
            c.set_glyph_mode(GlyphRasterMode::Outlines);
            c.set_scale(scale as f64);
            if clip != 0 {
                let radius = PaintCornerRadius { x: 7.5, y: 5.75 };
                c.replay_item(
                    &DrawCommand {
                        r#type: if clip == 3 {
                            CommandKind::kClipRoundedRect
                        } else {
                            CommandKind::kClipRect
                        },
                        rect: PaintRect {
                            x: 2.25,
                            y: 1.5,
                            width: 149.5,
                            height: 56.25,
                        },
                        corner_radii: PaintCornerRadii {
                            top_left: radius,
                            top_right: radius,
                            bottom_left: radius,
                            bottom_right: radius,
                        },
                        antialias: clip >= 2,
                        ..DrawCommand::default()
                    },
                    list,
                );
            }
            c.replay_item(command, list);
            c.finish_direct().into_vec()
        };
        if cached {
            f()
        } else {
            without_cache(f)
        }
    }
    fn list(bytes: &[u8]) -> ResourceContext<'_> {
        ResourceContext {
            resources: Some(ResourceCatalog {
                fonts: vec![FontFace {
                    bytes,
                    face_index: 0,
                    family: "Roboto",
                    native_family: "",
                    weight: 400.,
                    italic: false,
                    variations: vec![],
                }],
                images: vec![],
            }),
        }
    }
    fn command(bytes: &[u8]) -> DrawCommand {
        let face = Face::parse(bytes, 0).unwrap();
        DrawCommand {
            r#type: CommandKind::kDrawGlyphRun,
            font_size: 32.,
            synthetic_bold: true,
            color: Color {
                red: 0.8,
                green: 0.37,
                blue: 0.11,
                alpha: 0.5,
            },
            text_blob_origin: Offset { x: 12.25, y: 40.5 },
            glyphs: "MgWa"
                .chars()
                .enumerate()
                .map(|(i, ch)| PaintGlyph {
                    id: face.glyph_index(ch).unwrap().0 as u32,
                    offset: Offset {
                        x: i as f64 * 27.5,
                        y: 0.,
                    },
                    ..PaintGlyph::default()
                })
                .collect(),
            ..DrawCommand::default()
        }
    }
    #[test]
    fn resident_glyph_resources_match_legacy_integer_phase_clip_and_formats() {
        let bytes =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let list = list(bytes);
        purge();
        let mut item = command(bytes);
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for clip in 0..4 {
                for scale in [1., 2.] {
                    for (x, y) in [
                        (12., 40.),
                        (13., 41.),
                        (24., 52.),
                        (-7., 10.),
                        (7., -7.),
                        (297., 120.),
                        (100., 80.),
                    ] {
                        for phase in [0., 0.125, 0.499, 0.75] {
                            item.text_blob_origin = Offset {
                                x: x + phase,
                                y: y + phase,
                            };
                            let expected = render(&list, &item, format, clip, scale, false);
                            let cold = render(&list, &item, format, clip, scale, true);
                            let warm = render(&list, &item, format, clip, scale, true);
                            assert_eq!(cold,expected,"cold format={format:?} clip={clip} scale={scale} xy={x},{y} phase={phase}");
                            assert_eq!(warm,expected,"warm format={format:?} clip={clip} scale={scale} xy={x},{y} phase={phase}");
                        }
                    }
                }
            }
        }
        let (hits, misses) = stats();
        assert!(hits > 0 && misses > 0);
    }
    #[test]
    fn resident_glyph_resources_preserve_bold_stroke_italic_smoothing_and_paint_order() {
        let bytes =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let list = list(bytes);
        let mut item = command(bytes);
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for bold in [false, true] {
                for italic in [false, true] {
                    for stroke in [false, true] {
                        for aa in [FontSmoothing::kNone, FontSmoothing::kAuto] {
                            item.synthetic_bold = bold;
                            item.synthetic_italic = italic;
                            item.stroke_glyphs = stroke;
                            item.stroke_width = 1.5;
                            item.font_smoothing = aa;
                            for alpha in [0.125, 0.5, 1.] {
                                item.color.alpha = alpha;
                                let expected = render(&list, &item, format, 3, 2., false);
                                assert_eq!(render(&list,&item,format,3,2.,true),expected,"format={format:?} bold={bold} italic={italic} stroke={stroke} aa={aa:?} alpha={alpha}");
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn resident_glyph_resources_reuse_integer_positions_and_keep_clipped_variants() {
        let bytes =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let list = list(bytes);
        purge();
        let mut item = command(bytes);
        item.synthetic_bold = false;
        item.glyphs.truncate(1); // M has only line edges: integer phases are exact.
        item.text_blob_origin = Offset { x: 20., y: 60. };
        render(&list, &item, PixelFormat::Rgba8888, 0, 1., true);
        let (hits, misses) = stats();
        item.text_blob_origin = Offset { x: 30., y: 70. };
        render(&list, &item, PixelFormat::Bgrx8888, 0, 1., true);
        let (new_hits, new_misses) = stats();
        assert!(new_hits > hits);
        assert_eq!(new_misses, misses);
        for y in [5., -7., 5., -7.] {
            item.text_blob_origin.y = y;
            assert_eq!(
                render(&list, &item, PixelFormat::Bgrx8888, 0, 1., true),
                render(&list, &item, PixelFormat::Bgrx8888, 0, 1., false)
            );
        }
    }
    #[test]
    #[ignore]
    fn local_bold_glyph_raster_latency() {
        let bytes =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let list = list(bytes);
        let mut item = command(bytes);
        item.font_size = 44.;
        let face = Face::parse(bytes, 0).unwrap();
        item.glyphs = "Click and scroll test"
            .chars()
            .enumerate()
            .map(|(i, ch)| PaintGlyph {
                id: face.glyph_index(ch).unwrap().0 as u32,
                offset: Offset {
                    x: i as f64 * 20.,
                    y: 0.,
                },
                ..PaintGlyph::default()
            })
            .collect();
        for cached in [false, true] {
            let mut samples = Vec::new();
            for iteration in 0..25 {
                item.text_blob_origin = Offset {
                    x: 12.25,
                    y: 44.5 - ((iteration % 8) as f64 * 4.),
                };
                let now = std::time::Instant::now();
                let run = || {
                    let mut c = SkCanvas::make_raster_direct_with_format(
                        &list,
                        1280,
                        192,
                        5120,
                        PixelStorage::owned(vec![0; 1280 * 192 * 4]),
                        PixelFormat::Bgrx8888,
                    )
                    .unwrap();
                    c.set_glyph_mode(GlyphRasterMode::Outlines);
                    c.set_scale(2.);
                    c.replay_item(&item, &list);
                    c.finish_direct().into_vec()
                };
                let pixels = if cached { run() } else { without_cache(run) };
                std::hint::black_box(pixels);
                samples.push(now.elapsed().as_secs_f64() * 1000.);
            }
            samples.sort_by(f64::total_cmp);
            eprintln!(
                "glyph_raster cached={cached} median_ms={:.3} max_ms={:.3}",
                samples[samples.len() / 2],
                samples.last().unwrap()
            );
        }
    }
}

#[cfg(test)]
mod transform_and_budget_tests {
    use super::*;
    use crate::compat::commands::*;
    use crate::src::core::SkCanvas::{GlyphRasterMode, SkCanvas};
    use crate::{PixelFormat, PixelStorage};
    #[test]
    fn glyph_resources_preserve_shear_reflection_and_fractional_scale() {
        let bytes =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let face = ttf_parser::Face::parse(bytes, 0).unwrap();
        let list = ResourceContext {
            resources: Some(ResourceCatalog {
                fonts: vec![FontFace {
                    bytes,
                    face_index: 0,
                    family: "Roboto",
                    native_family: "",
                    weight: 400.,
                    italic: false,
                    variations: vec![],
                }],
                images: vec![],
            }),
        };
        let command = DrawCommand {
            r#type: CommandKind::kDrawGlyphRun,
            font_size: 32.,
            synthetic_bold: true,
            synthetic_italic: true,
            color: Color {
                red: 0.7,
                green: 0.2,
                blue: 0.83,
                alpha: 0.5,
            },
            text_blob_origin: Offset { x: 40.5, y: 65.25 },
            glyphs: vec![PaintGlyph {
                id: face.glyph_index('g').unwrap().0 as u32,
                ..PaintGlyph::default()
            }],
            ..DrawCommand::default()
        };
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for matrix in [
                Transform::from_row(0.7, 0., 0., 0.7, 0.125, 0.75),
                Transform::from_row(1., 0.2, -0.1, 1., 20., 10.),
                Transform::from_row(-1., 0., 0., 1., 130., 0.),
                Transform::from_row(0., 1., -1., 0., 130., 0.),
            ] {
                let render = || {
                    let mut c = SkCanvas::make_raster_direct_with_format(
                        &list,
                        240,
                        160,
                        960,
                        PixelStorage::owned(vec![0; 240 * 160 * 4]),
                        format,
                    )
                    .unwrap();
                    c.set_glyph_mode(GlyphRasterMode::Outlines);
                    c.state.transform = matrix;
                    c.replay_item(&command, &list);
                    c.finish_direct().into_vec()
                };
                let expected = without_cache(render);
                assert_eq!(render(), expected, "cold {format:?} {matrix:?}");
                assert_eq!(render(), expected, "warm {format:?} {matrix:?}");
            }
        }
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn glyph_resource_accounting_purges_keys_spans_and_path_owners() {
        purge();
        CACHE.with(|c| c.borrow_mut().set_limits(4096, 4));
        let path = crate::raster::PathBuilder::from_rect(
            crate::raster::Rect::from_xywh(0., 0., 25., 37.).unwrap(),
        );
        let owner = Arc::new(GlyphPaths {
            fill: path,
            stroke: None,
        });
        let weak = Arc::downgrade(&owner);
        let mut paint = Paint::default();
        paint.set_color_rgba8(91, 33, 173, 127);
        paint.anti_alias = true;
        let mut pixmap = Pixmap::new(96, 96).unwrap();
        for phase in 0..30 {
            assert!(draw(
                &mut pixmap,
                &owner,
                false,
                &paint,
                Transform::from_translate(20. + phase as f32 / 32., 40.125),
                None
            ));
            CACHE.with(|c| assert!(c.borrow().memory_used() <= 4096));
        }
        drop(owner);
        purge();
        assert!(weak.upgrade().is_none());
        CACHE.with(|c| {
            let mut c = c.borrow_mut();
            assert_eq!(c.memory_used(), 0);
            c.set_limits(
                crate::src::core::SkStrikeCache::SK_DEFAULT_FONT_CACHE_LIMIT,
                crate::src::core::SkStrikeCache::SK_DEFAULT_FONT_CACHE_COUNT_LIMIT,
            );
        });
    }
}

#[cfg(test)]
mod exact_descriptor_tests {
    use super::*;
    fn source() -> Arc<GlyphPaths> {
        Arc::new(GlyphPaths {
            fill: crate::raster::PathBuilder::from_rect(
                crate::raster::Rect::from_xywh(-3.25, -22.5, 19.75, 31.25).unwrap(),
            ),
            stroke: None,
        })
    }
    #[test]
    fn contained_bw_glyph_replay_matches_masked_scan_in_all_formats() {
        use crate::{PixelFormat, PixelStorage};
        let owner = source();
        let mut paint = Paint::default();
        paint.anti_alias = true;
        let matrix = Transform::from_translate(30.125, 40.75);
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for (x, y, w, h) in [(10, 10, 80, 80), (28, 20, 12, 24), (0, 0, 96, 96)] {
                let clip = crate::raster::IntRect::from_xywh(x, y, w, h).unwrap();
                let mut mask = Mask::new(96, 96).unwrap();
                for row in y..y + h as i32 {
                    mask.data_mut()[row as usize * 96 + x as usize
                        ..row as usize * 96 + (x as u32 + w) as usize]
                        .fill(255);
                }
                for alpha in [1, 127, 255] {
                    paint.set_color_rgba8(91, 33, 173, alpha);
                    let make = || {
                        Pixmap::install_pixels_with_format(
                            PixelStorage::owned(vec![255; 96 * 96 * 4]),
                            96,
                            96,
                            384,
                            format,
                        )
                        .unwrap()
                    };
                    let mut actual = make();
                    let mut expected = make();
                    assert!(draw_with_bw_clip(
                        &mut actual,
                        &owner,
                        false,
                        &paint,
                        matrix,
                        Some(&mask),
                        Some(clip)
                    ));
                    expected.fill_path(&owner.fill, &paint, FillRule::Winding, matrix, Some(&mask));
                    assert_eq!(
                        actual.data(),
                        expected.data(),
                        "format={format:?} clip={clip:?} alpha={alpha}"
                    );
                }
            }
        }
    }

    #[test]
    fn exact_transform_hits_bypass_path_and_fixed_edge_preparation() {
        purge();
        let owner = source();
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.set_color_rgba8(210, 33, 107, 127);
        let mut frame = Pixmap::new(96, 96).unwrap();
        let matrix = Transform::from_translate(11.125, 7.75); // crosses topviewport
        assert!(draw(&mut frame, &owner, false, &paint, matrix, None));
        let preparations = PREPARATIONS.with(|v| v.get());
        let generations = GENERATIONS.with(|v| v.get());
        let bytes = frame.data().to_vec();
        frame.fill(crate::raster::Color::TRANSPARENT);
        assert!(draw(&mut frame, &owner, false, &paint, matrix, None));
        assert_eq!(frame.data(), bytes);
        assert_eq!(PREPARATIONS.with(|v| v.get()), preparations);
        assert_eq!(GENERATIONS.with(|v| v.get()), generations);
        assert!(draw(
            &mut frame,
            &owner,
            false,
            &paint,
            Transform::from_translate(11.25, 7.75),
            None
        ));
        assert!(PREPARATIONS.with(|v| v.get()) > preparations);
        // Reverse to earliertransform uses its own exactrecord despite newphase.
        let preparations = PREPARATIONS.with(|v| v.get());
        assert!(draw(&mut frame, &owner, false, &paint, matrix, None));
        assert_eq!(PREPARATIONS.with(|v| v.get()), preparations);
    }
    #[test]
    fn exact_key_includes_viewport_and_aliases_share_charged_strike_lifetime() {
        purge();
        let owner = source();
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.set_color_rgba8(90, 40, 160, 127);
        let mut first = Pixmap::new(96, 96).unwrap();
        let mut second = Pixmap::new(32, 32).unwrap();
        let matrix = Transform::from_translate(23.125, 30.75);
        assert!(draw(&mut first, &owner, false, &paint, matrix, None));
        let preparations = PREPARATIONS.with(|v| v.get());
        assert!(draw(&mut second, &owner, false, &paint, matrix, None));
        assert!(PREPARATIONS.with(|v| v.get()) > preparations);
        let mut reference = Pixmap::new(32, 32).unwrap();
        reference.fill_path(&owner.fill, &paint, FillRule::Winding, matrix, None);
        assert_eq!(second.data(), reference.data());
        CACHE.with(|c| {
            let cache = c.borrow();
            let strike = cache.count();
            assert_eq!(strike, 1); // allprimary/exactrecords share one purgeunit
            assert!(cache.memory_used() > std::mem::size_of::<Descriptor>());
        });
        let weak = Arc::downgrade(&owner);
        drop(owner);
        purge();
        assert!(weak.upgrade().is_none());
    }
}

#[cfg(feature = "profiling")]
pub(crate) fn profile_stats() -> (usize, usize, u64, u64, u64) {
    CACHE.with(|c| {
        let c = c.borrow();
        (
            c.memory_used(),
            c.count(),
            c.hits(),
            c.misses(),
            c.evictions(),
        )
    })
}
