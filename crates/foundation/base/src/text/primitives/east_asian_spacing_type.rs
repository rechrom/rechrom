// C++: src/foundation/text/source/east_asian_spacing_type.h

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EastAsianSpacingType {
    kOther = 0,
    kNarrow,
    kConditional,
    kWide,
}
