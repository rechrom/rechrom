//! Paint-time image classification for forced dark mode.
//!
//! Source: third_party/blink/renderer/core/paint/paint_auto_dark_mode.{cc,h}.

#![allow(non_snake_case)]

use crate::PaintRect;
use layoutng_assembly::internal::layout_input::PaintStyle;
use layoutng_assembly::internal::layout_input_types::Color;
use paint_model::DisplayMaskLayer;

const MAX_IMAGE_LENGTH: f64 = 64.0;
const MAX_IMAGE_SEPARATOR_LENGTH: f64 = 8.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageType {
    #[default]
    kNone,
    kIcon,
    kSeparator,
    kPhoto,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ElementRole {
    kForeground,
    kListSymbol,
    #[default]
    kBackground,
    kSVG,
    kBorder,
    kSelection,
}

const FOREGROUND_BRIGHTNESS_THRESHOLD: i32 = 150;
const BACKGROUND_BRIGHTNESS_THRESHOLD: i32 = 205;

fn ColorBrightness(color: Color) -> i32 {
    let byte = |component: f32| (component.clamp(0.0, 1.0) * 255.0).round() as i32;
    (byte(color.red) * 299 + byte(color.green) * 587 + byte(color.blue) * 114) / 1000
}

fn ShouldInvertColor(color: Color, role: ElementRole) -> bool {
    let brightness = ColorBrightness(color);
    match role {
        ElementRole::kBackground | ElementRole::kSelection => {
            brightness > BACKGROUND_BRIGHTNESS_THRESHOLD
        }
        ElementRole::kForeground
        | ElementRole::kListSymbol
        | ElementRole::kSVG
        | ElementRole::kBorder => brightness < FOREGROUND_BRIGHTNESS_THRESHOLD,
    }
}

fn Multiply3(matrix: [[f32; 3]; 3], value: [f32; 3]) -> [f32; 3] {
    [
        matrix[0][0] * value[0] + matrix[0][1] * value[1] + matrix[0][2] * value[2],
        matrix[1][0] * value[0] + matrix[1][1] * value[1] + matrix[1][2] * value[2],
        matrix[2][0] * value[0] + matrix[2][1] * value[1] + matrix[2][2] * value[2],
    ]
}

// Chromium's D65 -> D50 Bradford-adapted sRGB transform from
// dark_mode_lab_color_space.h.
const SRGB_TO_XYZ_D50: [[f32; 3]; 3] = [
    [0.43487942, 0.3820189, 0.14731362],
    [0.22212312, 0.7159545, 0.06192239],
    [0.0070117023, 0.07839328, 0.7397832],
];
const XYZ_D50_TO_SRGB: [[f32; 3]; 3] = [
    [3.134231, -1.6188616, -0.48861766],
    [-0.9787889, 1.9152106, 0.034597307],
    [0.074013814, -0.18760723, 1.3527123],
];
const ILLUMINANT_D50: [f32; 3] = [0.964212, 1.0, 0.825188];

fn SrgbToLab(color: Color) -> [f32; 3] {
    let linear = |component: f32| {
        if component < 0.04045 {
            (component / 12.92).clamp(0.0, 1.0)
        } else {
            (((component + 0.055) / 1.055).powf(2.4)).clamp(0.0, 1.0)
        }
    };
    let xyz = Multiply3(
        SRGB_TO_XYZ_D50,
        [linear(color.red), linear(color.green), linear(color.blue)],
    );
    const SIGMA_2: f32 = 36.0 / 841.0;
    const SIGMA_3: f32 = 216.0 / 24389.0;
    let lab_f = |value: f32| {
        if value > SIGMA_3 {
            value.cbrt()
        } else {
            value / (3.0 * SIGMA_2) + 4.0 / 29.0
        }
    };
    let fx = lab_f(xyz[0] / ILLUMINANT_D50[0]);
    let fy = lab_f(xyz[1] / ILLUMINANT_D50[1]);
    let fz = lab_f(xyz[2] / ILLUMINANT_D50[2]);
    [
        (116.0 * fy - 16.0).clamp(0.0, 100.0),
        (500.0 * (fx - fy)).clamp(-128.0, 128.0),
        (200.0 * (fy - fz)).clamp(-128.0, 128.0),
    ]
}

fn LabToSrgb(lab: [f32; 3], alpha: f32) -> Color {
    const SIGMA: f32 = 6.0 / 29.0;
    const SIGMA_2: f32 = 36.0 / 841.0;
    let inverse_f = |value: f32| {
        if value > SIGMA {
            value * value * value
        } else {
            3.0 * SIGMA_2 * (value - 4.0 / 29.0)
        }
    };
    let lab = [
        lab[0].clamp(0.0, 100.0),
        lab[1].clamp(-128.0, 128.0),
        lab[2].clamp(-128.0, 128.0),
    ];
    let xyz = [
        inverse_f((lab[0] + 16.0) / 116.0 + lab[1] * 0.002) * ILLUMINANT_D50[0],
        inverse_f((lab[0] + 16.0) / 116.0) * ILLUMINANT_D50[1],
        inverse_f((lab[0] + 16.0) / 116.0 - lab[2] * 0.005) * ILLUMINANT_D50[2],
    ];
    let linear = Multiply3(XYZ_D50_TO_SRGB, xyz);
    let srgb = |component: f32| {
        if component < 0.0031308 {
            (12.92 * component).clamp(0.0, 1.0)
        } else {
            (1.055 * component.powf(1.0 / 2.4) - 0.055).clamp(0.0, 1.0)
        }
    };
    Color {
        red: srgb(linear[0]),
        green: srgb(linear[1]),
        blue: srgb(linear[2]),
        alpha,
    }
}

fn RelativeLuminance(color: Color) -> f32 {
    let linearize = |component: f32| {
        if component <= 0.04045 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linearize(color.red) + 0.7152 * linearize(color.green) + 0.0722 * linearize(color.blue)
}

fn ContrastRatio(lhs: Color, rhs: Color) -> f32 {
    let lhs = RelativeLuminance(lhs) + 0.05;
    let rhs = RelativeLuminance(rhs) + 0.05;
    lhs.max(rhs) / lhs.min(rhs)
}

fn QuantizeTo8Bit(value: f32) -> f32 {
    let scale = f32::from_bits(256.0f32.to_bits() - 1);
    (value * scale) as i32 as f32 / 255.0
}

fn Darken(color: Color) -> Color {
    let value = color.red.max(color.green).max(color.blue);
    let multiplier = if value == 0.0 {
        0.0
    } else {
        (value - 0.33).max(0.0) / value
    };
    Color {
        red: QuantizeTo8Bit(color.red * multiplier),
        green: QuantizeTo8Bit(color.green * multiplier),
        blue: QuantizeTo8Bit(color.blue * multiplier),
        alpha: color.alpha,
    }
}

fn EffectiveContrastBackground(background: Color) -> Color {
    if background == Color::default() {
        Color {
            red: 18.0 / 255.0,
            green: 18.0 / 255.0,
            blue: 18.0 / 255.0,
            alpha: 1.0,
        }
    } else {
        background
    }
}

fn AdjustBorderColor(mut color: Color, background: Color) -> Color {
    let background = EffectiveContrastBackground(background);
    while color.red != 0.0 || color.green != 0.0 || color.blue != 0.0 {
        if ContrastRatio(color, background) < 4.5 {
            break;
        }
        color = Darken(color);
    }
    color
}

fn ColorToBytes(color: Color) -> [u8; 3] {
    let byte = |component: f32| (component.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(color.red), byte(color.green), byte(color.blue)]
}

fn QuantizeColor(color: Color) -> Color {
    let [red, green, blue] = ColorToBytes(color);
    Color {
        red: f32::from(red) / 255.0,
        green: f32::from(green) / 255.0,
        blue: f32::from(blue) / 255.0,
        alpha: f32::from((color.alpha.clamp(0.0, 1.0) * 255.0).round() as u8) / 255.0,
    }
}

fn RgbToHsv(rgb: [u8; 3]) -> [f32; 3] {
    let [red, green, blue] = rgb;
    let minimum = red.min(green).min(blue);
    let maximum = red.max(green).max(blue);
    let delta = maximum - minimum;
    let value = f32::from(maximum) / 255.0;
    if delta == 0 {
        return [0.0, 0.0, value];
    }
    let saturation = f32::from(delta) / f32::from(maximum);
    let mut hue = if red == maximum {
        f32::from(green) - f32::from(blue)
    } else if green == maximum {
        2.0 * f32::from(delta) + f32::from(blue) - f32::from(red)
    } else {
        4.0 * f32::from(delta) + f32::from(red) - f32::from(green)
    } / f32::from(delta)
        * 60.0;
    if hue < 0.0 {
        hue += 360.0;
    }
    [hue, saturation, value]
}

fn HsvToColor(hsv: [f32; 3], alpha: f32) -> Color {
    let saturation = hsv[1].clamp(0.0, 1.0);
    let value = hsv[2].clamp(0.0, 1.0);
    let value_byte = (value * 255.0).round() as u8;
    let [red, green, blue] = if saturation.abs() <= 1.0 / 4096.0 {
        [value_byte; 3]
    } else {
        let hue = if (0.0..360.0).contains(&hsv[0]) {
            hsv[0] / 60.0
        } else {
            0.0
        };
        let sector = hue.floor() as u8;
        let fraction = hue - hue.floor();
        let channel = |component: f32| (component * value * 255.0).round() as u8;
        let p = channel(1.0 - saturation);
        let q = channel(1.0 - saturation * fraction);
        let t = channel(1.0 - saturation * (1.0 - fraction));
        match sector {
            0 => [value_byte, t, p],
            1 => [q, value_byte, p],
            2 => [p, value_byte, t],
            3 => [p, q, value_byte],
            4 => [t, p, value_byte],
            _ => [value_byte, p, q],
        }
    };
    Color {
        red: f32::from(red) / 255.0,
        green: f32::from(green) / 255.0,
        blue: f32::from(blue) / 255.0,
        alpha: f32::from((alpha.clamp(0.0, 1.0) * 255.0) as u8) / 255.0,
    }
}

fn AdjustSelectionColor(color: Color, background: Color) -> Color {
    let background = EffectiveContrastBackground(background);
    if ContrastRatio(color, background) >= 3.0 {
        return color;
    }
    let lab = SrgbToLab(color);
    let original_hue = RgbToHsv(ColorToBytes(color))[0];
    let mut best = color;
    let mut low = lab[0] as i32;
    let mut high = 101;
    while low < high {
        let lightness = (low + high) / 2;
        let adjusted = LabToSrgb([lightness as f32, lab[1], lab[2]], color.alpha);
        let mut hsv = RgbToHsv(ColorToBytes(adjusted));
        if hsv[0] != original_hue {
            hsv[0] = original_hue;
        }
        let adjusted = HsvToColor(hsv, color.alpha);
        if ContrastRatio(adjusted, background) > 3.0 {
            high = lightness;
            best = adjusted;
        } else {
            // This deliberately matches Chromium's current binary-search exit.
            low = high + 1;
        }
    }
    best
}

fn InvertLabColorIfNeeded(color: Color, role: ElementRole) -> Color {
    if !ShouldInvertColor(color, role) {
        return color;
    }
    let mut lab = SrgbToLab(color);
    lab[0] = (110.0 - lab[0]).min(100.0);
    let mut inverted = LabToSrgb(lab, color.alpha);
    // SkM44 builds the Bradford matrix at runtime. The equivalent flattened
    // 3x3 constants can differ by one final f32 ULP, so admit that ULP while
    // preserving DarkModeColorFilter's exact near-gray intent.
    let gray_epsilon = 2.0 * f32::EPSILON;
    let gray = (inverted.red - inverted.green).abs() < gray_epsilon
        && (inverted.red - inverted.blue).abs() < gray_epsilon;
    let adjusted = 18.0 / 255.0;
    if gray && inverted.red < 32.0 / 255.0 && inverted.red > adjusted {
        inverted.red = adjusted;
        inverted.green = adjusted;
        inverted.blue = adjusted;
    }
    inverted
}

/// The solid-color counterpart of Chromium's AutoDarkMode(role, enabled,
/// contrast_color) wrapper.
pub fn InvertColorIfEnabled(
    enabled: bool,
    contrast_background: Color,
    color: Color,
    role: ElementRole,
) -> Color {
    if !enabled {
        return color;
    }
    let inverted = InvertLabColorIfNeeded(color, role);
    let contrast_background =
        InvertLabColorIfNeeded(QuantizeColor(contrast_background), ElementRole::kBackground);
    match role {
        ElementRole::kBorder => AdjustBorderColor(inverted, contrast_background),
        ElementRole::kSelection => AdjustSelectionColor(inverted, contrast_background),
        _ => inverted,
    }
}

/// DarkModeFilter::InvertColorIfNeeded with the thresholds returned by
/// GetCurrentDarkModeSettings in this Chromium revision.
pub fn InvertColorIfNeeded(style: &PaintStyle, color: Color, role: ElementRole) -> Color {
    InvertColorIfEnabled(
        style.force_dark() && !style.auto_dark_mode_paused(),
        style.background_color(),
        color,
        role,
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageAutoDarkMode {
    pub role: ElementRole,
    pub enabled: bool,
    pub image_type: ImageType,
}

impl ImageAutoDarkMode {
    pub fn Disabled() -> Self {
        Self::default()
    }

    /// GraphicsContext first admits only icon/separator classifications, then
    /// the raster-side image decoder performs the pixel-content classification.
    pub fn ShouldUseRasterDarkMode(self) -> bool {
        self.enabled && matches!(self.image_type, ImageType::kIcon | ImageType::kSeparator)
    }
}

fn EnclosingLength(origin: f64, length: f64) -> f64 {
    (origin + length).ceil() - origin.floor()
}

fn GetImageType(destination: PaintRect, source: PaintRect) -> ImageType {
    let destination_width = EnclosingLength(destination.x, destination.width);
    let destination_height = EnclosingLength(destination.y, destination.height);
    if destination_width <= MAX_IMAGE_LENGTH && destination_height <= MAX_IMAGE_LENGTH {
        return ImageType::kIcon;
    }

    let source_width = EnclosingLength(source.x, source.width);
    let source_height = EnclosingLength(source.y, source.height);
    if source_width <= MAX_IMAGE_SEPARATOR_LENGTH || source_height <= MAX_IMAGE_SEPARATOR_LENGTH {
        return ImageType::kSeparator;
    }

    ImageType::kPhoto
}

/// Undo only LocalFrame::LayoutZoomFactor from the destination. The source is
/// intrinsic image space and is intentionally left unchanged.
pub fn GetImageTypeWithZoom(
    layout_zoom: f64,
    mut destination: PaintRect,
    source: PaintRect,
) -> ImageType {
    if layout_zoom > 0.0 && layout_zoom != 1.0 {
        destination.x /= layout_zoom;
        destination.y /= layout_zoom;
        destination.width /= layout_zoom;
        destination.height /= layout_zoom;
    }
    GetImageType(destination, source)
}

pub fn GetImageAutoDarkMode(
    style: &PaintStyle,
    layout_zoom: f64,
    destination: PaintRect,
    source: PaintRect,
) -> ImageAutoDarkMode {
    GetImageAutoDarkModeWithRole(
        style,
        layout_zoom,
        destination,
        source,
        ElementRole::kBackground,
    )
}

/// Resolve the ImageAutoDarkMode passed by SVGMaskPainter/BoxPainterBase after
/// mask geometry has produced Chromium's final destination and intrinsic tile
/// rectangles. Bitmap layers carry the admission to image decoding; generated
/// gradient layers carry it on their shader. SVG <mask> resource records have
/// no FillLayer image source rectangle and are intentionally left unchanged.
pub fn ApplyMaskLayerImageAutoDarkMode(
    style: &PaintStyle,
    layout_zoom: f64,
    layers: &mut [DisplayMaskLayer],
) {
    for layer in layers {
        if layer.source_rect.width <= 0.0 || layer.source_rect.height <= 0.0 {
            continue;
        }
        let use_dark_mode =
            GetImageAutoDarkMode(style, layout_zoom, layer.clip_rect, layer.source_rect)
                .ShouldUseRasterDarkMode();
        layer.use_dark_mode_for_image = use_dark_mode;
        if use_dark_mode {
            if let Some(shader) = &mut layer.paint_shader {
                shader.apply_dark_mode = true;
            }
        }
    }
}

pub fn GetImageAutoDarkModeWithRole(
    style: &PaintStyle,
    layout_zoom: f64,
    destination: PaintRect,
    source: PaintRect,
    role: ElementRole,
) -> ImageAutoDarkMode {
    if !style.force_dark() {
        return ImageAutoDarkMode::Disabled();
    }
    ImageAutoDarkMode {
        role,
        enabled: true,
        image_type: GetImageTypeWithZoom(layout_zoom, destination, source),
    }
}

/// Chromium gates SVG document-image classification behind a runtime feature.
/// The caller supplies that feature value because Rechrom keeps feature policy
/// outside the backend-neutral Paint record package.
pub fn GetSVGDocumentType(
    layout_zoom: f64,
    size: (f64, f64),
    size_threshold_enabled: bool,
) -> ImageType {
    if !size_threshold_enabled {
        return ImageType::kIcon;
    }
    let scale = if layout_zoom > 0.0 { layout_zoom } else { 1.0 };
    if size.0 / scale <= MAX_IMAGE_LENGTH && size.1 / scale <= MAX_IMAGE_LENGTH {
        ImageType::kIcon
    } else {
        ImageType::kPhoto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, width: f64, height: f64) -> PaintRect {
        PaintRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn classifies_icons_separators_and_photos_in_enclosing_pixel_space() {
        let source = rect(0.0, 0.0, 100.0, 100.0);
        assert_eq!(
            GetImageTypeWithZoom(1.0, rect(0.0, 0.0, 64.0, 64.0), source),
            ImageType::kIcon
        );
        assert_eq!(
            GetImageTypeWithZoom(1.0, rect(0.5, 0.0, 64.0, 64.0), source),
            ImageType::kPhoto
        );
        assert_eq!(
            GetImageTypeWithZoom(
                1.0,
                rect(0.0, 0.0, 200.0, 200.0),
                rect(0.0, 0.0, 8.0, 100.0),
            ),
            ImageType::kSeparator
        );
    }

    #[test]
    fn layout_zoom_affects_only_destination_classification() {
        assert_eq!(
            GetImageTypeWithZoom(
                2.0,
                rect(0.0, 0.0, 128.0, 128.0),
                rect(0.0, 0.0, 100.0, 100.0),
            ),
            ImageType::kIcon
        );
        assert_eq!(
            GetImageTypeWithZoom(
                2.0,
                rect(0.0, 0.0, 200.0, 200.0),
                rect(0.0, 0.0, 8.0, 100.0),
            ),
            ImageType::kSeparator
        );
    }

    #[test]
    fn force_dark_and_svg_threshold_gate_the_result() {
        let mut style = PaintStyle::default();
        let destination = rect(0.0, 0.0, 32.0, 32.0);
        let source = rect(0.0, 0.0, 100.0, 100.0);
        assert_eq!(
            GetImageAutoDarkMode(&style, 1.0, destination, source),
            ImageAutoDarkMode::Disabled()
        );
        style.force_dark = true;
        assert!(GetImageAutoDarkMode(&style, 1.0, destination, source).ShouldUseRasterDarkMode());
        style.auto_dark_mode_paused = true;
        assert!(
            GetImageAutoDarkMode(&style, 1.0, destination, source).ShouldUseRasterDarkMode(),
            "image classification ignores an SVG container's paused solid-color state"
        );
        style.auto_dark_mode_paused = false;
        assert_eq!(
            GetImageAutoDarkModeWithRole(&style, 1.0, destination, source, ElementRole::kSVG,).role,
            ElementRole::kSVG
        );
        assert_eq!(
            GetSVGDocumentType(2.0, (128.0, 128.0), true),
            ImageType::kIcon
        );
        assert_eq!(
            GetSVGDocumentType(1.0, (1000.0, 1000.0), false),
            ImageType::kIcon
        );
    }

    #[test]
    fn mask_layers_classify_bitmap_and_generated_images_but_not_svg_mask_records() {
        use layoutng_assembly::internal::paint_input::PaintShader;
        use paint_model::DisplayMaskLayer;

        let mut style = PaintStyle::default();
        style.force_dark = true;
        let mut layers = vec![
            DisplayMaskLayer {
                resource_id: 7,
                clip_rect: rect(0.0, 0.0, 32.0, 32.0),
                source_rect: rect(0.0, 0.0, 100.0, 100.0),
                ..Default::default()
            },
            DisplayMaskLayer {
                paint_shader: Some(PaintShader::default()),
                clip_rect: rect(0.0, 0.0, 32.0, 32.0),
                source_rect: rect(0.0, 0.0, 32.0, 32.0),
                ..Default::default()
            },
            DisplayMaskLayer {
                paint_shader: Some(PaintShader::default()),
                clip_rect: rect(0.0, 0.0, 32.0, 32.0),
                ..Default::default()
            },
        ];

        ApplyMaskLayerImageAutoDarkMode(&style, 1.0, &mut layers);

        assert!(layers[0].use_dark_mode_for_image);
        assert!(layers[1].use_dark_mode_for_image);
        assert!(layers[1].paint_shader.as_ref().unwrap().apply_dark_mode);
        assert!(!layers[2].use_dark_mode_for_image);
        assert!(!layers[2].paint_shader.as_ref().unwrap().apply_dark_mode);
    }

    #[test]
    fn background_colors_use_chromiums_lab_lightness_inversion() {
        let mut style = PaintStyle::default();
        style.force_dark = true;
        let white = Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 0.5,
        };
        let inverted = InvertColorIfNeeded(&style, white, ElementRole::kBackground);
        assert!(
            (inverted.red - 18.0 / 255.0).abs() < 0.00001,
            "{inverted:?}"
        );
        assert!((inverted.green - 18.0 / 255.0).abs() < 0.00001);
        assert!((inverted.blue - 18.0 / 255.0).abs() < 0.00001);
        assert_eq!(inverted.alpha, 0.5);

        let black = Color {
            alpha: 1.0,
            ..Default::default()
        };
        assert_eq!(
            InvertColorIfNeeded(&style, black, ElementRole::kBackground),
            black
        );
        let foreground = InvertColorIfNeeded(&style, black, ElementRole::kForeground);
        assert!((foreground.red - 1.0).abs() < 0.00001);
        assert!((foreground.green - 1.0).abs() < 0.00001);
        assert!((foreground.blue - 1.0).abs() < 0.00001);

        let border = InvertColorIfNeeded(&style, black, ElementRole::kBorder);
        assert_eq!(border.red, 87.0 / 255.0);
        assert_eq!(border.green, 87.0 / 255.0);
        assert_eq!(border.blue, 87.0 / 255.0);

        let selection = InvertColorIfNeeded(&style, black, ElementRole::kSelection);
        let dark_background = Color {
            red: 18.0 / 255.0,
            green: 18.0 / 255.0,
            blue: 18.0 / 255.0,
            alpha: 1.0,
        };
        assert!(ContrastRatio(selection, dark_background) > 3.0);

        style.auto_dark_mode_paused = true;
        assert_eq!(
            InvertColorIfNeeded(&style, white, ElementRole::kBackground),
            white
        );
        style.auto_dark_mode_paused = false;

        style.force_dark = false;
        assert_eq!(
            InvertColorIfNeeded(&style, white, ElementRole::kBackground),
            white
        );
    }
}
