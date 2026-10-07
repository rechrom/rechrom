// C++: font_engine/fonts/simple_font_data.h/.cc. This is a partial mapping.
// The shape cache's key and lookup algorithms, HanKerning, scaled-font
// creation, and the FontData virtual boundary remain partial.
use super::canvas_rotation_in_vertical::CanvasRotationInVertical;
use super::custom_font_data::CustomFontData;
use super::font_baseline::FontBaseline;
use super::font_data::FontData;
use super::font_height::FontHeight;
use super::font_metrics::FontMetrics;
use super::font_metrics_override::FontMetricsOverride;
use super::font_orientation::FontOrientation;
use super::font_platform_data::FontPlatformData;
use super::font_vertical_position_type::FontVerticalPositionType;
use super::glyph::Glyph;
use super::glyph_metrics_map::GlyphMetricsMap;
use super::shaping::han_kerning::FontData as HanKerningFontData;
use super::shaping::ng_shape_cache::NGShapeCache;
use crate::text::native::layout_locale::LayoutLocale;
use crate::text::native::opentype_font::{FontMetric, OpenTypeFont};
use foundation::gfx::{PointF, SizeF};
use foundation::{
    LayoutUnit, MakeGarbageCollected, Member, RectF, TextDirection, Traceable, Visitor,
};
use std::cell::RefCell;
use std::sync::OnceLock;

// cpp: font_engine/fonts/simple_font_data.h:55-68
#[derive(Clone, Copy, Debug)]
pub struct GlyphData {
    pub glyph: Glyph,
    pub font_data: *const SimpleFontData,
    pub canvas_rotation: CanvasRotationInVertical,
}

impl Default for GlyphData {
    fn default() -> Self {
        Self {
            glyph: 0,
            font_data: std::ptr::null(),
            canvas_rotation: CanvasRotationInVertical::kRegular,
        }
    }
}

impl GlyphData {
    pub fn new(
        glyph: Glyph,
        font_data: *const SimpleFontData,
        canvas_rotation: CanvasRotationInVertical,
    ) -> Self {
        Self {
            glyph,
            font_data,
            canvas_rotation,
        }
    }
}

// cpp: font_engine/fonts/simple_font_data.h:217-224
#[derive(Clone)]
struct HanKerningCacheEntry {
    locale: *const LayoutLocale,
    is_horizontal: bool,
    data: HanKerningFontData,
}

// cpp: font_engine/fonts/simple_font_data.h:74-238
pub struct SimpleFontData {
    font_metrics_: FontMetrics,
    max_char_width_: f32,
    avg_char_width_: f32,
    platform_data_: Member<FontPlatformData>,
    shape_cache_: Member<NGShapeCache>,
    space_glyph_: Glyph,
    space_width_: f32,
    zero_glyph_: Glyph,
    small_caps_: Member<SimpleFontData>,
    emphasis_mark_: Member<SimpleFontData>,
    custom_font_data_: Member<dyn CustomFontData>,
    ideographic_inline_size_: OnceLock<Option<f32>>,
    ideographic_advance_width_: OnceLock<Option<f32>>,
    ideographic_advance_height_: OnceLock<Option<f32>>,
    normalized_typo_ascent_descent_: RefCell<FontHeight>,
    han_kerning_cache_: RefCell<[Option<HanKerningCacheEntry>; 2]>,
    #[cfg(target_vendor = "apple")]
    glyph_to_bounds_map_: RefCell<Option<GlyphMetricsMap<RectF>>>,
}

// cpp: font_engine/fonts/simple_font_data.cc:30-54
fn PositiveMetric(font: &OpenTypeFont, metric: FontMetric, fallback: f32) -> f32 {
    font.Metric(metric)
        .map_or(fallback, |value| (value as f32).abs())
}

fn TypoAscenderAndDescender(font: &OpenTypeFont) -> Option<(i16, i16)> {
    let os2 = font.CopyTable(0x4f53_2f32);
    if os2.len() < 72 {
        return None;
    }
    let read_i16 = |offset: usize| i16::from_be_bytes([os2[offset], os2[offset + 1]]);
    Some((read_i16(68), read_i16(70).wrapping_neg()))
}

