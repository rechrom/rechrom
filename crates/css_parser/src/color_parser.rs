#![allow(non_snake_case)]

use layoutng::internal::layout_input_types::Color;

// cpp: css_parser/color_parser.cc:17-20
#[derive(Clone, Copy)]
struct NamedColor {
    name: &'static str,
    argb: u32,
}

// cpp: css_parser/color_parser.cc:22-178
const NAMED_COLORS: [NamedColor; 151] = [
    NamedColor {
        name: "aliceblue",
        argb: 0xfff0f8ff,
    },
    NamedColor {
        name: "antiquewhite",
        argb: 0xfffaebd7,
    },
    NamedColor {
        name: "aqua",
        argb: 0xff00ffff,
    },
    NamedColor {
        name: "aquamarine",
        argb: 0xff7fffd4,
    },
    NamedColor {
        name: "azure",
        argb: 0xfff0ffff,
    },
    NamedColor {
        name: "beige",
        argb: 0xfff5f5dc,
    },
    NamedColor {
        name: "bisque",
        argb: 0xffffe4c4,
    },
    NamedColor {
        name: "black",
        argb: 0xff000000,
    },
    NamedColor {
        name: "blanchedalmond",
        argb: 0xffffebcd,
    },
    NamedColor {
        name: "blue",
        argb: 0xff0000ff,
    },
    NamedColor {
        name: "blueviolet",
        argb: 0xff8a2be2,
    },
    NamedColor {
        name: "brown",
        argb: 0xffa52a2a,
    },
    NamedColor {
        name: "burlywood",
        argb: 0xffdeb887,
    },
    NamedColor {
        name: "cadetblue",
        argb: 0xff5f9ea0,
    },
    NamedColor {
        name: "chartreuse",
        argb: 0xff7fff00,
    },
    NamedColor {
        name: "chocolate",
        argb: 0xffd2691e,
    },
    NamedColor {
        name: "coral",
        argb: 0xffff7f50,
    },
    NamedColor {
        name: "cornflowerblue",
        argb: 0xff6495ed,
    },
    NamedColor {
        name: "cornsilk",
        argb: 0xfffff8dc,
    },
    NamedColor {
        name: "crimson",
        argb: 0xffdc143c,
    },
    NamedColor {
        name: "cyan",
        argb: 0xff00ffff,
    },
    NamedColor {
        name: "darkblue",
        argb: 0xff00008b,
    },
    NamedColor {
        name: "darkcyan",
        argb: 0xff008b8b,
    },
    NamedColor {
        name: "darkgoldenrod",
        argb: 0xffb8860b,
    },
    NamedColor {
        name: "darkgray",
        argb: 0xffa9a9a9,
    },
    NamedColor {
        name: "darkgreen",
        argb: 0xff006400,
    },
    NamedColor {
        name: "darkgrey",
        argb: 0xffa9a9a9,
    },
    NamedColor {
        name: "darkkhaki",
        argb: 0xffbdb76b,
    },
    NamedColor {
        name: "darkmagenta",
        argb: 0xff8b008b,
    },
    NamedColor {
        name: "darkolivegreen",
        argb: 0xff556b2f,
    },
    NamedColor {
        name: "darkorange",
        argb: 0xffff8c00,
    },
    NamedColor {
        name: "darkorchid",
        argb: 0xff9932cc,
    },
    NamedColor {
        name: "darkred",
        argb: 0xff8b0000,
    },
    NamedColor {
        name: "darksalmon",
        argb: 0xffe9967a,
    },
    NamedColor {
        name: "darkseagreen",
        argb: 0xff8fbc8f,
    },
    NamedColor {
        name: "darkslateblue",
        argb: 0xff483d8b,
    },
    NamedColor {
        name: "darkslategray",
        argb: 0xff2f4f4f,
    },
    NamedColor {
        name: "darkslategrey",
        argb: 0xff2f4f4f,
    },
    NamedColor {
        name: "darkturquoise",
        argb: 0xff00ced1,
    },
    NamedColor {
        name: "darkviolet",
        argb: 0xff9400d3,
    },
    NamedColor {
        name: "deeppink",
        argb: 0xffff1493,
    },
    NamedColor {
        name: "deepskyblue",
        argb: 0xff00bfff,
    },
    NamedColor {
        name: "dimgray",
        argb: 0xff696969,
    },
    NamedColor {
        name: "dimgrey",
        argb: 0xff696969,
    },
    NamedColor {
        name: "dodgerblue",
        argb: 0xff1e90ff,
    },
    NamedColor {
        name: "firebrick",
        argb: 0xffb22222,
    },
    NamedColor {
        name: "floralwhite",
        argb: 0xfffffaf0,
    },
    NamedColor {
        name: "forestgreen",
        argb: 0xff228b22,
    },
    NamedColor {
        name: "fuchsia",
        argb: 0xffff00ff,
    },
    NamedColor {
        name: "gainsboro",
        argb: 0xffdcdcdc,
    },
    NamedColor {
        name: "ghostwhite",
        argb: 0xfff8f8ff,
    },
    NamedColor {
        name: "gold",
        argb: 0xffffd700,
    },
    NamedColor {
        name: "goldenrod",
        argb: 0xffdaa520,
    },
    NamedColor {
        name: "gray",
        argb: 0xff808080,
    },
    NamedColor {
        name: "green",
        argb: 0xff008000,
    },
    NamedColor {
        name: "greenyellow",
        argb: 0xffadff2f,
    },
    NamedColor {
        name: "grey",
        argb: 0xff808080,
    },
    NamedColor {
        name: "honeydew",
        argb: 0xfff0fff0,
    },
    NamedColor {
        name: "hotpink",
        argb: 0xffff69b4,
    },
    NamedColor {
        name: "indianred",
        argb: 0xffcd5c5c,
    },
    NamedColor {
        name: "indigo",
        argb: 0xff4b0082,
    },
    NamedColor {
        name: "ivory",
        argb: 0xfffffff0,
    },
    NamedColor {
        name: "khaki",
        argb: 0xfff0e68c,
    },
    NamedColor {
        name: "lavender",
        argb: 0xffe6e6fa,
    },
    NamedColor {
        name: "lavenderblush",
        argb: 0xfffff0f5,
    },
    NamedColor {
        name: "lawngreen",
        argb: 0xff7cfc00,
    },
    NamedColor {
        name: "lemonchiffon",
        argb: 0xfffffacd,
    },
    NamedColor {
        name: "lightblue",
        argb: 0xffadd8e6,
    },
    NamedColor {
        name: "lightcoral",
        argb: 0xfff08080,
    },
    NamedColor {
        name: "lightcyan",
        argb: 0xffe0ffff,
    },
    NamedColor {
        name: "lightgoldenrodyellow",
        argb: 0xfffafad2,
    },
    NamedColor {
        name: "lightgray",
        argb: 0xffd3d3d3,
    },
    NamedColor {
        name: "lightgreen",
        argb: 0xff90ee90,
    },
    NamedColor {
        name: "lightgrey",
        argb: 0xffd3d3d3,
    },
    NamedColor {
        name: "lightpink",
        argb: 0xffffb6c1,
    },
    NamedColor {
        name: "lightsalmon",
        argb: 0xffffa07a,
    },
    NamedColor {
        name: "lightseagreen",
        argb: 0xff20b2aa,
    },
    NamedColor {
        name: "lightskyblue",
        argb: 0xff87cefa,
    },
    NamedColor {
        name: "lightslateblue",
        argb: 0xff8470ff,
    },
    NamedColor {
        name: "lightslategray",
        argb: 0xff778899,
    },
    NamedColor {
        name: "lightslategrey",
        argb: 0xff778899,
    },
    NamedColor {
        name: "lightsteelblue",
        argb: 0xffb0c4de,
    },
    NamedColor {
        name: "lightyellow",
        argb: 0xffffffe0,
    },
    NamedColor {
        name: "lime",
        argb: 0xff00ff00,
    },
    NamedColor {
        name: "limegreen",
        argb: 0xff32cd32,
    },
    NamedColor {
        name: "linen",
        argb: 0xfffaf0e6,
    },
    NamedColor {
        name: "magenta",
        argb: 0xffff00ff,
    },
    NamedColor {
        name: "maroon",
        argb: 0xff800000,
    },
    NamedColor {
        name: "mediumaquamarine",
        argb: 0xff66cdaa,
    },
    NamedColor {
        name: "mediumblue",
        argb: 0xff0000cd,
    },
    NamedColor {
        name: "mediumorchid",
        argb: 0xffba55d3,
    },
    NamedColor {
        name: "mediumpurple",
        argb: 0xff9370db,
    },
    NamedColor {
        name: "mediumseagreen",
        argb: 0xff3cb371,
    },
    NamedColor {
        name: "mediumslateblue",
        argb: 0xff7b68ee,
    },
    NamedColor {
        name: "mediumspringgreen",
        argb: 0xff00fa9a,
    },
    NamedColor {
        name: "mediumturquoise",
        argb: 0xff48d1cc,
    },
    NamedColor {
        name: "mediumvioletred",
        argb: 0xffc71585,
    },
    NamedColor {
        name: "midnightblue",
        argb: 0xff191970,
    },
    NamedColor {
        name: "mintcream",
        argb: 0xfff5fffa,
    },
    NamedColor {
        name: "mistyrose",
        argb: 0xffffe4e1,
    },
    NamedColor {
        name: "moccasin",
        argb: 0xffffe4b5,
    },
    NamedColor {
        name: "navajowhite",
        argb: 0xffffdead,
    },
    NamedColor {
        name: "navy",
        argb: 0xff000080,
    },
    NamedColor {
        name: "oldlace",
        argb: 0xfffdf5e6,
    },
    NamedColor {
        name: "olive",
        argb: 0xff808000,
    },
    NamedColor {
        name: "olivedrab",
        argb: 0xff6b8e23,
    },
    NamedColor {
        name: "orange",
        argb: 0xffffa500,
    },
    NamedColor {
        name: "orangered",
        argb: 0xffff4500,
    },
    NamedColor {
        name: "orchid",
        argb: 0xffda70d6,
    },
    NamedColor {
        name: "palegoldenrod",
        argb: 0xffeee8aa,
    },
    NamedColor {
        name: "palegreen",
        argb: 0xff98fb98,
    },
    NamedColor {
        name: "paleturquoise",
        argb: 0xffafeeee,
    },
    NamedColor {
        name: "palevioletred",
        argb: 0xffdb7093,
    },
    NamedColor {
        name: "papayawhip",
        argb: 0xffffefd5,
    },
    NamedColor {
        name: "peachpuff",
        argb: 0xffffdab9,
    },
    NamedColor {
        name: "peru",
        argb: 0xffcd853f,
    },
    NamedColor {
        name: "pink",
        argb: 0xffffc0cb,
    },
    NamedColor {
        name: "plum",
        argb: 0xffdda0dd,
    },
    NamedColor {
        name: "powderblue",
        argb: 0xffb0e0e6,
    },
    NamedColor {
        name: "purple",
        argb: 0xff800080,
    },
    NamedColor {
        name: "rebeccapurple",
        argb: 0xff663399,
    },
    NamedColor {
        name: "red",
        argb: 0xffff0000,
    },
    NamedColor {
        name: "rosybrown",
        argb: 0xffbc8f8f,
    },
    NamedColor {
        name: "royalblue",
        argb: 0xff4169e1,
    },
    NamedColor {
        name: "saddlebrown",
        argb: 0xff8b4513,
    },
    NamedColor {
        name: "salmon",
        argb: 0xfffa8072,
    },
    NamedColor {
        name: "sandybrown",
        argb: 0xfff4a460,
    },
    NamedColor {
        name: "seagreen",
        argb: 0xff2e8b57,
    },
    NamedColor {
        name: "seashell",
        argb: 0xfffff5ee,
    },
    NamedColor {
        name: "sienna",
        argb: 0xffa0522d,
    },
    NamedColor {
        name: "silver",
        argb: 0xffc0c0c0,
    },
    NamedColor {
        name: "skyblue",
        argb: 0xff87ceeb,
    },
    NamedColor {
        name: "slateblue",
        argb: 0xff6a5acd,
    },
    NamedColor {
        name: "slategray",
        argb: 0xff708090,
    },
    NamedColor {
        name: "slategrey",
        argb: 0xff708090,
    },
    NamedColor {
        name: "snow",
        argb: 0xfffffafa,
    },
    NamedColor {
        name: "springgreen",
        argb: 0xff00ff7f,
    },
    NamedColor {
        name: "steelblue",
        argb: 0xff4682b4,
    },
    NamedColor {
        name: "tan",
        argb: 0xffd2b48c,
    },
    NamedColor {
        name: "teal",
        argb: 0xff008080,
    },
    NamedColor {
        name: "thistle",
        argb: 0xffd8bfd8,
    },
    NamedColor {
        name: "tomato",
        argb: 0xffff6347,
    },
    NamedColor {
        name: "transparent",
        argb: 0x00000000,
    },
    NamedColor {
        name: "turquoise",
        argb: 0xff40e0d0,
    },
    NamedColor {
        name: "violet",
        argb: 0xffee82ee,
    },
    NamedColor {
        name: "violetred",
        argb: 0xffd02090,
    },
    NamedColor {
        name: "wheat",
        argb: 0xfff5deb3,
    },
    NamedColor {
        name: "white",
        argb: 0xffffffff,
    },
    NamedColor {
        name: "whitesmoke",
        argb: 0xfff5f5f5,
    },
    NamedColor {
        name: "yellow",
        argb: 0xffffff00,
    },
    NamedColor {
        name: "yellowgreen",
        argb: 0xff9acd32,
    },
];

