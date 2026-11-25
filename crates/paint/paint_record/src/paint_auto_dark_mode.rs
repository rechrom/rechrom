//! Paint-time image classification for forced dark mode.
//!
//! Source: third_party/blink/renderer/core/paint/paint_auto_dark_mode.{cc,h}.

#![allow(non_snake_case)]

use crate::PaintRect;
use layout::input::PaintStyle;
use paint_input_types::layout_input_types::Color;
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

pub use paint_model::dark_mode_filter::{ElementRole, InvertColorIfEnabled};

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
    use paint_model::dark_mode_filter::ContrastRatio;

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
        use paint_input_types::paint_input::PaintShader;
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