impl SimpleFontData {
    // cpp: font_engine/fonts/simple_font_data.cc:52-62
    // Allocate first so the cache can keep the stable GC address of `this`.
    pub fn Create(platform_data: *const FontPlatformData) -> *mut Self {
        assert!(!platform_data.is_null());
        let data = Self {
            font_metrics_: FontMetrics::default(),
            max_char_width_: 0.0,
            avg_char_width_: 0.0,
            platform_data_: Member::from_ptr(platform_data.cast_mut()),
            shape_cache_: Member::default(),
            space_glyph_: 0,
            space_width_: 0.0,
            zero_glyph_: 0,
            small_caps_: Member::default(),
            emphasis_mark_: Member::default(),
            custom_font_data_: Member::default(),
            ideographic_inline_size_: OnceLock::new(),
            ideographic_advance_width_: OnceLock::new(),
            ideographic_advance_height_: OnceLock::new(),
            normalized_typo_ascent_descent_: RefCell::new(FontHeight::default()),
            han_kerning_cache_: RefCell::new([None, None]),
            #[cfg(target_vendor = "apple")]
            glyph_to_bounds_map_: RefCell::new(None),
        };
        let ptr = MakeGarbageCollected(data);
        let cache = MakeGarbageCollected(NGShapeCache::new(ptr));
        let data = unsafe { &mut *ptr };
        data.shape_cache_ = Member::from_ptr(cache);
        data.PlatformInit(false, &FontMetricsOverride::default());
        data.PlatformGlyphInit();
        ptr
    }

    // cpp: font_engine/fonts/simple_font_data.h:93
    pub fn GetShapeCache(&self) -> &NGShapeCache {
        unsafe { &*self.shape_cache_.Get() }
    }

    // The inline package calls the mutable NGShapeCache through a const
    // SimpleFontData, as in the C++ GetShapeCache() API. Keep the GC-owned
    // cache's pointer in its source owner; the caller must serialize access.
    pub fn GetShapeCachePtrForInline(&self) -> *mut NGShapeCache {
        self.shape_cache_.Get()
    }

    // cpp: font_engine/fonts/simple_font_data.h:92-100,122-137,157-175
    pub fn PlatformData(&self) -> &FontPlatformData {
        let platform_data = self.platform_data_.Get();
        assert!(
            !platform_data.is_null(),
            "SimpleFontData has no platform data"
        );
        unsafe { &*platform_data }
    }

    pub fn GetFontMetrics(&self) -> &FontMetrics {
        &self.font_metrics_
    }
    pub fn GetFontMetricsMut(&mut self) -> &mut FontMetrics {
        &mut self.font_metrics_
    }
    pub fn InternalLeading(&self) -> f32 {
        self.font_metrics_.FloatHeight() - self.PlatformData().size()
    }
    pub fn MaxCharWidth(&self) -> f32 {
        self.max_char_width_
    }
    pub fn SetMaxCharWidth(&mut self, value: f32) {
        self.max_char_width_ = value;
    }
    pub fn AvgCharWidth(&self) -> f32 {
        self.avg_char_width_
    }
    pub fn SetAvgCharWidth(&mut self, value: f32) {
        self.avg_char_width_ = value;
    }
    pub fn SpaceWidth(&self) -> f32 {
        self.space_width_
    }
    pub fn SetSpaceWidth(&mut self, value: f32) {
        self.space_width_ = value;
    }
    pub fn SpaceGlyph(&self) -> Glyph {
        self.space_glyph_
    }
    pub fn SetSpaceGlyph(&mut self, value: Glyph) {
        self.space_glyph_ = value;
    }
    pub fn ZeroGlyph(&self) -> Glyph {
        self.zero_glyph_
    }
    pub fn SetZeroGlyph(&mut self, value: Glyph) {
        self.zero_glyph_ = value;
    }

