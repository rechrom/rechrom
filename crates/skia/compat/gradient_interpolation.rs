//! CSS Color 4 gradient interpolation coordinates.
//!
//! CSS gradients interpolate in the requested coordinate system and convert
//! the interpolated result to the destination color space. These helpers use
//! extended sRGB at the Paint/Raster boundary, matching Blink's Color value.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GradientInterpolationSpace {
    #[default]
    None,
    Srgb,
    SrgbLegacy,
    SrgbLinear,
    DisplayP3,
    DisplayP3Linear,
    Hsl,
    Hwb,
    A98Rgb,
    ProPhotoRgb,
    Rec2020,
    Rec2100Linear,
    Lab,
    Lch,
    Oklab,
    Oklch,
    XyzD50,
    XyzD65,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GradientHueMethod {
    #[default]
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

impl GradientInterpolationSpace {
    pub fn has_hue(self) -> bool {
        matches!(self, Self::Lch | Self::Oklch | Self::Hsl | Self::Hwb)
    }
}

fn mul(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    m.map(|row| row[0].mul_add(v[0], row[1].mul_add(v[1], row[2] * v[2])))
}
fn signed_pow(v: f32, p: f32) -> f32 {
    v.abs().powf(p).copysign(v)
}
fn decode(v: f32) -> f32 {
    if v.abs() <= 0.04045 {
        v / 12.92
    } else {
        signed_pow((v.abs() + 0.055) / 1.055, 2.4).copysign(v)
    }
}
fn encode(v: f32) -> f32 {
    if v.abs() <= 0.0031308 {
        12.92 * v
    } else {
        (1.055 * signed_pow(v.abs(), 1.0 / 2.4) - 0.055).copysign(v)
    }
}

const SRGB_TO_XYZ_D65: [[f32; 3]; 3] = [
    [0.4123908, 0.35758433, 0.1804808],
    [0.212639, 0.71516865, 0.07219232],
    [0.01933082, 0.11919478, 0.95053214],
];
const XYZ_D65_TO_SRGB: [[f32; 3]; 3] = [
    [3.24097, -1.5373832, -0.49861076],
    [-0.96924365, 1.8759675, 0.04155506],
    [0.05563008, -0.20397696, 1.0569715],
];
const P3_TO_XYZ_D65: [[f32; 3]; 3] = [
    [0.48657095, 0.2656677, 0.19821729],
    [0.22897457, 0.69173855, 0.07928691],
    [0.0, 0.04511338, 1.0439444],
];
const XYZ_D65_TO_P3: [[f32; 3]; 3] = [
    [2.493497, -0.9313836, -0.4027108],
    [-0.829489, 1.7626641, 0.02362469],
    [0.03584583, -0.07617239, 0.9568845],
];
const D65_TO_D50: [[f32; 3]; 3] = [
    [1.0479298, 0.0229468, -0.05019223],
    [0.02962782, 0.99043447, -0.01707383],
    [-0.00924306, 0.01505519, 0.75187427],
];
const D50_TO_D65: [[f32; 3]; 3] = [
    [0.9554734, -0.02309854, 0.06325931],
    [-0.0283697, 1.0099955, 0.02104144],
    [0.012314, -0.0205077, 1.3303659],
];
const A98_TO_XYZ_D50: [[f32; 3]; 3] = [
    [0.6097412, 0.20527649, 0.14918518],
    [0.31111145, 0.6256714, 0.06321716],
    [0.019470215, 0.06086731, 0.7445679],
];
const PROPHOTO_TO_XYZ_D50: [[f32; 3]; 3] = [
    [0.7977666, 0.1351813, 0.03134773],
    [0.2880748, 0.7118352, 0.00008994],
    [0.0, 0.0, 0.8251046],
];
const REC2020_TO_XYZ_D50: [[f32; 3]; 3] = [
    [0.673459, 0.165661, 0.125100],
    [0.279033, 0.675338, 0.0456288],
    [-0.00193139, 0.0299794, 0.797162],
];

fn inverse(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let d = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let r = 1.0 / d;
    [
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * r,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * r,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * r,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * r,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * r,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * r,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * r,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * r,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * r,
        ],
    ]
}

