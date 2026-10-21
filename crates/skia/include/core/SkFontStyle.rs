// Source: include/core/SkFontStyle.h:16-17
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SkFontStyle {
    // Source: include/core/SkFontStyle.h:79-82
    value: i32,
}
// Source: include/core/SkFontStyle.h:18-30
// The native named enum is retained as a distinct type. Existing i32 associated
// constants remain the constructor-facing spelling; C++ enum promotion is explicit.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    kInvisible_Weight = 0,
    kThin_Weight = 100,
    kExtraLight_Weight = 200,
    kLight_Weight = 300,
    kNormal_Weight = 400,
    kMedium_Weight = 500,
    kSemiBold_Weight = 600,
    kBold_Weight = 700,
    kExtraBold_Weight = 800,
    kBlack_Weight = 900,
    kExtraBlack_Weight = 1000,
}
// Source: include/core/SkFontStyle.h:32-42
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    kUltraCondensed_Width = 1,
    kExtraCondensed_Width = 2,
    kCondensed_Width = 3,
    kSemiCondensed_Width = 4,
    kNormal_Width = 5,
    kSemiExpanded_Width = 6,
    kExpanded_Width = 7,
    kExtraExpanded_Width = 8,
    kUltraExpanded_Width = 9,
}
impl SkFontStyle {
    // Source: include/core/SkFontStyle.h:18-30
    pub const kInvisible_Weight: i32 = 0;
    pub const kThin_Weight: i32 = 100;
    pub const kExtraLight_Weight: i32 = 200;
    pub const kLight_Weight: i32 = 300;
    pub const kNormal_Weight: i32 = 400;
    pub const kMedium_Weight: i32 = 500;
    pub const kSemiBold_Weight: i32 = 600;
    pub const kBold_Weight: i32 = 700;
    pub const kExtraBold_Weight: i32 = 800;
    pub const kBlack_Weight: i32 = 900;
    pub const kExtraBlack_Weight: i32 = 1000;
    // Source: include/core/SkFontStyle.h:32-42
    pub const kUltraCondensed_Width: i32 = 1;
    pub const kExtraCondensed_Width: i32 = 2;
    pub const kCondensed_Width: i32 = 3;
    pub const kSemiCondensed_Width: i32 = 4;
    pub const kNormal_Width: i32 = 5;
    pub const kSemiExpanded_Width: i32 = 6;
    pub const kExpanded_Width: i32 = 7;
    pub const kExtraExpanded_Width: i32 = 8;
    pub const kUltraExpanded_Width: i32 = 9;
}
// Source: include/core/SkFontStyle.h:44-48
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slant(pub u8);
impl Slant {
    pub const kUpright_Slant: Self = Self(0);
    pub const kItalic_Slant: Self = Self(1);
    pub const kOblique_Slant: Self = Self(2);
}
impl SkFontStyle {
    // Source: include/core/SkFontStyle.h:50-54
    pub const fn new(weight: i32, width: i32, slant: Slant) -> Self {
        let weight = if weight < 0 {
            0
        } else if weight > 1000 {
            1000
        } else {
            weight
        };
        let width = if width < 1 {
            1
        } else if width > 9 {
            9
        } else {
            width
        };
        // Source: include/core/SkFontStyle.h:53
        let slant = if slant.0 > 2 { 2 } else { slant.0 as i32 };
        Self {
            value: weight + (width << 16) + (slant << 24),
        }
    }
}
// Source: include/core/SkFontStyle.h:56-56
impl Default for SkFontStyle {
    fn default() -> Self {
        Self::Normal()
    }
}
// Source: include/core/SkFontStyle.h:58-60
impl PartialEq for SkFontStyle {
    fn eq(&self, rhs: &Self) -> bool {
        self.value == rhs.value
    }
}
impl SkFontStyle {
    // Source: include/core/SkFontStyle.h:62-64
    pub fn weight(&self) -> i32 {
        self.value & 0xffff
    }
    pub fn width(&self) -> i32 {
        (self.value >> 16) & 0xff
    }
    pub fn slant(&self) -> Slant {
        Slant(((self.value >> 24) & 0xff) as u8)
    }
    // Source: include/core/SkFontStyle.h:66-77
    pub const fn Normal() -> Self {
        Self::new(
            Self::kNormal_Weight,
            Self::kNormal_Width,
            Slant::kUpright_Slant,
        )
    }
    pub const fn Bold() -> Self {
        Self::new(
            Self::kBold_Weight,
            Self::kNormal_Width,
            Slant::kUpright_Slant,
        )
    }
    pub const fn Italic() -> Self {
        Self::new(
            Self::kNormal_Weight,
            Self::kNormal_Width,
            Slant::kItalic_Slant,
        )
    }
    pub const fn BoldItalic() -> Self {
        Self::new(
            Self::kBold_Weight,
            Self::kNormal_Width,
            Slant::kItalic_Slant,
        )
    }
}