// cpp: css_parser/color_parser.cc:180-194
fn TrimLower(input: &str) -> String {
    input
        .trim_matches(|c: char| c.is_ascii_whitespace())
        .to_ascii_lowercase()
}

// cpp: css_parser/color_parser.cc:196-203
fn Number(input: &str) -> Option<f64> {
    let value = TrimLower(input);
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    let begin = bytes.as_ptr().cast::<std::ffi::c_char>();
    let mut end = std::ptr::null_mut();
    unsafe extern "C" {
        fn strtod(begin: *const std::ffi::c_char, end: *mut *mut std::ffi::c_char) -> f64;
    }
    let number = unsafe { strtod(begin, &mut end) };
    let consumed = unsafe { end.offset_from(begin) };
    if consumed == 0 || consumed as usize != value.len() || !number.is_finite() {
        return None;
    }
    Some(number)
}

// cpp: css_parser/color_parser.cc:205-209
fn Percentage(input: &str) -> Option<f64> {
    let value = TrimLower(input);
    Number(value.strip_suffix('%')?)
}

// cpp: css_parser/color_parser.cc:211-233
fn AngleDegrees(input: &str, allow_unitless: bool) -> Option<f64> {
    let value = TrimLower(input);
    const PI: f64 = std::f64::consts::PI;
    if let Some(number) = value.strip_suffix("deg") {
        return Number(number);
    }
    if let Some(number) = value.strip_suffix("grad") {
        if let Some(number) = Number(number) {
            return Some(number * 0.9);
        }
    } else if let Some(number) = value.strip_suffix("rad") {
        if let Some(number) = Number(number) {
            return Some(number * 180.0 / PI);
        }
    } else if let Some(number) = value.strip_suffix("turn") {
        if let Some(number) = Number(number) {
            return Some(number * 360.0);
        }
    } else if allow_unitless {
        return Number(&value);
    }
    None
}

