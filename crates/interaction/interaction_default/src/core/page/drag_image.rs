//! DragImage's reusable image/font policy, core/page/drag_image.cc:78-98,129-269.
//! Raster remains an injected Paint/Raster leaf.
use font_engine::{
    plain_text_painter::{PlainTextPaint, PlainTextPainter},
    FontDescription, FontFamily, FontFamilyType, FontMetrics, FontSelectionValue,
};
use foundation::TextDirection;

// Preserve the existing import paths for controller-owned image policy.
pub use super::drag_controller::{ClampedImageScale, DragRectForImage};

// Source: drag_image.cc:78-98. The controller supplies its maximum image size.
pub(super) fn ClampedImageScaleForSize(
    image: (u32, u32),
    element: (i32, i32),
    max: (f32, f32),
) -> (f32, f32) {
    let clamp_x = if element.0 as f32 > max.0 {
        max.0 / element.0 as f32
    } else {
        1.
    };
    let clamp_y = if element.1 as f32 > max.1 {
        max.1 / element.1 as f32
    } else {
        1.
    };
    let clamp = clamp_x.min(clamp_y);
    (
        element.0 as f32 / image.0 as f32 * clamp,
        element.1 as f32 / image.1 as f32 * clamp,
    )
}
pub trait DragLabelFontContext {
    type Error;
    fn Metrics(&self, description: &FontDescription) -> Result<FontMetrics, Self::Error>;
    fn Shape(
        &self,
        text: &[u16],
        description: &FontDescription,
        direction: TextDirection,
        bidi: bool,
    ) -> Result<PlainTextPaint, Self::Error>;
}
pub struct LinkDragImage {
    pub size: (i32, i32),
    pub text: Vec<((f32, f32), PlainTextPaint)>,
}
pub fn CreateLinkDragImage<C: DragLabelFontContext>(
    context: &C,
    url: &str,
    label: &[u16],
    device_scale: f32,
) -> Result<LinkDragImage, C::Error> {
    let description = |size, weight| {
        let mut d = FontDescription::default();
        d.SetFamily(&FontFamily::new(
            foundation::AtomicString::from_utf16(
                &font_engine::plain_text_painter::DragLabelSystemFontFamily()
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            ),
            FontFamilyType::kFamilyName,
            None,
        ));
        d.SetWeight(FontSelectionValue::from_float(weight));
        d.SetSpecifiedSize(size);
        d.SetComputedSize(size);
        d
    };
    let label_font = description(11., 700.);
    let url_font = description(10., 400.);
    let label_metrics = context.Metrics(&label_font)?;
    let url_metrics = context.Metrics(&url_font)?;
    let start = label
        .iter()
        .position(|c| !foundation::unicode::IsSpaceOrNewline(*c))
        .unwrap_or(label.len());
    let end = label
        .iter()
        .rposition(|c| !foundation::unicode::IsSpaceOrNewline(*c))
        .map_or(start, |i| i + 1);
    let mut label = label[start..end].to_vec();
    let mut url: Vec<u16> = url.encode_utf16().collect();
    let draw_url = !label.is_empty();
    if !draw_url {
        label = url.clone();
    }
    let max = 292. / device_scale;
    let label_width = context
        .Shape(&label, &label_font, TextDirection::kLtr, true)?
        .width as i32;
    let clip_label = label_width as f32 > max;
    let label_width = if clip_label { max as i32 } else { label_width };
    let mut size = (
        label_width + 8,
        label_metrics.Ascent() + label_metrics.Descent() + 4,
    );
    let mut text = Vec::new();
    if draw_url {
        let width = context
            .Shape(&url, &url_font, TextDirection::kLtr, false)?
            .width as i32;
        size.1 += url_metrics.Ascent() + url_metrics.Descent();
        let clip = width as f32 > max;
        size.0 = if clip {
            max as i32
        } else {
            label_width.max(width) + 8
        };
        if clip {
            url = font_engine::string_truncator::TruncateString(
                &url,
                size.0 as f32 - 8.,
                font_engine::string_truncator::Truncation::Center,
                |t| {
                    context
                        .Shape(t, &url_font, TextDirection::kLtr, true)
                        .map(|p| p.width)
                },
            )?;
        }
        text.push((
            (4., (size.1 - 2 - url_metrics.Descent()) as f32),
            context.Shape(&url, &url_font, TextDirection::kLtr, false)?,
        ));
    }
    if clip_label {
        label = font_engine::string_truncator::TruncateString(
            &label,
            size.0 as f32 - 8.,
            font_engine::string_truncator::Truncation::Right,
            |t| {
                context
                    .Shape(t, &label_font, TextDirection::kLtr, true)
                    .map(|p| p.width)
            },
        )?;
    }
    let direction = PlainTextPainter::BaseDirection(&label);
    let paint = context.Shape(&label, &label_font, direction, true)?;
    let x = if direction == TextDirection::kRtl {
        (size.0 - 8) as f32 - paint.width.ceil()
    } else {
        4.
    };
    text.push(((x, 13.), paint));
    Ok(LinkDragImage { size, text })
}
