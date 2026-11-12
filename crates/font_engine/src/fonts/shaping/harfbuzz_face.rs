// C++: font_engine/fonts/shaping/harfbuzz_face.h.
use super::super::font_platform_data::FontPlatformData;
use crate::text::implementation::harfbuzz as hb;
use crate::text::implementation::opentype_font::HarfBuzzFont;
use foundation::{Member, Traceable, Visitor};
use std::ffi::c_void;

struct BackendScaledFont {
    font: *mut HarfBuzzFont,
    funcs: *mut hb::HbFontFuncs,
}

impl Drop for BackendScaledFont {
    fn drop(&mut self) {
        unsafe {
            // hb_font_t retains its function table, so release it first.
            hb::hb_font_destroy(self.font);
            hb::hb_font_funcs_destroy(self.funcs);
        }
    }
}

fn SkiaScalarToHarfBuzzPosition(value: f32) -> i32 {
    // cpp: skia/skia_text_metrics.cc:207-210. Rust's saturated float-to-int
    // cast truncates toward zero, matching ClampTo<int> for finite values.
    (value * 65536.0) as i32
}

unsafe fn BackendGlyphAdvance(font_data: *mut c_void, glyph: u32) -> i32 {
    let Ok(glyph) = u16::try_from(glyph) else {
        // kUnmatchedVSGlyphId is intentionally measured on the second pass.
        return 0;
    };
    let platform = unsafe { &*font_data.cast::<FontPlatformData>() };
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

unsafe extern "C" fn HarfBuzzGetGlyphHorizontalAdvance(
    _font: *mut HarfBuzzFont,
    font_data: *mut c_void,
    glyph: u32,
    _user_data: *mut c_void,
) -> i32 {
    if font_data.is_null() {
        return 0;
    }
    unsafe { BackendGlyphAdvance(font_data, glyph) }
}

unsafe extern "C" fn HarfBuzzGetGlyphHorizontalAdvances(
    _font: *mut HarfBuzzFont,
    font_data: *mut c_void,
    count: u32,
    first_glyph: *const u32,
    glyph_stride: u32,
    first_advance: *mut i32,
    advance_stride: u32,
    _user_data: *mut c_void,
) {
    if font_data.is_null() || first_glyph.is_null() || first_advance.is_null() {
        return;
    }
    let platform = unsafe { &*font_data.cast::<FontPlatformData>() };
    let Some(backend) = platform.Backend() else {
        for index in 0..count as usize {
            unsafe {
                first_advance
                    .cast::<u8>()
                    .add(index * advance_stride as usize)
                    .cast::<i32>()
                    .write_unaligned(0);
            }
        }
        return;
    };
    let glyphs: Vec<u16> = (0..count as usize)
        .map(|index| unsafe {
            first_glyph
                .cast::<u8>()
                .add(index * glyph_stride as usize)
                .cast::<u32>()
                .read_unaligned()
        })
        .map(|glyph| u16::try_from(glyph).unwrap_or(0))
        .collect();
    let mut advances = vec![0.0; glyphs.len()];
    backend.GlyphAdvancesForShaping(
        &glyphs,
        platform.size(),
        platform.SyntheticBold(),
        platform.SyntheticItalic(),
        &mut advances,
    );
    for (index, advance) in advances.into_iter().enumerate() {
        unsafe {
            first_advance
                .cast::<u8>()
                .add(index * advance_stride as usize)
                .cast::<i32>()
                .write_unaligned(SkiaScalarToHarfBuzzPosition(advance));
        }
    }
}

// cpp: font_engine/fonts/shaping/harfbuzz_face.h:16-29
pub struct HarfBuzzFace {
    platform_data_: Member<FontPlatformData>,
    backend_scaled_font_: Option<BackendScaledFont>,
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
        let backend_scaled_font_ = uses_backend_advances.then(|| unsafe {
            let font = hb::hb_font_create_sub_font(platform_data.RawFont().BorrowHarfBuzzFont());
            assert!(!font.is_null(), "HarfBuzz sub-font allocation failed");
            let funcs = hb::hb_font_funcs_create();
            assert!(
                !funcs.is_null(),
                "HarfBuzz font-functions allocation failed"
            );
            hb::hb_font_funcs_set_glyph_h_advance_func(
                funcs,
                Some(HarfBuzzGetGlyphHorizontalAdvance),
                std::ptr::null_mut(),
                None,
            );
            hb::hb_font_funcs_set_glyph_h_advances_func(
                funcs,
                Some(HarfBuzzGetGlyphHorizontalAdvances),
                std::ptr::null_mut(),
                None,
            );
            hb::hb_font_funcs_make_immutable(funcs);
            hb::hb_font_set_funcs(
                font,
                funcs,
                (platform_data as *const FontPlatformData).cast_mut().cast(),
                None,
            );
            BackendScaledFont { font, funcs }
        });
        Self {
            platform_data_: Member::from_ptr(
                platform_data as *const FontPlatformData as *mut FontPlatformData,
            ),
            backend_scaled_font_,
            space_in_gpos_: std::cell::Cell::new(None),
            space_in_gsub_: std::cell::Cell::new(None),
        }
    }

    /// harfbuzz_face.cc:321-389. Cache on the same retained face/font data.
    pub fn HasSpaceInLigaturesOrKerning(&self, features: u32) -> bool {
        use crate::fonts::typesetting_features::{kKerning, kLigatures};
        unsafe {
            let font = (&*self.platform_data_.Get()).RawFont().BorrowHarfBuzzFont();
            let mut space = u32::MAX;
            let glyphs = hb::hb_set_create();
            assert!(!glyphs.is_null(), "HarfBuzz set allocation");
            struct Set(*mut c_void);
            impl Drop for Set {
                fn drop(&mut self) {
                    unsafe { hb::hb_set_destroy(self.0) }
                }
            }
            let _set = Set(glyphs);
            let mut table_has_space = |tag: u32, has: bool| {
                if space == u32::MAX && hb::hb_font_get_nominal_glyph(font, 0x20, &mut space) == 0 {
                    return None;
                }
                if !has {
                    return Some(false);
                }
                let face = hb::hb_font_get_face(font);
                for i in 0..hb::hb_ot_layout_table_get_lookup_count(face, tag) {
                    hb::hb_ot_layout_lookup_collect_glyphs(
                        face,
                        tag,
                        i,
                        glyphs,
                        glyphs,
                        glyphs,
                        std::ptr::null_mut(),
                    );
                    if hb::hb_set_has(glyphs, space) != 0 {
                        return Some(true);
                    }
                }
                Some(false)
            };
            let face = hb::hb_font_get_face(font);
            if features & kKerning != 0 && self.space_in_gpos_.get().is_none() {
                let Some(value) = table_has_space(
                    u32::from_be_bytes(*b"GPOS"),
                    hb::hb_ot_layout_has_positioning(face) != 0,
                ) else {
                    return false;
                };
                self.space_in_gpos_.set(Some(value));
            }
            hb::hb_set_clear(glyphs);
            if features & kLigatures != 0 && self.space_in_gsub_.get().is_none() {
                let Some(value) = table_has_space(
                    u32::from_be_bytes(*b"GSUB"),
                    hb::hb_ot_layout_has_substitution(face) != 0,
                ) else {
                    return false;
                };
                self.space_in_gsub_.set(Some(value));
            }
            (features & kKerning != 0 && self.space_in_gpos_.get() == Some(true))
                || (features & kLigatures != 0 && self.space_in_gsub_.get() == Some(true))
        }
    }
    pub fn GetScaledFont(&self) -> *mut HarfBuzzFont {
        let platform_data = self.platform_data_.Get();
        assert!(
            !platform_data.is_null(),
            "HarfBuzzFace has no platform data"
        );
        let platform_data = unsafe { &*platform_data };
        if let Some(font) = &self.backend_scaled_font_ {
            let scale = SkiaScalarToHarfBuzzPosition(platform_data.size());
            unsafe {
                hb::hb_font_set_scale(font.font, scale, scale);
                hb::hb_font_set_ptem(font.font, platform_data.RawFont().SpecifiedSize() as f32);
            }
            font.font
        } else {
            platform_data.RawFont().BorrowHarfBuzzFont()
        }
    }
}

impl Traceable for HarfBuzzFace {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.platform_data_);
    }
}