// cpp: css_parser/color_parser.cc:235-246
const fn LegacyRgbChannel(value: u32) -> f32 {
    (value as f64 / 255.0) as f32
}
fn FromArgb(argb: u32) -> Color {
    Color {
        red: LegacyRgbChannel((argb >> 16) & 0xFF),
        green: LegacyRgbChannel((argb >> 8) & 0xFF),
        blue: LegacyRgbChannel(argb & 0xFF),
        alpha: ((argb >> 24) & 0xFF) as f32 / 255.0,
    }
}

// cpp: css_parser/color_parser.cc:248-277
fn HexDigit(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some(u32::from(c - b'0')),
        b'a'..=b'f' => Some(u32::from(c - b'a' + 10)),
        _ => None,
    }
}
fn ParseHex(value: &str) -> Option<Color> {
    let bytes = value.as_bytes();
    if bytes.first().copied() != Some(b'#') {
        return None;
    }
    let digits = bytes.len() - 1;
    if !matches!(digits, 3 | 4 | 6 | 8) {
        return None;
    }
    let mut channels = [0_u32, 0, 0, 255];
    let count = if digits == 3 || digits == 6 { 3 } else { 4 };
    for index in 0..count {
        if digits <= 4 {
            let nibble = HexDigit(bytes[index + 1])?;
            channels[index] = nibble * 17;
        } else {
            let high = HexDigit(bytes[1 + index * 2])?;
            let low = HexDigit(bytes[2 + index * 2])?;
            channels[index] = high * 16 + low;
        }
    }
    Some(Color {
        red: LegacyRgbChannel(channels[0]),
        green: LegacyRgbChannel(channels[1]),
        blue: LegacyRgbChannel(channels[2]),
        alpha: channels[3] as f32 / 255.0,
    })
}

