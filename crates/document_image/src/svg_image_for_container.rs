//! SVGImageForContainer per-use producer values. Source .h:65-86,
//! .cc:79-104,141-146; svg_image.cc:267-281,362-380,460-479,515-540.
use image_resource::ContainerKey;
use std::io;
pub struct SVGImageContainerUse {
    pub container: ContainerKey,
    pub image_size: (u32, u32),
}
impl SVGImageContainerUse {
    /// Source Create uses float reciprocal/multiply, then DrawInfo independently
    /// rounds the frame and outer image. Never borrow a previous use's viewport.
    /// An isolated SVG image has no enclosing SVG context-fill/context-stroke
    /// argument: those paints remain in its internal SVG marker/use owners.
    pub fn Create(
        target_size: (i32, i32),
        zoom: f32,
        view_fragment: &str,
        preferred_color_scheme: u8,
    ) -> io::Result<Self> {
        Self::CreateFloating(
            (target_size.0 as f32, target_size.1 as f32),
            zoom,
            view_fragment,
            preferred_color_scheme,
        )
    }
    pub fn CreateFloating(
        target_size: (f32, f32),
        zoom: f32,
        view_fragment: &str,
        preferred_color_scheme: u8,
    ) -> io::Result<Self> {
        if !target_size.0.is_finite()
            || !target_size.1.is_finite()
            || target_size.0 < 0.0
            || target_size.1 < 0.0
            || !zoom.is_finite()
            || zoom <= 0.0
        {
            return Err(io::Error::other(
                "invalid SVGImageForContainer source geometry",
            ));
        }
        let inverse_zoom = 1.0_f32 / zoom;
        let width = target_size.0 * inverse_zoom;
        let height = target_size.1 * inverse_zoom;
        let rounded = |v: f32| v.round().max(0.0) as i32 as u32;
        if !width.is_finite() || !height.is_finite() {
            return Err(io::Error::other(
                "SVG image container exceeds float geometry",
            ));
        }
        // SVGImage's non-ordinary Page has the native EmptyChromeClient screen
        // scale 1; DrawInfo zoom belongs only to the outer image operation.
        let mut container =
            ContainerKey::new(rounded(width), rounded(height), f64::from(zoom), 1.0);
        container.unrounded_size_bits =
            Some([f64::from(width).to_bits(), f64::from(height).to_bits()]);
        container.view_specification = view_fragment.to_owned();
        container.preferred_color_scheme = preferred_color_scheme;
        container.preferred_color_scheme_is_explicit = true;
        Ok(Self {
            container,
            image_size: (rounded(width * zoom), rounded(height * zoom)),
        })
    }
}
/// SVGImage::PaintRecordForCurrentFrame sets a LayoutUnit-floor SVG root
/// container independently from its rounded LocalFrameView viewport.
pub(crate) fn RootContainerSize(
    container: &ContainerKey,
) -> layoutng_assembly::internal::layout_input::Size {
    let (w, h) = container.unrounded_size();
    layoutng_assembly::internal::layout_input::Size {
        width: foundation::LayoutUnit::FromFloatFloor(w as f32).ToDouble(),
        height: foundation::LayoutUnit::FromFloatFloor(h as f32).ToDouble(),
    }
}
/// Stable SvgImageNonUniformScalingFix: source computes the non-scaling-stroke
/// scale from src_to_dst × residual_scale ÷ zoom (same float operation order).
pub(crate) fn ContainerScale(container: &ContainerKey) -> [f32; 2] {
    if container.unrounded_size_bits.is_none() {
        return [1.0, 1.0];
    }
    let (w, h) = container.unrounded_size();
    let zoom = container.zoom() as f32;
    let scale = |value: f32, rounded: u32| {
        if value == 0.0 || rounded == 0 {
            1.0
        } else {
            let dest = (value * zoom).round();
            let src_to_dst = dest / rounded as f32;
            let residual = rounded as f32 / value;
            (src_to_dst * residual) / zoom
        }
    };
    [
        scale(w as f32, container.width),
        scale(h as f32, container.height),
    ]
}
