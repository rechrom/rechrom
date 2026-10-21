// Source: src/core/SkFontPriv.h:20-21
pub struct SkFontPriv;
impl SkFontPriv {
    // Source: src/core/SkFontPriv.h:36-36
    pub const kCanonicalTextSizeForPaths: i32 = 64;
}
impl SkFontPriv {
    // Source: src/core/SkFontPriv.h:41-47
    pub fn MakeTextMatrix(
        size: f32,
        scale_x: f32,
        skew_x: f32,
    ) -> crate::src::core::SkMatrix::SkMatrix {
        let mut m = crate::src::core::SkMatrix::SkMatrix::Scale(size * scale_x, size);
        if skew_x != 0. {
            m.postSkew(skew_x, 0.);
        }
        m
    }
}
impl SkFontPriv {
    // Source: src/core/SkFontPriv.h:49-51
    pub fn MakeTextMatrixForFont(
        font: &crate::src::core::SkFont::SkFont,
    ) -> crate::src::core::SkMatrix::SkMatrix {
        Self::MakeTextMatrix(font.getSize(), font.getScaleX(), font.getSkewX())
    }
}
impl SkFontPriv {
    // Source: src/core/SkFontPriv.h:78-80
    pub fn IsFinite(font: &crate::src::core::SkFont::SkFont) -> bool {
        crate::src::core::SkFloatingPoint::SkIsFinite(&[
            font.getSize(),
            font.getScaleX(),
            font.getSkewX(),
        ])
    }
}
impl SkFontPriv {
    // Source: src/core/SkFontPriv.h:90-91
    pub fn Flags(font: &crate::src::core::SkFont::SkFont) -> u8 {
        font.fFlags
    }
}
// Native stack/heap scratch storage is represented without a self-referential span.
// Initialized u16 scratch replaces native unspecified bytes before glyph mapping.
// Adapted source: src/core/SkFontPriv.h:114-115: combined borrowed/inline/heap storage enum replaces two native members; field representation and initialized scratch differ
enum GlyphStorage<'a> {
    Borrowed(*const u16, usize, std::marker::PhantomData<&'a [u16]>),
    Inline([u16; 32], usize),
    Heap(Vec<u16>),
}
// Source: src/core/SkFontPriv.h:93-94
#[must_use]
pub struct SkAutoToGlyphs<'a> {
    storage: GlyphStorage<'a>,
}
impl<'a> SkAutoToGlyphs<'a> {
    /// The text allocation must remain readable for 'a, and glyph-ID text must
    /// be aligned for u16. UTF16/UTF32 retain their native alignment requirements.
    // Source: src/core/SkFontPriv.h:95-105
    pub unsafe fn new(
        font: &crate::src::core::SkFont::SkFont,
        text: *const std::ffi::c_void,
        length: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> Self {
        if encoding == crate::include::core::SkFontTypes::SkTextEncoding::kGlyphID || length == 0 {
            Self {
                storage: GlyphStorage::Borrowed(text.cast(), length >> 1, std::marker::PhantomData),
            }
        } else {
            let count = font.countText(text, length, encoding);
            let mut storage = if count <= 32 {
                GlyphStorage::Inline([0; 32], count)
            } else {
                GlyphStorage::Heap(vec![0; count])
            };
            let glyphs = match &mut storage {
                GlyphStorage::Inline(v, n) => &mut v[..*n],
                GlyphStorage::Heap(v) => v.as_mut_slice(),
                GlyphStorage::Borrowed(..) => unreachable!(),
            };
            let _ = font.textToGlyphs(text, length, encoding, glyphs);
            Self { storage }
        }
    }
}
impl<'a> SkAutoToGlyphs<'a> {
    // Source: src/core/SkFontPriv.h:107-107
    pub fn size(&self) -> usize {
        match &self.storage {
            GlyphStorage::Borrowed(_, n, _) | GlyphStorage::Inline(_, n) => *n,
            GlyphStorage::Heap(v) => v.len(),
        }
    }
}
impl<'a> SkAutoToGlyphs<'a> {
    // Source: src/core/SkFontPriv.h:108-108
    pub fn data(&self) -> *const u16 {
        match &self.storage {
            GlyphStorage::Borrowed(p, ..) => *p,
            GlyphStorage::Inline(v, _) => v.as_ptr(),
            GlyphStorage::Heap(v) => v.as_ptr(),
        }
    }
}
impl<'a> SkAutoToGlyphs<'a> {
    // Source: src/core/SkFontPriv.h:110-110
    pub fn count(&self) -> usize {
        self.size()
    }
}
impl<'a> SkAutoToGlyphs<'a> {
    // Source: src/core/SkFontPriv.h:111-111
    pub fn glyphs(&self) -> &[u16] {
        if self.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data(), self.size()) }
        }
    }
}