    // cpp: font_engine/fonts/simple_font_data.cc:75-169
    fn PlatformInit(&mut self, subpixel_ascent_descent: bool, overrides: &FontMetricsOverride) {
        // The GC address is stable for the owning heap scope; this borrow is
        // separate from the mutable metric fields of SimpleFontData.
        let platform_ptr = self.platform_data_.Get();
        assert!(
            !platform_ptr.is_null(),
            "SimpleFontData has no platform data"
        );
        let platform = unsafe { &*platform_ptr };
        let raw = platform.RawFont();
        let size = platform.size();
        if size == 0.0 {
            self.font_metrics_.Reset();
            self.avg_char_width_ = 0.0;
            self.max_char_width_ = 0.0;
            return;
        }
        let mut ascent = overrides.ascent_override.map_or_else(
            || PositiveMetric(raw, FontMetric::kHorizontalAscender, size * 0.8),
            |value| value * size,
        );
        let mut descent = overrides.descent_override.map_or_else(
            || PositiveMetric(raw, FontMetric::kHorizontalDescender, size * 0.2),
            |value| value * size,
        );
        let line_gap = overrides.line_gap_override.map_or_else(
            || PositiveMetric(raw, FontMetric::kHorizontalLineGap, 0.0),
            |value| value * size,
        );
        let backend_metrics = platform.Backend().map(|backend| {
            backend.Metrics(size, platform.SyntheticBold(), platform.SyntheticItalic())
        });
        if let Some(metrics) = backend_metrics {
            if overrides.ascent_override.is_none() {
                ascent = metrics.ascent;
            }
            if overrides.descent_override.is_none() {
                descent = metrics.descent;
            }
        }
        if !(subpixel_ascent_descent && (ascent < 3.0 || ascent + descent < 2.0)) {
            ascent = ascent.round();
            descent = descent.round();
        }
        #[cfg(target_vendor = "apple")]
        {
            let family = raw.FamilyName();
            if family == "Times" || family == "Helvetica" || family == "Courier" {
                ascent += ((ascent + descent) * 0.15 + 0.5).floor();
            }
        }
        self.font_metrics_.SetAscent(ascent);
        self.font_metrics_.SetDescent(descent);
        let resolved_line_gap = if overrides.line_gap_override.is_some() {
            line_gap
        } else {
            backend_metrics.map_or(line_gap, |metrics| metrics.leading)
        };
        self.font_metrics_.SetLineGap(resolved_line_gap);
        self.font_metrics_
            .SetLineSpacing(ascent.round() + descent.round() + resolved_line_gap.round());
        let mut x_height = backend_metrics
            .filter(|metrics| metrics.x_height != 0.0)
            .map_or_else(
                || PositiveMetric(raw, FontMetric::kXHeight, ascent * 0.56),
                |metrics| metrics.x_height,
            );
        if backend_metrics.is_some() {
            let x_glyph = self.GlyphForCharacter('x' as u32);
            if x_glyph != 0 {
                x_height = -self.PlatformBoundsForGlyph(x_glyph).y();
            }
        }
        self.font_metrics_.SetXHeight(x_height);
        let cap_height = backend_metrics
            .filter(|metrics| metrics.cap_height != 0.0)
            .map_or_else(
                || PositiveMetric(raw, FontMetric::kCapHeight, ascent),
                |metrics| metrics.cap_height,
            );
        self.font_metrics_.SetCapHeight(cap_height);
        if let Some(metrics) = backend_metrics.filter(|metrics| metrics.has_underline_position) {
            self.font_metrics_
                .SetUnderlinePosition(metrics.underline_position);
        } else if let Some(value) = raw.Metric(FontMetric::kUnderlineOffset) {
            self.font_metrics_.SetUnderlinePosition(-(value as f32));
        }
        if let Some(metrics) = backend_metrics.filter(|metrics| metrics.has_underline_thickness) {
            self.font_metrics_
                .SetUnderlineThickness(metrics.underline_thickness);
        } else if let Some(value) = raw.Metric(FontMetric::kUnderlineSize) {
            self.font_metrics_
                .SetUnderlineThickness((value as f32).abs());
        }
        self.avg_char_width_ = x_height;
        let x_glyph = self.GlyphForCharacter('x' as u32);
        if x_glyph != 0 {
            self.avg_char_width_ = self.WidthForGlyph(x_glyph);
        }
        self.max_char_width_ = self.avg_char_width_.max(ascent);
    }