// cpp: css_parser/color_parser.cc:279-289
fn Split(input: &str, delimiter: char) -> Vec<String> {
    input.split(delimiter).map(TrimLower).collect()
}

// cpp: css_parser/color_parser.cc:291-326
struct ModernArguments {
    components: Vec<String>,
    alpha: Option<String>,
}
fn ParseModernArguments(input: &str) -> Option<ModernArguments> {
    let bytes = input.as_bytes();
    let mut result = ModernArguments {
        components: Vec::new(),
        alpha: None,
    };
    let mut index = 0;
    let mut saw_slash = false;
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index == bytes.len() {
            break;
        }
        if bytes[index] == b'/' {
            if saw_slash {
                return None;
            }
            saw_slash = true;
            index += 1;
            continue;
        }
        let begin = index;
        while index < bytes.len() && bytes[index] != b'/' && !bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let token = TrimLower(&input[begin..index]);
        if token.is_empty() {
            return None;
        }
        if saw_slash {
            if result.alpha.is_some() {
                return None;
            }
            result.alpha = Some(token);
        } else {
            result.components.push(token);
        }
    }
    if saw_slash != result.alpha.is_some() {
        return None;
    }
    Some(result)
}

// cpp: css_parser/color_parser.cc:328-339
fn ParseAlpha(input: &str) -> Option<f32> {
    let value = if let Some(percentage) = Percentage(input) {
        percentage / 100.0
    } else {
        Number(input)?
    };
    let clamped = value.clamp(0.0, 1.0);
    Some(((clamped * 255.0).round() / 255.0) as f32)
}

