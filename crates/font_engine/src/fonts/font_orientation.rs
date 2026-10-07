// cpp: font_engine/fonts/font_orientation.h:35-49
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FontOrientation {
    kHorizontal = 0,
    kVerticalRotated = 1,
    kVerticalMixed = 2,
    kVerticalUpright = 3,
}

pub const kFontOrientationBitCount: u32 = 2;
pub const kFontOrientationAnyUprightMask: u32 = 2;

// cpp: font_engine/fonts/font_orientation.h:51-53
pub fn IsVerticalAnyUpright(orientation: FontOrientation) -> bool {
    (orientation as u32 & kFontOrientationAnyUprightMask) != 0
}

// cpp: font_engine/fonts/font_orientation.h:54-56
pub fn IsVerticalNonCJKUpright(orientation: FontOrientation) -> bool {
    orientation == FontOrientation::kVerticalUpright
}

// cpp: font_engine/fonts/font_orientation.h:58-60
pub fn IsVerticalBaseline(orientation: FontOrientation) -> bool {
    orientation != FontOrientation::kHorizontal
}

// The declarations at font_orientation.h:57,62-66 have no definitions in the
// supplied C++ font_engine tree; those calls remain unconnected.
