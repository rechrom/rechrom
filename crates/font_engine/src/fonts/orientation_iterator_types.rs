#![allow(non_camel_case_types, non_upper_case_globals)]

// The iterator behavior remains blocked; its nested value type is complete.
// cpp: font_engine/fonts/orientation_iterator.h:20-28
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderOrientation {
    #[default]
    kOrientationKeep = 0,
    kOrientationRotateSideways = 1,
    kOrientationInvalid = 2,
}

impl RenderOrientation {
    pub const kMaxEnumValue: Self = Self::kOrientationRotateSideways;
}
