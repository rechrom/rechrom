// cpp: layoutng/internal/style_variant.h:14-20
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum StyleVariant {
    kStandard = 0,
    kFirstLine = 1,
    kStandardEllipsis = 2,
    kFirstLineEllipsis = 3,
}

// cpp: layoutng/internal/style_variant.h:14-20
impl TryFrom<u8> for StyleVariant {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::kStandard),
            1 => Ok(Self::kFirstLine),
            2 => Ok(Self::kStandardEllipsis),
            3 => Ok(Self::kFirstLineEllipsis),
            _ => Err(()),
        }
    }
}

// cpp: layoutng/internal/style_variant.h:22-25
#[allow(non_snake_case)]
pub fn UsesFirstLineStyle(variant: StyleVariant) -> bool {
    (variant as u8 & StyleVariant::kFirstLine as u8) != 0
}

// cpp: layoutng/internal/style_variant.h:27-30
#[allow(non_snake_case)]
pub fn IsEllipsis(variant: StyleVariant) -> bool {
    (variant as u8 & StyleVariant::kStandardEllipsis as u8) != 0
}

// cpp: layoutng/internal/style_variant.h:32-37
#[allow(non_snake_case)]
pub fn ToParentStyleVariant(variant: StyleVariant) -> StyleVariant {
    let parent = (variant as u8) & !(StyleVariant::kStandardEllipsis as u8);
    match parent {
        0 => StyleVariant::kStandard,
        1 => StyleVariant::kFirstLine,
        _ => unreachable!("style variant uses only first-line and ellipsis bits"),
    }
}
