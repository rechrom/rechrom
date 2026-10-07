// C++: foundation/graphics_types/graphics/color.h and color_constants.cc.
// The supplied C++ tree lacks definitions for conversion, parsing, mixing,
// serialization, and the numeric Color hash; this module maps the inlines and
// the value representation without inventing those implementations.

pub type RGBA32 = u32;

// cpp: foundation/graphics_types/graphics/color.h:68-115
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ColorSpace {
    kSRGB,
    kSRGBLinear,
    kDisplayP3,
    kDisplayP3Linear,
    kA98RGB,
    kProPhotoRGB,
    kRec2020,
    kRec2100Linear,
    kXYZD50,
    kXYZD65,
    kLab,
    kOklab,
    kLch,
    kOklch,
    kSRGBLegacy,
    kHSL,
    kHWB,
    kNone,
}

// cpp: foundation/graphics_types/graphics/color.h:228-233
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum HueInterpolationMethod {
    kShorter,
    kLonger,
    kIncreasing,
    kDecreasing,
}

// C++ packs four one-bit none flags into a single byte. Rust keeps the same
// storage width and uses bit operations, preserving the 20-byte value layout.
// cpp: foundation/graphics_types/graphics/color.h:162-166,478-493
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color {
    color_space_: ColorSpace,
    none_flags_: u8,
    _padding: [u8; 2],
    param0_: f32,
    param1_: f32,
    param2_: f32,
    alpha_: f32,
}

const _: () = assert!(std::mem::size_of::<Color>() == 20);

impl Default for Color {
    fn default() -> Self {
        Self::kTransparent
    }
}

impl PartialEq for Color {
    // cpp: foundation/graphics_types/graphics/color.h:370-378
    fn eq(&self, other: &Self) -> bool {
        self.color_space_ == other.color_space_
            && self.none_flags_ == other.none_flags_
            && self.param0_ == other.param0_
            && self.param1_ == other.param1_
            && self.param2_ == other.param2_
            && self.alpha_ == other.alpha_
    }
}