// cpp: css_parser/color_parser.cc:341-347
fn ParseRgbChannel(input: &str) -> Option<f32> {
    let value = if let Some(percentage) = Percentage(input) {
        percentage / 100.0
    } else {
        Number(input)? / 255.0
    };
    Some(value.clamp(0.0, 1.0) as f32)
}

// cpp: css_parser/color_parser.cc:349-377
fn ParseRgb(input: &str) -> Option<Color> {
    let (components, alpha) = if input.contains(',') {
        let mut legacy = Split(input, ',');
        if !matches!(legacy.len(), 3 | 4) {
            return None;
        }
        let alpha = if legacy.len() == 4 {
            legacy.pop()
        } else {
            None
        };
        let percentage_channels = Percentage(&legacy[0]).is_some();
        if Percentage(&legacy[1]).is_some() != percentage_channels
            || Percentage(&legacy[2]).is_some() != percentage_channels
        {
            return None;
        }
        (legacy, alpha)
    } else {
        let modern = ParseModernArguments(input)?;
        if modern.components.len() != 3 {
            return None;
        }
        (modern.components, modern.alpha)
    };
    let red = ParseRgbChannel(&components[0])?;
    let green = ParseRgbChannel(&components[1])?;
    let blue = ParseRgbChannel(&components[2])?;
    let resolved_alpha = alpha.as_deref().map(ParseAlpha).unwrap_or(Some(1.0))?;
    Some(Color {
        red,
        green,
        blue,
        alpha: resolved_alpha,
    })
}

