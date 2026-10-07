// C++: font_engine/fonts/shaping/text_spacing_trim.h.

// cpp: font_engine/fonts/shaping/text_spacing_trim.h:11-22
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextSpacingTrim {
    #[default]
    kNormal = 0,
    kSpaceAll = 1,
    kSpaceFirst = 2,
    kTrimStart = 3,
}

impl TextSpacingTrim {
    pub const kInitial: Self = Self::kNormal;
}

pub const kTextSpacingTrimBitCount: u32 = 2;

// cpp: font_engine/fonts/shaping/text_spacing_trim.h:24-38
pub const fn ShouldTrimAdjacent(value: TextSpacingTrim) -> bool {
    !matches!(value, TextSpacingTrim::kSpaceAll)
}

pub const fn ShouldTrimStartOfParagraph(value: TextSpacingTrim) -> bool {
    matches!(value, TextSpacingTrim::kTrimStart)
}

pub const fn ShouldTrimStartOfWrappedLine(value: TextSpacingTrim) -> bool {
    matches!(
        value,
        TextSpacingTrim::kSpaceFirst | TextSpacingTrim::kTrimStart
    )
}

pub const fn ShouldTrimEnd(value: TextSpacingTrim) -> bool {
    !matches!(value, TextSpacingTrim::kSpaceAll)
}