fn a98_decode(v: f32) -> f32 {
    signed_pow(v, 563.0 / 256.0)
}
fn a98_encode(v: f32) -> f32 {
    signed_pow(v, 256.0 / 563.0)
}
fn prophoto_decode(v: f32) -> f32 {
    if v.abs() < 16.0 / 512.0 {
        v / 16.0
    } else {
        signed_pow(v, 1.8)
    }
}
fn prophoto_encode(v: f32) -> f32 {
    if v.abs() < 1.0 / 512.0 {
        v * 16.0
    } else {
        signed_pow(v, 1.0 / 1.8)
    }
}
fn rec2020_decode(v: f32) -> f32 {
    if v.abs() < 0.0812429 {
        v / 4.5
    } else {
        v.signum() * ((v.abs() + 0.0992968) / 1.0992968).powf(1.0 / 0.45)
    }
}
fn rec2020_encode(v: f32) -> f32 {
    if v.abs() < 0.01805397 {
        v * 4.5
    } else {
        v.signum() * (1.0992968 * v.abs().powf(0.45) - 0.0992968)
    }
}

fn xyz_d50_to_lab(v: [f32; 3]) -> [f32; 3] {
    let white = [0.96422, 1.0, 0.82521];
    let f = |x: f32| {
        let x = x;
        if x > 216.0 / 24389.0 {
            x.cbrt()
        } else {
            (24389.0 / 27.0 * x + 16.0) / 116.0
        }
    };
    let x = f(v[0] / white[0]);
    let y = f(v[1]);
    let z = f(v[2] / white[2]);
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}
fn lab_to_xyz_d50(v: [f32; 3]) -> [f32; 3] {
    let f1 = (v[0] + 16.0) / 116.0;
    let f0 = v[1] / 500.0 + f1;
    let f2 = f1 - v[2] / 200.0;
    let inv = |x: f32| {
        if x.powi(3) > 216.0 / 24389.0 {
            x.powi(3)
        } else {
            (116.0 * x - 16.0) / (24389.0 / 27.0)
        }
    };
    [inv(f0) * 0.96422, inv(f1), inv(f2) * 0.82521]
}
fn linear_srgb_to_oklab(v: [f32; 3]) -> [f32; 3] {
    let l = 0.41222146 * v[0] + 0.53633255 * v[1] + 0.051445995 * v[2];
    let m = 0.2119035 * v[0] + 0.6806995 * v[1] + 0.10739696 * v[2];
    let s = 0.08830246 * v[0] + 0.28171885 * v[1] + 0.6299787 * v[2];
    let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
    [
        0.21045426 * l + 0.7936178 * m - 0.004072047 * s,
        1.9779985 * l - 2.4285922 * m + 0.4505937 * s,
        0.025904037 * l + 0.78277177 * m - 0.80867577 * s,
    ]
}
fn oklab_to_linear_srgb(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] + 0.39633778 * v[1] + 0.21580376 * v[2]).powi(3);
    let m = (v[0] - 0.105561346 * v[1] - 0.06385417 * v[2]).powi(3);
    let s = (v[0] - 0.08948418 * v[1] - 1.2914855 * v[2]).powi(3);
    [
        4.0767417 * l - 3.3077116 * m + 0.23096994 * s,
        -1.268438 * l + 2.6097574 * m - 0.34131938 * s,
        -0.0041960863 * l - 0.7034186 * m + 1.7076147 * s,
    ]
}
fn lab_to_lch(v: [f32; 3]) -> [f32; 3] {
    [
        v[0],
        v[1].hypot(v[2]),
        v[2].atan2(v[1]).to_degrees().rem_euclid(360.0),
    ]
}
fn lch_to_lab(v: [f32; 3]) -> [f32; 3] {
    let h = v[2].to_radians();
    [v[0], v[1] * h.cos(), v[1] * h.sin()]
}

fn srgb_to_hsl(v: [f32; 3]) -> [f32; 3] {
    let max = v[0].max(v[1]).max(v[2]);
    let min = v[0].min(v[1]).min(v[2]);
    let lightness = (max + min) * 0.5;
    let delta = max - min;
    if delta == 0.0 {
        return [0.0, 0.0, lightness];
    }
    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let sector = if max == v[0] {
        ((v[1] - v[2]) / delta).rem_euclid(6.0)
    } else if max == v[1] {
        (v[2] - v[0]) / delta + 2.0
    } else {
        (v[0] - v[1]) / delta + 4.0
    };
    [sector * 60.0, saturation, lightness]
}

fn hsl_to_srgb(v: [f32; 3]) -> [f32; 3] {
    if v[1] == 0.0 {
        return [v[2]; 3];
    }
    let f = |offset: f32| {
        let k = (offset + v[0] / 30.0).rem_euclid(12.0);
        let a = v[1] * v[2].min(1.0 - v[2]);
        v[2] - a * (-1.0_f32).max((k - 3.0).min((9.0 - k).min(1.0)))
    };
    [f(0.0), f(8.0), f(4.0)]
}

