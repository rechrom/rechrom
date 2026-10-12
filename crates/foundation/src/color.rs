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

    // cpp: foundation/graphics_types/graphics/color.cc:241-276
    pub fn FromColorSpace(
        color_space: ColorSpace,
        param0: Option<f32>,
        param1: Option<f32>,
        param2: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        let mut result = Self {
            color_space_: color_space,
            none_flags_: u8::from(param0.is_none())
                | (u8::from(param1.is_none()) << 1)
                | (u8::from(param2.is_none()) << 2)
                | (u8::from(alpha.is_none()) << 3),
            _padding: [0; 2],
            param0_: param0.unwrap_or(0.0),
            param1_: param1.unwrap_or(0.0),
            param2_: param2.unwrap_or(0.0),
            alpha_: alpha.map_or(0.0, |value| value.clamp(0.0, 1.0)),
        };
        if Self::IsLightnessFirstComponent(color_space) && !result.param0_.is_nan() {
            result.param0_ = result.param0_.clamp(
                0.0,
                if matches!(color_space, ColorSpace::kLab | ColorSpace::kLch) {
                    100.0
                } else {
                    1.0
                },
            );
        }
        if Self::IsChromaSecondComponent(color_space) && !result.param1_.is_nan() {
            result.param1_ = result.param1_.max(0.0);
        }
        result
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

    // cpp: foundation/graphics_types/graphics/color.cc:575-616 Color::ToSRGB.
    // The fixed Chromium revision has baked gamut mapping disabled by default;
    // this returns extended sRGB and leaves final destination-gamut clipping to
    // the paint backend, just like Color::toSkColor4f().
    pub fn ToSRGB(&self) -> (f32, f32, f32) {
        match self.color_space_ {
            ColorSpace::kSRGB => (self.param0_, self.param1_, self.param2_),
            ColorSpace::kSRGBLegacy => (
                self.param0_ / 255.0,
                self.param1_ / 255.0,
                self.param2_ / 255.0,
            ),
            ColorSpace::kSRGBLinear => (
                srgb_encode(self.param0_),
                srgb_encode(self.param1_),
                srgb_encode(self.param2_),
            ),
            ColorSpace::kHSL => hsl_to_srgb(self.param0_, self.param1_, self.param2_),
            ColorSpace::kHWB => hwb_to_srgb(self.param0_, self.param1_, self.param2_),
            ColorSpace::kNone => (0.0, 0.0, 0.0),
            _ => xyz_d50_to_srgb(self.ToXYZD50()),
        }
    }

    // cpp: foundation/graphics_types/graphics/color.cc
    // Color::ConvertToColorSpace / Color::InterpolateColors.
    pub fn ConvertToColorSpace(&self, space: ColorSpace) -> Self {
        if self.color_space_ == space {
            return *self;
        }
        let xyz_d50 = self.ToXYZD50();
        let channels = match space {
            ColorSpace::kSRGB | ColorSpace::kSRGBLegacy => {
                let (r, g, b) = xyz_d50_to_srgb(xyz_d50);
                if space == ColorSpace::kSRGBLegacy {
                    [r * 255.0, g * 255.0, b * 255.0]
                } else {
                    [r, g, b]
                }
            }
            ColorSpace::kSRGBLinear => matrix_apply(matrix_inverse(SRGB_TO_XYZ_D50), xyz_d50),
            ColorSpace::kDisplayP3 => {
                matrix_apply(matrix_inverse(DISPLAY_P3_TO_XYZ_D50), xyz_d50).map(srgb_encode)
            }
            ColorSpace::kDisplayP3Linear => {
                matrix_apply(matrix_inverse(DISPLAY_P3_TO_XYZ_D50), xyz_d50)
            }
            ColorSpace::kA98RGB => matrix_apply(matrix_inverse(ADOBE_RGB_TO_XYZ_D50), xyz_d50)
                .map(|channel| signed_pow(channel, 256.0 / 563.0)),
            ColorSpace::kProPhotoRGB => {
                matrix_apply(matrix_inverse(PROPHOTO_RGB_TO_XYZ_D50), xyz_d50).map(prophoto_encode)
            }
            ColorSpace::kRec2020 => {
                matrix_apply(matrix_inverse(REC2020_TO_XYZ_D50), xyz_d50).map(rec2020_encode)
            }
            ColorSpace::kRec2100Linear => matrix_apply(matrix_inverse(REC2020_TO_XYZ_D50), xyz_d50),
            ColorSpace::kXYZD50 => xyz_d50,
            ColorSpace::kXYZD65 => matrix_apply(matrix_inverse(XYZ_D65_TO_D50), xyz_d50),
            ColorSpace::kLab => xyz_d50_to_lab(xyz_d50),
            ColorSpace::kLch => lab_to_lch(xyz_d50_to_lab(xyz_d50)),
            ColorSpace::kOklab => xyz_d50_to_oklab(xyz_d50),
            ColorSpace::kOklch => lab_to_lch(xyz_d50_to_oklab(xyz_d50)),
            ColorSpace::kHSL => {
                let (r, g, b) = xyz_d50_to_srgb(xyz_d50);
                srgb_to_hsl(r, g, b)
            }
            ColorSpace::kHWB => {
                let (r, g, b) = xyz_d50_to_srgb(xyz_d50);
                srgb_to_hwb(r, g, b)
            }
            _ => panic!("unsupported interpolation color space {space:?}"),
        };
        let none_flags = analogous_missing_components(self.color_space_, self.none_flags_, space);
        Self::FromColorSpace(
            space,
            (none_flags & 1 == 0).then_some(channels[0]),
            (none_flags & 2 == 0).then_some(channels[1]),
            (none_flags & 4 == 0).then_some(channels[2]),
            (none_flags & 8 == 0).then_some(self.Alpha()),
        )
    }

    pub fn InterpolateColors(
        space: ColorSpace,
        hue_method: HueInterpolationMethod,
        left: Self,
        right: Self,
        amount: f32,
    ) -> Self {
        let mut left = left.ConvertToColorSpace(space);
        let mut right = right.ConvertToColorSpace(space);
        // css-color-4 interpolation: after conversion, a missing component is
        // replaced by the corresponding component of the other endpoint. If
        // both endpoints omit it, the interpolation coordinate is zero.
        for bit in 0..4 {
            let mask = 1 << bit;
            let left_missing = left.none_flags_ & mask != 0;
            let right_missing = right.none_flags_ & mask != 0;
            let left_value = match bit {
                0 => &mut left.param0_,
                1 => &mut left.param1_,
                2 => &mut left.param2_,
                _ => &mut left.alpha_,
            };
            let right_value = match bit {
                0 => &mut right.param0_,
                1 => &mut right.param1_,
                2 => &mut right.param2_,
                _ => &mut right.alpha_,
            };
            match (left_missing, right_missing) {
                (true, false) => *left_value = *right_value,
                (false, true) => *right_value = *left_value,
                (true, true) => {
                    *left_value = 0.0;
                    *right_value = 0.0;
                }
                (false, false) => {}
            }
        }
        if Self::ColorSpaceHasHue(space) {
            if matches!(space, ColorSpace::kHSL | ColorSpace::kHWB) {
                (left.param0_, right.param0_) =
                    interpolate_hue(left.param0_, right.param0_, hue_method);
            } else {
                (left.param2_, right.param2_) =
                    interpolate_hue(left.param2_, right.param2_, hue_method);
            }
        }
        let inverse = 1.0 - amount;
        let alpha = left.alpha_ * inverse + right.alpha_ * amount;
        let mut channels = [left.param0_, left.param1_, left.param2_];
        let right_channels = [right.param0_, right.param1_, right.param2_];
        let hue_index = if matches!(space, ColorSpace::kHSL | ColorSpace::kHWB) {
            0
        } else {
            2
        };
        for index in 0..3 {
            if Self::ColorSpaceHasHue(space) && index == hue_index {
                channels[index] =
                    (channels[index] * inverse + right_channels[index] * amount).rem_euclid(360.0);
            } else if alpha == 0.0 {
                channels[index] = 0.0;
            } else {
                channels[index] = (channels[index] * left.alpha_ * inverse
                    + right_channels[index] * right.alpha_ * amount)
                    / alpha;
            }
        }
        Self::FromColorSpace(
            space,
            Some(channels[0]),
            Some(channels[1]),
            Some(channels[2]),
            Some(alpha),
        )
    }

    // cpp: foundation/graphics_types/graphics/color.cc:618-670 Color::ToXYZD50.
    fn ToXYZD50(&self) -> [f32; 3] {
        let channels = [self.param0_, self.param1_, self.param2_];
        match self.color_space_ {
            ColorSpace::kSRGBLegacy => matrix_apply(
                SRGB_TO_XYZ_D50,
                channels.map(|channel| srgb_decode(channel / 255.0)),
            ),
            ColorSpace::kSRGB => matrix_apply(SRGB_TO_XYZ_D50, channels.map(srgb_decode)),
            ColorSpace::kSRGBLinear => matrix_apply(SRGB_TO_XYZ_D50, channels),
            ColorSpace::kDisplayP3 => {
                matrix_apply(DISPLAY_P3_TO_XYZ_D50, channels.map(srgb_decode))
            }
            ColorSpace::kDisplayP3Linear => matrix_apply(DISPLAY_P3_TO_XYZ_D50, channels),
            ColorSpace::kA98RGB => matrix_apply(
                ADOBE_RGB_TO_XYZ_D50,
                channels.map(|x| signed_pow(x, 563.0 / 256.0)),
            ),
            ColorSpace::kProPhotoRGB => {
                matrix_apply(PROPHOTO_RGB_TO_XYZ_D50, channels.map(prophoto_decode))
            }
            ColorSpace::kRec2020 => matrix_apply(REC2020_TO_XYZ_D50, channels.map(rec2020_decode)),
            ColorSpace::kRec2100Linear => matrix_apply(REC2020_TO_XYZ_D50, channels),
            ColorSpace::kXYZD50 => channels,
            ColorSpace::kXYZD65 => matrix_apply(XYZ_D65_TO_D50, channels),
            ColorSpace::kLab => lab_to_xyz_d50(channels),
            ColorSpace::kOklab => oklab_to_xyz_d50(channels),
            ColorSpace::kLch => lab_to_xyz_d50(lch_to_lab(channels)),
            ColorSpace::kOklch => oklab_to_xyz_d50(lch_to_lab(channels)),
            ColorSpace::kHSL => matrix_apply(
                SRGB_TO_XYZ_D50,
                array_map(
                    hsl_to_srgb(self.param0_, self.param1_, self.param2_),
                    srgb_decode,
                ),
            ),
            ColorSpace::kHWB => matrix_apply(
                SRGB_TO_XYZ_D50,
                array_map(
                    hwb_to_srgb(self.param0_, self.param1_, self.param2_),
                    srgb_decode,
                ),
            ),
            ColorSpace::kNone => [0.0; 3],
        }
    }
}