impl Color {
    // cpp: foundation/graphics_types/graphics/color.h:193-197,421-428
    pub fn FromRGBAFloat(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            color_space_: ColorSpace::kSRGBLegacy,
            none_flags_: 0,
            _padding: [0; 2],
            param0_: r * 255.0,
            param1_: g * 255.0,
            param2_: b * 255.0,
            alpha_: a,
        }
    }

    const fn from_rgba32(color: RGBA32) -> Self {
        Self {
            color_space_: ColorSpace::kSRGBLegacy,
            none_flags_: 0,
            _padding: [0; 2],
            param0_: ((color >> 16) & 0xff) as f32,
            param1_: ((color >> 8) & 0xff) as f32,
            param2_: (color & 0xff) as f32,
            alpha_: ((color >> 24) & 0xff) as f32 / 255.0,
        }
    }

    // cpp: foundation/graphics_types/graphics/color_constants.cc:28-33
    pub const kBlack: Self = Self::from_rgba32(0xff000000);
    pub const kWhite: Self = Self::from_rgba32(0xffffffff);
    pub const kDarkGray: Self = Self::from_rgba32(0xff808080);
    pub const kGray: Self = Self::from_rgba32(0xffa0a0a0);
    pub const kLightGray: Self = Self::from_rgba32(0xffc0c0c0);
    pub const kTransparent: Self = Self::from_rgba32(0x00000000);

    // cpp: foundation/graphics_types/graphics/color.h:122-133
    pub const fn IsPredefinedColorSpace(space: ColorSpace) -> bool {
        matches!(
            space,
            ColorSpace::kSRGB
                | ColorSpace::kSRGBLinear
                | ColorSpace::kDisplayP3
                | ColorSpace::kDisplayP3Linear
                | ColorSpace::kA98RGB
                | ColorSpace::kProPhotoRGB
                | ColorSpace::kRec2020
                | ColorSpace::kRec2100Linear
                | ColorSpace::kXYZD50
                | ColorSpace::kXYZD65
        )
    }

    // cpp: foundation/graphics_types/graphics/color.h:135-139
    pub const fn IsLightnessFirstComponent(space: ColorSpace) -> bool {
        matches!(
            space,
            ColorSpace::kLab | ColorSpace::kOklab | ColorSpace::kLch | ColorSpace::kOklch
        )
    }

    // cpp: foundation/graphics_types/graphics/color.h:141-145
    pub const fn IsChromaSecondComponent(space: ColorSpace) -> bool {
        matches!(space, ColorSpace::kLch | ColorSpace::kOklch)
    }

    // cpp: foundation/graphics_types/graphics/color.h:149-152
    pub const fn IsLegacyColorSpace(space: ColorSpace) -> bool {
        matches!(
            space,
            ColorSpace::kSRGBLegacy | ColorSpace::kHSL | ColorSpace::kHWB
        )
    }

    // cpp: foundation/graphics_types/graphics/color.h:154-159
    pub const fn ColorSpaceHasHue(space: ColorSpace) -> bool {
        matches!(
            space,
            ColorSpace::kLch | ColorSpace::kOklch | ColorSpace::kHSL | ColorSpace::kHWB
        )
    }

    // cpp: foundation/graphics_types/graphics/color.h:182-192,430-432
    pub const fn FromRGB(r: i32, g: i32, b: i32) -> Self {
        Self::FromRGBA(r, g, b, 255)
    }

    pub const fn FromRGBA(r: i32, g: i32, b: i32, a: i32) -> Self {
        let color = ((Self::ClampInt255(a) as u32) << 24)
            | ((Self::ClampInt255(r) as u32) << 16)
            | ((Self::ClampInt255(g) as u32) << 8)
            | (Self::ClampInt255(b) as u32);
        Self::from_rgba32(color)
    }

    // cpp: foundation/graphics_types/graphics/color.h:265-267,412-420
    pub const fn FromRGBA32(color: RGBA32) -> Self {
        Self::from_rgba32(color)
    }

    pub const fn ClampInt255(value: i32) -> i32 {
        if value < 0 {
            0
        } else if value > 255 {
            255
        } else {
            value
        }
    }

    // cpp: foundation/graphics_types/graphics/color.h:305-322
    pub fn IsFullyTransparent(&self) -> bool {
        self.Alpha() <= 0.0
    }
    pub fn IsOpaque(&self) -> bool {
        self.Alpha() >= 1.0
    }
    pub fn Param0(&self) -> f32 {
        self.param0_
    }
    pub fn Param1(&self) -> f32 {
        self.param1_
    }
    pub fn Param2(&self) -> f32 {
        self.param2_
    }
    pub fn Alpha(&self) -> f32 {
        self.alpha_
    }
    pub fn Param0IsNone(&self) -> bool {
        self.none_flags_ & 1 != 0
    }
    pub fn Param1IsNone(&self) -> bool {
        self.none_flags_ & 2 != 0
    }
    pub fn Param2IsNone(&self) -> bool {
        self.none_flags_ & 4 != 0
    }
    pub fn AlphaIsNone(&self) -> bool {
        self.none_flags_ & 8 != 0
    }
    pub fn HasNoneParams(&self) -> bool {
        self.Param0IsNone() || self.Param1IsNone() || self.Param2IsNone() || self.AlphaIsNone()
    }
    pub fn SetAlpha(&mut self, alpha: f32) {
        self.alpha_ = alpha;
    }

    // cpp: foundation/graphics_types/graphics/color.h:333-335
    pub fn AlphaAsInteger(&self) -> i32 {
        (self.alpha_ * 255.0).round_ties_even() as i32
    }

    // cpp: foundation/graphics_types/graphics/color.h:346-350
    pub fn MakeOpaque(&self) -> Self {
        let mut opaque = *self;
        opaque.SetAlpha(1.0);
        opaque
    }

    // cpp: foundation/graphics_types/graphics/color.h:387
    pub fn GetColorSpace(&self) -> ColorSpace {
        self.color_space_
    }
}
