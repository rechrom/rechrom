//! Native text shaping and glyph-paint export owned by `layout_fragment`.
//!
//! Included at the reciprocal native boundary so paint-time glyph creation,
//! text-combine projection, and list-marker payloads retain their native
//! inputs without losing package attribution.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:832-844
struct GlyphExportContext {
    runs: *mut Vec<PaintGlyphRunBuilder>,
    writing_mode: WritingMode,
    current_font: *const SimpleFontData,
    current_rotation: CanvasRotationInVertical,
    optical_size: f64,
    baseline: f64,
    last_run: usize,
    last_glyph: usize,
    last_total_advance: f32,
    has_last_glyph: bool,
}

// Keep the mutable glyph buffer outside its final Arc while the text blob is
// being built. Chromium's SkTextBlobBuilder writes directly into its unique
// run buffer; performing Arc copy-on-write checks for every glyph adds work
// that does not exist in that source path.
struct PaintGlyphRunBuilder {
    run: PaintGlyphRun,
    glyphs: Vec<PaintGlyph>,
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:846-929
fn ExportGlyph(
    opaque: *mut c_void,
    character_index: u32,
    glyph: u16,
    glyph_offset: gfx::Vector2dF,
    total_advance: f32,
    is_horizontal: bool,
    canvas_rotation: CanvasRotationInVertical,
    font: *const SimpleFontData,
) {
    let context = unsafe { &mut *(opaque as *mut GlyphExportContext) };
    let runs = unsafe { &mut *context.runs };
    if context.has_last_glyph {
        runs[context.last_run].glyphs[context.last_glyph].advance =
            (total_advance - context.last_total_advance) as f64;
    }
    if runs.is_empty()
        || context.current_font != font
        || runs
            .last()
            .is_some_and(|builder| builder.run.horizontal != is_horizontal)
        || context.current_rotation != canvas_rotation
    {
        context.current_font = font;
        context.current_rotation = canvas_rotation;
        let font = unsafe { &*font };
        let platform = font.PlatformData();
        let raw_font = platform.RawFont();
        // Shaping may select a platform size different from the CSS computed size.
        let run_font_size = platform.size() as f64;
        let default_thickness = 1.0_f64.max(run_font_size / 16.0);
        let underline_position = -raw_font
            .Metric(FontMetric::kUnderlineOffset)
            .unwrap_or(-run_font_size / 10.0);
        let underline_thickness = raw_font
            .Metric(FontMetric::kUnderlineSize)
            .unwrap_or(default_thickness)
            .abs();
        let strikeout_position = raw_font
            .Metric(FontMetric::kStrikeoutOffset)
            .unwrap_or(run_font_size * 0.3);
        let strikeout_thickness = raw_font
            .Metric(FontMetric::kStrikeoutSize)
            .unwrap_or(default_thickness)
            .abs();
        let mut variations = platform.ResourceVariations().to_vec();
        // The platform font selector can encode optical sizing in the native
        // typeface rather than in its explicit variation list. Paint records
        // must nevertheless retain the specified CSS optical size: raster is
        // allowed to scale the glyph to the zoomed/device size, but must not
        // silently select a different optical design at that later boundary.
        if raw_font.HasVariationAxis(OPTICAL_SIZE_TAG)
            && !variations
                .iter()
                .any(|variation| variation.tag == OPTICAL_SIZE_TAG)
        {
            variations.push(FontVariation {
                tag: OPTICAL_SIZE_TAG,
                value: context.optical_size as f32,
            });
        }
        runs.push(PaintGlyphRunBuilder {
            run: PaintGlyphRun {
                font_face_index: platform.ResourceFaceIndex(),
                font_variations: variations,
                font_size: run_font_size,
                ascent: font.GetFontMetrics().FloatAscent() as f64,
                // TextFragmentPainter establishes the origin from the primary font.
                baseline: context.baseline,
                underline_position,
                underline_thickness,
                strikeout_position,
                strikeout_thickness,
                horizontal: is_horizontal,
                rtl: false,
                synthetic_bold: platform.SyntheticBold(),
                synthetic_italic: platform.SyntheticItalic(),
                writing_mode: context.writing_mode,
                ..PaintGlyphRun::default()
            },
            glyphs: Vec::new(),
        });
    }
    let run = runs.last_mut().expect("glyph run");
    let mut vertical_baseline_x_offset = 0.0;
    if !is_horizontal && IsCanvasRotationInVerticalUpright(canvas_rotation) {
        let metrics = unsafe { &*font }.GetFontMetrics();
        vertical_baseline_x_offset =
            (metrics.FloatAscent() - metrics.FloatAscentFor(FontBaseline::kCentralBaseline)) as f64;
    }
    run.glyphs.push(PaintGlyph {
        id: glyph as u32,
        character_index,
        canvas_rotation: canvas_rotation as u8,
        offset: if is_horizontal {
            Offset {
                x: (total_advance + glyph_offset.x()) as f64,
                y: glyph_offset.y() as f64,
            }
        } else {
            Offset {
                x: glyph_offset.x() as f64 + vertical_baseline_x_offset,
                y: (total_advance + glyph_offset.y()) as f64,
            }
        },
        ..PaintGlyph::default()
    });
    context.last_glyph = run.glyphs.len() - 1;
    context.last_run = runs.len() - 1;
    context.last_total_advance = total_advance;
    context.has_last_glyph = true;
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:931-951
#[inline(never)]
pub(super) fn ExportShapeResult(
    shape: *const ShapeResultView,
    optical_size: f64,
    baseline: f64,
    writing_mode: WritingMode,
    runs: &mut Vec<PaintGlyphRun>,
) {
    if shape.is_null() {
        return;
    }
    let shape = unsafe { &*shape };
    let mut builders = Vec::new();
    let mut context = GlyphExportContext {
        runs: &mut builders,
        writing_mode,
        current_font: std::ptr::null(),
        current_rotation: CanvasRotationInVertical::kRegular,
        optical_size,
        baseline,
        last_run: 0,
        last_glyph: 0,
        last_total_advance: 0.0,
        has_last_glyph: false,
    };
    shape.ForEachGlyph(0.0, ExportGlyph, &mut context as *mut _ as *mut c_void);
    if context.has_last_glyph {
        builders[context.last_run].glyphs[context.last_glyph].advance =
            (shape.Width() - context.last_total_advance) as f64;
    }
    let rtl = shape.IsRtl();
    runs.reserve(builders.len());
    runs.extend(builders.into_iter().map(|builder| {
        let mut run = builder.run;
        run.rtl = rtl;
        run.glyphs = Arc::new(builder.glyphs);
        run
    }));
}

struct FirstGlyphFont {
    font: *const SimpleFontData,
    glyph: u16,
}

fn CaptureFirstGlyphFont(
    opaque: *mut c_void,
    _character_index: u32,
    glyph: u16,
    _glyph_offset: gfx::Vector2dF,
    _total_advance: f32,
    _is_horizontal: bool,
    _canvas_rotation: CanvasRotationInVertical,
    font: *const SimpleFontData,
) {
    let context = unsafe { &mut *(opaque as *mut FirstGlyphFont) };
    if context.font.is_null() {
        context.font = font;
        context.glyph = glyph;
    }
}

// text_combine_painter.cc:116-160. Combined text is treated as one U+FFFC-like
// glyph for emphasis positioning; Chromium shapes U+3042 to obtain that
// cluster's advance, then centers one emphasis glyph over it.
pub(super) fn ExportTextCombineEmphasis(
    style: &NativeComputedStyle,
) -> Option<TextCombineEmphasisPaintData> {
    use layoutng_style::style::computed_style_constants::LineLogicalSide;

    if style.GetTextEmphasisMark() == TextEmphasisMark::kNone {
        return None;
    }
    let mark_units = style.TextEmphasisMarkString().utf16_units()?;
    if mark_units.is_empty() {
        return None;
    }
    let font = unsafe { &*style.GetFont() };
    let primary = font.PrimaryFont();
    if primary.is_null() {
        return None;
    }

    let mark_text = BlinkString::from_utf16(&mark_units);
    let mark_shape =
        HarfBuzzShaper::new(mark_text).Shape(style.GetFont(), foundation::TextDirection::kLtr);
    let mark_view = ShapeResultView::CreateFromResult(mark_shape);
    if mark_view.is_null() {
        return None;
    }
    let mut glyph_font = FirstGlyphFont {
        font: std::ptr::null(),
        glyph: 0,
    };
    unsafe { &*mark_view }.ForEachGlyph(
        0.0,
        CaptureFirstGlyphFont,
        &mut glyph_font as *mut _ as *mut c_void,
    );
    if glyph_font.font.is_null() {
        return None;
    }

    let placeholder_text = BlinkString::from_utf16(&[0x3042]);
    let placeholder_shape = HarfBuzzShaper::new(placeholder_text)
        .Shape(style.GetFont(), foundation::TextDirection::kLtr);
    let placeholder_view = ShapeResultView::CreateFromResult(placeholder_shape);
    if placeholder_view.is_null() {
        return None;
    }

    let mut glyph_runs = Vec::new();
    let description = font.GetFontDescription();
    ExportShapeResult(
        mark_view,
        description.SpecifiedSize() as f64,
        0.0,
        WritingMode::kHorizontalTb,
        &mut glyph_runs,
    );
    let smoothing = match description.FontSmoothing() {
        FontSmoothingMode::kNoSmoothing => FontSmoothing::kNone,
        FontSmoothingMode::kAntialiased => FontSmoothing::kAntialiased,
        FontSmoothingMode::kSubpixelAntialiased => FontSmoothing::kSubpixelAntialiased,
        FontSmoothingMode::kAutoSmoothing => FontSmoothing::kAuto,
    };
    for run in &mut glyph_runs {
        run.font_smoothing = smoothing;
    }
    // FillTextEmphasisGlyphsNG centers the chosen emphasis glyph's platform
    // bounds, rather than the shaped mark run's aggregate ink bounds.
    let mark_bounds = unsafe { &*glyph_font.font }.BoundsForGlyph(glyph_font.glyph);
    let mut x = unsafe { &*placeholder_view }.Width() as f64 / 2.0
        - f64::from(mark_bounds.x() + mark_bounds.width() / 2.0);
    // TextEmphasisLetterSpacing is enabled in the Chromium revision this
    // translation targets; the runtime feature accessor has not otherwise
    // crossed the foundation boundary.
    x -= f64::from(description.LetterSpacing()) / 2.0;
    let base_metrics = unsafe { &*primary }.GetFontMetrics();
    let mark_metrics = unsafe { &*glyph_font.font }.GetFontMetrics();
    let emphasis_offset = if style.GetTextEmphasisLineLogicalSide() == LineLogicalSide::kOver {
        let offset = -base_metrics.FixedAscent(FontBaseline::kAlphabeticBaseline)
            - mark_metrics.FixedDescent(FontBaseline::kAlphabeticBaseline);
        offset.Floor() as f64
    } else {
        let offset = base_metrics.FixedDescent(FontBaseline::kAlphabeticBaseline)
            + mark_metrics.FixedAscent(FontBaseline::kAlphabeticBaseline);
        offset.Ceil() as f64
    };
    Some(TextCombineEmphasisPaintData {
        glyph_runs,
        origin: Offset {
            x,
            y: base_metrics.Ascent() as f64 + emphasis_offset,
        },
        color: crate::internal::boundary::native_paint_style::TextEmphasisColor(style),
    })
}

// text_fragment_painter.cc:520-525. Text-combine can reshape with a
// compressed Font that is owned by its anonymous LayoutTextCombine parent.
// Keep the paint origin's primary-font metrics tied to that same instance;
// the FragmentItem style font is only the fallback when compression was not
// selected.
fn TextCombineForItem(item: &FragmentItem) -> *mut LayoutTextCombine {
    let object = item.GetLayoutObject();
    if object.is_null() {
        return std::ptr::null_mut();
    }
    DynamicTo::<LayoutTextCombine>(unsafe { &*object }.Parent())
}

fn TextPaintFont(item: &FragmentItem) -> &Font {
    let text_combine = TextCombineForItem(item);
    if !text_combine.is_null() {
        let compressed = unsafe { &*text_combine }.CompressedFont();
        if !compressed.is_null() {
            return unsafe { &*compressed };
        }
    }
    item.ScaledFont()
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:953-981
fn TextPaintBaseline(item: &FragmentItem) -> f64 {
    let paint_font = TextPaintFont(item);
    let primary_font = paint_font.PrimaryFont();
    let text_combine = TextCombineForItem(item);
    if !text_combine.is_null() {
        // layout_text_combine.cc:178-186. Combined horizontal text paints in
        // a vertical line box. Its origin is based on the anonymous wrapper's
        // uncompressed font and half internal-leading adjustment, even when
        // shaping selected a compressed font instance.
        let wrapper_font_ptr = unsafe { &*text_combine }.StyleRef().GetFont();
        let wrapper_font = unsafe { &*wrapper_font_ptr };
        let wrapper_data = wrapper_font.PrimaryFont();
        if wrapper_data.is_null() {
            return 0.0;
        }
        let wrapper_data = unsafe { &*wrapper_data };
        return Number(LayoutUnit::from_f32(
            wrapper_data.GetFontMetrics().Ascent() as f32 - wrapper_data.InternalLeading() / 2.0,
        ));
    }
    if primary_font.is_null() {
        return 0.0;
    }
    let ascent = unsafe { &*primary_font }.GetFontMetrics().Ascent();
    // text_fragment_painter.cc:531-540. Normal text uses the integer
    // primary-font ascent. `text-fit` is the one HTML branch that scales
    // that ascent before constructing the LayoutUnit text origin.
    let text_fit_scale = if item.IsSvgText() {
        1.0
    } else {
        item.GetTextFitScale()
    };
    if text_fit_scale == 1.0 {
        ascent as f64
    } else {
        Number(LayoutUnit::from_f32(ascent as f32 * text_fit_scale))
    }
}

/// Build the immutable glyph payload at Paint recording time. Chromium keeps
/// ShapeResult beside FragmentItem and creates its text blob only when the
/// drawing item is actually recorded; Layout publication never serializes all
/// glyphs. This helper remains crate-private because it needs Layout's native
/// font resolver, while callers reach it only through NativePaintFragmentCursor.
#[inline(never)]
pub(crate) fn BuildPaintGlyphRuns(
    item: &FragmentItem,
    writing_mode: WritingMode,
) -> Arc<Vec<PaintGlyphRun>> {
    if !item.IsText() {
        return Arc::new(Vec::new());
    }
    let paint_font = TextPaintFont(item);
    let font_description = paint_font.GetFontDescription();
    let optical_size = font_description.SpecifiedSize() as f64;
    let mut glyph_runs = Vec::new();
    ExportShapeResult(
        item.TextShapeResult(),
        optical_size,
        TextPaintBaseline(item),
        writing_mode,
        &mut glyph_runs,
    );
    let smoothing = match item.Style().GetFontDescription().FontSmoothing() {
        FontSmoothingMode::kNoSmoothing => FontSmoothing::kNone,
        FontSmoothingMode::kAntialiased => FontSmoothing::kAntialiased,
        FontSmoothingMode::kSubpixelAntialiased => FontSmoothing::kSubpixelAntialiased,
        FontSmoothingMode::kAutoSmoothing => FontSmoothing::kAuto,
    };
    for run in &mut glyph_runs {
        run.font_smoothing = smoothing;
    }
    Arc::new(glyph_runs)
}

pub(crate) fn NativeTextPaintBaseline(item: &FragmentItem) -> Option<f64> {
    item.IsText().then(|| TextPaintBaseline(item))
}

pub(super) fn ExportTextCombineInlineOffset(item: &FragmentItem, paint: &mut PaintProperties) {
    let text_combine = TextCombineForItem(item);
    if text_combine.is_null() {
        return;
    }
    let text_combine_ref = unsafe { &*text_combine };
    if let Some(scale_x) = text_combine_ref.ScaleXForInline() {
        // layout_text_combine.cc:168-176. Spacing precedes the wrapper's scale
        // transform, hence the division by scale_x.
        paint.text_combine_inline_offset = Number(LayoutUnit::from_f32(
            text_combine_ref.ComputeInlineSpacing() / scale_x,
        ));
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:983-1001
pub(super) fn ExportListMarkerSymbol(item: &FragmentItem, paint: &mut PaintProperties) {
    if !item.IsSymbolMarker() {
        return;
    }
    let mut object = item.GetLayoutObject();
    while !object.is_null() {
        let layout_object = unsafe { &*object };
        if layout_object.IsLayoutInsideListMarker() {
            paint.list_marker_inside = true;
        }
        let node = layout_object.GetNode();
        if !node.is_null() && unsafe { &*node }.InputKind() == NodeKind::kListItem {
            let defaults = ExtendedStyle::default();
            let style = unsafe { &*node }.InputStyle();
            paint.list_marker_symbol = Some(
                style
                    .extended
                    .as_deref()
                    .unwrap_or(&defaults)
                    .list_style_type,
            );
            return;
        }
        object = layout_object.Parent();
    }
    panic!("symbol list marker has no list-item ancestor");
}