type Matrix3 = [[f32; 3]; 3];

// These are the same to-XYZ-D50 matrices selected by
// Color::GetSkColorSpaceParams at Chromium 6c1d401fcca5. The sRGB and Adobe
// matrices retain Skia's 16.16 constants; the other values are Skia's named
// gamuts (ProPhoto comes from its named CSS Color 4 primaries).
const SRGB_TO_XYZ_D50: Matrix3 = [
    [
        0x6fa2 as f32 / 65536.0,
        0x6299 as f32 / 65536.0,
        0x24a0 as f32 / 65536.0,
    ],
    [
        0x38f5 as f32 / 65536.0,
        0xb785 as f32 / 65536.0,
        0x0f84 as f32 / 65536.0,
    ],
    [
        0x0390 as f32 / 65536.0,
        0x18da as f32 / 65536.0,
        0xb6cf as f32 / 65536.0,
    ],
];
const ADOBE_RGB_TO_XYZ_D50: Matrix3 = [
    [
        0x9c18 as f32 / 65536.0,
        0x348d as f32 / 65536.0,
        0x2631 as f32 / 65536.0,
    ],
    [
        0x4fa5 as f32 / 65536.0,
        0xa02c as f32 / 65536.0,
        0x102f as f32 / 65536.0,
    ],
    [
        0x04fc as f32 / 65536.0,
        0x0f95 as f32 / 65536.0,
        0xbe9c as f32 / 65536.0,
    ],
];
const DISPLAY_P3_TO_XYZ_D50: Matrix3 = [
    [0.515102, 0.291965, 0.157153],
    [0.241182, 0.692236, 0.0665819],
    [-0.00104941, 0.0418818, 0.784378],
];
const REC2020_TO_XYZ_D50: Matrix3 = [
    [0.673459, 0.165661, 0.125100],
    [0.279033, 0.675338, 0.0456288],
    [-0.00193139, 0.0299794, 0.797162],
];
const PROPHOTO_RGB_TO_XYZ_D50: Matrix3 = [
    [0.7977666, 0.1351813, 0.03134773],
    [0.2880748, 0.7118352, 0.00008994],
    [0.0, 0.0, 0.8251046],
];
const XYZ_D65_TO_D50: Matrix3 = [
    [1.0478112, 0.0228866, -0.0501270],
    [0.0295424, 0.9904844, -0.0170491],
    [-0.0092345, 0.0150436, 0.7521316],
];
const XYZ_D65_TO_LMS: Matrix3 = [
    [0.81902244, 0.36190626, -0.12887378],
    [0.03298367, 0.92928685, 0.03614467],
    [0.04817720, 0.26423952, 0.63354784],
];
const OKLAB_TO_LMS: Matrix3 = [
    [1.0, 0.3963378, 0.21580376],
    [1.0, -0.10556134, -0.06385417],
    [1.0, -0.08948418, -1.2914855],
];

