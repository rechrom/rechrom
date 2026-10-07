// C++: font_engine/fonts/font_width_variant.h. Its ToString declaration has
// no definition in the supplied source tree.
// cpp: font_engine/fonts/font_width_variant.h:35-46
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontWidthVariant {
    kRegularWidth,
    kHalfWidth,
    kThirdWidth,
    kQuarterWidth,
}

impl FontWidthVariant {
    pub const kLastFontWidthVariant: Self = Self::kQuarterWidth;
}
pub const kFontWidthVariantWidth: u32 = 2;
const _: () =
    assert!((FontWidthVariant::kLastFontWidthVariant as u32) >> kFontWidthVariantWidth == 0);