fn srgb_to_hwb(v: [f32; 3]) -> [f32; 3] {
    [
        srgb_to_hsl(v)[0],
        v[0].min(v[1]).min(v[2]),
        1.0 - v[0].max(v[1]).max(v[2]),
    ]
}

fn hwb_to_srgb(v: [f32; 3]) -> [f32; 3] {
    if v[1] + v[2] >= 1.0 {
        return [v[1] / (v[1] + v[2]); 3];
    }
    let mut rgb = hsl_to_srgb([v[0], 1.0, 0.5]);
    for channel in &mut rgb {
        *channel += v[1] - (v[1] + v[2]) * *channel;
    }
    rgb
}

pub fn srgb_to_space(rgba: [f32; 4], space: GradientInterpolationSpace) -> [f32; 4] {
    let rgb = [rgba[0], rgba[1], rgba[2]];
    let linear = rgb.map(decode);
    let xyz65 = mul(SRGB_TO_XYZ_D65, linear);
    let c = match space {
        GradientInterpolationSpace::None | GradientInterpolationSpace::Srgb => rgb,
        GradientInterpolationSpace::SrgbLegacy => rgb.map(|channel| channel * 255.0),
        GradientInterpolationSpace::SrgbLinear => linear,
        GradientInterpolationSpace::DisplayP3 => mul(XYZ_D65_TO_P3, xyz65).map(encode),
        GradientInterpolationSpace::DisplayP3Linear => mul(XYZ_D65_TO_P3, xyz65),
        GradientInterpolationSpace::Hsl => srgb_to_hsl(rgb),
        GradientInterpolationSpace::Hwb => srgb_to_hwb(rgb),
        GradientInterpolationSpace::A98Rgb => {
            mul(inverse(A98_TO_XYZ_D50), mul(D65_TO_D50, xyz65)).map(a98_encode)
        }
        GradientInterpolationSpace::ProPhotoRgb => {
            mul(inverse(PROPHOTO_TO_XYZ_D50), mul(D65_TO_D50, xyz65)).map(prophoto_encode)
        }
        GradientInterpolationSpace::Rec2020 => {
            mul(inverse(REC2020_TO_XYZ_D50), mul(D65_TO_D50, xyz65)).map(rec2020_encode)
        }
        GradientInterpolationSpace::Rec2100Linear => {
            mul(inverse(REC2020_TO_XYZ_D50), mul(D65_TO_D50, xyz65))
        }
        GradientInterpolationSpace::XyzD65 => xyz65,
        GradientInterpolationSpace::XyzD50 => mul(D65_TO_D50, xyz65),
        GradientInterpolationSpace::Lab => xyz_d50_to_lab(mul(D65_TO_D50, xyz65)),
        GradientInterpolationSpace::Lch => lab_to_lch(xyz_d50_to_lab(mul(D65_TO_D50, xyz65))),
        GradientInterpolationSpace::Oklab => linear_srgb_to_oklab(linear),
        GradientInterpolationSpace::Oklch => lab_to_lch(linear_srgb_to_oklab(linear)),
    };
    [c[0], c[1], c[2], rgba[3]]
}
pub fn space_to_srgb(rgba: [f32; 4], space: GradientInterpolationSpace) -> [f32; 4] {
    let c = [rgba[0], rgba[1], rgba[2]];
    let rgb = match space {
        GradientInterpolationSpace::None | GradientInterpolationSpace::Srgb => c,
        GradientInterpolationSpace::SrgbLegacy => c.map(|channel| channel / 255.0),
        GradientInterpolationSpace::SrgbLinear => c.map(encode),
        GradientInterpolationSpace::DisplayP3 => {
            mul(XYZ_D65_TO_SRGB, mul(P3_TO_XYZ_D65, c.map(decode))).map(encode)
        }
        GradientInterpolationSpace::DisplayP3Linear => {
            mul(XYZ_D65_TO_SRGB, mul(P3_TO_XYZ_D65, c)).map(encode)
        }
        GradientInterpolationSpace::Hsl => hsl_to_srgb(c),
        GradientInterpolationSpace::Hwb => hwb_to_srgb(c),
        GradientInterpolationSpace::A98Rgb => mul(
            XYZ_D65_TO_SRGB,
            mul(D50_TO_D65, mul(A98_TO_XYZ_D50, c.map(a98_decode))),
        )
        .map(encode),
        GradientInterpolationSpace::ProPhotoRgb => mul(
            XYZ_D65_TO_SRGB,
            mul(D50_TO_D65, mul(PROPHOTO_TO_XYZ_D50, c.map(prophoto_decode))),
        )
        .map(encode),
        GradientInterpolationSpace::Rec2020 => mul(
            XYZ_D65_TO_SRGB,
            mul(D50_TO_D65, mul(REC2020_TO_XYZ_D50, c.map(rec2020_decode))),
        )
        .map(encode),
        GradientInterpolationSpace::Rec2100Linear => {
            mul(XYZ_D65_TO_SRGB, mul(D50_TO_D65, mul(REC2020_TO_XYZ_D50, c))).map(encode)
        }
        GradientInterpolationSpace::XyzD65 => mul(XYZ_D65_TO_SRGB, c).map(encode),
        GradientInterpolationSpace::XyzD50 => mul(XYZ_D65_TO_SRGB, mul(D50_TO_D65, c)).map(encode),
        GradientInterpolationSpace::Lab => {
            mul(XYZ_D65_TO_SRGB, mul(D50_TO_D65, lab_to_xyz_d50(c))).map(encode)
        }
        GradientInterpolationSpace::Lch => mul(
            XYZ_D65_TO_SRGB,
            mul(D50_TO_D65, lab_to_xyz_d50(lch_to_lab(c))),
        )
        .map(encode),
        GradientInterpolationSpace::Oklab => oklab_to_linear_srgb(c).map(encode),
        GradientInterpolationSpace::Oklch => oklab_to_linear_srgb(lch_to_lab(c)).map(encode),
    };
    [rgb[0], rgb[1], rgb[2], rgba[3]]
}