    // cpp: font_engine/fonts/simple_font_data.cc:171-195
    fn PlatformGlyphInit(&mut self) {
        if self.PlatformData().RawFont().GlyphCount() == 0 {
            return;
        }
        self.space_glyph_ = self.GlyphForCharacter(' ' as u32);
        self.space_width_ = self.WidthForGlyph(self.space_glyph_);
        self.zero_glyph_ = self.GlyphForCharacter('0' as u32);
        let zero_width = self.ZeroInlineSize();
        self.font_metrics_.SetZeroWidth(zero_width);
    }

    pub fn FontDataForCharacter(&self, _character: u32) -> *const Self {
        self
    }

    pub fn GlyphForCharacter(&self, character: u32) -> Glyph {
        let glyph = self.PlatformData().RawFont().GlyphForCharacter(character);
        glyph
            .filter(|glyph| *glyph <= Glyph::MAX as u32)
            .map_or(0, |glyph| glyph as Glyph)
    }

    pub fn GlyphForMathCharacter(&self, character: u32, _direction: TextDirection) -> Glyph {
        self.GlyphForCharacter(character)
    }

    pub fn IsSegmented(&self) -> bool {
        false
    }

    // cpp: font_engine/fonts/simple_font_data.h:179-199
    pub fn IsCustomFont(&self) -> bool {
        self.custom_font_data_.GetNonNull().is_some()
    }
    pub fn IsLoading(&self) -> bool {
        self.CustomData().is_some_and(CustomFontData::IsLoading)
    }
    pub fn IsLoadingFallback(&self) -> bool {
        self.CustomData()
            .is_some_and(CustomFontData::IsLoadingFallback)
    }
    pub fn IsPendingDataUrlCustomFont(&self) -> bool {
        self.CustomData()
            .is_some_and(CustomFontData::IsPendingDataUrl)
    }
    pub fn ShouldSkipDrawing(&self) -> bool {
        self.CustomData()
            .is_some_and(CustomFontData::ShouldSkipDrawing)
    }
    pub fn GetCustomFontData(&self) -> Option<&dyn CustomFontData> {
        self.CustomData()
    }

    fn CustomData(&self) -> Option<&dyn CustomFontData> {
        self.custom_font_data_
            .GetNonNull()
            .map(|pointer| unsafe { pointer.as_ref() })
    }

    // cpp: font_engine/fonts/simple_font_data.cc:224-263
    fn TrySetNormalizedTypoAscentAndDescent(&self, ascent: f32, descent: f32) -> bool {
        let height = ascent + descent;
        if height <= 0.0 || ascent < 0.0 || ascent > height {
            return false;
        }
        let normalized_ascent =
            LayoutUnit::FromFloatRound(ascent * self.PlatformData().size() / height);
        *self.normalized_typo_ascent_descent_.borrow_mut() = FontHeight::new(
            normalized_ascent,
            LayoutUnit::FromFloatRound(self.PlatformData().size()) - normalized_ascent,
        );
        true
    }

    fn ComputeNormalizedTypoAscentAndDescent(&self) {
        if let Some((ascent, descent)) = TypoAscenderAndDescender(self.PlatformData().RawFont()) {
            if ascent > 0
                && self.TrySetNormalizedTypoAscentAndDescent(ascent as f32, descent as f32)
            {
                return;
            }
        }
        self.TrySetNormalizedTypoAscentAndDescent(
            self.font_metrics_.FloatAscent(),
            self.font_metrics_.FloatDescent(),
        );
    }

    pub fn NormalizedTypoAscentAndDescent(&self, baseline: FontBaseline) -> FontHeight {
        if baseline == FontBaseline::kAlphabeticBaseline {
            if self.normalized_typo_ascent_descent_.borrow().ascent == LayoutUnit::from_signed(0) {
                self.ComputeNormalizedTypoAscentAndDescent();
            }
            return *self.normalized_typo_ascent_descent_.borrow();
        }
        let height = LayoutUnit::FromFloatRound(self.PlatformData().size());
        FontHeight::new(height - height / 2, height / 2)
    }

    pub fn NormalizedTypoAscent(&self, baseline: FontBaseline) -> LayoutUnit {
        self.NormalizedTypoAscentAndDescent(baseline).ascent
    }
    pub fn NormalizedTypoDescent(&self, baseline: FontBaseline) -> LayoutUnit {
        self.NormalizedTypoAscentAndDescent(baseline).descent
    }