fn array_map(values: (f32, f32, f32), f: fn(f32) -> f32) -> [f32; 3] {
    [f(values.0), f(values.1), f(values.2)]
}

fn matrix_apply(matrix: Matrix3, value: [f32; 3]) -> [f32; 3] {
    matrix.map(|row| row[0] * value[0] + row[1] * value[1] + row[2] * value[2])
}

fn matrix_inverse(matrix: Matrix3) -> Matrix3 {
    let determinant = matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
        - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
        + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]);
    let reciprocal = 1.0 / determinant;
    [
        [
            (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1]) * reciprocal,
            (matrix[0][2] * matrix[2][1] - matrix[0][1] * matrix[2][2]) * reciprocal,
            (matrix[0][1] * matrix[1][2] - matrix[0][2] * matrix[1][1]) * reciprocal,
        ],
        [
            (matrix[1][2] * matrix[2][0] - matrix[1][0] * matrix[2][2]) * reciprocal,
            (matrix[0][0] * matrix[2][2] - matrix[0][2] * matrix[2][0]) * reciprocal,
            (matrix[0][2] * matrix[1][0] - matrix[0][0] * matrix[1][2]) * reciprocal,
        ],
        [
            (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]) * reciprocal,
            (matrix[0][1] * matrix[2][0] - matrix[0][0] * matrix[2][1]) * reciprocal,
            (matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0]) * reciprocal,
        ],
    ]
}