pub fn prepare(rgba: [f32; 4], space: GradientInterpolationSpace) -> [f32; 4] {
    let mut c = srgb_to_space(rgba, space);
    let a = c[3];
    let hue_index = if matches!(
        space,
        GradientInterpolationSpace::Hsl | GradientInterpolationSpace::Hwb
    ) {
        0
    } else {
        2
    };
    for i in 0..3 {
        if !(space.has_hue() && i == hue_index) {
            c[i] *= a;
        }
    }
    c
}
pub fn finish(
    mut rgba: [f32; 4],
    space: GradientInterpolationSpace,
    premultiplied: bool,
) -> [f32; 4] {
    let a = rgba[3];
    let hue_index = if matches!(
        space,
        GradientInterpolationSpace::Hsl | GradientInterpolationSpace::Hwb
    ) {
        0
    } else {
        2
    };
    if premultiplied && a != 0.0 {
        for i in 0..3 {
            if !(space.has_hue() && i == hue_index) {
                rgba[i] /= a;
            }
        }
    }
    space_to_srgb(rgba, space)
}
pub fn adjust_hue(left: f32, right: f32, method: GradientHueMethod) -> (f32, f32) {
    let mut l = left;
    let mut r = right;
    let d = r - l;
    match method {
        GradientHueMethod::Shorter => {
            if d > 180.0 {
                l += 360.0
            } else if d < -180.0 {
                r += 360.0
            }
        }
        GradientHueMethod::Longer => {
            if 0.0 < d && d < 180.0 {
                l += 360.0
            } else if -180.0 < d && d <= 0.0 {
                r += 360.0
            }
        }
        GradientHueMethod::Increasing => {
            if r < l {
                r += 360.0
            }
        }
        GradientHueMethod::Decreasing => {
            if l < r {
                l += 360.0
            }
        }
    }
    (l, r)
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

fn component_kinds(space: GradientInterpolationSpace) -> [ComponentKind; 3] {
    use ComponentKind::*;
    match space {
        GradientInterpolationSpace::None
        | GradientInterpolationSpace::Srgb
        | GradientInterpolationSpace::SrgbLegacy
        | GradientInterpolationSpace::SrgbLinear
        | GradientInterpolationSpace::DisplayP3
        | GradientInterpolationSpace::DisplayP3Linear
        | GradientInterpolationSpace::A98Rgb
        | GradientInterpolationSpace::ProPhotoRgb
        | GradientInterpolationSpace::Rec2020
        | GradientInterpolationSpace::Rec2100Linear
        | GradientInterpolationSpace::XyzD50
        | GradientInterpolationSpace::XyzD65 => [RedX, GreenY, BlueZ],
        GradientInterpolationSpace::Lab | GradientInterpolationSpace::Oklab => [Lightness, A, B],
        GradientInterpolationSpace::Lch | GradientInterpolationSpace::Oklch => {
            [Lightness, Colorfulness, Hue]
        }
        GradientInterpolationSpace::Hsl => [Hue, Colorfulness, Lightness],
        GradientInterpolationSpace::Hwb => [Hue, Whiteness, Blackness],
    }
}

fn analogous_missing(
    source: GradientInterpolationSpace,
    flags: u8,
    target: GradientInterpolationSpace,
) -> u8 {
    let source = component_kinds(source);
    let target = component_kinds(target);
    let mut result = flags & 8;
    for source_index in 0..3 {
        if flags & (1 << source_index) == 0 {
            continue;
        }
        for target_index in 0..3 {
            if source[source_index] == target[target_index] {
                result |= 1 << target_index;
            }
        }
    }
    result
}

/// Converts one adjacent stop pair, performs CSS Color 4 missing-component
/// carry forward, adjusts hue, and conditionally premultiplies interpolation
/// channels. The operation is pair-local because a shared stop can inherit a
/// different component from the interval on each side.
pub fn prepare_pair(
    left: [f32; 4],
    left_source: GradientInterpolationSpace,
    left_missing: u8,
    right: [f32; 4],
    right_source: GradientInterpolationSpace,
    right_missing: u8,
    space: GradientInterpolationSpace,
    hue: GradientHueMethod,
    premultiplied: bool,
) -> ([f32; 4], [f32; 4]) {
    let convert = |components, source| {
        if source == space {
            components
        } else if source == GradientInterpolationSpace::None {
            srgb_to_space(components, space)
        } else {
            srgb_to_space(space_to_srgb(components, source), space)
        }
    };
    let mut left = convert(left, left_source);
    let mut right = convert(right, right_source);
    let left_missing = analogous_missing(left_source, left_missing, space);
    let right_missing = analogous_missing(right_source, right_missing, space);
    for index in 0..4 {
        match (
            left_missing & (1 << index) != 0,
            right_missing & (1 << index) != 0,
        ) {
            (true, false) => left[index] = right[index],
            (false, true) => right[index] = left[index],
            (true, true) => {
                left[index] = 0.0;
                right[index] = 0.0;
            }
            (false, false) => {}
        }
    }
    let hue_index = if matches!(
        space,
        GradientInterpolationSpace::Hsl | GradientInterpolationSpace::Hwb
    ) {
        0
    } else {
        2
    };
    if space.has_hue() {
        (left[hue_index], right[hue_index]) = adjust_hue(left[hue_index], right[hue_index], hue);
    }
    if premultiplied {
        for color in [&mut left, &mut right] {
            let alpha = color[3];
            for index in 0..3 {
                if !(space.has_hue() && index == hue_index) {
                    color[index] *= alpha;
                }
            }
        }
    }
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_spaces_round_trip_and_hue_direction_is_retained() {
        let source = [0.82, 0.14, 0.63, 0.7];
        for space in [
            GradientInterpolationSpace::Srgb,
            GradientInterpolationSpace::SrgbLinear,
            GradientInterpolationSpace::DisplayP3,
            GradientInterpolationSpace::Hsl,
            GradientInterpolationSpace::Hwb,
            GradientInterpolationSpace::A98Rgb,
            GradientInterpolationSpace::ProPhotoRgb,
            GradientInterpolationSpace::Rec2020,
            GradientInterpolationSpace::Lab,
            GradientInterpolationSpace::Lch,
            GradientInterpolationSpace::Oklab,
            GradientInterpolationSpace::Oklch,
            GradientInterpolationSpace::XyzD50,
            GradientInterpolationSpace::XyzD65,
        ] {
            let output = space_to_srgb(srgb_to_space(source, space), space);
            for channel in 0..3 {
                assert!(
                    (output[channel] - source[channel]).abs() < 3e-4,
                    "{space:?}"
                );
            }
        }
        assert_eq!(
            adjust_hue(350.0, 10.0, GradientHueMethod::Shorter),
            (350.0, 370.0)
        );
        assert_eq!(
            adjust_hue(350.0, 10.0, GradientHueMethod::Decreasing),
            (350.0, 10.0)
        );
        assert_eq!(
            adjust_hue(10.0, 350.0, GradientHueMethod::Increasing),
            (10.0, 350.0)
        );

        let (left, right) = prepare_pair(
            [0.0, 1.0, 0.5, 1.0],
            GradientInterpolationSpace::Hsl,
            1,
            [120.0, 1.0, 0.5, 1.0],
            GradientInterpolationSpace::Hsl,
            0,
            GradientInterpolationSpace::Hsl,
            GradientHueMethod::Shorter,
            true,
        );
        assert!(
            (left[0] - right[0]).abs() < 1e-4,
            "missing hue must carry from the adjacent stop"
        );
    }
}