    // cpp: font_engine/fonts/simple_font_data.cc:265-279
    pub fn VerticalPosition(
        &self,
        position_type: FontVerticalPositionType,
        baseline: FontBaseline,
    ) -> LayoutUnit {
        match position_type {
            FontVerticalPositionType::TextTop => {
                LayoutUnit::from_signed(self.font_metrics_.AscentFor(baseline))
            }
            FontVerticalPositionType::TextBottom => {
                LayoutUnit::from_signed(-self.font_metrics_.DescentFor(baseline))
            }
            FontVerticalPositionType::TopOfEmHeight => self.NormalizedTypoAscent(baseline),
            FontVerticalPositionType::BottomOfEmHeight => -self.NormalizedTypoDescent(baseline),
        }
    }

    // cpp: font_engine/fonts/simple_font_data.cc:281-310
    pub fn IdeographicAdvanceWidth(&self) -> &Option<f32> {
        self.ideographic_advance_width_.get_or_init(|| {
            let glyph = self.GlyphForCharacter('水' as u32);
            (glyph != 0).then(|| self.WidthForGlyph(glyph))
        })
    }
    pub fn IdeographicAdvanceHeight(&self) -> &Option<f32> {
        self.ideographic_advance_height_.get_or_init(|| {
            let glyph = self.GlyphForCharacter('水' as u32);
            (glyph != 0).then(|| {
                self.PlatformData()
                    .RawFont()
                    .VerticalAdvance(glyph as u32)
                    .abs() as f32
            })
        })
    }
    pub fn IdeographicInlineSize(&self) -> &Option<f32> {
        self.ideographic_inline_size_.get_or_init(|| {
            if self.PlatformData().Orientation() == FontOrientation::kVerticalUpright {
                *self.IdeographicAdvanceHeight()
            } else {
                *self.IdeographicAdvanceWidth()
            }
        })
    }
    pub fn TextAutoSpaceInlineSize(&self) -> f32 {
        self.IdeographicInlineSize()
            .unwrap_or(self.PlatformData().size())
            / 8.0
    }

    // C++ returns a const reference to one of two mutable cache slots. The
    // small immutable record is cloned in Rust so RefCell's borrow does not
    // escape while later calls may shift the cache entries.
    // cpp: font_engine/fonts/simple_font_data.cc:312-326
    pub fn HanKerningData(&self, locale: &LayoutLocale, is_horizontal: bool) -> HanKerningFontData {
        let locale_ptr = locale as *const LayoutLocale;
        {
            let cache = self.han_kerning_cache_.borrow();
            for entry in cache.iter().flatten() {
                if entry.locale == locale_ptr && entry.is_horizontal == is_horizontal {
                    return entry.data.clone();
                }
            }
        }
        let data = HanKerningFontData::new(self, locale, is_horizontal);
        let mut cache = self.han_kerning_cache_.borrow_mut();
        cache[1] = cache[0].take();
        cache[0] = Some(HanKerningCacheEntry {
            locale: locale_ptr,
            is_horizontal,
            data: data.clone(),
        });
        data
    }

    // cpp: font_engine/fonts/simple_font_data.cc:328-390
    pub fn PlatformBoundsForGlyph(&self, glyph: Glyph) -> RectF {
        let platform = self.PlatformData();
        if let Some(backend) = platform.Backend() {
            let metrics = backend.GlyphMetrics(
                glyph,
                platform.size(),
                platform.SyntheticBold(),
                platform.SyntheticItalic(),
            );
            return rect(metrics.x, metrics.y, metrics.width, metrics.height);
        }
        let Some(extents) = platform.RawFont().Extents(glyph as u32) else {
            return RectF::default();
        };
        let mut x = extents.x_bearing as f32;
        let mut y = -extents.y_bearing as f32;
        let mut width = extents.width as f32;
        let mut height = -extents.height as f32;
        if platform.SyntheticItalic() {
            let top_x = -0.25 * y;
            let bottom_x = -0.25 * (y + height);
            x += top_x.min(bottom_x);
            width += (top_x - bottom_x).abs();
        }
        if platform.SyntheticBold() {
            let size = platform.size();
            let scale = if size <= 9.0 {
                1.0 / 24.0
            } else if size >= 36.0 {
                1.0 / 32.0
            } else {
                1.0 / 24.0 + (size - 9.0) / 27.0 * (1.0 / 32.0 - 1.0 / 24.0)
            };
            let extra = size * scale;
            x -= extra * 0.5;
            y -= extra * 0.5;
            width += extra;
            height += extra;
        }
        rect(x, y, width, height)
    }

