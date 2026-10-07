// cpp: layoutng_style/css/color_scheme_flags.h:10-17
/// Flags extracted from the computed list of color schemes.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ColorSchemeFlag {
    kNormal = 0,
    kDark = 1,
    kLight = 2,
    kOnly = 4,
}

// cpp: layoutng_style/css/color_scheme_flags.h:19-20
/// Bitset of ColorSchemeFlag values, retaining the C++ u8 representation.
pub type ColorSchemeFlags = u8;
