//! Immutable resource catalogs and shaped glyph values retained with a
//! fragment result (`layoutng_fragment_tree/fragment_tree.h`).

use foundation::{
    FontSmoothing, FontVariation, LayoutOffset, LayoutPaintWritingMode, LayoutViewportGeometry,
    PaintColor,
};
use std::sync::Arc;

/// Immutable resources and host geometry retained with a fragment result.
// cpp: layoutng_fragment_tree/fragment_tree.h:12-17
#[derive(Clone, PartialEq)]
pub struct PaintResources {
    pub fonts: font_engine::FontFaceCatalog,
    /// Shared with the publishing ConstraintSpace: each publication takes a
    /// reference, never a copy of the catalog.
    pub images: Arc<[image_resource::PaintImage]>,
    /// Immutable front-buffer snapshots published by Canvas contexts. They
    /// remain a distinct producer catalog even though Raster lowers both
    /// canvas and decoded bitmap frames to SkImage resources.
    pub canvases: Arc<[canvas::CanvasPaintResource]>,
    pub device_pixel_ratio: f64,
    pub preferred_color_scheme: u8,
    /// Page/browser zoom already applied to retained Layout coordinates.
    pub layout_zoom_factor: f64,
    /// LocalFrameView::BaseBackgroundColor() after host/color-scheme policy.
    /// Kept independently from the admission bit because transparent and
    /// non-white embedder colors are observable during printing.
    pub base_background_color: PaintColor,
    pub paints_opaque_base_background: bool,
    /// Settings::GetShouldClearDocumentBackground. When the base backdrop is
    /// painted, ViewPainter uses source replacement instead of source-over.
    pub should_clear_document_background: bool,
    pub viewport: Option<LayoutViewportGeometry>,
}

impl Default for PaintResources {
    fn default() -> Self {
        Self {
            fonts: font_engine::FontFaceCatalog::default(),
            images: Default::default(),
            canvases: Default::default(),
            device_pixel_ratio: 1.0,
            preferred_color_scheme: 0,
            layout_zoom_factor: 1.0,
            base_background_color: PaintColor {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
            paints_opaque_base_background: true,
            should_clear_document_background: true,
            viewport: None,
        }
    }
}

impl PaintResources {
    pub fn ArtifactToDeviceScale(&self) -> f64 {
        self.device_pixel_ratio / self.layout_zoom_factor.max(f64::MIN_POSITIVE)
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct PaintGlyph {
    pub id: u32,
    pub character_index: u32,
    pub canvas_rotation: u8,
    pub offset: LayoutOffset,
    pub advance: f64,
}

/// Immutable source shaping output retained by Paint. The glyph vector is
/// shared so crossing the Layout/Fragment boundary never copies a run.
// cpp: layoutng_fragment_tree/fragment_tree.h:133-153
#[derive(Clone, PartialEq)]
pub struct PaintGlyphRun {
    pub font_face_index: u32,
    pub font_variations: Vec<FontVariation>,
    pub font_size: f64,
    pub ascent: f64,
    pub baseline: f64,
    pub underline_position: f64,
    pub underline_thickness: Option<f64>,
    pub strikeout_position: f64,
    pub strikeout_thickness: f64,
    pub horizontal: bool,
    pub rtl: bool,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
    pub font_smoothing: FontSmoothing,
    pub writing_mode: LayoutPaintWritingMode,
    pub glyphs: Arc<Vec<PaintGlyph>>,
}

impl Default for PaintGlyphRun {
    fn default() -> Self {
        Self {
            font_face_index: 0,
            font_variations: Vec::new(),
            font_size: 16.0,
            ascent: 0.0,
            baseline: 0.0,
            underline_position: 0.0,
            underline_thickness: None,
            strikeout_position: 0.0,
            strikeout_thickness: 1.0,
            horizontal: true,
            rtl: false,
            synthetic_bold: false,
            synthetic_italic: false,
            font_smoothing: FontSmoothing::kAuto,
            writing_mode: LayoutPaintWritingMode::kHorizontalTb,
            glyphs: Arc::new(Vec::new()),
        }
    }
}