// cpp: css_parser/color_parser.cc:379-398
fn Hsl(mut hue: f64, saturation: f64, lightness: f64, alpha: f32) -> Color {
    hue %= 360.0;
    if hue < 0.0 {
        hue += 360.0;
    }
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let section = hue / 60.0;
    let x = chroma * (1.0 - (section % 2.0 - 1.0).abs());
    let (mut red, mut green, mut blue) = (0.0, 0.0, 0.0);
    if section < 1.0 {
        red = chroma;
        green = x;
    } else if section < 2.0 {
        red = x;
        green = chroma;
    } else if section < 3.0 {
        green = chroma;
        blue = x;
    } else if section < 4.0 {
        green = x;
        blue = chroma;
    } else if section < 5.0 {
        red = x;
        blue = chroma;
    } else {
        red = chroma;
        blue = x;
    }
    let match_value = lightness - chroma / 2.0;
    Color {
        red: (red + match_value) as f32,
        green: (green + match_value) as f32,
        blue: (blue + match_value) as f32,
        alpha,
    }
}

// cpp: css_parser/color_parser.cc:400-426
fn ParseHsl(input: &str) -> Option<Color> {
    let (components, alpha) = if input.contains(',') {
        let mut legacy = Split(input, ',');
        if !matches!(legacy.len(), 3 | 4) {
            return None;
        }
        let alpha = if legacy.len() == 4 {
            legacy.pop()
        } else {
            None
        };
        (legacy, alpha)
    } else {
        let modern = ParseModernArguments(input)?;
        if modern.components.len() != 3 {
            return None;
        }
        (modern.components, modern.alpha)
    };
    let hue = AngleDegrees(&components[0], true)?;
    let saturation = Percentage(&components[1])?;
    let lightness = Percentage(&components[2])?;
    let resolved_alpha = alpha.as_deref().map(ParseAlpha).unwrap_or(Some(1.0))?;
    Some(Hsl(
        hue,
        (saturation / 100.0).clamp(0.0, 1.0),
        (lightness / 100.0).clamp(0.0, 1.0),
        resolved_alpha,
    ))
}

// cpp: css_parser/color_parser.cc:428-449
fn ParseHwb(input: &str) -> Option<Color> {
    let arguments = ParseModernArguments(input)?;
    if arguments.components.len() != 3 {
        return None;
    }
    let hue = AngleDegrees(&arguments.components[0], true)?;
    let whiteness = Percentage(&arguments.components[1])?;
    let blackness = Percentage(&arguments.components[2])?;
    let alpha = arguments
        .alpha
        .as_deref()
        .map(ParseAlpha)
        .unwrap_or(Some(1.0))?;
    let white = (whiteness / 100.0).clamp(0.0, 1.0);
    let black = (blackness / 100.0).clamp(0.0, 1.0);
    if white + black >= 1.0 {
        let gray = (white / (white + black)) as f32;
        return Some(Color {
            red: gray,
            green: gray,
            blue: gray,
            alpha,
        });
    }
    let mut pure = Hsl(hue, 1.0, 0.5, alpha);
    let factor = 1.0 - white - black;
    pure.red = (f64::from(pure.red) * factor + white) as f32;
    pure.green = (f64::from(pure.green) * factor + white) as f32;
    pure.blue = (f64::from(pure.blue) * factor + white) as f32;
    Some(pure)
}

// cpp: css_parser/color_parser.h:12-14
// cpp: css_parser/color_parser.cc:453-476
pub fn ParseCSSColor(input: &str) -> Option<Color> {
    let value = TrimLower(input);
    if value.is_empty() {
        return None;
    }
    if let Some(hex) = ParseHex(&value) {
        return Some(hex);
    }
    if let Ok(index) = NAMED_COLORS.binary_search_by_key(&value.as_str(), |entry| entry.name) {
        return Some(FromArgb(NAMED_COLORS[index].argb));
    }
    let open = value.find('(')?;
    if !value.ends_with(')')
        || value[open..].find(')').map(|position| open + position) != Some(value.len() - 1)
    {
        return None;
    }
    let name = &value[..open];
    let arguments = &value[open + 1..value.len() - 1];
    match name {
        "rgb" | "rgba" => ParseRgb(arguments),
        "hsl" | "hsla" => ParseHsl(arguments),
        "hwb" => ParseHwb(arguments),
        _ => None,
    }
}