fn signed_pow(value: f32, exponent: f32) -> f32 {
    value.signum() * value.abs().powf(exponent)
}

fn srgb_decode(value: f32) -> f32 {
    if value.abs() <= 0.04045 {
        value / 12.92
    } else {
        value.signum() * ((value.abs() + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_encode(value: f32) -> f32 {
    if value.abs() <= 0.0031308 {
        value * 12.92
    } else {
        value.signum() * (1.055 * value.abs().powf(1.0 / 2.4) - 0.055)
    }
}

fn rec2020_decode(value: f32) -> f32 {
    if value.abs() <= 0.0812429 {
        value * 0.222222
    } else {
        value.signum() * (0.909672 * value.abs() + 0.0903276).powf(2.22222)
    }
}

fn rec2020_encode(value: f32) -> f32 {
    if value.abs() < 0.01805397 {
        value * 4.5
    } else {
        value.signum() * (1.0992968 * value.abs().powf(0.45) - 0.0992968)
    }
}

fn prophoto_encode(value: f32) -> f32 {
    if value.abs() < 1.0 / 512.0 {
        value * 16.0
    } else {
        signed_pow(value, 1.0 / 1.8)
    }
}

fn xyz_d50_to_srgb(value: [f32; 3]) -> (f32, f32, f32) {
    let linear = matrix_apply(matrix_inverse(SRGB_TO_XYZ_D50), value);
    (
        srgb_encode(linear[0]),
        srgb_encode(linear[1]),
        srgb_encode(linear[2]),
    )
}

fn lab_to_xyz_d50([lightness, a, b]: [f32; 3]) -> [f32; 3] {
    let fy = (lightness + 16.0) / 116.0;
    let inverse = |value: f32| {
        if value <= 24.0 / 116.0 {
            (108.0 / 841.0) * (value - 16.0 / 116.0)
        } else {
            value * value * value
        }
    };
    [
        inverse(fy + a / 500.0) * 0.9642,
        inverse(fy),
        inverse(fy - b / 200.0) * 0.8251,
    ]
}

fn xyz_d50_to_lab(value: [f32; 3]) -> [f32; 3] {
    let convert = |value: f32| {
        if value > 216.0 / 24389.0 {
            value.cbrt()
        } else {
            (24389.0 / 27.0 * value + 16.0) / 116.0
        }
    };
    let x = convert(value[0] / 0.9642);
    let y = convert(value[1]);
    let z = convert(value[2] / 0.8251);
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

fn lch_to_lab([lightness, chroma, hue]: [f32; 3]) -> [f32; 3] {
    let radians = hue.to_radians();
    [lightness, chroma * radians.cos(), chroma * radians.sin()]
}

fn lab_to_lch([lightness, a, b]: [f32; 3]) -> [f32; 3] {
    [
        lightness,
        a.hypot(b),
        b.atan2(a).to_degrees().rem_euclid(360.0),
    ]
}

fn xyz_d50_to_oklab(value: [f32; 3]) -> [f32; 3] {
    let xyz_d65 = matrix_apply(matrix_inverse(XYZ_D65_TO_D50), value);
    let lms = matrix_apply(XYZ_D65_TO_LMS, xyz_d65).map(f32::cbrt);
    matrix_apply(matrix_inverse(OKLAB_TO_LMS), lms)
}

fn interpolate_hue(left: f32, right: f32, method: HueInterpolationMethod) -> (f32, f32) {
    let (mut left, mut right) = (left, right);
    let delta = right - left;
    match method {
        HueInterpolationMethod::kShorter => {
            if delta > 180.0 {
                left += 360.0
            } else if delta < -180.0 {
                right += 360.0
            }
        }
        HueInterpolationMethod::kLonger => {
            if delta > 0.0 && delta < 180.0 {
                left += 360.0
            } else if delta > -180.0 && delta <= 0.0 {
                right += 360.0
            }
        }
        HueInterpolationMethod::kIncreasing => {
            if right < left {
                right += 360.0
            }
        }
        HueInterpolationMethod::kDecreasing => {
            if left < right {
                left += 360.0
            }
        }
    }
    (left, right)
}

fn oklab_to_xyz_d50(value: [f32; 3]) -> [f32; 3] {
    let lms = matrix_apply(OKLAB_TO_LMS, value);
    let xyz_d65 = matrix_apply(
        matrix_inverse(XYZ_D65_TO_LMS),
        lms.map(|channel| channel * channel * channel),
    );
    matrix_apply(XYZ_D65_TO_D50, xyz_d65)
}

fn hsl_to_srgb(hue: f32, saturation: f32, lightness: f32) -> (f32, f32, f32) {
    if saturation == 0.0 {
        return (lightness, lightness, lightness);
    }
    let f = |offset: f32| {
        let k = (offset + hue / 30.0).rem_euclid(12.0);
        let a = saturation * lightness.min(1.0 - lightness);
        lightness - a * (-1.0_f32).max((k - 3.0).min((9.0 - k).min(1.0)))
    };
    (f(0.0), f(8.0), f(4.0))
}

fn hwb_to_srgb(hue: f32, white: f32, black: f32) -> (f32, f32, f32) {
    if white + black >= 1.0 {
        let gray = white / (white + black);
        return (gray, gray, gray);
    }
    let (mut red, mut green, mut blue) = hsl_to_srgb(hue, 1.0, 0.5);
    red += white - (white + black) * red;
    green += white - (white + black) * green;
    blue += white - (white + black) * blue;
    (red, green, blue)
}

fn srgb_to_hsl(red: f32, green: f32, blue: f32) -> [f32; 3] {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) * 0.5;
    let delta = max - min;
    if delta == 0.0 {
        return [0.0, 0.0, lightness];
    }
    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue_sector = if max == red {
        ((green - blue) / delta).rem_euclid(6.0)
    } else if max == green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    [hue_sector * 60.0, saturation, lightness]
}

fn srgb_to_hwb(red: f32, green: f32, blue: f32) -> [f32; 3] {
    let hue = srgb_to_hsl(red, green, blue)[0];
    [
        hue,
        red.min(green).min(blue),
        1.0 - red.max(green).max(blue),
    ]
}

fn prophoto_decode(value: f32) -> f32 {
    if value.abs() < 16.0 / 512.0 {
        value / 16.0
    } else {
        signed_pow(value, 1.8)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComponentKind {
    RedX,
    GreenY,
    BlueZ,
    Lightness,
    Colorfulness,
    Hue,
    A,
    B,
    Whiteness,
    Blackness,
}

fn component_kinds(space: ColorSpace) -> [ComponentKind; 3] {
    use ComponentKind::*;
    match space {
        ColorSpace::kSRGB
        | ColorSpace::kSRGBLinear
        | ColorSpace::kDisplayP3
        | ColorSpace::kDisplayP3Linear
        | ColorSpace::kA98RGB
        | ColorSpace::kProPhotoRGB
        | ColorSpace::kRec2020
        | ColorSpace::kRec2100Linear
        | ColorSpace::kSRGBLegacy => [RedX, GreenY, BlueZ],
        ColorSpace::kXYZD50 | ColorSpace::kXYZD65 => [RedX, GreenY, BlueZ],
        ColorSpace::kLab | ColorSpace::kOklab => [Lightness, A, B],
        ColorSpace::kLch | ColorSpace::kOklch => [Lightness, Colorfulness, Hue],
        ColorSpace::kHSL => [Hue, Colorfulness, Lightness],
        ColorSpace::kHWB => [Hue, Whiteness, Blackness],
        ColorSpace::kNone => [RedX, GreenY, BlueZ],
    }
}

// css-color-4 analogous-component carry forward. Missing flags survive a
// conversion only when the destination has a component with the same meaning.
fn analogous_missing_components(source: ColorSpace, flags: u8, target: ColorSpace) -> u8 {
    let source_kinds = component_kinds(source);
    let target_kinds = component_kinds(target);
    let mut result = flags & 8;
    for source_index in 0..3 {
        if flags & (1 << source_index) == 0 {
            continue;
        }
        for target_index in 0..3 {
            if source_kinds[source_index] == target_kinds[target_index] {
                result |= 1 << target_index;
            }
        }
    }
    result
}
