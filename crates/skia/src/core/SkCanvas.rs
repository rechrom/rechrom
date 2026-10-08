//! CPU canvas subset; replay adapters live in compat/canvas.rs.

use crate::compat::commands::*;
use crate::compat::geometry::*;
use crate::src::core::SkMaskBlurFilter::blur_alpha_mask;

use std::collections::HashMap;

use crate::raster::{
    ColorU8, FillRule, FilterQuality, GradientStop, IntSize, LineCap, LineJoin, LinearGradient,
    Mask, Paint, Path, PathBuilder, Pattern, Pixmap, Point, Rect, SpreadMode, Stroke, StrokeDash,
    Transform,
};
use ttf_parser::{Face, GlyphId, OutlineBuilder, Tag};

fn path_gap(path: &Path, top: f32, bottom: f32) -> Option<(f32, f32)> {
    use crate::raster::path64::{cubic64::Cubic64, line_cubic_intersections, point64::Point64};
    use crate::src::core::SkPath::PathSegment;
    let mut left = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    let mut expand = |x: f32| {
        if x.is_finite() {
            left = left.min(x);
            right = right.max(x);
        }
    };
    let mut current = Point::zero();
    for segment in path.segments() {
        let points: Vec<Point> = match segment {
            PathSegment::MoveTo(point) => {
                current = point;
                continue;
            }
            PathSegment::LineTo(end) => vec![current, end],
            PathSegment::QuadTo(control, end) => vec![current, control, end],
            PathSegment::CubicTo(control1, control2, end) => {
                vec![current, control1, control2, end]
            }
            PathSegment::Close => continue,
        };
        current = *points.last().unwrap();
        let segment_top = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let segment_bottom = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        if top > segment_bottom || segment_top > bottom {
            continue;
        }
        for point in &points {
            if top < point.y && point.y < bottom {
                expand(point.x);
            }
        }
        for y in [top, bottom] {
            match points.as_slice() {
                [a, b] => {
                    let t = (y - a.y) / (b.y - a.y);
                    if (0.0..1.0).contains(&t) {
                        expand(a.x + t * (b.x - a.x));
                    }
                }
                [a, b, c] => {
                    let roots = crate::src::core::SkQuads::RootsReal(
                        f64::from(a.y - 2.0 * b.y + c.y),
                        f64::from(2.0 * (b.y - a.y)),
                        f64::from(a.y - y),
                    );
                    // SkBezierQuad::Intersect accepts both endpoints. This is
                    // deliberately different from the line case's half-open
                    // range and matters for glyph contours whose extrema land
                    // exactly on the decoration stripe.
                    for t in roots.into_iter().filter(|t| (0.0..=1.0).contains(t)) {
                        let u = 1.0 - t;
                        expand(
                            (u * u * f64::from(a.x)
                                + 2.0 * u * t * f64::from(b.x)
                                + t * t * f64::from(c.x)) as f32,
                        );
                    }
                }
                [a, b, c, d] => {
                    let cubic = Cubic64::new([
                        Point64::from_point(*a),
                        Point64::from_point(*b),
                        Point64::from_point(*c),
                        Point64::from_point(*d),
                    ]);
                    let mut roots = [0.0; 3];
                    let count = line_cubic_intersections::horizontal_intersect(
                        &cubic,
                        f64::from(y),
                        &mut roots,
                    );
                    for &t in &roots[..count] {
                        expand(cubic.point_at_t(t).x as f32);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    (left < right).then_some((left, right))
}

use crate::cpu::hvgl::HvglTable;
use crate::cpu::image_sampling::{build_mip_image, draw_image_bitmap_opaque, mip_level_for_size};

#[derive(Clone)]
pub(crate) struct MCRec {
    pub(crate) transform: Transform,
    // SkAAClip shares its run storage across saved raster clips; copy only
    // when a subsequent clip mutates the saved coverage.
    pub(crate) clip: Option<std::sync::Arc<Mask>>,
    pub(crate) clip_summary: Option<crate::src::core::SkRasterClip::SkRasterClip>,
    pub(crate) clip_is_aa: bool,
    pub(crate) clip_runs: Option<std::sync::Arc<Vec<usize>>>,
    pub(crate) clip_encoding: crate::src::core::SkColorGlyphClip::CanvasClipOwner,
    pub(crate) clip_aa_tiles: Option<std::sync::Arc<Vec<bool>>>,
    pub(crate) device_origin: (u32, u32),
    // Layer device -> root raster coordinates; keeps outer replay tile anchors.
    pub(crate) tile_origin: (i32, i32),
}

// LOCAL_ADAPTER: owned immutable decoded sources cross raster canvases.
// SkMipmapAccessor::try_load_mips and SkBitmapCacheDesc generation/subset
// key retain source pixels independently of destination draw state.
// Retained immutable owners authenticate shared sources in constant time.
// Borrowed sources still require exact bytes and dimensions, never addresses.
struct ImageSourceSnapshot {
    width: u32,
    height: u32,
    rgba: std::sync::Arc<Vec<u8>>,
    opaque: bool,
    last_used: u64,
}
#[derive(Clone, Copy, Default, Debug)]
pub struct RasterImageCacheStats {
    pub source_conversions: u64,
    pub source_reuses: u64,
    pub source_invalidations: u64,
    pub mip_builds: u64,
}
/// Owned immutable source/mip cache. No composited pixels or rendered frame
/// are cached. IDs and input addresses alone never authenticate a new catalog.
pub struct RasterImageCache {
    images: HashMap<u64, Pixmap>,
    mip_images: HashMap<(u64, u32, i32, i32, u32, u32), Pixmap>,
    sources: HashMap<u64, ImageSourceSnapshot>,
    pixel_byte_limit: usize,
    clock: u64,
    stats: RasterImageCacheStats,
}
impl Default for RasterImageCache {
    fn default() -> Self {
        Self::with_pixel_byte_limit(64 * 1024 * 1024)
    }
}
impl RasterImageCache {
    pub fn with_pixel_byte_limit(pixel_byte_limit: usize) -> Self {
        Self {
            images: HashMap::new(),
            mip_images: HashMap::new(),
            sources: HashMap::new(),
            pixel_byte_limit,
            clock: 0,
            stats: RasterImageCacheStats::default(),
        }
    }
    pub fn stats(&self) -> RasterImageCacheStats {
        self.stats
    }
    pub fn resident_pixel_bytes(&self) -> usize {
        self.sources
            .values()
            .map(|s| s.rgba.len())
            .chain(self.images.values().map(|p| p.data().len()))
            .chain(self.mip_images.values().map(|p| p.data().len()))
            .fold(0usize, usize::saturating_add)
    }
    fn evict_to_limit(&mut self) {
        while self.resident_pixel_bytes() > self.pixel_byte_limit {
            let Some((&id, _)) = self.sources.iter().min_by_key(|(_, s)| s.last_used) else {
                break;
            };
            self.sources.remove(&id);
            self.images.remove(&id);
            self.mip_images.retain(|key, _| key.0 != id);
        }
    }
}

// Owned, cleared layer scratch only. A rendering thread may create a fresh
// canvas per frame, so keep allocation reuse outside individual canvas state.
// This never retains a root framebuffer, native mapping, image or clip mask.
#[derive(Default)]
struct LayerScratchPool {
    pixmaps: Vec<Pixmap>,
    bytes: usize,
}
impl LayerScratchPool {
    const BYTE_LIMIT: usize = 32 * 1024 * 1024;
    const ENTRY_LIMIT: usize = 8;

    fn take(&mut self, width: u32, height: u32) -> Option<Pixmap> {
        let at = self.pixmaps.iter().rposition(|p| {
            p.width() == width
                && p.height() == height
                && p.format == crate::raster::PixelFormat::Rgba8888
        })?;
        let pixmap = self.pixmaps.remove(at);
        self.bytes -= pixmap.data().len();
        Some(pixmap)
    }

    fn put(&mut self, pixmap: Pixmap) {
        let bytes = pixmap.data().len();
        if bytes > Self::BYTE_LIMIT {
            return;
        }
        while self.pixmaps.len() >= Self::ENTRY_LIMIT || self.bytes > Self::BYTE_LIMIT - bytes {
            let evicted = self.pixmaps.remove(0);
            self.bytes -= evicted.data().len();
        }
        self.bytes += bytes;
        self.pixmaps.push(pixmap);
    }
}
std::thread_local! {
    static LAYER_SCRATCH: std::cell::RefCell<LayerScratchPool> =
        std::cell::RefCell::new(LayerScratchPool::default());
}
fn cleared_layer_pixmap(width: u32, height: u32) -> Pixmap {
    if let Some(mut pixmap) = LAYER_SCRATCH
        .try_with(|pool| pool.borrow_mut().take(width, height))
        .ok()
        .flatten()
    {
        // Original saveLayer clear, including every transparent padding pixel.
        pixmap.data_mut().fill(0);
        pixmap
    } else {
        Pixmap::new(width, height).expect("valid bounded raster layer")
    }
}
fn recycle_layer_pixmap(pixmap: Pixmap) {
    // Only the completed child allocation is passed here. Reject external
    // storage explicitly, even if a future layer adapter installs a mapping.
    if pixmap.data.external_owner().is_some()
        || pixmap.format != crate::raster::PixelFormat::Rgba8888
    {
        return;
    }
    let _ = LAYER_SCRATCH.try_with(|pool| pool.borrow_mut().put(pixmap));
}

pub struct SkCanvas {
    pub(crate) pixmap: Pixmap,
    // Local conservative alpha metadata: white initialization and the admitted
    // SrcOver commands preserve opaque destination alpha. Unknown operations
    // invalidate it and retain the measured alpha-reduction readback.
    pub(crate) known_opaque: bool,
    pub(crate) images: HashMap<u64, Pixmap>,
    pub(crate) mip_images: HashMap<(u64, u32, i32, i32, u32, u32), Pixmap>,
    image_cache_enabled: bool,
    image_sources: HashMap<u64, ImageSourceSnapshot>,
    image_cache_validated: std::collections::HashSet<u64>,
    image_cache_pixel_byte_limit: usize,
    image_cache_clock: u64,
    image_cache_stats: RasterImageCacheStats,
    pub(crate) clip_product_cache: Option<crate::RasterClipProductCache>,
    pub(crate) state: MCRec,
    pub(crate) stack: Vec<SavedFrame>,
    pub(crate) mask_stack: Vec<Vec<crate::compat::commands::MaskLayer>>,
    pub(crate) glyph_mode: GlyphRasterMode,
    pub(crate) f16_surface: Option<crate::cpu::f16_surface::Surface>,
}

pub(crate) struct SavedFrame {
    pub(crate) state: MCRec,
    pub(crate) layer: Option<(Pixmap, f32, Option<crate::cpu::f16_surface::Surface>)>,
    pub(crate) filters: Vec<(f32, f32)>,
    pub(crate) layer_origin: (i32, i32),
    pub(crate) parent_opaque: bool,
    // PaintChunksToCcLayer masks restore with SkBlendMode::kDstIn.
    pub(crate) dst_in: bool,
}

impl SkCanvas {
    pub fn new(list: &ResourceContext, width: u32, height: u32) -> Self {
        let size = IntSize::from_wh(width, height).expect("valid raster size");
        let bytes =
            crate::src::core::SkPixmap::data_len_for_size(size).expect("valid raster byte length");
        // Allocate directly in the white erase state. This equivalent ownership
        // adapter avoids zero-initializing and then overwriting the same N32
        // storage; every new full-redraw framebuffer is still freshly allocated.
        let pixmap = Pixmap::from_vec(vec![255; bytes], size).expect("valid white raster size");
        Self::with_pixmap(list, pixmap)
    }

    /// Restricted SkCanvas::MakeRasterDirect / SkSurfaces::WrapPixels adapter.
    /// Raster directly into the supplied tightly packed RGBA8888 target. Its
    /// owner stays attached through saveLayer/restore and full command replay.
    /// Like new(), this browser factory starts with an opaque white backdrop.
    /// It initializes the supplied mapping, never allocates a replacement.
    pub fn make_raster_direct(
        list: &ResourceContext,
        width: u32,
        height: u32,
        row_bytes: usize,
        storage: crate::raster::PixelStorage,
    ) -> Option<Self> {
        Self::make_raster_direct_with_format(
            list,
            width,
            height,
            row_bytes,
            storage,
            crate::raster::PixelFormat::Rgba8888,
        )
    }

    /// Wrap target storage with explicit destination color/alpha encoding.
    /// BGRA follows SkRasterPipeline's load/store plus swap_rb stages. BGRX
    /// uses opaque alpha and writes zero to the host's unused byte; shader
    /// registers, RGBA source images and intermediate layers stay canonical.
    pub fn make_raster_direct_with_format(
        list: &ResourceContext,
        width: u32,
        height: u32,
        row_bytes: usize,
        storage: crate::raster::PixelStorage,
        format: crate::raster::PixelFormat,
    ) -> Option<Self> {
        let mut pixmap =
            Pixmap::install_pixels_with_format(storage, width, height, row_bytes, format)?;
        pixmap.fill(crate::raster::Color::WHITE);
        Some(Self::with_pixmap(list, pixmap))
    }

    /// Wrap initialized target pixels without clearing them. The caller owns
    /// their content and color encoding. A semantic clip may change native AA
    /// rounding; it is not an arbitrary final-write damage restriction. A
    /// retained renderer must close damage over affected geometry/layer bounds
    /// and prove complete pixel equivalence, or replay the full frame.
    /// `opaque` promises that the existing destination is
    /// opaque; BGRX represents alpha implicitly regardless of its unused byte.
    /// Like SkSurfaces::WrapPixels, this factory does not replace the mapping.
    pub fn make_raster_direct_with_format_preserving(
        list: &ResourceContext,
        width: u32,
        height: u32,
        row_bytes: usize,
        storage: crate::raster::PixelStorage,
        format: crate::raster::PixelFormat,
        opaque: bool,
    ) -> Option<Self> {
        let pixmap = Pixmap::install_pixels_with_format(storage, width, height, row_bytes, format)?;
        let mut canvas = Self::with_pixmap(list, pixmap);
        canvas.known_opaque = opaque || format == crate::raster::PixelFormat::Bgrx8888;
        Some(canvas)
    }

    /// Conservative device-space ink bounds for the supported platform mask
    /// path. Uses the same prepared font, variation selection and quantized
    /// baseline as drawGlyphRunList. A missing whole-font extent falls back to
    /// per-glyph metrics from the same native font instance. Unknown outline/color bounds
    /// return None so callers retain their complete active-clip damage.
    pub fn conservative_platform_glyph_run_bounds(
        list: &ResourceContext,
        item: &DrawCommand,
        transform: Transform,
    ) -> Option<PaintRect> {
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (list, item, transform);
            None
        }
        #[cfg(target_os = "macos")]
        {
            use crate::cpu::scaler_context_mac_ct::{with_scaler, SkScalerContextMac};
            if item.r#type != CommandKind::kDrawGlyphRun
                || item.transform != TransformMatrix::default()
                || item.synthetic_bold
                || item.synthetic_italic
                || item.stroke_glyphs
                || item.glyphs.iter().any(|g| g.canvas_rotation != 0)
                || ![
                    transform.sx,
                    transform.sy,
                    transform.kx,
                    transform.ky,
                    transform.tx,
                    transform.ty,
                ]
                .into_iter()
                .all(f32::is_finite)
                || !item.font_size.is_finite()
                || item.font_size <= 0.0
            {
                return None;
            }
            if item.glyphs.is_empty() {
                return Some(PaintRect::default());
            }
            let font = list
                .resources
                .as_ref()?
                .fonts
                .get(item.font_face_index as usize)?;
            let variations = if item.font_variations.is_empty() {
                &font.variations
            } else {
                &item.font_variations
            };
            let matrix = crate::cpu::glyph_position::mask_transform(transform);
            if !SkScalerContextMac::supports_transform(matrix) {
                return None;
            }
            with_scaler(|scaler| {
                let prepared = scaler.prepare_glyph_run(
                    item.font_face_index,
                    font,
                    item.font_size as f32,
                    variations,
                    matrix,
                );
                let mut union = [
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                ];
                for glyph in &item.glyphs {
                    let position = crate::cpu::glyph_position::mask_position_transform(
                        transform,
                        item.text_blob_origin.x as f32 + glyph.offset.x as f32,
                        item.text_blob_origin.y as f32 + glyph.offset.y as f32,
                    );
                    let bounds = if let Some(bounds) = prepared.conservative_bounds_at(position) {
                        bounds
                    } else {
                        let glyph_id = u16::try_from(glyph.id).ok()?;
                        prepared.native_allocation_bounds_at(glyph_id, position)?
                    };
                    if bounds[0] >= bounds[2] || bounds[1] >= bounds[3] {
                        continue;
                    }
                    union[0] = union[0].min(bounds[0]);
                    union[1] = union[1].min(bounds[1]);
                    union[2] = union[2].max(bounds[2]);
                    union[3] = union[3].max(bounds[3]);
                }
                if union[0] == f64::INFINITY {
                    Some(PaintRect::default())
                } else {
                    Some(PaintRect {
                        x: union[0],
                        y: union[1],
                        width: union[2] - union[0],
                        height: union[3] - union[1],
                    })
                }
            })
        }
    }

    /// Move source/mip data into a fresh canvas before any image draws.
    /// Every used source is revalidated; target initialization and all draw,
    /// sampling, clip, blend and layer operations remain unchanged.
    pub fn install_image_cache(&mut self, cache: RasterImageCache) {
        assert!(
            self.images.is_empty() && self.mip_images.is_empty(),
            "image cache must be installed before image draws"
        );
        self.images = cache.images;
        self.mip_images = cache.mip_images;
        self.image_sources = cache.sources;
        self.image_cache_pixel_byte_limit = cache.pixel_byte_limit;
        self.image_cache_clock = cache.clock.saturating_add(1);
        self.image_cache_stats = cache.stats;
        self.image_cache_validated.clear();
        self.image_cache_enabled = true;
    }
    /// Recover source/mip ownership after replay, before finishing the target.
    /// Budget eviction cannot change pixels already drawn in this canvas.
    pub fn take_image_cache(&mut self) -> RasterImageCache {
        if !self.image_cache_enabled {
            return RasterImageCache::default();
        }
        let mut cache = RasterImageCache {
            images: std::mem::take(&mut self.images),
            mip_images: std::mem::take(&mut self.mip_images),
            sources: std::mem::take(&mut self.image_sources),
            pixel_byte_limit: self.image_cache_pixel_byte_limit,
            clock: self.image_cache_clock,
            stats: self.image_cache_stats,
        };
        cache.evict_to_limit();
        self.image_cache_validated.clear();
        self.image_cache_enabled = false;
        cache
    }
    fn authenticate_cached_image(&mut self, resource: &Image<'_>) {
        if !self.image_cache_enabled || !self.image_cache_validated.insert(resource.id) {
            return;
        }
        let shared = resource.shared_pixels();
        let identical = self.image_sources.get(&resource.id).is_some_and(|old| {
            old.width == resource.width
                && old.height == resource.height
                && (shared.is_some_and(|owner| std::sync::Arc::ptr_eq(&old.rgba, owner))
                    || old.rgba.as_slice() == resource.rgba8)
        });
        if identical {
            let old = self.image_sources.get_mut(&resource.id).unwrap();
            old.last_used = self.image_cache_clock;
            // Adopt equal bytes' current owner so subsequent frames also avoid
            // a byte scan. Holding it prevents address recycling; edits use COW.
            if let Some(owner) = shared {
                if !std::sync::Arc::ptr_eq(&old.rgba, owner) {
                    old.rgba = owner.clone();
                }
            }
            self.image_cache_stats.source_reuses += 1;
        } else {
            if self.image_sources.contains_key(&resource.id) {
                self.image_cache_stats.source_invalidations += 1;
            }
            // Clear every subset/level, including recycled pointer addresses.
            self.images.remove(&resource.id);
            self.mip_images.retain(|key, _| key.0 != resource.id);
            self.image_sources.insert(
                resource.id,
                ImageSourceSnapshot {
                    width: resource.width,
                    height: resource.height,
                    rgba: shared
                        .cloned()
                        .unwrap_or_else(|| std::sync::Arc::new(resource.rgba8.to_vec())),
                    opaque: resource.rgba8.chunks_exact(4).all(|pixel| pixel[3] == 255),
                    last_used: self.image_cache_clock,
                },
            );
        }
    }

    fn with_pixmap(list: &ResourceContext, pixmap: Pixmap) -> Self {
        let images = HashMap::new();
        if let Some(resources) = &list.resources {
            let mut ids = std::collections::HashSet::new();
            for resource in &resources.images {
                assert!(resource.id != 0 && resource.width != 0 && resource.height != 0);
                assert_eq!(
                    resource.rgba8.len(),
                    resource.width as usize * resource.height as usize * 4,
                    "paint image RGBA byte count differs from dimensions"
                );
                assert!(ids.insert(resource.id), "duplicate paint image id");
            }
        }
        let (width, height) = (pixmap.width(), pixmap.height());
        // Like deferred SkImage raster access, decode/convert only when a
        // visible image draw actually requests pixels. The borrowed catalog
        // remains at replay's boundary; admitted images become owned here.
        Self {
            pixmap,
            known_opaque: true,
            images,
            mip_images: HashMap::new(),
            image_cache_enabled: false,
            image_sources: HashMap::new(),
            image_cache_validated: std::collections::HashSet::new(),
            image_cache_pixel_byte_limit: 64 * 1024 * 1024,
            image_cache_clock: 0,
            image_cache_stats: RasterImageCacheStats::default(),

            clip_product_cache: None,
            state: MCRec {
                transform: Transform::identity(),
                clip: None,
                clip_summary: None,
                clip_runs: None,
                clip_encoding: crate::src::core::SkColorGlyphClip::CanvasClipOwner::device(
                    crate::src::core::SkColorGlyphClip::Bounds([0, 0, width as i32, height as i32]),
                ),
                clip_aa_tiles: None,
                clip_is_aa: false,
                device_origin: (0, 0),
                tile_origin: (0, 0),
            },
            stack: Vec::new(),
            mask_stack: Vec::new(),
            glyph_mode: GlyphRasterMode::Platform,
            f16_surface: None,
        }
    }

    pub fn install_clip_product_cache(&mut self, cache: crate::RasterClipProductCache) {
        self.clip_product_cache = Some(cache);
    }
    pub fn take_clip_product_cache(&mut self) -> crate::RasterClipProductCache {
        self.clip_product_cache.take().unwrap_or_default()
    }

    pub(crate) fn save(&mut self) {
        self.stack.push(SavedFrame {
            state: self.state.clone(),
            layer: None,
            filters: Vec::new(),
            layer_origin: (0, 0),
            parent_opaque: self.known_opaque,
            dst_in: false,
        });
    }

    // SkCanvas.cpp internalSaveLayer / get_layer_mapping_and_bounds:
    // allocate the rounded-out intersection, then install its device origin.
    // The parent's AA clip is applied once, when drawDevice restores the layer.
    pub(crate) fn internalSaveLayer(&mut self, item: &DrawCommand, use_f16: bool) {
        self.save_layer_device(item, use_f16, Vec::new());
    }

    pub(crate) fn save_layer_filter(&mut self, item: &DrawCommand) {
        let tr = self.state.transform;
        let filters = item
            .filters
            .iter()
            .filter(|f| f.r#type == PaintFilterType::kBlur && f.amount > 0.0)
            .map(|f| {
                (
                    (f.amount as f32) * tr.sx.hypot(tr.kx),
                    (f.amount as f32) * tr.ky.hypot(tr.sy),
                )
            })
            .collect();
        self.save_layer_device(&DrawCommand::default(), false, filters);
    }

    fn save_layer_device(&mut self, item: &DrawCommand, use_f16: bool, filters: Vec<(f32, f32)>) {
        use crate::src::core::SkRasterClip::SkRasterClip;
        if !filters.is_empty() {
            self.ensure_clip_mask();
        }
        let device =
            crate::raster::IntRect::from_xywh(0, 0, self.pixmap.width(), self.pixmap.height())
                .expect("valid parent device bounds");
        let mut bounds = if self.state.clip_summary.is_some() || self.state.clip.is_some() {
            self.state
                .clip_summary
                .as_ref()
                .and_then(|c| c.getBounds())
                .and_then(|b| b.intersect(&device))
        } else {
            Some(device)
        };
        if item.rect.width > 0.0 && item.rect.height > 0.0 {
            let content = rect(item.rect)
                .and_then(|r| r.transform(self.state.transform))
                .and_then(|r| {
                    crate::raster::IntRect::from_ltrb(
                        r.left().floor() as i32,
                        r.top().floor() as i32,
                        r.right().ceil() as i32,
                        r.bottom().ceil() as i32,
                    )
                });
            bounds = bounds.and_then(|b| content.and_then(|c| b.intersect(&c)));
        }
        // Pixmap has no zero-size device. A one-pixel empty clip represents
        // Skia's no-pixels save layer and still participates in save/restore.
        let filtered = !filters.is_empty();
        if filtered {
            // getInputBounds: blur needs input beyond the output clip. Each
            // filter expands its input; the extra transparent pixel matches
            // internalSaveLayer's decal-sampling padding.
            let bx = filters
                .iter()
                .map(|(x, _)| (x * 3.0).ceil() as i32)
                .sum::<i32>()
                + 1;
            let by = filters
                .iter()
                .map(|(_, y)| (y * 3.0).ceil() as i32)
                .sum::<i32>()
                + 1;
            bounds = bounds.and_then(|b| {
                crate::raster::IntRect::from_ltrb(
                    b.left() - bx,
                    b.top() - by,
                    b.right() + bx,
                    b.bottom() + by,
                )
            });
        }
        let (left, top, width, height) =
            bounds.map_or((0, 0, 1, 1), |b| (b.left(), b.top(), b.width(), b.height()));
        let child = cleared_layer_pixmap(width, height);
        let parent = std::mem::replace(&mut self.pixmap, child);
        let previous = self.f16_surface.take();
        if use_f16 {
            self.f16_surface = Some(crate::cpu::f16_surface::Surface::new(
                width as usize * height as usize,
            ));
        }
        self.stack.push(SavedFrame {
            state: self.state.clone(),
            layer: Some((parent, item.opacity.clamp(0.0, 1.0), previous)),
            filters,
            layer_origin: (left, top),
            parent_opaque: self.known_opaque,
            dst_in: false,
        });
        self.known_opaque = false;
        self.state.transform.tx -= left as f32;
        self.state.transform.ty -= top as f32;
        let mut clip_summary = SkRasterClip::default();
        let child_clip = bounds.and_then(|_| {
            if filtered {
                crate::raster::IntRect::from_xywh(1, 1, width - 2, height - 2)
            } else {
                crate::raster::IntRect::from_xywh(0, 0, width, height)
            }
        });
        clip_summary.setRect(child_clip);
        self.state.clip = if filtered || bounds.is_none() {
            let mut mask = Mask::new(width, height).unwrap();
            if let Some(b) = child_clip {
                for y in b.top() as usize..b.bottom() as usize {
                    mask.data_mut()[y * width as usize + b.left() as usize
                        ..y * width as usize + b.right() as usize]
                        .fill(255);
                }
            }
            Some(std::sync::Arc::new(mask))
        } else {
            None
        };
        self.state.clip_encoding = child_clip.map_or_else(
            || {
                let mut empty = crate::src::core::SkColorGlyphClip::CanvasClipOwner::device(
                    crate::src::core::SkColorGlyphClip::Bounds([0, 0, 1, 1]),
                );
                empty.set_empty();
                empty
            },
            |b| {
                crate::src::core::SkColorGlyphClip::CanvasClipOwner::device(
                    crate::src::core::SkColorGlyphClip::Bounds([
                        b.left(),
                        b.top(),
                        b.right(),
                        b.bottom(),
                    ]),
                )
            },
        );
        self.state.clip_summary = Some(clip_summary);
        self.state.device_origin = (0, 0);
        self.state.tile_origin.0 += left;
        self.state.tile_origin.1 += top;
        self.state.clip_is_aa = false;
        self.state.clip_runs = None;
        self.state.clip_aa_tiles = None;
    }

    pub(crate) fn restore(&mut self) {
        if self.stack.last().is_some_and(|frame| frame.dst_in) {
            self.restore_mask_layer(true);
            return;
        }
        if let Some(frame) = self.stack.pop() {
            if let Some((parent, opacity, mut previous)) = frame.layer {
                #[cfg(feature = "profiling")]
                let layer_started =
                    std::env::var_os("SKIA_TRACE_LAYERS").map(|_| std::time::Instant::now());
                #[cfg(feature = "profiling")]
                let layer_kind = (
                    self.f16_surface.is_some(),
                    previous.is_some(),
                    self.pixmap.width(),
                    self.pixmap.height(),
                    frame.filters.len(),
                );
                for (sigma_x, sigma_y) in frame.filters {
                    crate::cpu::layer_filters::blur(&mut self.pixmap, sigma_x, sigma_y);
                }
                #[cfg(feature = "profiling")]
                let blur_elapsed = layer_started.map(|started| started.elapsed());
                let child_pixmap = std::mem::replace(&mut self.pixmap, parent);
                let format = self.pixmap.format;
                let parent_width = self.pixmap.width() as usize;
                let child_width = child_pixmap.width() as usize;
                let parent_height = self.pixmap.height() as i32;
                let origin = frame.layer_origin;
                if let Some(child) = self.f16_surface.take() {
                    child.composite_into_mask(
                        self.pixmap.data_mut(),
                        previous.as_mut(),
                        opacity,
                        frame.state.clip.as_deref(),
                        format,
                        child_width,
                        parent_width,
                        origin,
                    );
                } else {
                    // N32 restore uses SkOpts' variable-source row blitter.
                    // Retain this adapter's byte-exact opacity rounding; build
                    // its table once per layer instead of multiplying four
                    // floats at every destination pixel. F16 keeps its path.
                    let opacity_table: Option<[u8; 256]> = (previous.is_none() && opacity != 1.0)
                        .then(|| {
                            std::array::from_fn(|v| (v as f32 * opacity).round_ties_even() as u8)
                        });
                    let mut scaled_row = if opacity_table.is_some() {
                        vec![0; child_width * 4]
                    } else {
                        Vec::new()
                    };
                    for (child_row, src_row) in child_pixmap
                        .data()
                        .chunks_exact(child_width * 4)
                        .enumerate()
                    {
                        let y = child_row as i32 + origin.1;
                        if y < 0 || y >= parent_height {
                            continue;
                        }
                        let left = (-origin.0).max(0) as usize;
                        let right =
                            child_width.min((parent_width as i32 - origin.0).max(0) as usize);
                        let (mut left, mut right) = (left, right);
                        if let Some(mask) = frame.state.clip.as_deref() {
                            let bounds = mask.storage_bounds();
                            if y < bounds.top() || y >= bounds.bottom() {
                                continue;
                            }
                            left = left
                                .max((i64::from(bounds.left()) - i64::from(origin.0)).max(0)
                                    as usize);
                            right = right
                                .min((i64::from(bounds.right()) - i64::from(origin.0)).max(0)
                                    as usize);
                        }
                        if left >= right {
                            continue;
                        }
                        let parent_start =
                            y as usize * parent_width + (origin.0 + left as i32) as usize;
                        let dst_row = &mut self.pixmap.data_mut()
                            [parent_start * 4..(parent_start + right - left) * 4];
                        if previous.is_none() {
                            let source = &src_row[left * 4..right * 4];
                            let source = if let Some(table) = &opacity_table {
                                for (out, &value) in
                                    scaled_row[..source.len()].iter_mut().zip(source)
                                {
                                    *out = table[value as usize];
                                }
                                &scaled_row[..source.len()]
                            } else {
                                source
                            };
                            let clip = frame.state.clip.as_deref().map(|mask| {
                                mask.row_range(
                                    y as u32,
                                    (parent_start % parent_width) as u32,
                                    (parent_start % parent_width + right - left) as u32,
                                )
                            });
                            let mut x = 0;
                            while x < right - left {
                                let coverage = clip.map_or(255, |row| row[x]);
                                let end = clip.map_or(right - left, |row| {
                                    crate::compat::analytic_masks::equal_byte_run_end(
                                        row, x, coverage,
                                    )
                                });
                                if coverage == 255 {
                                    crate::src::opts::SkBlitRow_opts::blit_row_s32a_opaque(
                                        &mut dst_row[x * 4..end * 4],
                                        &source[x * 4..end * 4],
                                        format,
                                    );
                                } else if coverage != 0 {
                                    // Preserve SkDraw::drawDevice's combined
                                    // SrcOver/AA arithmetic at partial edges.
                                    let scale = u32::from(coverage) + 1;
                                    for (src, dst) in source[x * 4..end * 4]
                                        .chunks_exact(4)
                                        .zip(dst_row[x * 4..end * 4].chunks_exact_mut(4))
                                    {
                                        let inverse = if src[3] == 255 {
                                            256 - scale
                                        } else {
                                            let p = 65535 - u32::from(src[3]) * scale;
                                            (p + (p >> 8)) >> 8
                                        };
                                        for c in 0..4 {
                                            let channel = format.channel(c);
                                            dst[channel] = ((u32::from(src[c]) * scale
                                                + u32::from(dst[channel]) * inverse)
                                                >> 8)
                                                .min(255)
                                                as u8;
                                        }
                                        if format == crate::raster::PixelFormat::Bgrx8888 {
                                            dst[3] = 0;
                                        }
                                    }
                                }
                                x = end;
                            }
                            continue;
                        }
                        let source = &src_row[left * 4..right * 4];
                        let clip = frame.state.clip.as_deref().map(|mask| {
                            mask.row_range(
                                y as u32,
                                (parent_start % parent_width) as u32,
                                (parent_start % parent_width + right - left) as u32,
                            )
                        });
                        let surface = previous.as_mut().expect("F16 parent restore");
                        let mut x = 0;
                        while x < right - left {
                            let coverage = clip.map_or(255, |row| row[x]);
                            let end = clip.map_or(right - left, |row| {
                                crate::compat::analytic_masks::equal_byte_run_end(row, x, coverage)
                            });
                            surface.composite_n32_span(
                                parent_start + x,
                                &source[x * 4..end * 4],
                                &mut dst_row[x * 4..end * 4],
                                opacity,
                                coverage,
                                format,
                            );
                            x = end;
                        }
                    }
                }
                #[cfg(feature = "profiling")]
                if let (Some(started), Some(blur_elapsed)) = (layer_started, blur_elapsed) {
                    eprintln!("raster-layer child_f16={} parent_f16={} width={} height={} filters={} blur_ms={:.3} composite_ms={:.3}",
                        layer_kind.0, layer_kind.1, layer_kind.2, layer_kind.3, layer_kind.4,
                        blur_elapsed.as_secs_f64()*1000.0, (started.elapsed()-blur_elapsed).as_secs_f64()*1000.0);
                }
                // The compositor has finished borrowing the child. Retain
                // only its owned scratch allocation, never the restored parent.
                recycle_layer_pixmap(child_pixmap);
                self.f16_surface = previous;
                // Restoring a layer composites with SrcOver. An opaque parent
                // remains opaque independently of the child's alpha/filters.
                self.known_opaque = frame.parent_opaque;
            }
            self.state = frame.state;
        }
    }

    // cpp: skia_renderer/skia_renderer.cc:1074-1110

    pub(crate) fn concat(&mut self, matrix: [f64; 16]) {
        let transform = Transform::from_row(
            matrix[0] as f32,
            matrix[1] as f32,
            matrix[4] as f32,
            matrix[5] as f32,
            matrix[12] as f32,
            matrix[13] as f32,
        );
        self.state.transform = self.state.transform.pre_concat(transform);
    }

    /// Compatibility bridge for pixel-mask-only raster backends. The native
    /// BW rectangle state stays as SkRasterClip bounds until one such backend
    /// requires its alpha plane; save() continues sharing that plane via Arc.
    pub(crate) fn ensure_clip_mask(&mut self) {
        if self.state.clip.is_some() {
            return;
        }
        let Some(summary) = self.state.clip_summary else {
            return;
        };
        // Opaque rectangular coverage needs only its device-relative bounds.
        // Keep the authoritative AA/BW receiver and encoded clip unchanged;
        // full-device rectangles retain dense storage for legacy byte views.
        if let Some(bounds) = summary.getBounds().filter(|_| summary.isRect()) {
            if bounds.width() != self.pixmap.width() || bounds.height() != self.pixmap.height() {
                let mut mask = Mask::new_bounded(self.pixmap.width(), self.pixmap.height(), bounds)
                    .expect("valid bounded rectangular device clip");
                for y in bounds.top() as u32..bounds.bottom() as u32 {
                    mask.row_range_mut(y, bounds.left() as u32, bounds.right() as u32)
                        .fill(255);
                }
                self.state.clip = Some(std::sync::Arc::new(mask));
                return;
            }
        }
        let width = self.pixmap.width() as usize;
        let mut mask = Mask::new(self.pixmap.width(), self.pixmap.height())
            .expect("valid device clip dimensions");
        if let Some(bounds) = summary.getBounds() {
            for y in bounds.top() as usize..bounds.bottom() as usize {
                mask.data_mut()
                    [y * width + bounds.left() as usize..y * width + bounds.right() as usize]
                    .fill(255);
            }
        }
        self.state.clip = Some(std::sync::Arc::new(mask));
    }

    pub(crate) fn refresh_clip_summary(&mut self) {
        self.state.clip_summary = self.state.clip.as_deref().map(|mask| {
            crate::src::core::SkRasterClip::SkRasterClip::from_mask_builder(
                mask,
                Some(mask.storage_bounds()),
            )
            .expect("valid raster clip dimensions")
        });
    }

    pub(crate) fn update_clip_kind(&mut self, antialias: bool) {
        // An interior rectangular intersection can retain SkAAClip RLE state.
        self.state.clip_is_aa |= antialias;
        self.refresh_clip_summary();
    }

    pub(crate) fn refresh_encoded_clip_helpers(&mut self) {
        self.state.clip_is_aa = !self.state.clip_encoding.is_bw();
        self.state.clip_runs = self
            .state
            .clip_encoding
            .row_run_starts(self.pixmap.width() as usize)
            .map(std::sync::Arc::new);
        // The current analytic walker selects its real receiver from the
        // encoded clip/clip_is_aa and ignores the legacy tile-mode argument.
        // Keep native run boundaries, without rescanning mask pixels to build
        // a virtual-tile classification that no draw consumes.
        self.state.clip_aa_tiles = None;
    }

    pub(crate) fn clipPath(&mut self, path: &Path, antialias: bool) {
        self.clip_path_with_rule(path, antialias, FillRule::Winding);
    }

    pub(crate) fn clip_path_with_rule(&mut self, path: &Path, antialias: bool, rule: FillRule) {
        self.clip_path_with_mask(path, antialias, rule, None);
    }

    pub(crate) fn clip_path_with_mask(
        &mut self,
        path: &Path,
        antialias: bool,
        rule: FillRule,
        mut prepared: Option<crate::src::core::SkColorGlyphClip::PathProduct>,
    ) {
        use crate::src::core::SkRasterClip::SkRasterClip;
        // setPath already scans within the encoded receiver's old bounds.
        // An unmaterialized opaque rectangle contributes only that boundary;
        // do not allocate/fill a device-sized 255 mask just to discard it.
        // Keep AA/BW owner selection and every complex-coverage path intact.
        let rectangular_bounds_only =
            self.state.clip.is_none() && self.state.clip_summary.is_some_and(|c| c.isRect());
        if !rectangular_bounds_only {
            self.ensure_clip_mask();
        }
        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let shape = path.bounds().transform(self.state.transform).and_then(|r| {
            crate::raster::IntRect::from_ltrb(
                (r.left().floor() - 1.0).max(0.0).min(width as f32) as i32,
                (r.top().floor() - 1.0).max(0.0).min(height as f32) as i32,
                (r.right().ceil() + 1.0).max(0.0).min(width as f32) as i32,
                (r.bottom().ceil() + 1.0).max(0.0).min(height as f32) as i32,
            )
        });
        let support = if self.state.clip.is_some() || rectangular_bounds_only {
            shape.and_then(|b| {
                self.state
                    .clip_summary
                    .as_ref()
                    .and_then(|c| c.getBounds())
                    .and_then(|c| b.intersect(&c))
            })
        } else {
            shape
        };
        // One receiver product updates metadata even on the existing
        // prepared-mask and fallback routes; no stale clip state survives.
        let transform = self.state.transform;
        let mut mask = None;
        let cache = &mut self.clip_product_cache;
        self.state
            .clip_encoding
            .intersect_path(antialias, |bounds, kind| {
                let product = prepared.take().unwrap_or_else(|| {
                    crate::src::core::SkColorGlyphClip::produce_path_cached(
                        cache, path, None, rule, antialias, transform, width, height, bounds, kind,
                    )
                });
                mask = Some(product.mask);
                product.owner
            });
        let mut mask =
            mask.unwrap_or_else(|| Mask::new(width, height).expect("valid empty raster clip"));
        if let Some(shape) = shape.filter(|_| self.state.clip.is_some() || rectangular_bounds_only)
        {
            // A path product owns zero coverage outside its physical storage.
            // Intersect only actual rows; a legacy full-device mutable slice
            // would expand the pristine packed mask before every AA clip.
            if let Some(area) = shape.intersect(&mask.storage_bounds()) {
                let rectangular_old = self.state.clip_summary.is_some_and(|c| c.isRect());
                // Crossing the old packed bounds uses a bounded scratch row;
                // it must not request old.row_range() outside its storage.
                let mut old_row = if rectangular_old {
                    Vec::new()
                } else {
                    vec![0; area.width() as usize]
                };
                for y in area.top() as u32..area.bottom() as u32 {
                    let row = mask.row_range_mut(y, area.left() as u32, area.right() as u32);
                    if let Some(b) = support
                        .filter(|b| y as i32 >= b.top() && (y as i32) < b.bottom())
                        .and_then(|b| b.intersect(&area))
                    {
                        let left = (b.left() - area.left()) as usize;
                        let right = (b.right() - area.left()) as usize;
                        row[..left].fill(0);
                        row[right..].fill(0);
                        if !rectangular_old {
                            let previous = &mut old_row[..right - left];
                            self.state
                                .clip
                                .as_ref()
                                .expect("complex clip coverage is materialized")
                                .copy_row_range(y, b.left() as u32, previous);
                            for (a, &b) in row[left..right].iter_mut().zip(previous.iter()) {
                                *a = crate::cpu::mask_blitter::mul_div_255_round(*a, b);
                            }
                        }
                    } else {
                        row.fill(0);
                    }
                }
            }
        }
        self.state.clip_summary = SkRasterClip::from_mask_builder(&mask, support);
        self.state.clip = Some(std::sync::Arc::new(mask));
        self.refresh_encoded_clip_helpers();
    }

    /// Dense compatibility bytes for a native encoded Difference operation.
    /// Ownership/class/run boundaries have already been updated from the real
    /// receiver. Never infer an AA owner from these completed coverage bytes.
    pub(crate) fn install_difference_clip(
        &mut self,
        old_bounds: crate::src::core::SkColorGlyphClip::Bounds,
        hole: Option<Mask>,
        bw_rect: Option<crate::src::core::SkColorGlyphClip::Bounds>,
    ) {
        use crate::src::core::SkRasterClip::SkRasterClip;
        if self.state.clip_encoding.bounds().is_none() {
            self.state.clip = None;
            self.state.clip_summary = Some(SkRasterClip::default());
            self.refresh_encoded_clip_helpers();
            return;
        }
        if hole.is_none() && bw_rect.is_none() {
            self.refresh_encoded_clip_helpers();
            return;
        }
        let bounds = crate::raster::IntRect::from_ltrb(
            old_bounds.0[0],
            old_bounds.0[1],
            old_bounds.0[2],
            old_bounds.0[3],
        )
        .expect("nonempty device-contained difference bounds");
        let mut result = Mask::new_bounded(self.pixmap.width(), self.pixmap.height(), bounds)
            .expect("valid difference clip storage");
        let mut operand = vec![0; bounds.width() as usize];
        for y in bounds.top() as u32..bounds.bottom() as u32 {
            let row = result.row_range_mut(y, bounds.left() as u32, bounds.right() as u32);
            if let Some(old) = &self.state.clip {
                old.copy_row_range(y, bounds.left() as u32, row);
            } else {
                row.fill(255);
            }
            if let Some(hole) = &hole {
                hole.copy_row_range(y, bounds.left() as u32, &mut operand);
                for (old, &hole) in row.iter_mut().zip(&operand) {
                    *old = crate::cpu::mask_blitter::mul_div_255_round(*old, 255 - hole);
                }
            } else if let Some(rect) = bw_rect.filter(|r| y as i32 >= r.0[1] && (y as i32) < r.0[3])
            {
                let left = rect.0[0].max(bounds.left());
                let right = rect.0[2].min(bounds.right());
                if left < right {
                    row[(left - bounds.left()) as usize..(right - bounds.left()) as usize].fill(0);
                }
            }
        }
        self.state.clip_summary = SkRasterClip::from_mask_builder(&result, Some(bounds));
        self.state.clip = Some(std::sync::Arc::new(result));
        self.refresh_encoded_clip_helpers();
    }

    pub(crate) fn drawPath(
        &mut self,
        path: &Path,
        color_value: Color,
        antialias: bool,
        rule: FillRule,
    ) {
        if self.f16_surface.is_some() {
            let mut mask =
                Mask::new(self.pixmap.width(), self.pixmap.height()).expect("valid surface");
            mask.fill_path(path, rule, antialias, self.state.transform);
            self.blend_f16_mask_rows(
                &mask,
                color_value,
                0,
                self.pixmap.width() as usize,
                0,
                self.pixmap.height() as usize,
            );
            return;
        }
        self.ensure_clip_mask();
        let paint = solid_paint(color_value, antialias);
        self.pixmap.fill_path(
            path,
            &paint,
            rule,
            self.state.transform,
            self.state.clip.as_deref(),
        );
    }

    pub(crate) fn drawRect(&mut self, bounds: PaintRect, color_value: Color, antialias: bool) {
        // SkCanvas::drawRect sorts its source rectangle before SkDraw rejects
        // empty fills. Test emptiness before device-space rounding.
        let l = bounds.x as f32;
        let t = bounds.y as f32;
        let r = l + bounds.width as f32;
        let b = t + bounds.height as f32;
        if ![l, t, r, b].into_iter().all(f32::is_finite) || l == r || t == b {
            return;
        }
        if let Some(bounds) = Rect::from_ltrb(l.min(r), t.min(b), l.max(r), t.max(b)) {
            // SkDraw routes an AA rectangle through path scanning when its
            // matrix does not preserve axis-aligned rectangles.
            let t = self.state.transform;
            let stays_rect = (t.kx == 0.0 && t.ky == 0.0) || (t.sx == 0.0 && t.sy == 0.0);
            // skia_renderer.cc:1106-1110 disables edge AA for every
            // rect-preserving matrix, including a cropped layer's translation.
            // Testing only identity made damage-layer bounds change pixels.
            let antialias = antialias && !stays_rect;
            if antialias {
                if let Some(commands) = rect_commands(PaintRect {
                    x: bounds.left() as f64,
                    y: bounds.top() as f64,
                    width: bounds.width() as f64,
                    height: bounds.height() as f64,
                }) {
                    if let Some(mask) = crate::cpu::analytic_aa::path_spans_with_clip(
                        &commands,
                        false,
                        t,
                        self.pixmap.width(),
                        self.pixmap.height(),
                        self.state.clip.as_deref(),
                        self.state.device_origin,
                        self.state.clip_is_aa,
                        self.state.clip_aa_tiles.as_deref().map(|v| v.as_slice()),
                        self.state.clip_summary.as_ref(),
                    ) {
                        self.blend_rounded_spans(&mask, color_value);
                        return;
                    }
                }
            }
            // SkDraw's aliased identity-matrix rectangle uses SkRect::round,
            // which floors coordinate + 0.5, including half-pixel edges.
            let bounds = if self.state.transform.is_identity() {
                let Some(bounds) = Rect::from_ltrb(
                    (bounds.left() + 0.5).floor(),
                    (bounds.top() + 0.5).floor(),
                    (bounds.right() + 0.5).floor(),
                    (bounds.bottom() + 0.5).floor(),
                ) else {
                    return;
                };
                bounds
            } else {
                bounds
            };
            if self.f16_surface.is_some() {
                self.drawPath(
                    &PathBuilder::from_rect(bounds),
                    color_value,
                    antialias && !self.state.transform.is_identity(),
                    FillRule::Winding,
                );
                return;
            }
            {
                if let Some(device) = bounds.transform(self.state.transform) {
                    let t = self.state.transform;
                    if t.kx == 0.0
                        && t.ky == 0.0
                        && [device.left(), device.top(), device.right(), device.bottom()]
                            .iter()
                            .all(|v| v.fract() == 0.0)
                    {
                        // SkBlitter emits horizontal spans. The AA clip can
                        // split each span into constant-coverage runs.
                        let source = crate::cpu::mask_blitter::premultiply(color_value);
                        let left = device.left().max(0.0).min(self.pixmap.width() as f32) as u32;
                        let right = device.right().max(0.0).min(self.pixmap.width() as f32) as u32;
                        // The opaque blitRect operation is one contiguous
                        // constant fill when its span covers the target stride.
                        // Clip by BW bounds first; complex AA masks keep the
                        // original row-by-row coverage dispatch.
                        let full_rows = self
                            .state
                            .clip_summary
                            .as_ref()
                            .and_then(|clip| {
                                clip.isRect()
                                    .then(|| clip.getBounds())
                                    .flatten()
                                    .filter(|b| {
                                        b.left() == 0 && b.right() == self.pixmap.width() as i32
                                    })
                                    .map(|b| (b.top() as u32, b.bottom() as u32))
                            })
                            .or_else(|| {
                                (self.state.clip_summary.is_none() && self.state.clip.is_none())
                                    .then_some((0, self.pixmap.height()))
                            });
                        if source[3] == 255 && left == 0 && right == self.pixmap.width() {
                            if let Some((clip_top, clip_bottom)) = full_rows {
                                let top = (device.top().max(0.0) as u32).max(clip_top);
                                let bottom =
                                    (device.bottom().min(self.pixmap.height() as f32).max(0.0)
                                        as u32)
                                        .min(clip_bottom);
                                if top < bottom {
                                    let row_bytes = self.pixmap.width() as usize * 4;
                                    let format = self.pixmap.format;

                                    crate::src::core::SkBlitRow_D32::blend_span_format(
                                        &mut self.pixmap.data_mut()
                                            [top as usize * row_bytes..bottom as usize * row_bytes],
                                        source,
                                        255,
                                        false,
                                        format,
                                    );
                                }
                                return;
                            }
                        }
                        for y in device.top().max(0.0) as u32
                            ..device.bottom().min(self.pixmap.height() as f32).max(0.0) as u32
                        {
                            self.blend_solid_span(
                                (y * self.pixmap.width() + left) as usize,
                                (right - left) as usize,
                                source,
                                255,
                                false,
                            );
                        }
                        return;
                    }
                }
            }
            let paint = solid_paint(
                color_value,
                antialias && !self.state.transform.is_identity(),
            );
            self.ensure_clip_mask();
            self.pixmap.fill_rect(
                bounds,
                &paint,
                self.state.transform,
                self.state.clip.as_deref(),
            );
        }
    }

    /// Draws a filled rectangle while retaining Skia's edge coverage.
    ///
    /// Blink's text-decoration painter snaps the block axis, but keeps the
    /// fractional inline end and asks GraphicsContext::DrawRect for edge AA.
    /// Generic layout rectangles intentionally take `drawRect`'s aliased
    /// rect-preserving fast path; this entry point is therefore limited to
    /// decoration geometry whose paint contract explicitly requests AA.
    pub(crate) fn drawTextDecorationRect(&mut self, bounds: Rect, color_value: Color) {
        let transform = self.state.transform;
        let (bounds, transform) = if transform.kx == 0.0 && transform.ky == 0.0 {
            let Some(device_bounds) = bounds.transform(transform) else {
                return;
            };
            // RasterPipeline's dedicated rect scanner carries 8-bit edge
            // coverage. Supplying the already mapped device rect avoids the
            // generic transformed-path fallback and its coarser path AA.
            (device_bounds, Transform::identity())
        } else {
            (bounds, transform)
        };
        if self.f16_surface.is_some() {
            let mut mask = Mask::new(self.pixmap.width(), self.pixmap.height())
                .expect("valid decoration mask");
            mask.fill_path(
                &PathBuilder::from_rect(bounds),
                FillRule::Winding,
                true,
                transform,
            );
            self.blend_f16_mask_rows(
                &mask,
                color_value,
                0,
                self.pixmap.width() as usize,
                0,
                self.pixmap.height() as usize,
            );
            return;
        }
        self.ensure_clip_mask();
        let paint = solid_paint(color_value, true);
        self.pixmap
            .fill_rect(bounds, &paint, transform, self.state.clip.as_deref());
    }

    pub(crate) fn blend_coverage(&mut self, index: usize, color: Color, coverage: u8, pair: bool) {
        if self.state.clip.is_none() {
            if let Some(clip) = self.state.clip_summary {
                let Some(bounds) = clip.getBounds() else {
                    return;
                };
                let width = self.pixmap.width() as usize;
                let x = index % width;
                let y = index / width;
                if x < bounds.left() as usize
                    || x >= bounds.right() as usize
                    || y < bounds.top() as usize
                    || y >= bounds.bottom() as usize
                {
                    return;
                }
            }
        }
        let coverage = self.state.clip.as_deref().map_or(coverage, |c| {
            crate::cpu::mask_blitter::mul_div_255_round(
                coverage,
                c.alpha_at(
                    (index % c.width() as usize) as u32,
                    (index / c.width() as usize) as u32,
                ),
            )
        });
        let format = self.pixmap.format;
        let dst = &mut self.pixmap.data_mut()[index * 4..index * 4 + 4];
        if let Some(surface) = self.f16_surface.as_mut() {
            surface.blend(index, color, coverage);
            dst.copy_from_slice(&format.encode(surface.rgba8(index)));
        } else if pair && !self.state.clip_is_aa {
            crate::cpu::mask_blitter::blend_anti_h2_format(dst, color, coverage, format);
        } else {
            crate::cpu::mask_blitter::blend_mask_format(dst, color, coverage, format);
        }
    }
    pub(crate) fn blend_solid_span(
        &mut self,
        start: usize,
        count: usize,
        source: [u8; 4],
        coverage: u8,
        pair: bool,
    ) {
        debug_assert!(self.f16_surface.is_none());
        let format = self.pixmap.format;
        let end = start + count;
        if count == 0 {
            return;
        }
        if let Some(clip) = self
            .state
            .clip_summary
            .as_ref()
            .filter(|c| c.isRect() || c.getBounds().is_none())
        {
            let Some(bounds) = clip.getBounds() else {
                return;
            };
            let width = self.pixmap.width() as usize;
            let y = start / width;
            debug_assert_eq!(y, (end - 1) / width);
            if y < bounds.top() as usize || y >= bounds.bottom() as usize {
                return;
            }
            let row_start = y * width;
            let left = start.max(row_start + bounds.left() as usize);
            let right = end.min(row_start + bounds.right() as usize);
            if left < right {
                crate::src::core::SkBlitRow_D32::blend_span_format(
                    &mut self.pixmap.data_mut()[left * 4..right * 4],
                    source,
                    coverage,
                    pair && !self.state.clip_is_aa,
                    format,
                );
            }
            return;
        }
        let (mut at, mut end) = (start, end);
        let width = self.pixmap.width() as usize;
        if let Some(clip) = self.state.clip.as_deref() {
            let bounds = clip.storage_bounds();
            let y = start / width;
            if y < bounds.top() as usize || y >= bounds.bottom() as usize {
                return;
            }
            at = at.max(y * width + bounds.left() as usize);
            end = end.min(y * width + bounds.right() as usize);
        }
        while at < end {
            let (run_end, alpha) = if let Some(clip) = &self.state.clip {
                let row = clip.row_range(
                    (at / width) as u32,
                    (at % width) as u32,
                    (end - at + at % width) as u32,
                );
                let alpha = row[0];
                let n = crate::compat::analytic_masks::equal_byte_run_end(row, 0, alpha);
                (
                    at + n,
                    crate::cpu::mask_blitter::mul_div_255_round(coverage, alpha),
                )
            } else {
                (end, coverage)
            };
            crate::src::core::SkBlitRow_D32::blend_span_format(
                &mut self.pixmap.data_mut()[at * 4..run_end * 4],
                source,
                alpha,
                pair && !self.state.clip_is_aa,
                format,
            );
            at = run_end;
        }
    }

    fn blend_f16_solid_span(&mut self, start: usize, count: usize, color: Color, coverage: u8) {
        if count == 0 {
            return;
        }
        let (mut at, mut end) = (start, start + count);
        let width = self.pixmap.width() as usize;
        if let Some(clip) = self
            .state
            .clip_summary
            .as_ref()
            .filter(|_| self.state.clip.is_none())
        {
            let Some(bounds) = clip.getBounds() else {
                return;
            };
            let y = start / width;
            debug_assert_eq!(y, (end - 1) / width);
            if y < bounds.top() as usize || y >= bounds.bottom() as usize {
                return;
            }
            at = at.max(y * width + bounds.left() as usize);
            end = end.min(y * width + bounds.right() as usize);
        }
        if let Some(clip) = self.state.clip.as_deref() {
            let bounds = clip.storage_bounds();
            let y = start / width;
            if y < bounds.top() as usize || y >= bounds.bottom() as usize {
                return;
            }
            at = at.max(y * width + bounds.left() as usize);
            end = end.min(y * width + bounds.right() as usize);
        }
        let format = self.pixmap.format;
        while at < end {
            let (run_end, alpha) = if let Some(clip) = self.state.clip.as_deref() {
                let row = clip.row_range(
                    (at / width) as u32,
                    (at % width) as u32,
                    (end - at + at % width) as u32,
                );
                let a = row[0];
                (
                    at + crate::compat::analytic_masks::equal_byte_run_end(row, 0, a),
                    crate::cpu::mask_blitter::mul_div_255_round(coverage, a),
                )
            } else {
                (end, coverage)
            };
            self.f16_surface
                .as_mut()
                .expect("F16 span target")
                .blend_color_span(
                    at,
                    &mut self.pixmap.data_mut()[at * 4..run_end * 4],
                    color,
                    alpha,
                    format,
                );
            at = run_end;
        }
    }

    fn blend_f16_mask_rows(
        &mut self,
        mask: &Mask,
        color: Color,
        left: usize,
        right: usize,
        top: usize,
        bottom: usize,
    ) {
        let width = self.pixmap.width() as usize;
        for y in top..bottom {
            let row = &mask.data()[y * width..(y + 1) * width];
            let mut x = left;
            while x < right {
                let coverage = row[x];
                let end =
                    crate::compat::analytic_masks::equal_byte_run_end(&row[..right], x, coverage);
                if coverage != 0 {
                    self.blend_f16_solid_span(y * width + x, end - x, color, coverage);
                }
                x = end;
            }
        }
    }

    pub(crate) fn blend_rounded_spans(
        &mut self,
        spans: &crate::compat::analytic_masks::RoundedSpans,
        color: Color,
    ) {
        let source = crate::cpu::mask_blitter::premultiply(color);
        for span in spans.direct.iter().chain(spans.accumulated.iter()) {
            if self.f16_surface.is_some() {
                self.blend_f16_solid_span(span.start, span.count, color, span.coverage);
            } else {
                self.blend_solid_span(span.start, span.count, source, span.coverage, span.pair);
            }
        }
    }

    pub(crate) fn blend_analytic_mask(
        &mut self,
        mask: &crate::cpu::analytic_aa::RoundedCoverage,
        color: Color,
    ) {
        // Direct anti spans precede the accumulated RLE scan-line flush.
        // Combining both coverages first changes SrcOver at intersections.
        for &(i, coverage, pair) in &mask.blits {
            self.blend_coverage(i, color, coverage, pair);
        }
        let Some(bounds) = mask.support_bounds else {
            return;
        };
        let width = self.pixmap.width() as usize;
        if self.f16_surface.is_some() {
            self.blend_f16_mask_rows(
                &mask.mask,
                color,
                bounds.left() as usize,
                bounds.right() as usize,
                bounds.top() as usize,
                bounds.bottom() as usize,
            );
            return;
        }
        let source = crate::cpu::mask_blitter::premultiply(color);
        for y in bounds.top() as usize..bounds.bottom() as usize {
            let row = &mask.mask.data()[y * width..(y + 1) * width];
            let mut x = bounds.left() as usize;
            let right = bounds.right() as usize;
            while x < right {
                let coverage = row[x];
                if coverage == 0 {
                    x += row[x..right]
                        .iter()
                        .position(|&a| a != 0)
                        .unwrap_or(right - x);
                    continue;
                }
                let pair = mask.pairs[y * width + x];
                let mut end = x + 1;
                while end < right && row[end] == coverage && mask.pairs[y * width + end] == pair {
                    end += 1;
                }
                self.blend_solid_span(y * width + x, end - x, source, coverage, pair);
                x = end;
            }
        }
    }

    pub(crate) fn drawRRect(&mut self, item: &DrawCommand) {
        if item.antialias && item.corner_radii.HasRadius() {
            if let Some(spans) = crate::cpu::analytic_aa::rounded_rect_spans_in_device(
                item.rect,
                item.corner_radii,
                self.state.transform,
                self.pixmap.width(),
                self.pixmap.height(),
                self.state.clip.as_deref(),
                self.state.clip_summary.as_ref(),
                self.state.tile_origin,
            ) {
                self.blend_rounded_spans(&spans, item.color);
                return;
            }
            if let Some(mask) = crate::cpu::analytic_aa::rounded_rect_mask_with_clip(
                item.rect,
                item.corner_radii,
                self.state.transform,
                self.pixmap.width(),
                self.pixmap.height(),
                false,
                self.state.clip.as_deref(),
                self.state.clip_summary.as_ref(),
            ) {
                self.blend_analytic_mask(&mask, item.color);
                return;
            }
        }
        if let Some(path) = rounded_rect_path(item.rect, item.corner_radii) {
            self.drawPath(
                &path,
                item.color,
                item.antialias && item.corner_radii.HasRadius(),
                FillRule::Winding,
            );
        }
    }

    pub(crate) fn drawDRRect(&mut self, item: &DrawCommand) {
        // GraphicsContext::FillDRRect source uniform border fast path;
        // <=1px AA stroke is converted by SkDraw to a hairline.
        if item.antialias
            && item.paint_shader.is_none()
            && item.blend_mode == PaintBlendMode::kNormal
            && item.blur_radius == 0.0
        {
            if let Some((path, width)) = crate::compat::border::uniform_border_path(item) {
                if crate::cpu::hairline::draw_with_clip(
                    &mut self.pixmap,
                    self.f16_surface.as_mut(),
                    self.state.clip.as_deref(),
                    self.state.clip_is_aa,
                    self.state.clip_summary.as_ref(),
                    self.state.transform,
                    &path,
                    item.color,
                    width,
                ) {
                    return;
                }
                // GraphicsContext::FillDRRect preserves StrokeStyle when the
                // transformed width exceeds the hairline limit. SkStrokeRec
                // constructs the outline from the same rational conics in
                // source coordinates using the CTM resolution scale.
                let t = self.state.transform;
                // SkMatrixPriv::ComputeResScaleForStroking uses both CTM
                // columns and falls back to one for zero/nonfinite scales.
                // Point::length also retains SkPoint's double overflow path.
                let sx = Point::from_xy(t.sx, t.ky).length();
                let sy = Point::from_xy(t.kx, t.sy).length();
                let scale = sx.max(sy);
                let resolution_scale = if sx.is_finite() && sy.is_finite() && scale > 0.0 {
                    scale
                } else {
                    1.0
                };
                if let Some(outline) = crate::cpu::stroke::outline_with_scale(
                    &path,
                    width,
                    SvgStrokeLineCap::kButt,
                    SvgStrokeLineJoin::kMiter,
                    4.0,
                    resolution_scale,
                ) {
                    if let Some(spans) = crate::cpu::analytic_aa::path_spans_with_clip(
                        &outline,
                        false,
                        t,
                        self.pixmap.width(),
                        self.pixmap.height(),
                        self.state.clip.as_deref(),
                        self.state.device_origin,
                        self.state.clip_is_aa,
                        self.state.clip_aa_tiles.as_deref().map(|v| v.as_slice()),
                        self.state.clip_summary.as_ref(),
                    ) {
                        self.blend_rounded_spans(&spans, item.color);
                        return;
                    }
                }
            }
        }
        let (Some(outer), Some(inner)) = (
            rounded_rect_path(item.rect, item.corner_radii),
            rounded_rect_path(item.inner_rect, item.inner_corner_radii),
        ) else {
            return;
        };
        let mut builder = PathBuilder::new();
        builder.push_path(&outer);
        builder.push_path(&inner);
        if let Some(path) = builder.finish() {
            self.drawPath(&path, item.color, item.antialias, FillRule::EvenOdd);
        }
    }

    // SkStroke::lineTo and SkStrokerPriv::RoundCapper: an axis-aligned
    // single line uses a rectangle, or a capsule with four rational conics.

    // cpp: skia_renderer/skia_renderer.cc:1147-1156
    pub(crate) fn stroke_path(&mut self, path: &Path, item: &DrawCommand) {
        assert_eq!(item.blend_mode, PaintBlendMode::kNormal);
        assert!(
            item.paint_shader.is_none(),
            "SVG stroke shader is not installed in pure Rust replay"
        );
        let mut stroke = Stroke {
            width: item.stroke_width as f32,
            miter_limit: item.miter_limit as f32,
            line_cap: if item.round_cap || item.svg_line_cap == SvgStrokeLineCap::kRound {
                LineCap::Round
            } else if item.svg_line_cap == SvgStrokeLineCap::kSquare {
                LineCap::Square
            } else {
                LineCap::Butt
            },
            line_join: match item.svg_line_join {
                SvgStrokeLineJoin::kMiter => LineJoin::Miter,
                SvgStrokeLineJoin::kRound => LineJoin::Round,
                SvgStrokeLineJoin::kBevel => LineJoin::Bevel,
            },
            ..Stroke::default()
        };
        if !item.dash_intervals.is_empty() {
            stroke.dash = StrokeDash::new(
                item.dash_intervals
                    .iter()
                    .map(|value| *value as f32)
                    .collect(),
                item.dash_offset as f32,
            );
        }
        if item.antialias
            && item.stroke_width > 1.0
            && item.blur_radius == 0.0
            && item.dash_intervals.is_empty()
        {
            let t = self.state.transform;
            {
                let commands = if item.path.is_empty() {
                    commands_from_path(path)
                } else {
                    item.path.clone()
                };
                let source_outline = crate::cpu::stroke::outline_with_scale(
                    &commands,
                    stroke.width,
                    if item.round_cap {
                        SvgStrokeLineCap::kRound
                    } else {
                        item.svg_line_cap
                    },
                    item.svg_line_join,
                    stroke.miter_limit,
                    (t.sx * t.sx + t.ky * t.ky)
                        .sqrt()
                        .max((t.kx * t.kx + t.sy * t.sy).sqrt()),
                )
                .or_else(|| path.stroke(&stroke, 1.0).map(|p| commands_from_path(&p)));
                if let Some(outline) = source_outline {
                    if let Some(mask) = crate::cpu::analytic_aa::path_spans_with_clip(
                        &outline,
                        false,
                        t,
                        self.pixmap.width(),
                        self.pixmap.height(),
                        self.state.clip.as_deref(),
                        self.state.device_origin,
                        self.state.clip_is_aa,
                        self.state.clip_aa_tiles.as_deref().map(|v| v.as_slice()),
                        self.state.clip_summary.as_ref(),
                    ) {
                        self.blend_rounded_spans(&mask, item.color);
                        return;
                    }
                }
            }
        }
        self.ensure_clip_mask();
        let paint = solid_paint(item.color, item.antialias);
        self.pixmap.stroke_path(
            path,
            &paint,
            &stroke,
            self.state.transform,
            self.state.clip.as_deref(),
        );
    }

    // cpp: skia_renderer/skia_renderer.cc:628-689
    pub(crate) fn drawImageRect(
        &mut self,
        item: &DrawCommand,
        destination: PaintRect,
        list: &ResourceContext,
    ) {
        assert_eq!(item.blend_mode, PaintBlendMode::kNormal);
        let Some(resource) = list
            .resources
            .as_ref()
            .and_then(|r| r.images.iter().find(|image| image.id == item.resource_id))
        else {
            return;
        };
        let Some(bounds) = rect(destination) else {
            return;
        };
        let mut source = item.source_rect;
        if source.width <= 0.0 || source.height <= 0.0 {
            return;
        }
        // SkCanvas::onDrawImageRect2 rejects against the device clip before
        // the bitmap device creates a subset/mipmap. Admit only our positive
        // scale+translation path here; other matrices retain normal replay.
        let tr = self.state.transform;
        if tr.sx > 0.0 && tr.sy > 0.0 && tr.kx == 0.0 && tr.ky == 0.0 {
            let left = destination.x as f32 * tr.sx + tr.tx;
            let top = destination.y as f32 * tr.sy + tr.ty;
            let right = left + destination.width as f32 * tr.sx;
            let bottom = top + destination.height as f32 * tr.sy;
            let clip = if let Some(summary) = self.state.clip_summary {
                let Some(clip) = summary.getBounds() else {
                    return;
                };
                (
                    clip.left() as f32,
                    clip.top() as f32,
                    clip.right() as f32,
                    clip.bottom() as f32,
                )
            } else {
                (
                    0.0,
                    0.0,
                    self.pixmap.width() as f32,
                    self.pixmap.height() as f32,
                )
            };
            // Conservative one-pixel outset also covers raster rounding/AA.
            if [left, top, right, bottom].iter().all(|v| v.is_finite())
                && (right < clip.0 - 1.0
                    || bottom < clip.1 - 1.0
                    || left > clip.2 + 1.0
                    || top > clip.3 + 1.0)
            {
                return;
            }
        }
        self.authenticate_cached_image(resource);
        let known_opaque = self
            .image_sources
            .get(&resource.id)
            .is_some_and(|image| image.opaque);
        let stats = &mut self.image_cache_stats;
        let cache_enabled = self.image_cache_enabled;
        let image = self.images.entry(resource.id).or_insert_with(|| {
            if cache_enabled {
                stats.source_conversions += 1;
            }
            let mut premultiplied = vec![0; resource.rgba8.len()];
            use crate::src::core::SkConvertPixels::{Rgba8888AlphaType, SkConvertPixels};
            assert!(SkConvertPixels(
                &mut premultiplied,
                resource.rgba8,
                Rgba8888AlphaType::Unpremul,
                Rgba8888AlphaType::Premul
            ));
            Pixmap::from_vec(
                premultiplied,
                IntSize::from_wh(resource.width, resource.height).unwrap(),
            )
            .unwrap()
        });
        let uses_full_source = source.x == 0.0
            && source.y == 0.0
            && source.width == f64::from(image.width())
            && source.height == f64::from(image.height());
        // SkMipmapAccessor selects from the inverse device matrix. A CSS
        // destination size alone over-minifies images on high-DPI surfaces.
        let w = destination.width as f32;
        let h = destination.height as f32;
        let target_width = (Point::from_xy(w * tr.sx, w * tr.ky).length().round() as u32).max(1);
        let target_height = (Point::from_xy(h * tr.kx, h * tr.sy).length().round() as u32).max(1);
        // Chromium SoftwareImageDecodeCache also generates scaled decodes for
        // rounded-out source subsets. Keeping the full-resolution subset here
        // changes the bitmap filter's input even when JPEG decoding is exact.
        let subset_bounds = crate::raster::IntRect::from_ltrb(
            (source.x as f32).floor().max(0.0) as i32,
            (source.y as f32).floor().max(0.0) as i32,
            ((source.x + source.width) as f32)
                .ceil()
                .min(image.width() as f32) as i32,
            ((source.y + source.height) as f32)
                .ceil()
                .min(image.height() as f32) as i32,
        );
        let level = subset_bounds.map_or(0, |bounds| {
            mip_level_for_size(bounds.width(), bounds.height(), target_width, target_height)
        });
        let image = if level > 0 && uses_full_source {
            let image = self
                .mip_images
                .entry((item.resource_id, level, 0, 0, image.width(), image.height()))
                .or_insert_with(|| {
                    if cache_enabled {
                        stats.mip_builds += 1;
                    }
                    build_mip_image(image, level)
                });
            source = PaintRect {
                x: 0.0,
                y: 0.0,
                width: f64::from(image.width()),
                height: f64::from(image.height()),
            };
            &*image
        } else if level > 0 {
            let bounds = subset_bounds.unwrap();
            // The same strict source subset can appear in several draws.
            // Like the full-image path, generate its immutable scaled decode
            // once within this canvas replay, keyed by the complete subset.
            let scaled_subset = self
                .mip_images
                .entry((
                    item.resource_id,
                    level,
                    bounds.x(),
                    bounds.y(),
                    bounds.width(),
                    bounds.height(),
                ))
                .or_insert_with(|| {
                    if cache_enabled {
                        stats.mip_builds += 1;
                    }
                    let subset = image.clone_rect(bounds).unwrap();
                    build_mip_image(&subset, level)
                });
            let sx = f64::from(scaled_subset.width()) / f64::from(bounds.width());
            let sy = f64::from(scaled_subset.height()) / f64::from(bounds.height());
            source = PaintRect {
                x: (source.x - f64::from(bounds.x())) * sx,
                y: (source.y - f64::from(bounds.y())) * sy,
                width: source.width * sx,
                height: source.height * sy,
            };
            &*scaled_subset
        } else {
            image
        };
        if draw_image_bitmap_opaque(
            &mut self.pixmap,
            self.state.clip.as_deref(),
            self.state.transform,
            image,
            destination,
            source,
            self.state.device_origin,
            self.state.tile_origin,
            self.state.clip_runs.as_deref().map(|v| v.as_slice()),
            self.state.clip_summary.as_ref(),
            known_opaque,
        ) {
            return;
        }
        let fallback_clip = if self.state.clip.is_none() {
            self.state.clip_summary.map(|summary| {
                let mut mask = Mask::new(self.pixmap.width(), self.pixmap.height()).unwrap();
                if let Some(bounds) = summary.getBounds() {
                    let width = self.pixmap.width() as usize;
                    for y in bounds.top() as usize..bounds.bottom() as usize {
                        mask.data_mut()[y * width + bounds.left() as usize
                            ..y * width + bounds.right() as usize]
                            .fill(255);
                    }
                }
                mask
            })
        } else {
            None
        };
        let sx = (destination.width / source.width) as f32;
        let sy = (destination.height / source.height) as f32;
        let transform = Transform::from_row(
            sx,
            0.0,
            0.0,
            sy,
            (destination.x - source.x * f64::from(sx)) as f32,
            (destination.y - source.y * f64::from(sy)) as f32,
        );
        let mut paint = Paint::default();
        paint.shader = Pattern::new(
            image.as_ref(),
            SpreadMode::Pad,
            FilterQuality::Bilinear,
            1.0,
            transform,
        );
        paint.anti_alias = item.antialias;
        self.pixmap.fill_rect(
            bounds,
            &paint,
            self.state.transform,
            self.state.clip.as_deref().or(fallback_clip.as_ref()),
        );
    }

    pub(crate) fn glyph_run_intercepts(
        list: &ResourceContext,
        item: &DrawCommand,
        top: f64,
        bottom: f64,
    ) -> Vec<(f64, f64)> {
        if item.glyphs.is_empty()
            || item.transform != TransformMatrix::default()
            || item.glyphs.iter().any(|glyph| glyph.canvas_rotation != 0)
        {
            return Vec::new();
        }
        let Some(font) = list
            .resources
            .as_ref()
            .and_then(|resources| resources.fonts.get(item.font_face_index as usize))
        else {
            return Vec::new();
        };
        let variations = if item.font_variations.is_empty() {
            &font.variations
        } else {
            &item.font_variations
        };
        let descriptor = crate::compat::glyph_paths::Descriptor {
            font: 0,
            face_index: font.face_index,
            size: (item.font_size as f32).to_bits(),
            variations: variations
                .iter()
                .map(|axis| (axis.tag, axis.value.to_bits()))
                .collect(),
            italic: item.synthetic_italic,
            stroke_width: 0,
            resolution: 1.0f32.to_bits(),
            expand_stroke: false,
        };
        let relative_top = top - item.text_blob_origin.y;
        let relative_bottom = bottom - item.text_blob_origin.y;
        crate::compat::glyph_paths::with_paths(font.bytes, descriptor, |strike| {
            let mut face_data = None;
            let mut intervals = Vec::new();
            for glyph in &item.glyphs {
                let cached = strike.prepare_path(
                    glyph.id as u16,
                    || {
                        let (face, hvgl, scale, normalized_axes) =
                            face_data.get_or_insert_with(|| {
                                let mut face = Face::parse(&font.bytes, font.face_index & 0xffff)
                                    .expect("pure Rust rasterizer could not parse the font face");
                                let hvgl = if face.tables().glyf.is_none()
                                    && face.tables().cff.is_none()
                                    && face.tables().cff2.is_none()
                                {
                                    let data = face
                                        .raw_face()
                                        .table(Tag::from_bytes(b"hvgl"))
                                        .unwrap_or_else(|| {
                                            panic!(
                                                "font {} has no supported vector outlines",
                                                font.family
                                            )
                                        });
                                    Some(
                                        HvglTable::parse(data)
                                            .expect("invalid HVGL glyph outline table"),
                                    )
                                } else {
                                    None
                                };
                                for axis in variations {
                                    let _ = face.set_variation(
                                        Tag::from_bytes(&axis.tag.to_be_bytes()),
                                        axis.value,
                                    );
                                }
                                let optical_size = Tag::from_bytes(b"opsz");
                                if face
                                    .variation_axes()
                                    .into_iter()
                                    .any(|axis| axis.tag == optical_size)
                                {
                                    let _ = face.set_variation(optical_size, item.font_size as f32);
                                }
                                let scale = item.font_size as f32 / f32::from(face.units_per_em());
                                let normalized_axes: Vec<f32> = face
                                    .variation_coordinates()
                                    .iter()
                                    .map(|coordinate| f32::from(coordinate.get()) / 16384.0)
                                    .collect();
                                (face, hvgl, scale, normalized_axes)
                            });
                        let id = GlyphId(glyph.id as u16);
                        let mut outline =
                            GlyphPathBuilder::new(0.0, 0.0, *scale, item.synthetic_italic);
                        let has_outline = if let Some(table) = hvgl {
                            table
                                .outline_glyph(id.0, normalized_axes, &mut outline)
                                .expect("invalid HVGL glyph outline")
                        } else {
                            face.outline_glyph(id, &mut outline).is_some()
                        };
                        if !has_outline {
                            return None;
                        }
                        outline
                            .path
                            .finish()
                            .map(|fill| crate::compat::glyph_paths::GlyphPaths {
                                fill,
                                stroke: None,
                            })
                    },
                    crate::compat::glyph_paths::GlyphPaths::heap_bytes,
                );
                let Some(path) = cached else { continue };
                let glyph_top = (relative_top - glyph.offset.y) as f32;
                let glyph_bottom = (relative_bottom - glyph.offset.y) as f32;
                if let Some((left, right)) = path_gap(&path.fill, glyph_top, glyph_bottom) {
                    intervals.push((
                        item.text_blob_origin.x + glyph.offset.x + f64::from(left),
                        item.text_blob_origin.x + glyph.offset.x + f64::from(right),
                    ));
                }
            }
            intervals
        })
    }

    pub(crate) fn drawGlyphRunList(&mut self, item: &DrawCommand, list: &ResourceContext) {
        if item.glyphs.is_empty() {
            return;
        }
        assert_eq!(item.transform, TransformMatrix::default());
        assert!(item.glyphs.iter().all(|glyph| glyph.canvas_rotation == 0));
        let resources = list
            .resources
            .as_ref()
            .expect("glyphs require font resources");
        let font = resources
            .fonts
            .get(item.font_face_index as usize)
            .expect("glyph font face index is outside paint resources");
        let variations = if item.font_variations.is_empty() {
            &font.variations
        } else {
            &item.font_variations
        };
        #[cfg(target_os = "macos")]
        if self.glyph_mode == GlyphRasterMode::Platform
            && !item.synthetic_bold
            && !item.stroke_glyphs
            && (!item.synthetic_italic || {
                // The scaled CTFont subset excludes synthetic skew. Preserve
                // the existing unit-axis italic path; scale uses outlines.
                crate::cpu::scaler_context_mac_ct::SkScalerContextMac::supports_unit_axis(
                    crate::cpu::glyph_position::mask_transform(self.state.transform),
                )
            })
            && crate::cpu::scaler_context_mac_ct::SkScalerContextMac::supports_transform(
                crate::cpu::glyph_position::mask_transform(self.state.transform),
            )
        {
            let mut mask_matrix = crate::cpu::glyph_position::mask_transform(self.state.transform);
            if item.synthetic_italic {
                // ConfigureFontForBlink sets SkFont::skewX=-0.25. Glyph origins
                // stay unsheared; only the prepared linear mask matrix changes.
                mask_matrix.kx = mask_matrix.sx * -0.25 + mask_matrix.kx;
                mask_matrix.sy = mask_matrix.ky * -0.25 + mask_matrix.sy;
            }
            crate::cpu::scaler_context_mac_ct::with_scaler(|scaler| {
                let prepared = scaler.prepare_glyph_run(
                    item.font_face_index,
                    font,
                    item.font_size as f32,
                    variations,
                    mask_matrix,
                );
                let paint = crate::cpu::scaler_context_mac_ct::PreparedGlyphPaint::new(
                    item.color,
                    item.font_smoothing,
                );
                crate::cpu::scaler_context_mac_ct::with_glyph_images(&prepared, &paint, |strike| {
                    for glyph in &item.glyphs {
                        let position = crate::cpu::glyph_position::mask_position_transform(
                            self.state.transform,
                            item.text_blob_origin.x as f32 + glyph.offset.x as f32,
                            item.text_blob_origin.y as f32 + glyph.offset.y as f32,
                        );
                        crate::cpu::scaler_context_mac_ct::SkScalerContextMac::draw_with_clip_kind(
                            &mut self.pixmap,
                            strike,
                            self.state.clip.as_deref(),
                            self.state.clip_summary.as_ref(),
                            &self.state.clip_encoding,
                            &prepared,
                            glyph.id as u16,
                            position,
                            &paint,
                            self.f16_surface.as_mut(),
                        );
                    }
                });
            });
            return;
        }
        #[cfg(target_os = "macos")]
        if self.glyph_mode == GlyphRasterMode::Platform
            && (item.synthetic_bold || item.stroke_glyphs || item.synthetic_italic)
            && crate::cpu::scaler_context_mac_ct::SkScalerContextMac::supports_transform(
                crate::cpu::glyph_position::mask_transform(self.state.transform),
            )
        {
            let rendered_color = crate::cpu::scaler_context_mac_ct::with_scaler(|scaler| {
                let mut prepared = scaler.prepare_glyph_run(
                    item.font_face_index,
                    font,
                    item.font_size as f32,
                    variations,
                    crate::cpu::glyph_position::mask_transform(self.state.transform),
                );
                if !prepared.is_color() {
                    return false;
                }
                if item.synthetic_italic {
                    prepared.apply_color_italic();
                }
                // Mac neverRequestPath sets an empty path for every color-font
                // glyph. Fake bold/stroke then fall back to CoreText image,
                // including ordinary outlines within that same color typeface.
                let paint = crate::cpu::scaler_context_mac_ct::PreparedGlyphPaint::new(
                    item.color,
                    item.font_smoothing,
                );
                crate::cpu::scaler_context_mac_ct::with_glyph_images(&prepared, &paint, |strike| {
                    for glyph in &item.glyphs {
                        let position = crate::cpu::glyph_position::mask_position_transform(
                            self.state.transform,
                            item.text_blob_origin.x as f32 + glyph.offset.x as f32,
                            item.text_blob_origin.y as f32 + glyph.offset.y as f32,
                        );
                        crate::cpu::scaler_context_mac_ct::SkScalerContextMac::draw_with_clip_kind(
                            &mut self.pixmap,
                            strike,
                            self.state.clip.as_deref(),
                            self.state.clip_summary.as_ref(),
                            &self.state.clip_encoding,
                            &prepared,
                            glyph.id as u16,
                            position,
                            &paint,
                            self.f16_surface.as_mut(),
                        );
                    }
                });
                true
            });
            if rendered_color {
                return;
            }
        }
        let stroke_width = (item.font_size as f32 * 0.03).max(0.5);
        let tr = self.state.transform;
        let expand_stroke = item.synthetic_bold
            && !item.stroke_glyphs
            && tr.kx == 0.0
            && tr.ky == 0.0
            && tr.sx.abs() == tr.sy.abs()
            && stroke_width * tr.sx.abs() >= 1.0;
        let resolution = crate::src::core::SkStroke::PathStroker::compute_resolution_scale(&tr);
        let cache_glyph_scans = crate::compat::glyph_rasters::enabled();
        let descriptor = crate::compat::glyph_paths::Descriptor {
            font: 0,
            face_index: font.face_index,
            size: (item.font_size as f32).to_bits(),
            variations: variations
                .iter()
                .map(|a| (a.tag, a.value.to_bits()))
                .collect(),
            italic: item.synthetic_italic,
            stroke_width: if item.synthetic_bold {
                stroke_width.to_bits()
            } else {
                0
            },
            resolution: resolution.to_bits(),
            expand_stroke,
        };
        // SkStrike keeps generated paths across full redraws. Their color, clip,
        // placement and framebuffer remain draw-local and are replayed each time.
        crate::compat::glyph_paths::with_paths(font.bytes, descriptor, |strike| {
            let mut face_data = None;
            for glyph in &item.glyphs {
                let cached = strike.prepare_path(
                    glyph.id as u16,
                    || {
                        let (face, hvgl, scale, normalized_axes) =
                            face_data.get_or_insert_with(|| {
                                let mut face = Face::parse(&font.bytes, font.face_index & 0xffff)
                                    .expect("pure Rust rasterizer could not parse the font face");
                                let hvgl = if face.tables().glyf.is_none()
                                    && face.tables().cff.is_none()
                                    && face.tables().cff2.is_none()
                                {
                                    let data = face
                                        .raw_face()
                                        .table(Tag::from_bytes(b"hvgl"))
                                        .unwrap_or_else(|| {
                                            panic!(
                                                "font {} has no supported vector outlines",
                                                font.family
                                            )
                                        });
                                    Some(
                                        HvglTable::parse(data)
                                            .expect("invalid HVGL font outline table"),
                                    )
                                } else {
                                    None
                                };
                                for axis in variations {
                                    let _ = face.set_variation(
                                        Tag::from_bytes(&axis.tag.to_be_bytes()),
                                        axis.value,
                                    );
                                }
                                let optical_size = Tag::from_bytes(b"opsz");
                                if face
                                    .variation_axes()
                                    .into_iter()
                                    .any(|axis| axis.tag == optical_size)
                                {
                                    let _ = face.set_variation(optical_size, item.font_size as f32);
                                }
                                let scale = item.font_size as f32 / f32::from(face.units_per_em());
                                let antialias = item.font_smoothing != FontSmoothing::kNone;
                                let normalized_axes: Vec<f32> = face
                                    .variation_coordinates()
                                    .iter()
                                    .map(|coordinate| f32::from(coordinate.get()) / 16384.0)
                                    .collect();

                                (face, hvgl, scale, normalized_axes)
                            });
                        let id = GlyphId(glyph.id as u16);
                        let mut outline =
                            GlyphPathBuilder::new(0.0, 0.0, *scale, item.synthetic_italic);
                        let has_outline = if let Some(table) = hvgl {
                            table
                                .outline_glyph(id.0, normalized_axes, &mut outline)
                                .expect("invalid HVGL glyph outline")
                        } else {
                            face.outline_glyph(id, &mut outline).is_some()
                        };
                        if !has_outline {
                            assert!(
                                face.glyph_bounding_box(id).is_none(),
                                "glyph has bounds without supported outline"
                            );
                            return None;
                        }
                        let path = outline.path.finish()?;
                        let stroke = if expand_stroke {
                            path.stroke(
                                &Stroke {
                                    width: stroke_width,
                                    ..Stroke::default()
                                },
                                resolution,
                            )
                        } else {
                            None
                        };
                        Some(crate::compat::glyph_paths::GlyphPaths { fill: path, stroke })
                    },
                    crate::compat::glyph_paths::GlyphPaths::heap_bytes,
                );
                let Some(cached) = cached else {
                    continue;
                };
                let glyph_transform = crate::cpu::glyph_position::position_transform(
                    self.state.transform,
                    item.text_blob_origin.x as f32 + glyph.offset.x as f32,
                    item.text_blob_origin.y as f32 + glyph.offset.y as f32,
                );
                let antialias = item.font_smoothing != FontSmoothing::kNone;
                let paint = solid_paint(item.color, antialias);
                // The strike image's run blitter accepts a BW region directly.
                // Avoid expanding an alpha plane even when a glyph crosses the
                // clip edge; its ordered runs are cropped by that region.
                if cache_glyph_scans && !item.stroke_glyphs && !item.synthetic_bold {
                    if self
                        .state
                        .clip_summary
                        .is_some_and(|c| c.getBounds().is_none())
                    {
                        continue;
                    }
                    if crate::compat::glyph_rasters::draw_with_bw_clip(
                        &mut self.pixmap,
                        &cached,
                        false,
                        &paint,
                        glyph_transform,
                        self.state.clip.as_deref(),
                        self.state
                            .clip_summary
                            .as_ref()
                            .filter(|c| c.isRect())
                            .and_then(|c| c.getBounds()),
                    ) {
                        continue;
                    }
                }
                // SkDraw's BW rectangular clip can be omitted only when it
                // contains the conservative bounds of every path in this glyph.
                let mut omit_clip = false;
                if glyph_transform.kx == 0.0
                    && glyph_transform.ky == 0.0
                    && glyph_transform.sx > 0.0
                    && glyph_transform.sy > 0.0
                {
                    if let Some(bounds) = self
                        .state
                        .clip_summary
                        .as_ref()
                        .filter(|c| c.isRect())
                        .and_then(|c| c.getBounds())
                    {
                        let contains = |path: &Path, pad: f32| {
                            let p = path.bounds();
                            p.left() * glyph_transform.sx + glyph_transform.tx - pad
                                >= bounds.left() as f32
                                && p.top() * glyph_transform.sy + glyph_transform.ty - pad
                                    >= bounds.top() as f32
                                && p.right() * glyph_transform.sx + glyph_transform.tx + pad
                                    <= bounds.right() as f32
                                && p.bottom() * glyph_transform.sy + glyph_transform.ty + pad
                                    <= bounds.bottom() as f32
                        };
                        let pad = if item.stroke_glyphs {
                            item.stroke_width as f32 * resolution * 4.0 + 2.0
                        } else if item.synthetic_bold && !expand_stroke {
                            stroke_width * resolution * 4.0 + 2.0
                        } else {
                            2.0
                        };
                        if contains(&cached.fill, pad)
                            && cached.stroke.as_ref().is_none_or(|p| contains(p, 2.0))
                        {
                            omit_clip = true;
                        }
                    }
                }
                if !omit_clip {
                    self.ensure_clip_mask();
                }
                let clip = if omit_clip {
                    None
                } else {
                    self.state.clip.as_deref()
                };
                if item.stroke_glyphs {
                    self.pixmap.stroke_path(
                        &cached.fill,
                        &paint,
                        &Stroke {
                            width: item.stroke_width as f32,
                            ..Stroke::default()
                        },
                        glyph_transform,
                        clip,
                    );
                } else {
                    if !cache_glyph_scans
                        || !crate::compat::glyph_rasters::draw_with_bw_clip(
                            &mut self.pixmap,
                            &cached,
                            false,
                            &paint,
                            glyph_transform,
                            clip,
                            self.state
                                .clip_summary
                                .as_ref()
                                .filter(|c| c.isRect())
                                .and_then(|c| c.getBounds()),
                        )
                    {
                        self.pixmap.fill_path(
                            &cached.fill,
                            &paint,
                            FillRule::Winding,
                            glyph_transform,
                            clip,
                        );
                    }
                    if item.synthetic_bold {
                        if expand_stroke {
                            if let Some(stroke) = &cached.stroke {
                                if !cache_glyph_scans
                                    || !crate::compat::glyph_rasters::draw_with_bw_clip(
                                        &mut self.pixmap,
                                        &cached,
                                        true,
                                        &paint,
                                        glyph_transform,
                                        clip,
                                        self.state
                                            .clip_summary
                                            .as_ref()
                                            .filter(|c| c.isRect())
                                            .and_then(|c| c.getBounds()),
                                    )
                                {
                                    self.pixmap.fill_path(
                                        stroke,
                                        &paint,
                                        FillRule::Winding,
                                        glyph_transform,
                                        clip,
                                    );
                                }
                            }
                        } else {
                            self.pixmap.stroke_path(
                                &cached.fill,
                                &paint,
                                &Stroke {
                                    width: stroke_width,
                                    ..Stroke::default()
                                },
                                glyph_transform,
                                clip,
                            );
                        }
                    }
                }
            }
        });
    }
}

/// Font-mask provider; the remaining replay uses the local Rust Skia rasterizer.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphRasterMode {
    Platform,
    Outlines,
}
impl SkCanvas {
    /// Locate this device's pixel (0, 0) in the containing raster grid.
    /// Shader/scan-conversion tile anchors use this metadata; it does not move
    /// geometry, alter the CTM or translate the clip. A tile replay sets it
    /// before saving state, including any guard pixels outside the tile.
    /// saveLayer adds its child device offset; restore recovers the parent
    /// origin from the saved MCRec together with the other canvas state.
    pub fn set_raster_origin(&mut self, origin: (i32, i32)) {
        self.state.tile_origin = origin;
    }

    /// Consume this device and read its real raster clip as A8 coverage.
    /// This reads the completed clip, not the rendered surface's alpha. Like
    /// finish_direct, callers transfer any reusable resource caches beforehand.
    pub fn finish_clip_alpha(self) -> Vec<u8> {
        let width = self.pixmap.width() as usize;
        let height = self.pixmap.height() as usize;
        let mut alpha = vec![0; width * height];
        let device = crate::raster::IntRect::from_xywh(0, 0, width as u32, height as u32)
            .expect("valid raster clip device");
        // No explicit summary is the initial full-device clip. An explicit
        // empty summary must remain empty, even if old storage is retained.
        let bounds = match self.state.clip_summary {
            Some(summary) => summary.getBounds().and_then(|b| b.intersect(&device)),
            None => Some(device),
        };
        let Some(bounds) = bounds else {
            return alpha;
        };
        let left = bounds.left() as usize;
        let right = bounds.right() as usize;
        if let Some(mask) = &self.state.clip {
            // The packed mask reader copies actual AA/BW alpha and zero-pads
            // outside its storage; it never promotes to a dense RGBA device.
            for y in bounds.top() as usize..bounds.bottom() as usize {
                mask.copy_row_range(
                    y as u32,
                    left as u32,
                    &mut alpha[y * width + left..y * width + right],
                );
            }
        } else {
            // Complex region/path coverage is materialized by clip receivers.
            // An unmaterialized nonempty clip therefore owns true BW bounds.
            debug_assert!(self
                .state
                .clip_summary
                .is_none_or(|summary| summary.isRect()));
            for y in bounds.top() as usize..bounds.bottom() as usize {
                alpha[y * width + left..y * width + right].fill(255);
            }
        }
        alpha
    }

    pub fn set_scale(&mut self, scale: f64) {
        assert!(scale.is_finite() && scale > 0.0);
        self.state.transform = Transform::from_scale(scale as f32, scale as f32);
    }
    pub fn set_glyph_mode(&mut self, mode: GlyphRasterMode) {
        self.glyph_mode = mode;
    }
    pub fn clear_transparent(&mut self) {
        self.known_opaque = false;
        self.pixmap.fill(crate::raster::Color::TRANSPARENT);
    }
    pub fn finish(self) -> Vec<u8> {
        if self.known_opaque {
            #[cfg(feature = "profiling")]
            if std::env::var_os("SKIA_VERIFY_OPAQUE_READBACK").is_some() {
                assert!(
                    self.pixmap.as_ref().computeIsOpaque(),
                    "invalid opaque readback metadata"
                );
            }
            // SkConvertPixels elides unpremultiplication for opaque alpha;
            // consume this already-owned RGBA allocation without rescanning it.
            self.pixmap.take()
        } else {
            self.pixmap.take_demultiplied()
        }
    }
    /// Return the original CPU target in native premultiplied RGBA form.
    /// No opacity reduction, unpremultiply, readback or allocation takes place.
    /// Match SkCanvas destruction's restoration of outstanding save layers so
    /// the returned ownership always belongs to the original supplied target.
    pub fn finish_direct(mut self) -> crate::raster::PixelStorage {
        while !self.stack.is_empty() {
            self.restore();
        }
        self.pixmap.take_storage()
    }
    pub fn finish_native(self) -> Vec<u8> {
        crate::compat::canvas::source_rgba_readback(self.pixmap.data())
    }
    #[cfg(feature = "profiling")]
    pub fn clip_profile(&self) -> (bool, usize) {
        (
            self.state.clip_is_aa,
            self.state.clip.as_deref().map_or(0, |mask| {
                mask.data()
                    .iter()
                    .filter(|&&alpha| alpha != 0 && alpha != 255)
                    .count()
            }),
        )
    }
}

/// Rust compatibility alias; canonical class spelling follows SkCanvas.h.
pub type Canvas = SkCanvas;

#[cfg(test)]
mod readback_and_path_tests {
    use super::*;
    #[test]
    fn f16_mask_runs_preserve_per_pixel_clipping_and_half_storage() {
        use crate::src::core::SkRasterClip::SkRasterClip;
        let resources = ResourceContext::default();
        let (w, h) = (37, 19);
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for clipping in 0..4 {
                let mut expected = SkCanvas::new(&resources, w, h);
                let mut actual = SkCanvas::new(&resources, w, h);
                let mut clip = Mask::new(w, h).unwrap();
                let mut mask = Mask::new(w, h).unwrap();
                for (i, v) in clip.data_mut().iter_mut().enumerate() {
                    *v = if i % 37 < 3 || i / 37 < 2 {
                        0
                    } else {
                        (i / 7 * 31) as u8
                    };
                }
                for (i, v) in mask.data_mut().iter_mut().enumerate() {
                    *v = (i / 5 * 47) as u8;
                }
                for c in [&mut expected, &mut actual] {
                    c.pixmap.format = format;
                    c.f16_surface = Some(crate::cpu::f16_surface::Surface::new((w * h) as usize));
                    c.state.clip = (clipping == 1).then(|| std::sync::Arc::new(clip.clone()));
                    c.state.clip_summary = match clipping {
                        1 => SkRasterClip::from_mask(w, h, clip.data()),
                        2 => {
                            let mut rect_clip = SkRasterClip::default();
                            rect_clip.setRect(crate::raster::IntRect::from_xywh(3, 2, 27, 14));
                            Some(rect_clip)
                        }
                        3 => SkRasterClip::from_mask(w, h, &vec![0; (w * h) as usize]),
                        _ => None,
                    };
                }
                for alpha in [0.0, 0.37, 1.0] {
                    let color = Color {
                        red: 0.17,
                        green: 0.59,
                        blue: 0.83,
                        alpha,
                    };
                    for (i, &coverage) in mask.data().iter().enumerate() {
                        if coverage != 0 {
                            expected.blend_coverage(i, color, coverage, false);
                        }
                    }
                    actual.blend_f16_mask_rows(&mask, color, 0, w as usize, 0, h as usize);
                    assert_eq!(
                        actual.pixmap.data(),
                        expected.pixmap.data(),
                        "format={format:?} clip={clipping} alpha={alpha}"
                    );
                    assert_eq!(
                        actual.f16_surface.as_ref().unwrap().pixels(),
                        expected.f16_surface.as_ref().unwrap().pixels()
                    );
                }
            }
        }
    }
    #[test]
    fn f16_rounded_spans_preserve_dense_mask_storage() {
        use crate::src::core::SkRasterClip::SkRasterClip;
        let resources = ResourceContext::default();
        let radius = PaintCornerRadius { x: 19.25, y: 13.75 };
        let radii = PaintCornerRadii {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        };
        for x in [-5.25, 15.5, 251.125] {
            for aa_clip in [false, true] {
                let rect = PaintRect {
                    x,
                    y: 19.375,
                    width: 310.25,
                    height: 98.75,
                };
                let mut clip = Mask::new(520, 160).unwrap();
                for y in 25..125 {
                    clip.data_mut()[y * 520 + 31..y * 520 + 491].fill(255);
                    clip.data_mut()[y * 520 + 31] = (y * 37) as u8;
                }
                let clip = aa_clip.then(|| std::sync::Arc::new(clip));
                let summary = clip
                    .as_ref()
                    .map(|m| SkRasterClip::from_mask(520, 160, m.data()).unwrap());
                let dense = crate::cpu::analytic_aa::rounded_rect_mask_with_clip(
                    rect,
                    radii,
                    Transform::identity(),
                    520,
                    160,
                    false,
                    clip.as_deref(),
                    summary.as_ref(),
                )
                .unwrap();
                let spans = crate::cpu::analytic_aa::rounded_rect_spans(
                    rect,
                    radii,
                    Transform::identity(),
                    520,
                    160,
                    clip.as_deref(),
                    summary.as_ref(),
                )
                .unwrap();
                for format in [
                    crate::PixelFormat::Rgba8888,
                    crate::PixelFormat::Bgra8888,
                    crate::PixelFormat::Bgrx8888,
                ] {
                    let mut expected = SkCanvas::new(&resources, 520, 160);
                    let mut actual = SkCanvas::new(&resources, 520, 160);
                    for c in [&mut expected, &mut actual] {
                        c.pixmap.format = format;
                        c.f16_surface = Some(crate::cpu::f16_surface::Surface::new(520 * 160));
                        c.state.clip = clip.clone();
                        c.state.clip_summary = summary;
                        c.state.clip_is_aa = aa_clip;
                    }
                    for alpha in [0.37, 0.71, 1.0] {
                        let color = Color {
                            red: 0.17,
                            green: 0.59,
                            blue: 0.83,
                            alpha,
                        };
                        expected.blend_analytic_mask(&dense, color);
                        actual.blend_rounded_spans(&spans, color);
                        assert_eq!(
                            actual.pixmap.data(),
                            expected.pixmap.data(),
                            "x={x} aa_clip={aa_clip} format={format:?} alpha={alpha}"
                        );
                        assert_eq!(
                            actual.f16_surface.as_ref().unwrap().pixels(),
                            expected.f16_surface.as_ref().unwrap().pixels(),
                            "F16 storage x={x} aa_clip={aa_clip} format={format:?} alpha={alpha}"
                        );
                    }
                }
            }
        }
    }

    fn pixels(commands: &[DrawCommand], transparent: bool) -> Vec<u8> {
        let list = ResourceContext::default();
        let mut c = SkCanvas::new(&list, 80, 64);
        if transparent {
            c.clear_transparent();
        }
        for command in commands {
            c.replay_item(command, &list);
        }
        if c.known_opaque {
            assert!(c.pixmap.as_ref().computeIsOpaque());
        }
        let expected = c.pixmap.clone().take_demultiplied();
        let actual = c.finish();
        assert_eq!(actual, expected);
        actual
    }
    #[test]
    fn restore_rows_preserve_opacity_clip_origins_and_storage_formats() {
        use crate::raster::PixelFormat;
        let resources = ResourceContext::default();
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for opacity in [0.0, 0.001, 0.5, 0.73, 1.0] {
                for origin in [(0, 0), (-2, 3), (4, -3)] {
                    for clipped in [false, true] {
                        let mut c = SkCanvas::new(&resources, 37, 19);
                        c.pixmap.format = format;
                        for (i, p) in c.pixmap.data_mut().chunks_exact_mut(4).enumerate() {
                            p.copy_from_slice(&format.encode([
                                (i * 13) as u8,
                                (i * 31) as u8,
                                (i * 73) as u8,
                                255,
                            ]));
                        }
                        if clipped {
                            let mut clip = Mask::new(37, 19).unwrap();
                            for (i, a) in clip.data_mut().iter_mut().enumerate() {
                                *a = match i % 37 {
                                    0..=2 => 0,
                                    3..=7 => 91,
                                    8..=30 => 255,
                                    _ => 183,
                                };
                            }
                            c.state.clip = Some(std::sync::Arc::new(clip));
                            c.refresh_clip_summary();
                        }
                        c.internalSaveLayer(
                            &DrawCommand {
                                opacity,
                                ..Default::default()
                            },
                            false,
                        );
                        c.stack.last_mut().unwrap().layer_origin = origin;
                        for (i, p) in c.pixmap.data_mut().chunks_exact_mut(4).enumerate() {
                            let a = (i * 43) as u8;
                            p.copy_from_slice(&[a / 2, a / 3, a / 4, a]);
                        }
                        let frame = c.stack.last().unwrap();
                        let parent = &frame.layer.as_ref().unwrap().0;
                        let mut expected = parent.data().to_vec();
                        let width = c.pixmap.width() as usize;
                        for (i, s) in c.pixmap.data().chunks_exact(4).enumerate() {
                            let x = (i % width) as i32 + origin.0;
                            let y = (i / width) as i32 + origin.1;
                            if x < 0 || x >= 37 || y < 0 || y >= 19 || s == [0; 4] {
                                continue;
                            }
                            let at = y as usize * 37 + x as usize;
                            let coverage = frame.state.clip.as_ref().map_or(255, |m| m.data()[at]);
                            if coverage == 0 {
                                continue;
                            }
                            let src: [u8; 4] = std::array::from_fn(|c| {
                                (f32::from(s[c]) * opacity).round_ties_even() as u8
                            });
                            let dst = &mut expected[at * 4..at * 4 + 4];
                            for channel in 0..4 {
                                let k = format.channel(channel);
                                dst[k] = if coverage == 255 {
                                    (u32::from(src[channel])
                                        + (u32::from(dst[k]) * u32::from(255 - src[3]) + 127) / 255)
                                        .min(255) as u8
                                } else {
                                    let scale = u32::from(coverage) + 1;
                                    let inverse = if src[3] == 255 {
                                        256 - scale
                                    } else {
                                        let p = 65535 - u32::from(src[3]) * scale;
                                        (p + (p >> 8)) >> 8
                                    };
                                    ((u32::from(src[channel]) * scale
                                        + u32::from(dst[k]) * inverse)
                                        >> 8)
                                        .min(255) as u8
                                };
                            }
                            if format == PixelFormat::Bgrx8888 {
                                dst[3] = 0;
                            }
                        }
                        c.restore();
                        assert_eq!(
                            c.pixmap.data(),
                            expected,
                            "{format:?} opacity={opacity} origin={origin:?} clipped={clipped}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn alpha_only_rrect_rect_events_match_original_dense_mask() {
        let radius = PaintCornerRadius { x: 9.25, y: 13.75 };
        let radii = PaintCornerRadii {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        };
        for scale in [0.8, 1.5, 2.0] {
            for x in [-3.125, 39.75, 139.125] {
                for height in [38.125, 99.0, 260.5] {
                    for force_rle in [false, true] {
                        let bounds = PaintRect {
                            x,
                            y: 81.125,
                            width: 400.25,
                            height,
                        };
                        let transform = Transform::from_scale(scale, scale);
                        let old = crate::cpu::analytic_aa::rounded_rect_mask(
                            bounds, radii, transform, 1040, 720, force_rle, None,
                        )
                        .unwrap();
                        let new = crate::cpu::analytic_aa::rounded_rect_alpha_mask(
                            bounds, radii, transform, 1040, 720, force_rle,
                        )
                        .unwrap();
                        assert_eq!(
                            new.data(),
                            old.mask.data(),
                            "scale={scale} x={x} height={height} force_rle={force_rle}"
                        );
                        if force_rle {
                            let clip = crate::cpu::analytic_aa::rounded_rect_clip_mask(
                                bounds, radii, transform, 1040, 720, None, None,
                            )
                            .unwrap();
                            // SkAAClip::setPath's Builder uses the snug
                            // path.roundOut intersection as its actual scan
                            // bounds. Full shadow masks keep the device clip;
                            // they may differ at a fixed-edge overshoot.
                            let left = (bounds.x as f32 * scale).floor() as i32;
                            let top = (bounds.y as f32 * scale).floor() as i32;
                            let right = (bounds.x as f32 * scale + bounds.width as f32 * scale)
                                .ceil() as i32;
                            let bottom = (bounds.y as f32 * scale + bounds.height as f32 * scale)
                                .ceil() as i32;
                            let snug = crate::raster::IntRect::from_ltrb(left, top, right, bottom)
                                .and_then(|b| {
                                    b.intersect(
                                        &crate::raster::IntRect::from_xywh(0, 0, 1040, 720)
                                            .unwrap(),
                                    )
                                });
                            let mut source_clip =
                                crate::src::core::SkRasterClip::SkRasterClip::default();
                            source_clip.setRect(snug);
                            let dense_clip =
                                crate::compat::analytic_masks::rounded_rect_mask_with_clip(
                                    bounds,
                                    radii,
                                    transform,
                                    1040,
                                    720,
                                    true,
                                    None,
                                    Some(&source_clip),
                                )
                                .unwrap();
                            assert_eq!(clip.mask.data(), dense_clip.mask.data());
                            if scale == 1.5 && x == 39.75 && height == 38.125 {
                                // Independent original SkCanvas transparent
                                // clipRRect whole-mask oracle: source AA clip
                                // alpha248/0, full-mask alpha247/8. Preserve
                                // this original failing geometry and exact
                                // source values; no pixel tolerance.
                                assert_eq!(old.mask.data()[142 * 1040 + 659], 247);
                                assert_eq!(old.mask.data()[142 * 1040 + 660], 8);
                                assert_eq!(clip.mask.data()[142 * 1040 + 659], 248);
                                assert_eq!(clip.mask.data()[142 * 1040 + 660], 0);
                            }
                            let mut old_runs = dense_clip.run_starts.clone();
                            let mut new_runs = clip.run_starts.clone();
                            old_runs.sort_unstable();
                            old_runs.dedup();
                            new_runs.sort_unstable();
                            new_runs.dedup();
                            assert_eq!(
                                new_runs, old_runs,
                                "run metadata scale={scale} x={x} height={height}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn save_shares_clip_storage_and_clip_mutation_preserves_saved_coverage() {
        let resources = ResourceContext::default();
        let mut canvas = SkCanvas::new(&resources, 520, 360);
        let mut mask = Mask::new(520, 360).unwrap();
        for y in 12..344 {
            mask.data_mut()[y * 520 + 16..y * 520 + 504].fill(255);
        }
        canvas.state.clip = Some(std::sync::Arc::new(mask));
        canvas.refresh_clip_summary();
        let saved = canvas.state.clip.clone().unwrap();
        canvas.save();
        assert!(std::sync::Arc::ptr_eq(
            &saved,
            canvas.state.clip.as_ref().unwrap()
        ));
        canvas.replay_item(
            &DrawCommand {
                r#type: CommandKind::kClipRect,
                rect: PaintRect {
                    x: 80.0,
                    y: 60.0,
                    width: 200.0,
                    height: 180.0,
                },
                antialias: false,
                ..Default::default()
            },
            &resources,
        );
        canvas.ensure_clip_mask();
        assert!(!std::sync::Arc::ptr_eq(
            &saved,
            canvas.state.clip.as_ref().unwrap()
        ));
        assert_eq!(saved.data()[20 * 520 + 20], 255);
        assert_eq!(canvas.state.clip.as_ref().unwrap().data()[20 * 520 + 20], 0);
        canvas.restore();
        assert!(std::sync::Arc::ptr_eq(
            &saved,
            canvas.state.clip.as_ref().unwrap()
        ));
    }

    #[test]
    fn path_span_target_preserves_dense_blits_with_formats_transforms_and_aa_clip() {
        use crate::src::core::SkRasterClip::SkRasterClip;
        let resources = ResourceContext::default();
        let mut builder = PathBuilder::new();
        builder.move_to(41.125, 27.25);
        builder.line_to(127.75, 31.5);
        builder.line_to(67.125, 79.25);
        builder.line_to(148.75, 144.125);
        builder.line_to(38.25, 137.5);
        builder.close();
        let polygon = commands_from_path(&builder.finish().unwrap());
        let mut builder = PathBuilder::new();
        builder.move_to(68.25, 44.5);
        builder.cubic_to(34.125, 89.25, 156.5, 145.75, 199.25, 101.125);
        builder.quad_to(158.125, 15.25, 68.25, 44.5);
        builder.close();
        let curve = commands_from_path(&builder.finish().unwrap());
        let rectangle = rect_commands(PaintRect {
            x: 78.25,
            y: 39.5,
            width: 71.125,
            height: 83.25,
        })
        .unwrap();
        for commands in [&polygon, &curve, &rectangle] {
            for transform in [
                Transform::identity(),
                Transform::from_scale(2.0, 2.0),
                Transform::from_row(1.5, 0.125, 0.25, 1.5, -17.125, 4.75),
            ] {
                for aa_clip in [false, true] {
                    let mut clip = Mask::new(520, 360).unwrap();
                    for y in 23..330 {
                        clip.data_mut()[y * 520 + 32..y * 520 + 480].fill(255);
                        clip.data_mut()[y * 520 + 97] = (y % 255) as u8;
                    }
                    let clip = aa_clip.then(|| std::sync::Arc::new(clip));
                    let summary = clip
                        .as_ref()
                        .map(|m| SkRasterClip::from_mask(520, 360, m.data()).unwrap());
                    let dense = crate::cpu::analytic_aa::path_mask_with_clip(
                        commands,
                        false,
                        transform,
                        520,
                        360,
                        clip.as_deref(),
                        (0, 0),
                        aa_clip,
                        None,
                        summary.as_ref(),
                    )
                    .unwrap();
                    let spans = crate::cpu::analytic_aa::path_spans_with_clip(
                        commands,
                        false,
                        transform,
                        520,
                        360,
                        clip.as_deref(),
                        (0, 0),
                        aa_clip,
                        None,
                        summary.as_ref(),
                    )
                    .unwrap();
                    for format in [
                        crate::PixelFormat::Rgba8888,
                        crate::PixelFormat::Bgra8888,
                        crate::PixelFormat::Bgrx8888,
                    ] {
                        for alpha in [0.39, 1.0] {
                            let mut expected = SkCanvas::new(&resources, 520, 360);
                            let mut actual = SkCanvas::new(&resources, 520, 360);
                            for canvas in [&mut expected, &mut actual] {
                                canvas.pixmap.format = format;
                                canvas.state.clip = clip.clone();
                                canvas.state.clip_summary = summary;
                                canvas.state.clip_is_aa = aa_clip;
                            }
                            let color = Color {
                                red: 0.19,
                                green: 0.57,
                                blue: 0.83,
                                alpha,
                            };
                            expected.blend_analytic_mask(&dense, color);
                            actual.blend_rounded_spans(&spans, color);
                            assert_eq!(actual.pixmap.data(), expected.pixmap.data(),
                                "format={format:?} alpha={alpha} transform={transform:?} aa_clip={aa_clip}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn bounded_generic_clip_matches_dense_intersection_under_scale_and_skew() {
        let resources = ResourceContext::default();
        let r = PaintCornerRadius { x: 9.25, y: 13.5 };
        let radii = PaintCornerRadii {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        };
        for sx in [1.0, 1.5, 2.0] {
            for skew in [0.0, 0.25] {
                let mut canvas = SkCanvas::new(&resources, 520, 360);
                canvas.state.transform = Transform::from_row(sx, 0.0, skew, sx, 5.125, -7.75);
                let mut old: Option<Mask> = None;
                for rect in [
                    PaintRect {
                        x: 120.125,
                        y: 99.25,
                        width: 87.5,
                        height: 37.5,
                    },
                    PaintRect {
                        x: 110.75,
                        y: 103.125,
                        width: 103.0,
                        height: 26.75,
                    },
                    PaintRect {
                        x: -15.0,
                        y: -8.0,
                        width: 241.75,
                        height: 210.0,
                    },
                    PaintRect {
                        x: 500.0,
                        y: 400.0,
                        width: 12.0,
                        height: 14.0,
                    },
                ] {
                    let path = rounded_rect_path(rect, radii).unwrap();
                    let mut expected = old.clone().unwrap_or_else(|| Mask::new(520, 360).unwrap());
                    if old.is_some() {
                        expected.intersect_path(
                            &path,
                            FillRule::Winding,
                            true,
                            canvas.state.transform,
                        );
                    } else {
                        expected.fill_path(&path, FillRule::Winding, true, canvas.state.transform);
                    }
                    canvas.clipPath(&path, true);
                    assert_eq!(
                        canvas.state.clip.as_deref().unwrap().data(),
                        expected.data(),
                        "scale={sx} skew={skew} rect={rect:?}"
                    );
                    assert_eq!(
                        canvas.state.clip_summary,
                        crate::src::core::SkRasterClip::SkRasterClip::from_mask(
                            520,
                            360,
                            expected.data()
                        )
                    );
                    old = Some(expected);
                }
            }
        }
    }

    #[test]
    fn save_layer_owns_only_intersected_device_pixels_and_restores_parent() {
        let resources = ResourceContext::default();
        let mut canvas = SkCanvas::new(&resources, 2560, 1542);
        canvas.internalSaveLayer(
            &DrawCommand {
                rect: PaintRect {
                    x: 930.25,
                    y: 701.5,
                    width: 17.0,
                    height: 16.0,
                },
                opacity: 0.63,
                ..Default::default()
            },
            true,
        );
        assert_eq!((canvas.pixmap.width(), canvas.pixmap.height()), (18, 17));
        assert_eq!(canvas.f16_surface.as_ref().unwrap().pixels().len(), 18 * 17);
        assert_eq!(
            (canvas.state.transform.tx, canvas.state.transform.ty),
            (-930.0, -701.0)
        );
        canvas.internalSaveLayer(
            &DrawCommand {
                rect: PaintRect {
                    x: 934.0,
                    y: 704.0,
                    width: 4.0,
                    height: 5.0,
                },
                ..Default::default()
            },
            false,
        );
        assert_eq!((canvas.pixmap.width(), canvas.pixmap.height()), (4, 5));
        canvas.restore();
        assert_eq!((canvas.pixmap.width(), canvas.pixmap.height()), (18, 17));
        assert!(canvas.f16_surface.is_some());
        canvas.restore();
        assert_eq!(
            (canvas.pixmap.width(), canvas.pixmap.height()),
            (2560, 1542)
        );
        assert!(canvas.state.transform.is_identity());
    }

    #[test]
    fn opaque_readback_is_conservative_and_matches_scanned_conversion() {
        for alpha in [0.0, 0.125, 0.5, 1.0] {
            let command = DrawCommand {
                r#type: CommandKind::kDrawRoundedRect,
                rect: PaintRect {
                    x: 2.25,
                    y: 3.5,
                    width: 57.0,
                    height: 31.0,
                },
                color: Color {
                    red: 0.2,
                    green: 0.5,
                    blue: 0.7,
                    alpha,
                },
                antialias: true,
                corner_radii: PaintCornerRadii {
                    top_left: PaintCornerRadius { x: 7.5, y: 6.0 },
                    top_right: PaintCornerRadius { x: 7.5, y: 6.0 },
                    bottom_left: PaintCornerRadius { x: 7.5, y: 6.0 },
                    bottom_right: PaintCornerRadius { x: 7.5, y: 6.0 },
                },
                ..DrawCommand::default()
            };
            assert!(pixels(&[command.clone()], false)
                .chunks_exact(4)
                .all(|p| p[3] == 255));
            assert!(pixels(&[command], true).chunks_exact(4).any(|p| p[3] < 255));
        }
        let list = ResourceContext::default();
        let mut c = SkCanvas::new(&list, 8, 8);
        c.replay_item(
            &DrawCommand {
                r#type: CommandKind::kSaveLayerAlpha,
                opacity: 0.5,
                ..DrawCommand::default()
            },
            &list,
        );
        assert!(!c.known_opaque);
        c.replay_item(
            &DrawCommand {
                r#type: CommandKind::kRestore,
                ..DrawCommand::default()
            },
            &list,
        );
        assert!(c.known_opaque);
        assert_eq!(c.finish(), vec![255; 8 * 8 * 4]);
    }
    #[test]
    fn warmed_outline_paths_preserve_placement_color_scale_clip_and_stroke() {
        let bytes = include_bytes!(
            "../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
        );
        let list = ResourceContext {
            resources: Some(ResourceCatalog {
                fonts: vec![FontFace {
                    bytes,
                    face_index: 0,
                    family: "Roboto",
                    native_family: "",
                    weight: 400.0,
                    italic: false,
                    variations: vec![],
                }],
                images: vec![],
            }),
        };
        let render = |phase: f64, scale: f64, bold: bool, stroke: bool, color: Color| {
            let mut c = SkCanvas::new(&list, 180, 100);
            c.set_glyph_mode(GlyphRasterMode::Outlines);
            c.set_scale(scale);
            c.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kClipRect,
                    rect: PaintRect {
                        x: 4.5,
                        y: 3.0,
                        width: 65.0,
                        height: 35.0,
                    },
                    ..DrawCommand::default()
                },
                &list,
            );
            c.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kDrawGlyphRun,
                    font_size: 24.0,
                    synthetic_bold: bold,
                    stroke_glyphs: stroke,
                    stroke_width: 0.7,
                    color,
                    text_blob_origin: Offset {
                        x: 6.0 + phase,
                        y: 30.0 + phase,
                    },
                    glyphs: vec![
                        PaintGlyph {
                            id: 36,
                            ..PaintGlyph::default()
                        },
                        PaintGlyph {
                            id: 74,
                            offset: Offset { x: 24.0, y: 0.0 },
                            ..PaintGlyph::default()
                        },
                    ],
                    ..DrawCommand::default()
                },
                &list,
            );
            c.finish()
        };
        for scale in [0.75, 1.0, 2.0] {
            for phase in [0.0, 0.125, 0.5] {
                for (bold, stroke) in [(false, false), (true, false), (false, true)] {
                    for alpha in [0.5, 1.0] {
                        let color = Color {
                            red: 0.2,
                            green: 0.4,
                            blue: 0.6,
                            alpha,
                        };
                        crate::compat::glyph_paths::purge();
                        let cold = render(phase, scale, bold, stroke, color);
                        assert_eq!(cold, render(phase, scale, bold, stroke, color));
                        let shifted = render(phase + 0.25, scale, bold, stroke, color);
                        crate::compat::glyph_paths::purge();
                        assert_eq!(shifted, render(phase + 0.25, scale, bold, stroke, color));
                    }
                }
            }
        }
    }

    #[test]
    fn blurred_glyph_run_uses_bounded_normal_blur() {
        let bytes = include_bytes!(
            "../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
        );
        let list = ResourceContext {
            resources: Some(ResourceCatalog {
                fonts: vec![FontFace {
                    bytes,
                    face_index: 0,
                    family: "Roboto",
                    native_family: "",
                    weight: 400.0,
                    italic: false,
                    variations: vec![],
                }],
                images: vec![],
            }),
        };
        let render = |blur_radius| {
            let mut canvas = SkCanvas::new(&list, 96, 64);
            canvas.set_glyph_mode(GlyphRasterMode::Outlines);
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kDrawGlyphRun,
                    font_size: 28.0,
                    color: Color {
                        red: 1.0,
                        alpha: 1.0,
                        ..Color::default()
                    },
                    text_blob_origin: Offset { x: 28.0, y: 42.0 },
                    glyphs: vec![PaintGlyph {
                        id: 36,
                        ..PaintGlyph::default()
                    }],
                    blur_radius,
                    ..DrawCommand::default()
                },
                &list,
            );
            canvas.finish()
        };
        let sharp = render(0.0);
        let blurred = render(4.0);
        assert_ne!(blurred, sharp);
        let changed_pixels = |pixels: &[u8]| {
            let background = &pixels[..4];
            pixels
                .chunks_exact(4)
                .filter(|pixel| *pixel != background)
                .count()
        };
        assert!(changed_pixels(&blurred) > changed_pixels(&sharp));
        assert_eq!(blurred, render(4.0), "blurred replay must be deterministic");
    }
}

#[cfg(test)]
mod direct_target_tests {
    use super::*;
    use crate::raster::PixelStorage;
    use core::ptr::NonNull;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    struct Mapping {
        bytes: Box<[u8]>,
        drops: Arc<AtomicUsize>,
    }
    impl Drop for Mapping {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn external(len: usize, drops: &Arc<AtomicUsize>) -> PixelStorage {
        let mut owner = Box::new(Mapping {
            bytes: vec![7; len].into_boxed_slice(),
            drops: drops.clone(),
        });
        let ptr = NonNull::new(owner.bytes.as_mut_ptr()).unwrap();
        // SAFETY: Box backing bytes stay fixed, ownership transfers exclusively,
        // Mapping is Send and drops its allocation once with storage ownership.
        unsafe { PixelStorage::from_external(ptr, len, owner) }.unwrap()
    }
    fn draw(c: &mut SkCanvas, list: &ResourceContext) {
        c.replay_item(
            &DrawCommand {
                r#type: CommandKind::kDrawRect,
                rect: PaintRect {
                    x: 1.,
                    y: 1.,
                    width: 4.,
                    height: 3.,
                },
                color: Color {
                    red: 0.8,
                    green: 0.4,
                    blue: 0.2,
                    alpha: 0.5,
                },
                ..DrawCommand::default()
            },
            list,
        );
    }
    #[test]
    fn direct_target_identity_clone_and_owner_return() {
        let list = ResourceContext::default();
        let drops = Arc::new(AtomicUsize::new(0));
        let target = external(8 * 6 * 4, &drops);
        let ptr = target.as_ptr();
        let mut c = SkCanvas::make_raster_direct(&list, 8, 6, 32, target).unwrap();
        assert_eq!(c.pixmap.data().as_ptr(), ptr);
        assert!(c.pixmap.data().iter().all(|&b| b == 255));
        draw(&mut c, &list);
        let cloned = c.pixmap.clone();
        assert_ne!(cloned.data().as_ptr(), ptr);
        assert!(cloned.data.external_owner().is_none());
        let mut target = c.finish_direct();
        assert_eq!(target.as_ptr(), ptr);
        assert_eq!(target.as_slice(), cloned.data());
        target.as_mut_slice()[0] = 17;
        assert_eq!(cloned.data()[0], 255);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(target.external_owner().unwrap().is::<Mapping>());
        let owner = target
            .into_external_owner()
            .unwrap()
            .downcast::<Mapping>()
            .unwrap();
        assert_eq!(owner.bytes.as_ptr(), ptr);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(owner);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn direct_target_layers_restore_base_and_drop_once() {
        let list = ResourceContext::default();
        for finish in [false, true] {
            for f16 in [false, true] {
                let drops = Arc::new(AtomicUsize::new(0));
                let target = external(8 * 6 * 4, &drops);
                let ptr = target.as_ptr();
                let mut c = SkCanvas::make_raster_direct(&list, 8, 6, 32, target).unwrap();
                c.clear_transparent();
                c.save();
                c.internalSaveLayer(
                    &DrawCommand {
                        opacity: 0.5,
                        ..DrawCommand::default()
                    },
                    f16,
                );
                assert_ne!(c.pixmap.data().as_ptr(), ptr);
                assert!(c.pixmap.data.external_owner().is_none());
                assert!(
                    c.pixmap.data().iter().all(|&byte| byte == 0),
                    "fresh and recycled layers must begin fully transparent"
                );
                draw(&mut c, &list);
                c.internalSaveLayer(&DrawCommand::default(), false);
                draw(&mut c, &list);
                assert_eq!(drops.load(Ordering::SeqCst), 0);
                if finish {
                    let target = c.finish_direct();
                    assert_eq!(target.as_ptr(), ptr);
                    assert!(target.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 255));
                    assert_eq!(drops.load(Ordering::SeqCst), 0);
                    drop(target);
                } else {
                    drop(c);
                }
                assert_eq!(drops.load(Ordering::SeqCst), 1);
            }
        }
    }
    #[test]
    fn direct_target_premul_matches_legacy_and_image_sources_are_separate() {
        let source = [200, 80, 40, 128, 30, 70, 190, 255];
        let list = ResourceContext {
            resources: Some(ResourceCatalog {
                images: vec![Image::borrowed(1, 2, 1, &source)],
                ..ResourceCatalog::default()
            }),
        };
        let drops = Arc::new(AtomicUsize::new(0));
        let mut direct =
            SkCanvas::make_raster_direct(&list, 8, 6, 32, external(8 * 6 * 4, &drops)).unwrap();
        let mut legacy = SkCanvas::new(&list, 8, 6);
        for c in [&mut direct, &mut legacy] {
            c.clear_transparent();
            draw(c, &list);
            c.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kDrawImageRect,
                    resource_id: 1,
                    source_rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 2.0,
                        height: 1.0,
                    },
                    rect: PaintRect {
                        x: 1.,
                        y: 4.,
                        width: 2.,
                        height: 1.,
                    },
                    ..DrawCommand::default()
                },
                &list,
            );
        }
        assert!(direct.images[&1].data.external_owner().is_none());
        assert_ne!(direct.images[&1].data().as_ptr(), source.as_ptr());
        assert_ne!(
            direct.images[&1].data().as_ptr(),
            direct.pixmap.data().as_ptr()
        );
        let expected_premul = legacy.pixmap.data().to_vec();
        let expected_straight = legacy.finish();
        let target = direct.finish_direct();
        assert_eq!(target.as_slice(), expected_premul);
        assert_ne!(target.as_slice(), expected_straight);
        for pixel in target.chunks_exact(4) {
            assert!(pixel[..3].iter().all(|&channel| channel <= pixel[3]));
        }
        let size = IntSize::from_wh(8, 6).unwrap();
        assert_eq!(
            Pixmap::from_vec(target.as_slice().to_vec(), size)
                .unwrap()
                .take_demultiplied(),
            expected_straight
        );
        assert_eq!(source, [200, 80, 40, 128, 30, 70, 190, 255]);
        drop(target);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn direct_target_rejects_bad_dimensions_stride_length_and_releases() {
        let list = ResourceContext::default();
        let drops = Arc::new(AtomicUsize::new(0));
        for (width, height, stride, len) in [
            (0, 6, 0, 192),
            (8, 0, 32, 192),
            (8, 6, 36, 192),
            (8, 6, 32, 191),
            (8, 6, 32, 193),
            (u32::MAX, 6, 32, 192),
        ] {
            assert!(SkCanvas::make_raster_direct(
                &list,
                width,
                height,
                stride,
                external(len, &drops)
            )
            .is_none());
        }
        assert_eq!(drops.load(Ordering::SeqCst), 6);
    }
    #[test]
    fn external_legacy_readback_copies_before_release() {
        let list = ResourceContext::default();
        let drops = Arc::new(AtomicUsize::new(0));
        let target = external(8 * 6 * 4, &drops);
        let ptr = target.as_ptr();
        let mut c = SkCanvas::make_raster_direct(&list, 8, 6, 32, target).unwrap();
        c.clear_transparent();
        draw(&mut c, &list);
        let expected = c.pixmap.clone().take_demultiplied();
        let result = c.finish();
        assert_eq!(result, expected);
        assert_ne!(result.as_ptr(), ptr);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

#[cfg(test)]
mod color_layout_tests {
    use super::*;
    use crate::{PixelFormat, PixelStorage};
    fn render(
        list: &ResourceContext,
        commands: &[DrawCommand],
        format: PixelFormat,
        transparent: bool,
        mode: GlyphRasterMode,
    ) -> Vec<u8> {
        let mut c = SkCanvas::make_raster_direct_with_format(
            list,
            96,
            72,
            384,
            PixelStorage::owned(vec![13; 96 * 72 * 4]),
            format,
        )
        .unwrap();
        c.set_glyph_mode(mode);
        if transparent {
            c.clear_transparent();
        }
        for command in commands {
            c.replay_item(command, list);
        }
        c.finish_direct().into_vec()
    }
    fn compare(list: &ResourceContext, commands: &[DrawCommand], mode: GlyphRasterMode) {
        for transparent in [false, true] {
            let rgba = render(list, commands, PixelFormat::Rgba8888, transparent, mode);
            let bgra = render(list, commands, PixelFormat::Bgra8888, transparent, mode);
            for (i, (r, b)) in rgba.chunks_exact(4).zip(bgra.chunks_exact(4)).enumerate() {
                assert_eq!(b, [r[2], r[1], r[0], r[3]], "BGRA pixel {i}");
            }
            if !transparent {
                let bgrx = render(list, commands, PixelFormat::Bgrx8888, false, mode);
                for (i, (r, b)) in rgba.chunks_exact(4).zip(bgrx.chunks_exact(4)).enumerate() {
                    assert_eq!(b, [r[2], r[1], r[0], 0], "BGRX pixel {i}");
                }
            }
        }
    }
    fn color(alpha: f32) -> Color {
        Color {
            red: 0.83,
            green: 0.37,
            blue: 0.12,
            alpha,
        }
    }
    fn rect() -> PaintRect {
        PaintRect {
            x: 7.25,
            y: 5.5,
            width: 73.5,
            height: 58.25,
        }
    }
    fn radii() -> PaintCornerRadii {
        let r = PaintCornerRadius { x: 8.25, y: 7.75 };
        PaintCornerRadii {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        }
    }
    #[test]
    fn subset_mip_keys_reuse_only_matching_rectangles_and_match_fresh_draws() {
        let source: Vec<u8> = (0..256 * 256 * 4)
            .map(|i| {
                if i % 4 == 3 {
                    255
                } else {
                    (i * 31 + i / 17) as u8
                }
            })
            .collect();
        let list = ResourceContext {
            resources: Some(ResourceCatalog {
                images: vec![Image::borrowed(1, 256, 256, &source)],
                ..Default::default()
            }),
        };
        let mut canvas = SkCanvas::new(&list, 32, 24);
        for (x, y) in [
            (10.25, 13.75),
            (10.25, 13.75),
            (110.25, 13.75),
            (110.25, 13.75),
        ] {
            let command = DrawCommand {
                resource_id: 1,
                source_rect: PaintRect {
                    x,
                    y,
                    width: 80.5,
                    height: 90.5,
                },
                ..Default::default()
            };
            let destination = PaintRect {
                x: 2.5,
                y: 3.25,
                width: 18.0,
                height: 17.0,
            };
            canvas.clear_transparent();
            canvas.drawImageRect(&command, destination, &list);
            let mut fresh = SkCanvas::new(&list, 32, 24);
            fresh.clear_transparent();
            fresh.drawImageRect(&command, destination, &list);
            assert_eq!(canvas.pixmap.data(), fresh.pixmap.data());
        }
        assert_eq!(
            canvas.mip_images.len(),
            2,
            "different rectangles cannot share a scaled subset"
        );
    }
    #[test]
    fn offscreen_images_reject_before_building_mipmaps_and_boundary_still_draws() {
        let source = vec![255u8; 256 * 256 * 4];
        let list = ResourceContext {
            resources: Some(ResourceCatalog {
                images: vec![Image::borrowed(1, 256, 256, &source)],
                ..Default::default()
            }),
        };
        let mut canvas = SkCanvas::new(&list, 32, 24);
        let command = DrawCommand {
            resource_id: 1,
            source_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 256.0,
                height: 256.0,
            },
            ..Default::default()
        };
        for (x, y) in [(40.0, 0.0), (-40.0, 0.0), (0.0, 40.0), (0.0, -40.0)] {
            canvas.drawImageRect(
                &command,
                PaintRect {
                    x,
                    y,
                    width: 8.0,
                    height: 8.0,
                },
                &list,
            );
            assert!(
                canvas.mip_images.is_empty(),
                "culled images do not generate decodes"
            );
            assert!(
                canvas.images.is_empty(),
                "culled images do not convert source pixels"
            );
        }
        canvas.clear_transparent();
        canvas.drawImageRect(
            &command,
            PaintRect {
                x: 31.4,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
            &list,
        );
        assert!(!canvas.mip_images.is_empty());
        assert_eq!(&canvas.pixmap.data()[31 * 4..32 * 4], &[255; 4]);
    }
    #[test]
    fn bgra_bgrx_shapes_aa_clip_layers_f16_images_and_shadows() {
        let source = [
            230, 40, 70, 128, 10, 120, 220, 255, 77, 33, 11, 200, 20, 90, 100, 255,
        ];
        let list = ResourceContext {
            resources: Some(ResourceCatalog {
                images: vec![Image::borrowed(1, 2, 2, &source)],
                ..ResourceCatalog::default()
            }),
        };
        for clip in [false, true] {
            for layer in [
                None,
                Some(CommandKind::kSaveLayer),
                Some(CommandKind::kSaveLayerAlpha),
            ] {
                for kind in [
                    CommandKind::kDrawRect,
                    CommandKind::kDrawRoundedRect,
                    CommandKind::kDrawEllipse,
                    CommandKind::kDrawImageRect,
                    CommandKind::kDrawBoxShadow,
                ] {
                    let mut commands = vec![];
                    if clip {
                        commands.push(DrawCommand {
                            r#type: CommandKind::kClipRoundedRect,
                            rect: rect(),
                            corner_radii: radii(),
                            antialias: true,
                            ..DrawCommand::default()
                        });
                    }
                    if let Some(kind) = layer {
                        commands.push(DrawCommand {
                            r#type: kind,
                            opacity: 0.63,
                            ..DrawCommand::default()
                        });
                    }
                    commands.push(DrawCommand {
                        r#type: kind,
                        rect: rect(),
                        corner_radii: radii(),
                        color: color(0.5),
                        resource_id: 1,
                        source_rect: PaintRect {
                            x: 0.,
                            y: 0.,
                            width: 2.,
                            height: 2.,
                        },
                        antialias: true,
                        shadow_offset: Offset { x: 2., y: 3. },
                        blur_radius: 3.,
                        ..DrawCommand::default()
                    });
                    if layer.is_some() {
                        commands.push(DrawCommand {
                            r#type: CommandKind::kRestore,
                            ..DrawCommand::default()
                        });
                    }
                    compare(&list, &commands, GlyphRasterMode::Outlines);
                }
            }
        }
    }
    #[test]
    fn bgra_bgrx_gradient_vector_tail_scalar_and_clip() {
        let list = ResourceContext::default();
        for vertical in [false, true] {
            for alpha in [1., 0.5] {
                for stops in [2, 3] {
                    let shader = PaintShader {
                        start: Offset { x: 0., y: 0. },
                        end: if vertical {
                            Offset { x: 0., y: 72. }
                        } else {
                            Offset { x: 96., y: 30. }
                        },
                        stops: (0..stops)
                            .map(|i| PaintColorStop {
                                offset: i as f64 / (stops - 1) as f64,
                                offset_length: 0.0,
                                color: if i == 0 {
                                    color(alpha)
                                } else {
                                    Color {
                                        red: 0.13,
                                        green: 0.7,
                                        blue: 0.87,
                                        alpha,
                                    }
                                },
                            })
                            .collect(),
                        ..PaintShader::default()
                    };
                    for clip in [false, true] {
                        let mut commands = vec![];
                        if clip {
                            commands.push(DrawCommand {
                                r#type: CommandKind::kClipRoundedRect,
                                rect: rect(),
                                corner_radii: radii(),
                                antialias: true,
                                ..DrawCommand::default()
                            });
                        }
                        commands.push(DrawCommand {
                            r#type: CommandKind::kDrawGradientRect,
                            rect: PaintRect {
                                x: 0.,
                                y: 0.,
                                width: 91.,
                                height: 72.,
                            },
                            paint_shader: Some(shader.clone()),
                            ..DrawCommand::default()
                        });
                        compare(&list, &commands, GlyphRasterMode::Outlines);
                    }
                }
            }
        }
    }
    #[test]
    fn bgra_bgrx_platform_and_outline_glyphs() {
        let bytes = include_bytes!(
            "../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
        );
        let face = Face::parse(bytes, 0).unwrap();
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
        for mode in [GlyphRasterMode::Platform, GlyphRasterMode::Outlines] {
            for bold in [false, true] {
                let command = DrawCommand {
                    r#type: CommandKind::kDrawGlyphRun,
                    color: color(0.5),
                    font_size: 25.,
                    synthetic_bold: bold,
                    text_blob_origin: Offset { x: 3.25, y: 33.5 },
                    glyphs: vec![
                        PaintGlyph {
                            id: face.glyph_index('M').unwrap().0 as u32,
                            offset: Offset { x: 0., y: 0. },
                            ..PaintGlyph::default()
                        },
                        PaintGlyph {
                            id: face.glyph_index('a').unwrap().0 as u32,
                            offset: Offset { x: 25., y: 0. },
                            ..PaintGlyph::default()
                        },
                    ],
                    ..DrawCommand::default()
                };
                compare(&list, &[command], mode);
            }
        }
    }
    #[test]
    fn bgra_bgrx_canonical_blends_and_gamma_pipeline() {
        use crate::raster::{BlendMode, ColorSpace};
        for format in [PixelFormat::Bgra8888, PixelFormat::Bgrx8888] {
            for highp in [false, true] {
                for gamma in [ColorSpace::Linear, ColorSpace::FullSRGBGamma] {
                    for mode in [
                        BlendMode::kSrcOver,
                        BlendMode::kDstOver,
                        BlendMode::kSrcIn,
                        BlendMode::kDstIn,
                        BlendMode::kMultiply,
                        BlendMode::kScreen,
                        BlendMode::kColor,
                        BlendMode::kLuminosity,
                    ] {
                        let mut expected = Pixmap::new(33, 5).unwrap();
                        expected.fill(crate::raster::Color::from_rgba8(11, 83, 201, 255));
                        let mut target = Pixmap::install_pixels_with_format(
                            PixelStorage::owned(vec![0; 33 * 5 * 4]),
                            33,
                            5,
                            132,
                            format,
                        )
                        .unwrap();
                        target.fill(crate::raster::Color::from_rgba8(11, 83, 201, 255));
                        let mut paint = Paint::default();
                        paint.set_color_rgba8(217, 53, 101, 131);
                        paint.blend_mode = mode;
                        paint.force_hq_pipeline = highp;
                        paint.colorspace = gamma;
                        paint.anti_alias = true;
                        for p in [&mut expected, &mut target] {
                            p.fill_rect(
                                Rect::from_xywh(0.25, 0.5, 31.75, 3.5).unwrap(),
                                &paint,
                                Transform::identity(),
                                None,
                            );
                        }
                        for (r, b) in expected
                            .data()
                            .chunks_exact(4)
                            .zip(target.data().chunks_exact(4))
                        {
                            assert_eq!(
                                &b[..3],
                                &[r[2], r[1], r[0]],
                                "format={format:?} highp={highp} gamma={gamma:?} blend={mode:?}"
                            );
                            assert_eq!(
                                b[3],
                                if format == PixelFormat::Bgrx8888 {
                                    0
                                } else {
                                    r[3]
                                }
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod identity_clip_tests {
    use super::*;
    #[test]
    fn contiguous_opaque_rect_matches_scanline_blits_with_clip_and_formats() {
        use crate::{PixelFormat, PixelStorage};
        let resources = ResourceContext::default();
        let width = 37u32;
        let height = 23u32;
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for clip in [
                None,
                Some((0, 4, 37, 15)),
                Some((3, 4, 29, 15)),
                Some((0, 0, 0, 0)),
            ] {
                for (left, top, right, bottom) in [
                    (-5, -7, 42, 30),
                    (0, 5, 37, 14),
                    (4, 3, 29, 20),
                    (0, 25, 37, 29),
                ] {
                    for color in [
                        Color {
                            red: 1.,
                            green: 1.,
                            blue: 1.,
                            alpha: 1.,
                        },
                        Color {
                            red: 0.13,
                            green: 0.41,
                            blue: 0.79,
                            alpha: 1.,
                        },
                        Color {
                            red: 0.4,
                            green: 0.8,
                            blue: 0.2,
                            alpha: 0.57,
                        },
                    ] {
                        let make = || {
                            let mut c = SkCanvas::make_raster_direct_with_format(
                                &resources,
                                width,
                                height,
                                width as usize * 4,
                                PixelStorage::owned(vec![0; (width * height * 4) as usize]),
                                format,
                            )
                            .unwrap();
                            if let Some((x, y, w, h)) = clip {
                                c.replay_item(
                                    &DrawCommand {
                                        r#type: CommandKind::kClipRect,
                                        rect: PaintRect {
                                            x: x as f64,
                                            y: y as f64,
                                            width: w as f64,
                                            height: h as f64,
                                        },
                                        ..Default::default()
                                    },
                                    &resources,
                                );
                            }
                            c
                        };
                        let mut actual = make();
                        let mut expected = make();
                        actual.drawRect(
                            PaintRect {
                                x: left as f64,
                                y: top as f64,
                                width: (right - left) as f64,
                                height: (bottom - top) as f64,
                            },
                            color,
                            true,
                        );
                        let source = crate::cpu::mask_blitter::premultiply(color);
                        let l = left.clamp(0, width as i32) as usize;
                        let r = right.clamp(0, width as i32) as usize;
                        for y in top.clamp(0, height as i32)..bottom.clamp(0, height as i32) {
                            expected.blend_solid_span(
                                y as usize * width as usize + l,
                                r - l,
                                source,
                                255,
                                false,
                            );
                        }
                        assert_eq!(actual.finish_direct().into_vec(), expected.finish_direct().into_vec(), "format={format:?} clip={clip:?} rect={left},{top},{right},{bottom} alpha={}", color.alpha);
                    }
                }
            }
        }
    }

    #[test]
    fn device_bw_clip_matches_previous_dense_plane_for_aa_edges_and_layers() {
        let resources = ResourceContext::default();
        for phase in [0.0, 0.125, 0.5, 0.875] {
            let render = |dense: bool| {
                let mut canvas = SkCanvas::new(&resources, 83, 59);
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kClipRect,
                        rect: PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 83.0,
                            height: 59.0,
                        },
                        antialias: true,
                        ..Default::default()
                    },
                    &resources,
                );
                assert!(canvas.state.clip.is_none());
                if dense {
                    let mut mask = Mask::new(83, 59).unwrap();
                    mask.data_mut().fill(255);
                    canvas.state.clip = Some(std::sync::Arc::new(mask));
                }
                let radius = PaintCornerRadius { x: 7.25, y: 6.5 };
                for (kind, rect) in [
                    (
                        CommandKind::kDrawRoundedRect,
                        PaintRect {
                            x: -3.25 + phase,
                            y: -1.5 + phase,
                            width: 91.5,
                            height: 65.25,
                        },
                    ),
                    (
                        CommandKind::kStrokeRect,
                        PaintRect {
                            x: 1.0 + phase,
                            y: 2.5 + phase,
                            width: 79.75,
                            height: 54.25,
                        },
                    ),
                ] {
                    canvas.replay_item(
                        &DrawCommand {
                            r#type: kind,
                            rect,
                            corner_radii: PaintCornerRadii {
                                top_left: radius,
                                top_right: radius,
                                bottom_left: radius,
                                bottom_right: radius,
                            },
                            stroke_width: 0.75,
                            antialias: true,
                            color: Color {
                                red: 0.2,
                                green: 0.65,
                                blue: 0.8,
                                alpha: 0.57,
                            },
                            ..Default::default()
                        },
                        &resources,
                    );
                }
                canvas.save();
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kClipRect,
                        rect: PaintRect {
                            x: 3.25,
                            y: 4.5,
                            width: 70.75,
                            height: 41.25,
                        },
                        antialias: true,
                        ..Default::default()
                    },
                    &resources,
                );
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kDrawRect,
                        rect: PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 83.0,
                            height: 59.0,
                        },
                        antialias: true,
                        color: Color {
                            red: 0.8,
                            green: 0.15,
                            blue: 0.5,
                            alpha: 0.43,
                        },
                        ..Default::default()
                    },
                    &resources,
                );
                canvas.restore();
                canvas.finish()
            };
            assert_eq!(render(false), render(true), "phase={phase}");
        }
    }
}

#[cfg(test)]
mod bounded_clip_replay_round6 {
    use super::*;
    use crate::src::core::SkRasterClip::SkRasterClip;
    #[test]
    fn packed_solid_rounded_and_layer_replay_matches_dense_without_expansion() {
        let resources = ResourceContext::default();
        let radius = PaintCornerRadius { x: 9.25, y: 13.75 };
        let radii = PaintCornerRadii {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        };
        let rect = PaintRect {
            x: 31.75,
            y: 41.25,
            width: 99.5,
            height: 71.75,
        };
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            let clip = crate::cpu::analytic_aa::rounded_rect_clip_mask(
                rect,
                radii,
                Transform::identity(),
                320,
                300,
                None,
                None,
            )
            .unwrap();
            let summary =
                SkRasterClip::from_mask_builder(&clip.mask, Some(clip.mask.storage_bounds()))
                    .unwrap();
            let bytes = clip.mask.allocated_bytes();
            let mut actual = SkCanvas::new(&resources, 320, 300);
            let mut expected = SkCanvas::new(&resources, 320, 300);
            actual.pixmap.format = format;
            expected.pixmap.format = format;
            expected.state.clip = Some(std::sync::Arc::new(
                Mask::from_vec(
                    clip.mask.clone().take(),
                    IntSize::from_wh(320, 300).unwrap(),
                )
                .unwrap(),
            ));
            actual.state.clip = Some(std::sync::Arc::new(clip.mask));
            for c in [&mut actual, &mut expected] {
                c.state.clip_summary = Some(summary);
                c.state.clip_is_aa = true;
            }
            let color = Color {
                red: 0.19,
                green: 0.57,
                blue: 0.83,
                alpha: 0.73,
            };
            for kind in [CommandKind::kDrawRect, CommandKind::kDrawRoundedRect] {
                let command = DrawCommand {
                    r#type: kind,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 300.0,
                        height: 260.0,
                    },
                    corner_radii: radii,
                    color,
                    antialias: true,
                    ..Default::default()
                };
                for c in [&mut actual, &mut expected] {
                    c.replay_item(&command, &resources);
                }
                assert_eq!(actual.pixmap.data(), expected.pixmap.data());
                assert_eq!(
                    actual.state.clip.as_deref().unwrap().allocated_bytes(),
                    bytes
                );
            }
            for f16 in [false, true] {
                for c in [&mut actual, &mut expected] {
                    c.internalSaveLayer(
                        &DrawCommand {
                            opacity: 0.73,
                            ..Default::default()
                        },
                        f16,
                    );
                    c.replay_item(
                        &DrawCommand {
                            rect: PaintRect {
                                x: 0.0,
                                y: 0.0,
                                width: 320.0,
                                height: 300.0,
                            },
                            color,
                            ..Default::default()
                        },
                        &resources,
                    );
                    c.restore();
                }
                assert_eq!(
                    actual.pixmap.data(),
                    expected.pixmap.data(),
                    "{format:?} f16={f16}"
                );
                assert_eq!(
                    actual.state.clip.as_deref().unwrap().allocated_bytes(),
                    bytes,
                    "restore must retain packed clip"
                );
            }
        }
    }
}

#[cfg(test)]
mod preserving_damage_tests {
    use super::*;
    use crate::{PixelFormat, PixelStorage};

    #[test]
    fn preserving_target_matches_full_replay_when_damage_closes_over_aa_geometry() {
        let resources = ResourceContext::default();
        let (width, height) = (106u32, 82u32);
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for scale in [1.0, 2.0] {
                for opacity in [0.25, 1.0] {
                    let rectangle = |x, y, width, height| PaintRect {
                        x,
                        y,
                        width,
                        height,
                    };
                    let commands = [
                        DrawCommand {
                            r#type: CommandKind::kDrawRect,
                            rect: rectangle(0., 0., 53., 41.),
                            color: Color {
                                red: 1.,
                                green: 1.,
                                blue: 1.,
                                alpha: 1.,
                            },
                            antialias: false,
                            ..Default::default()
                        },
                        DrawCommand {
                            r#type: CommandKind::kSave,
                            ..Default::default()
                        },
                        DrawCommand {
                            r#type: CommandKind::kDrawRoundedRect,
                            rect: rectangle(2.125, 3.375, 26.5, 22.25),
                            corner_radii: PaintCornerRadii {
                                top_left: PaintCornerRadius { x: 5., y: 5. },
                                top_right: PaintCornerRadius { x: 5., y: 5. },
                                bottom_left: PaintCornerRadius { x: 5., y: 5. },
                                bottom_right: PaintCornerRadius { x: 5., y: 5. },
                            },
                            color: Color {
                                red: 0.8,
                                green: 0.2,
                                blue: 0.4,
                                alpha: 0.75,
                            },
                            ..Default::default()
                        },
                        DrawCommand {
                            r#type: CommandKind::kSaveLayerAlpha,
                            opacity,
                            ..Default::default()
                        },
                        DrawCommand {
                            r#type: CommandKind::kDrawRect,
                            rect: rectangle(4.25, 7.375, 25.5, 18.25),
                            color: Color {
                                red: 0.1,
                                green: 0.6,
                                blue: 0.3,
                                alpha: 0.6,
                            },
                            ..Default::default()
                        },
                        DrawCommand {
                            r#type: CommandKind::kRestore,
                            ..Default::default()
                        },
                        DrawCommand {
                            r#type: CommandKind::kRestore,
                            ..Default::default()
                        },
                    ];
                    let mut full = SkCanvas::make_raster_direct_with_format(
                        &resources,
                        width,
                        height,
                        width as usize * 4,
                        PixelStorage::owned(vec![19; width as usize * height as usize * 4]),
                        format,
                    )
                    .unwrap();
                    full.set_scale(scale);
                    for command in &commands {
                        full.replay_item(command, &resources);
                    }
                    let expected = full.finish_direct().into_vec();
                    let initial: Vec<u8> = (0..width as usize * height as usize)
                        .flat_map(|i| {
                            format.encode([(i * 17) as u8, (i * 31) as u8, (i * 7) as u8, 255])
                        })
                        .collect();
                    let mut partial = SkCanvas::make_raster_direct_with_format_preserving(
                        &resources,
                        width,
                        height,
                        width as usize * 4,
                        PixelStorage::owned(initial.clone()),
                        format,
                        true,
                    )
                    .unwrap();
                    assert_eq!(partial.pixmap.data(), initial);
                    // A semantic clip can switch the native AA blitter's
                    // integer rounding when it cuts a shape horizontally.
                    // This supported damage closes over every participating
                    // rounded shape and keeps the layer's device origin (0,0).
                    // Clip in device coordinates before CSS scale.
                    let right = (36.0 * scale) as usize;
                    let bottom = (32.0 * scale) as usize;
                    partial.replay_item(
                        &DrawCommand {
                            r#type: CommandKind::kClipRect,
                            rect: rectangle(0., 0., right as f64, bottom as f64),
                            antialias: false,
                            ..Default::default()
                        },
                        &resources,
                    );
                    partial.set_scale(scale);
                    for command in &commands {
                        partial.replay_item(command, &resources);
                    }
                    // An extra restore at the root cannot remove its damage clip.
                    partial.restore();
                    let actual = partial.finish_direct().into_vec();
                    for y in 0..height as usize {
                        for x in 0..width as usize {
                            let at = (y * width as usize + x) * 4;
                            let reference = if x < right && y < bottom {
                                &expected
                            } else {
                                &initial
                            };
                            assert_eq!(
                                &actual[at..at + 4],
                                &reference[at..at + 4],
                                "format={format:?} scale={scale} opacity={opacity} x={x} y={y}"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn arbitrary_semantic_clip_is_not_a_full_frame_pixel_crop() {
        // Original fixture retained as an unsupported-contract regression.
        // Independent originalSkia proof: damage-native-semantics-round2.log
        // has identical Rust/native clipped bytes, and this same full/clip
        // pixel difference. Do not erase H2 provenance or loosen tolerance.
        let resources = ResourceContext::default();
        let render = |clipped: bool| {
            let mut canvas = SkCanvas::new(&resources, 53, 41);
            if clipped {
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kClipRect,
                        rect: PaintRect {
                            x: 7.,
                            y: 9.,
                            width: 29.,
                            height: 23.,
                        },
                        antialias: false,
                        ..Default::default()
                    },
                    &resources,
                );
            }
            let radius = PaintCornerRadius { x: 5., y: 5. };
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kDrawRoundedRect,
                    rect: PaintRect {
                        x: 2.125,
                        y: 3.375,
                        width: 26.5,
                        height: 22.25,
                    },
                    corner_radii: PaintCornerRadii {
                        top_left: radius,
                        top_right: radius,
                        bottom_left: radius,
                        bottom_right: radius,
                    },
                    color: Color {
                        red: 0.8,
                        green: 0.2,
                        blue: 0.4,
                        alpha: 0.75,
                    },
                    ..Default::default()
                },
                &resources,
            );
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kSaveLayerAlpha,
                    opacity: 0.25,
                    ..Default::default()
                },
                &resources,
            );
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kDrawRect,
                    rect: PaintRect {
                        x: 4.25,
                        y: 7.375,
                        width: 25.5,
                        height: 18.25,
                    },
                    color: Color {
                        red: 0.1,
                        green: 0.6,
                        blue: 0.3,
                        alpha: 0.6,
                    },
                    ..Default::default()
                },
                &resources,
            );
            canvas.restore();
            canvas.finish()
        };
        let full = render(false);
        let clipped = render(true);
        let at = (25 * 53 + 7) * 4;
        assert_eq!(&full[at..at + 4], &[197, 142, 155, 255]);
        assert_eq!(&clipped[at..at + 4], &[196, 144, 156, 255]);
        assert_ne!(&full[at..at + 4], &clipped[at..at + 4]);
    }
}

#[cfg(all(test, target_os = "macos"))]
mod conservative_glyph_damage_tests {
    use super::*;
    #[test]
    fn whole_font_bounds_enclose_all_drawn_native_mask_pixels() {
        let bytes = include_bytes!(
            "../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
        );
        let face = ttf_parser::Face::parse(bytes, 0).unwrap();
        let font = FontFace {
            family: "Roboto",
            native_family: "",
            weight: 400.,
            italic: false,
            bytes,
            face_index: 0,
            variations: Vec::new(),
        };
        let list = ResourceContext {
            resources: Some(ResourceCatalog {
                fonts: vec![font],
                ..Default::default()
            }),
        };
        let glyphs = "RgQij"
            .chars()
            .enumerate()
            .map(|(i, ch)| PaintGlyph {
                id: face.glyph_index(ch).unwrap().0 as u32,
                offset: Offset {
                    x: i as f64 * 12.125,
                    y: 0.,
                },
                ..Default::default()
            })
            .collect();
        let base = DrawCommand {
            r#type: CommandKind::kDrawGlyphRun,
            font_size: 14.,
            glyphs,
            text_blob_origin: Offset {
                x: 25.125,
                y: 30.375,
            },
            color: Color {
                red: 0.1,
                green: 0.3,
                blue: 0.7,
                alpha: 1.,
            },
            ..Default::default()
        };
        for scale in [0.5, 1., 2.] {
            for offset in [-23.125, 0., 7.375] {
                let mut canvas = SkCanvas::new(&list, 240, 160);
                canvas.state.transform =
                    Transform::from_scale(scale, scale).post_translate(offset, 3.125);
                let bounds = SkCanvas::conservative_platform_glyph_run_bounds(
                    &list,
                    &base,
                    canvas.state.transform,
                )
                .unwrap();
                canvas.drawGlyphRunList(&base, &list);
                let mut drawn = 0;
                for (i, pixel) in canvas.pixmap.data().chunks_exact(4).enumerate() {
                    if pixel == [255; 4] {
                        continue;
                    }
                    let (x, y) = (i % 240, i / 240);
                    assert!(
                        x as f64 >= bounds.x
                            && y as f64 >= bounds.y
                            && (x as f64) < bounds.x + bounds.width
                            && (y as f64) < bounds.y + bounds.height,
                        "scale={scale} offset={offset} pixel={x},{y} bounds={bounds:?}"
                    );
                    drawn += 1;
                }
                assert!(drawn > 0);
            }
        }
        for mut unsupported in [base.clone(), base.clone(), base.clone()] {
            unsupported.synthetic_bold = true;
            assert!(SkCanvas::conservative_platform_glyph_run_bounds(
                &list,
                &unsupported,
                Transform::identity()
            )
            .is_none());
        }
        let mut unsupported = base.clone();
        unsupported.synthetic_italic = true;
        assert!(SkCanvas::conservative_platform_glyph_run_bounds(
            &list,
            &unsupported,
            Transform::identity()
        )
        .is_none());
        unsupported = base.clone();
        unsupported.stroke_glyphs = true;
        assert!(SkCanvas::conservative_platform_glyph_run_bounds(
            &list,
            &unsupported,
            Transform::identity()
        )
        .is_none());
    }
}

#[cfg(test)]
mod persistent_image_cache_tests {
    use super::*;
    fn input(width: u32, height: u32) -> Vec<u8> {
        (0..width * height)
            .flat_map(|i| {
                [
                    (i * 31) as u8,
                    (i * 17 + 29) as u8,
                    (i * 47 + 11) as u8,
                    (i * 13 + 97) as u8,
                ]
            })
            .collect()
    }
    fn render(
        rgba: &[u8],
        width: u32,
        height: u32,
        format: crate::PixelFormat,
        scale: f64,
        frame: u32,
        clip: bool,
        layer: bool,
        cache: Option<RasterImageCache>,
    ) -> (Vec<u8>, RasterImageCache) {
        let resources = ResourceContext {
            resources: Some(ResourceCatalog {
                images: vec![Image::borrowed(1, width, height, rgba)],
                ..Default::default()
            }),
        };
        let mut canvas = SkCanvas::make_raster_direct_with_format(
            &resources,
            226,
            166,
            226 * 4,
            crate::PixelStorage::owned(vec![93; 226 * 166 * 4]),
            format,
        )
        .unwrap();
        canvas.set_scale(scale);
        if let Some(cache) = cache {
            canvas.install_image_cache(cache);
        }
        if clip {
            let radius = PaintCornerRadius { x: 8.25, y: 14.375 };
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kClipRoundedRect,
                    rect: PaintRect {
                        x: 3.125,
                        y: 4.375,
                        width: 105.5,
                        height: 73.25,
                    },
                    corner_radii: PaintCornerRadii {
                        top_left: radius,
                        top_right: radius,
                        bottom_left: radius,
                        bottom_right: radius,
                    },
                    antialias: true,
                    ..Default::default()
                },
                &resources,
            );
        }
        if layer {
            canvas.internalSaveLayer(
                &DrawCommand {
                    opacity: 0.35,
                    rect: PaintRect {
                        x: 2.,
                        y: 3.,
                        width: 108.,
                        height: 78.,
                    },
                    ..Default::default()
                },
                false,
            );
        }
        for subset in [false, true] {
            let source = if subset {
                PaintRect {
                    x: 4.125 + f64::from(frame % 2) * 0.5,
                    y: 2.375,
                    width: 63.5,
                    height: 71.125,
                }
            } else {
                PaintRect {
                    x: 0.,
                    y: 0.,
                    width: f64::from(width),
                    height: f64::from(height),
                }
            };
            let destination = if subset {
                PaintRect {
                    x: 59.125,
                    y: 9.375 + f64::from(frame % 3),
                    width: 23.25,
                    height: 29.5,
                }
            } else {
                PaintRect {
                    x: 6.25,
                    y: 12.375 + f64::from(frame % 3),
                    width: if frame % 2 == 0 { 61.25 } else { 23.25 },
                    height: if frame % 2 == 0 { 43.5 } else { 17.5 },
                }
            };
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kDrawImageRect,
                    resource_id: 1,
                    rect: destination,
                    source_rect: source,
                    ..Default::default()
                },
                &resources,
            );
        }
        if layer {
            canvas.restore();
        }
        let cache = canvas.take_image_cache();
        (canvas.finish_direct().into_vec(), cache)
    }
    #[test]
    fn cached_decodes_replay_every_changed_frame_like_cold_canvas() {
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for scale in [1., 2.] {
                for clip in [false, true] {
                    for layer in [false, true] {
                        let mut rgba = input(129, 97);
                        let mut cache = RasterImageCache::default();
                        let mut previous: Option<Vec<u8>> = None;
                        for frame in 0..6 {
                            // Same id and same allocation address changes content;
                            // then same byte count changes shape. Both invalidate all
                            // formerly derived full/subset mip levels exactly.
                            if frame == 3 {
                                for p in rgba.chunks_exact_mut(4) {
                                    p[0] ^= 255;
                                    p[1] ^= 127;
                                    p[2] ^= 63;
                                }
                            }
                            let (width, height) = if frame >= 4 { (97, 129) } else { (129, 97) };
                            let (old, _) = render(
                                &rgba, width, height, format, scale, frame, clip, layer, None,
                            );
                            let (new, next) = render(
                                &rgba,
                                width,
                                height,
                                format,
                                scale,
                                frame,
                                clip,
                                layer,
                                Some(cache),
                            );
                            let difference = new.iter().zip(&old).position(|(a, b)| a != b);
                            assert!(new.len()==old.len() && difference.is_none(),
                        "format={format:?} scale={scale} frame={frame} clip={clip} layer={layer} firstdiff={difference:?} cached_len={} cold_len={}",new.len(),old.len());
                            if let Some(previous) = &previous {
                                assert!(new!=*previous,"changed actualdraw must differ: format={format:?} scale={scale} frame={frame} clip={clip} layer={layer}");
                            }
                            previous = Some(new.clone());
                            cache = next;
                            assert!(cache.resident_pixel_bytes() <= 64 * 1024 * 1024);
                        }
                        assert_eq!(cache.stats().source_conversions, 3);
                        assert_eq!(cache.stats().source_invalidations, 2);
                        assert_eq!(cache.stats().source_reuses, 3);
                        assert!(cache.stats().mip_builds > 0);
                    }
                }
            }
        }
    }
    #[test]
    fn shared_image_identity_retains_owner_and_invalidates_cow_and_dimensions() {
        use std::sync::Arc;
        let mut bytes = Arc::new(input(4, 3));
        let mut cache = RasterImageCache::default();
        let authenticate = |image: Image<'_>, cache: RasterImageCache| {
            let resources = ResourceContext {
                resources: Some(ResourceCatalog {
                    images: vec![image],
                    ..Default::default()
                }),
            };
            let mut canvas = SkCanvas::new(&resources, 4, 3);
            canvas.install_image_cache(cache);
            canvas.authenticate_cached_image(&resources.resources.as_ref().unwrap().images[0]);
            canvas.take_image_cache()
        };
        cache = authenticate(Image::shared(1, 4, 3, &bytes), cache);
        assert!(Arc::ptr_eq(&cache.sources[&1].rgba, &bytes));
        cache = authenticate(Image::shared(1, 4, 3, &bytes), cache);
        assert_eq!(cache.stats().source_reuses, 1);
        let equal = Arc::new(bytes.as_ref().clone());
        cache = authenticate(Image::shared(1, 4, 3, &equal), cache);
        assert!(Arc::ptr_eq(&cache.sources[&1].rgba, &equal));
        cache = authenticate(Image::shared(1, 4, 3, &bytes), cache);
        let retained = bytes.clone();
        Arc::make_mut(&mut bytes)[0] ^= 255;
        assert_ne!(bytes[0], retained[0]);
        cache = authenticate(Image::shared(1, 4, 3, &bytes), cache);
        assert_eq!(cache.stats().source_invalidations, 1);
        cache = authenticate(Image::shared(1, 3, 4, &bytes), cache);
        assert_eq!(cache.stats().source_invalidations, 2);
        let mut borrowed = bytes.as_ref().clone();
        let mut reassigned = Image::shared(1, 3, 4, &bytes);
        borrowed[3] = 0;
        reassigned.rgba8 = &borrowed;
        assert!(reassigned.shared_pixels().is_none());
        cache = authenticate(reassigned, cache);
        assert_eq!(cache.stats().source_invalidations, 3);
        assert!(!cache.sources[&1].opaque);
        borrowed[0] ^= 255;
        cache = authenticate(Image::borrowed(1, 3, 4, &borrowed), cache);
        assert_eq!(cache.stats().source_invalidations, 4);
    }
    #[test]
    fn opaque_cached_image_stores_match_src_over_and_alpha_invalidation() {
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            let mut rgba = input(129, 97);
            for pixel in rgba.chunks_exact_mut(4) {
                pixel[3] = 255;
            }
            let mut cache = RasterImageCache::default();
            for frame in 0..3 {
                if frame == 2 {
                    rgba[(40 * 129 + 40) * 4 + 3] = 64;
                }
                let (expected, _) = render(&rgba, 129, 97, format, 2., frame, true, false, None);
                let (actual, next) =
                    render(&rgba, 129, 97, format, 2., frame, true, false, Some(cache));
                assert_eq!(actual, expected, "format={format:?} frame={frame}");
                assert_eq!(next.sources[&1].opaque, frame != 2);
                cache = next;
            }
            assert_eq!(cache.stats().source_invalidations, 1);
        }
    }
    #[test]
    fn cache_eviction_and_resource_pointer_aba_never_reuse_stale_pixels() {
        let rgba = input(129, 97);
        let mut cache = RasterImageCache::with_pixel_byte_limit(1);
        for frame in 0..3 {
            let (old, _) = render(
                &rgba,
                129,
                97,
                crate::PixelFormat::Rgba8888,
                1.,
                frame,
                false,
                false,
                None,
            );
            let (new, next) = render(
                &rgba,
                129,
                97,
                crate::PixelFormat::Rgba8888,
                1.,
                frame,
                false,
                false,
                Some(cache),
            );
            assert_eq!(new, old);
            cache = next;
            assert_eq!(cache.resident_pixel_bytes(), 0);
        }
        assert_eq!(cache.stats().source_conversions, 3);
        // A distinct allocation with identical immutable content is reusable.
        let mut cache = RasterImageCache::default();
        let (_, next) = render(
            &rgba,
            129,
            97,
            crate::PixelFormat::Bgra8888,
            1.,
            0,
            false,
            false,
            Some(cache),
        );
        cache = next;
        let distinct = rgba.clone();
        assert_ne!(distinct.as_ptr(), rgba.as_ptr());
        let (old, _) = render(
            &distinct,
            129,
            97,
            crate::PixelFormat::Bgra8888,
            1.,
            1,
            false,
            false,
            None,
        );
        let (new, next) = render(
            &distinct,
            129,
            97,
            crate::PixelFormat::Bgra8888,
            1.,
            1,
            false,
            false,
            Some(cache),
        );
        assert_eq!(old, new);
        assert_eq!(next.stats().source_conversions, 1);
        assert_eq!(next.stats().source_reuses, 1);
    }
}

#[cfg(all(test, target_os = "macos"))]
mod variable_glyph_damage_tests {
    use super::*;
    #[test]
    fn selected_variable_instance_bounds_enclose_native_draw_pixels() {
        for bytes in [
            include_bytes!("../../../../../src/third_party/skia/resources/fonts/Distortable.ttf")
                .as_slice(),
            include_bytes!("../../../../../src/third_party/skia/resources/fonts/Variable.ttf")
                .as_slice(),
        ] {
            let parsed = ttf_parser::Face::parse(bytes, 0).unwrap();
            assert!(!parsed.variation_axes().is_empty());
            for phase in 0..3 {
                let axes = parsed
                    .variation_axes()
                    .into_iter()
                    .map(|axis| FontVariation {
                        tag: u32::from_be_bytes(axis.tag.to_bytes()),
                        value: match phase {
                            0 => axis.min_value,
                            1 => axis.def_value,
                            _ => axis.max_value,
                        },
                    })
                    .collect();
                let list = ResourceContext {
                    resources: Some(ResourceCatalog {
                        fonts: vec![FontFace {
                            family: "",
                            native_family: "",
                            weight: 400.,
                            italic: false,
                            bytes,
                            face_index: 0,
                            variations: axes,
                        }],
                        ..Default::default()
                    }),
                };
                let command = DrawCommand {
                    r#type: CommandKind::kDrawGlyphRun,
                    font_size: 18.,
                    text_blob_origin: Offset {
                        x: 40.125,
                        y: 60.375,
                    },
                    glyphs: (1..parsed.number_of_glyphs().min(9))
                        .enumerate()
                        .map(|(i, id)| PaintGlyph {
                            id: id as u32,
                            offset: Offset {
                                x: i as f64 * 15.125,
                                y: 0.,
                            },
                            ..Default::default()
                        })
                        .collect(),
                    color: Color {
                        red: 0.15,
                        green: 0.35,
                        blue: 0.65,
                        alpha: 1.,
                    },
                    ..Default::default()
                };
                for scale in [0.5, 1., 2.] {
                    let mut canvas = SkCanvas::new(&list, 480, 240);
                    canvas.set_scale(scale);
                    let bounds = SkCanvas::conservative_platform_glyph_run_bounds(
                        &list,
                        &command,
                        canvas.state.transform,
                    )
                    .expect("actual instance metrics fallback");
                    assert!(bounds.width < 480. && bounds.height < 240.);
                    canvas.drawGlyphRunList(&command, &list);
                    let mut drawn = 0;
                    for (i, pixel) in canvas.pixmap.data().chunks_exact(4).enumerate() {
                        if pixel == [255; 4] {
                            continue;
                        }
                        let (x, y) = ((i % 480) as f64, (i / 480) as f64);
                        assert!(
                            x >= bounds.x
                                && y >= bounds.y
                                && x < bounds.x + bounds.width
                                && y < bounds.y + bounds.height,
                            "phase={phase} scale={scale} pixel={x},{y}"
                        );
                        drawn += 1;
                    }
                    assert!(
                        drawn > 0,
                        "fontbytes={} phase={phase} scale={scale} axes={:?} bounds={bounds:?}",
                        bytes.len(),
                        list.resources.as_ref().unwrap().fonts[0].variations
                    );
                    let mut unsupported = command.clone();
                    unsupported.synthetic_italic = true;
                    assert!(SkCanvas::conservative_platform_glyph_run_bounds(
                        &list,
                        &unsupported,
                        canvas.state.transform
                    )
                    .is_none());
                }
            }
        }
    }
}