    pub fn PreciseBoundsForGlyph(&self, glyph: Glyph) -> RectF {
        self.PlatformBoundsForGlyph(glyph)
    }
    pub fn BoundsForGlyphs(&self, glyphs: &[Glyph], bounds: &mut Vec<RectF>) {
        bounds.resize(glyphs.len(), RectF::default());
        for (index, glyph) in glyphs.iter().enumerate() {
            bounds[index] = self.PlatformBoundsForGlyph(*glyph);
        }
    }
    pub fn WidthForGlyph(&self, glyph: Glyph) -> f32 {
        if let Some(backend) = self.PlatformData().Backend() {
            return backend
                .GlyphMetrics(
                    glyph,
                    self.PlatformData().size(),
                    self.PlatformData().SyntheticBold(),
                    self.PlatformData().SyntheticItalic(),
                )
                .advance;
        }
        self.PlatformData()
            .RawFont()
            .HorizontalAdvance(glyph as u32) as f32
    }

    // cpp: font_engine/fonts/simple_font_data.cc:392-402
    pub fn ZeroInlineSize(&self) -> f32 {
        if self.zero_glyph_ != 0 {
            return if self.PlatformData().Orientation() == FontOrientation::kVerticalUpright {
                self.PlatformData()
                    .RawFont()
                    .VerticalAdvance(self.zero_glyph_ as u32)
                    .abs() as f32
            } else {
                self.WidthForGlyph(self.zero_glyph_)
            };
        }
        if self.PlatformData().Orientation() == FontOrientation::kVerticalUpright {
            self.PlatformData().size()
        } else {
            self.PlatformData().size() * 0.5
        }
    }

    // cpp: font_engine/fonts/simple_font_data.h:241-259
    pub fn BoundsForGlyph(&self, glyph: Glyph) -> RectF {
        #[cfg(not(target_vendor = "apple"))]
        {
            self.PlatformBoundsForGlyph(glyph)
        }
        #[cfg(target_vendor = "apple")]
        {
            let mut cache = self.glyph_to_bounds_map_.borrow_mut();
            if let Some(map) = cache.as_mut() {
                if let Some(bounds) = map.MetricsForGlyph(glyph) {
                    return bounds;
                }
            }
            let bounds = self.PlatformBoundsForGlyph(glyph);
            cache
                .get_or_insert_with(GlyphMetricsMap::new)
                .SetMetricsForGlyph(glyph, bounds);
            bounds
        }
    }
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> RectF {
    RectF::new(PointF::new(x, y), SizeF::new(width, height))
}

impl Traceable for SimpleFontData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.platform_data_);
        visitor.Trace(&self.shape_cache_);
        visitor.Trace(&self.small_caps_);
        visitor.Trace(&self.emphasis_mark_);
        visitor.Trace(&self.custom_font_data_);
    }
}

// cpp: font_engine/fonts/simple_font_data.h:157-194
impl FontData for SimpleFontData {
    fn FontDataForCharacter(&self, character: u32) -> *const SimpleFontData {
        SimpleFontData::FontDataForCharacter(self, character)
    }
    fn IsCustomFont(&self) -> bool {
        SimpleFontData::IsCustomFont(self)
    }
    fn IsLoading(&self) -> bool {
        SimpleFontData::IsLoading(self)
    }
    fn IsLoadingFallback(&self) -> bool {
        SimpleFontData::IsLoadingFallback(self)
    }
    fn IsSegmented(&self) -> bool {
        SimpleFontData::IsSegmented(self)
    }
    fn ShouldSkipDrawing(&self) -> bool {
        SimpleFontData::ShouldSkipDrawing(self)
    }
}
