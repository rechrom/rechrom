// C++: font_engine/fonts/shaping/harfbuzz_face.h.
use super::super::font_platform_data::FontPlatformData;
use crate::text::implementation::opentype_layout;
use foundation::{Member, Traceable, Visitor};
use harfrust::{Advances, FontFuncs, GlyphId, ShaperFont, Tag};
use read_fonts::TableProvider;

fn SkiaScalarToHarfBuzzPosition(value: f32) -> i32 {
    // cpp: skia/skia_text_metrics.cc:207-210. Rust's saturated float-to-int
    // cast truncates toward zero, matching ClampTo<int> for finite values.
    (value * 65536.0) as i32
}

/// Blink's HarfBuzzFace sub-font: horizontal advances come from the platform
/// strike, every other font function from the OpenType parent.
/// cpp: font_engine/fonts/shaping/harfbuzz_face.cc HarfBuzzGetGlyphHorizontalAdvance(s).
struct BackendAdvances {
    platform_data: *const FontPlatformData,
}

impl BackendAdvances {
    fn platform(&self) -> &FontPlatformData {
        // The HarfBuzzFace that owns these callbacks is owned by, and never
        // outlives, its FontPlatformData.
        unsafe { &*self.platform_data }
    }
}

impl FontFuncs for BackendAdvances {
    fn glyph_h_advance(&self, _font: &ShaperFont, glyph: GlyphId) -> i32 {
        let Ok(glyph) = u16::try_from(glyph.to_u32()) else {
            // kUnmatchedVSGlyphId is intentionally measured on the second pass.
            return 0;
        };
        let platform = self.platform();
        let Some(backend) = platform.Backend() else {
            return 0;
        };
        SkiaScalarToHarfBuzzPosition(backend.GlyphAdvanceForShaping(
            glyph,
            platform.size(),
            platform.SyntheticBold(),
            platform.SyntheticItalic(),
        ))
    }

    fn glyph_h_advances(&self, _font: &ShaperFont, advances: Advances<'_>) {
        let platform = self.platform();
        let Some(backend) = platform.Backend() else {
            for (_, advance) in advances {
                *advance = 0;
            }
            return;
        };
        let mut glyphs = Vec::with_capacity(advances.len());
        let mut outputs = Vec::with_capacity(advances.len());
        for (glyph, advance) in advances {
            glyphs.push(u16::try_from(glyph.to_u32()).unwrap_or(0));
            outputs.push(advance);
        }
        let mut values = vec![0.0; glyphs.len()];
        backend.GlyphAdvancesForShaping(
            &glyphs,
            platform.size(),
            platform.SyntheticBold(),
            platform.SyntheticItalic(),
            &mut values,
        );
        for (output, value) in outputs.into_iter().zip(values) {
            *output = SkiaScalarToHarfBuzzPosition(value);
        }
    }
}

// cpp: font_engine/fonts/shaping/harfbuzz_face.h:16-29
pub struct HarfBuzzFace {
    platform_data_: Member<FontPlatformData>,
    backend_advances_: Option<BackendAdvances>,
    space_in_gpos_: std::cell::Cell<Option<bool>>,
    space_in_gsub_: std::cell::Cell<Option<bool>>,
}

impl HarfBuzzFace {
    pub fn new(platform_data: &FontPlatformData) -> Self {
        let uses_backend_advances = platform_data.Backend().is_some() && {
            #[cfg(target_vendor = "apple")]
            {
                // Chromium keeps HarfBuzz advances for AAT trak unless
                // sbix requires the Skia strike path.
                !platform_data.RawFont().UsesOpenTypeHorizontalAdvances()
            }
            #[cfg(not(target_vendor = "apple"))]
            {
                true
            }
        };
        Self {
            platform_data_: Member::from_ptr(
                platform_data as *const FontPlatformData as *mut FontPlatformData,
            ),
            backend_advances_: uses_backend_advances.then_some(BackendAdvances {
                platform_data: platform_data as *const FontPlatformData,
            }),
            space_in_gpos_: std::cell::Cell::new(None),
            space_in_gsub_: std::cell::Cell::new(None),
        }
    }

    fn platform_data(&self) -> &FontPlatformData {
        let platform_data = self.platform_data_.Get();
        assert!(
            !platform_data.is_null(),
            "HarfBuzzFace has no platform data"
        );
        unsafe { &*platform_data }
    }

    /// The OpenType font this face scales.
    pub(crate) fn Font(&self) -> &crate::text::implementation::opentype_font::OpenTypeFont {
        self.platform_data().RawFont()
    }

    /// harfbuzz_face.cc:321-389. Cache on the same retained face/font data.
    pub fn HasSpaceInLigaturesOrKerning(&self, features: u32) -> bool {
        use crate::fonts::typesetting_features::{kKerning, kLigatures};
        let font = self.platform_data().RawFont();
        let Some(space) = font.GlyphForCharacter(0x20) else {
            return false;
        };
        let Ok(space) = u16::try_from(space) else {
            return false;
        };
        let tables = font.HarfRustFont().tables();
        let table_has_space = |tag: &[u8; 4]| {
            let data = tables.data_for_tag(Tag::new(tag));
            data.is_some_and(|data| {
                opentype_layout::LookupsReferenceGlyph(data.as_bytes(), tag == b"GPOS", space)
            })
        };
        if features & kKerning != 0 && self.space_in_gpos_.get().is_none() {
            self.space_in_gpos_.set(Some(table_has_space(b"GPOS")));
        }
        if features & kLigatures != 0 && self.space_in_gsub_.get().is_none() {
            self.space_in_gsub_.set(Some(table_has_space(b"GSUB")));
        }
        (features & kKerning != 0 && self.space_in_gpos_.get() == Some(true))
            || (features & kLigatures != 0 && self.space_in_gsub_.get() == Some(true))
    }

    /// The 16.16 scale of `GetScaledFont` (harfbuzz_face.cc): the platform
    /// strike size when its advances are used, otherwise the font's own.
    pub fn ScaledFontScale(&self) -> i32 {
        let platform_data = self.platform_data();
        if self.backend_advances_.is_some() {
            SkiaScalarToHarfBuzzPosition(platform_data.size())
        } else {
            platform_data.RawFont().Scale()
        }
    }

    /// The font-functions override of `GetScaledFont`, if any.
    pub fn ScaledFontFuncs(&self) -> Option<&dyn FontFuncs> {
        self.backend_advances_
            .as_ref()
            .map(|funcs| funcs as &dyn FontFuncs)
    }
}

impl Traceable for HarfBuzzFace {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.platform_data_);
    }
}
