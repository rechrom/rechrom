//! Pixel-storage adapters for the translated render-surface filter pipeline.
//!
//! The public functions operate on caller-owned premultiplied RGBA8888
//! storage.  They are the CPU-renderer counterpart of Chromium's
//! `RenderSurfaceFilters::BuildImageFilter`: callers retain ownership of the
//! ordered CSS filter chain while this module owns Skia's pixel semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMatrixKind {
    Grayscale,
    Sepia,
    Saturate,
    HueRotate,
    Invert,
    Opacity,
    Brightness,
    Contrast,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderSurfaceFilter {
    /// Source: SkKnownRuntimeEffects.cpp:419-430; sksl_rt_shader.sksl:5-7.
    Luma,
    ColorMatrix([f32; 20]),
    /// cc/paint/paint_filter.cc:502-542: exact integer region mask value.
    AlphaThreshold {
        region: crate::include::core::SkRegion::SkRegion,
        local_to_device: [f32; 6],
    },
    /// SkImageFilters::Offset samples the immutable input in translated space.
    Offset {
        x: f32,
        y: f32,
    },
    Blur {
        sigma_x: f32,
        sigma_y: f32,
    },
    DropShadow {
        sigma_x: f32,
        sigma_y: f32,
        offset_x: f32,
        offset_y: f32,
        color: [f32; 4],
    },
    SvgGraph(RenderSvgFilterGraph),
    /// SVG graph evaluated in its axis-aligned filter raster space, then
    /// sampled through the complete affine surface-quad mapping. This keeps
    /// primitiveUnits/filterUnits local under rotation and skew.
    AffineSvgGraph {
        graph: RenderSvgFilterGraph,
        filter_to_device: [f32; 6],
    },
    /// Device-space affine matrix mapping source pixels to their reflected
    /// destination.  Chromium's BoxReflection matrix is self-inverse.
    BoxReflect {
        matrix: [f32; 6],
        mask: Option<RenderReflectionMask>,
    },
}

/// Raster-owned inputs for the private NinePieceImage PaintRecord used by
/// -webkit-box-reflect. Images are already selected for image-set/container
/// state; Skia only samples these immutable pixels in the effect coordinate
/// space. Reflection masks always consume alpha, never CSS mask luminance.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderReflectionMaskLayer {
    pub resource_id: u64,
    pub image_width: u32,
    pub image_height: u32,
    pub image_rgba: std::sync::Arc<Vec<u8>>,
    pub image_interpolation_quality: crate::compat::commands::ImageInterpolationQuality,
    pub paint_shader: Option<crate::compat::commands::PaintShader>,
    pub clip_rect: crate::compat::commands::PaintRect,
    pub source_rect: crate::compat::commands::PaintRect,
    pub tile_rect: crate::compat::commands::PaintRect,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub tile_spacing: crate::compat::commands::Size,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderReflectionMask {
    pub layers: Vec<RenderReflectionMaskLayer>,
    /// Local/property coordinates to root device coordinates.
    pub local_to_device: [f32; 6],
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSvgFilterGraph {
    pub primitives: Vec<RenderSvgFilterPrimitive>,
    pub region: [f32; 4],
    pub fill_paint: Option<[f32; 4]>,
    pub stroke_paint: Option<[f32; 4]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RenderSvgFilterPrimitive {
    pub input: String,
    pub result: String,
    pub region: [f32; 4],
    pub linear_rgb: bool,
    pub kind: RenderSvgFilterKind,
}
#[derive(Clone, Debug, PartialEq)]
pub enum RenderSvgFilterKind {
    GaussianBlur {
        x: f32,
        y: f32,
        edge_mode: RenderSvgEdgeMode,
    },
    ColorMatrix([f32; 20]),
    Offset {
        x: f32,
        y: f32,
    },
    Flood([f32; 4]),
    Blend {
        input2: String,
        mode: RenderSvgBlend,
    },
    Composite {
        input2: String,
        operator: RenderSvgComposite,
        k: [f32; 4],
    },
    Merge(Vec<String>),
    ComponentTransfer([RenderSvgTransfer; 4]),
    Morphology {
        dilate: bool,
        x: u32,
        y: u32,
    },
    DisplacementMap {
        input2: String,
        scale: f32,
        x_channel: usize,
        y_channel: usize,
    },
    Turbulence {
        base_x: f32,
        base_y: f32,
        octaves: u32,
        seed: f32,
        stitch: bool,
        fractal_noise: bool,
    },
    Tile,
    Image {
        href: String,
        local_fragment: bool,
        preserve_none: bool,
        width: u32,
        height: u32,
        pixels: std::sync::Arc<Vec<u8>>,
    },
    Convolve {
        order_x: u32,
        order_y: u32,
        kernel: Vec<f32>,
        divisor: f32,
        bias: f32,
        target_x: u32,
        target_y: u32,
        edge_mode: RenderSvgEdgeMode,
        unit_x: f32,
        unit_y: f32,
        preserve_alpha: bool,
    },
    Lighting {
        lighting_type: RenderSvgLightingType,
        light: Option<RenderSvgLight>,
        color: [f32; 3],
        surface_scale: f32,
        constant: f32,
        shininess: f32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderSvgLightingType {
    Diffuse,
    Specular,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderSvgLight {
    Distant {
        direction: [f32; 3],
    },
    Point {
        position: [f32; 3],
    },
    Spot {
        position: [f32; 3],
        target: [f32; 3],
        falloff_exponent: f32,
        limiting_cone_angle: f32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderSvgEdgeMode {
    None,
    Duplicate,
    Wrap,
}
#[derive(Clone, Debug, PartialEq)]
pub enum RenderSvgTransfer {
    Identity,
    Table(Vec<f32>),
    Discrete(Vec<f32>),
    Linear {
        slope: f32,
        intercept: f32,
    },
    Gamma {
        amplitude: f32,
        exponent: f32,
        offset: f32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderSvgBlend {
    Normal,
    Multiply,
    Screen,
    Darken,
    Lighten,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderSvgComposite {
    Over,
    In,
    Out,
    Atop,
    Xor,
    Arithmetic,
}

/// Resolve a logical SVG filter graph for a concrete pattern-tile pixel grid.
/// The document graph translator runs once at the Layout/Raster boundary;
/// pattern tiles apply their use-specific device scale here without rebuilding
/// or approximating the graph.
pub fn scale_svg_filter_graph(
    graph: &RenderSvgFilterGraph,
    scale_x: f32,
    scale_y: f32,
) -> RenderSvgFilterGraph {
    let mut graph = graph.clone();
    let scale_region = |region: &mut [f32; 4]| {
        region[0] *= scale_x;
        region[1] *= scale_y;
        region[2] *= scale_x;
        region[3] *= scale_y;
    };
    scale_region(&mut graph.region);
    let scalar = (scale_x.abs() * scale_y.abs()).sqrt();
    for primitive in &mut graph.primitives {
        scale_region(&mut primitive.region);
        match &mut primitive.kind {
            RenderSvgFilterKind::GaussianBlur { x, y, .. }
            | RenderSvgFilterKind::Offset { x, y } => {
                *x *= scale_x;
                *y *= scale_y;
            }
            RenderSvgFilterKind::Morphology { x, y, .. } => {
                *x = ((*x as f32) * scale_x).round().max(0.0) as u32;
                *y = ((*y as f32) * scale_y).round().max(0.0) as u32;
            }
            RenderSvgFilterKind::DisplacementMap { scale, .. } => *scale *= scalar,
            RenderSvgFilterKind::Convolve { unit_x, unit_y, .. } => {
                *unit_x *= scale_x;
                *unit_y *= scale_y;
            }
            RenderSvgFilterKind::Lighting {
                light,
                surface_scale,
                ..
            } => {
                *surface_scale *= scalar;
                if let Some(light) = light {
                    match light {
                        RenderSvgLight::Distant { .. } => {}
                        RenderSvgLight::Point { position } => {
                            position[0] *= scale_x;
                            position[1] *= scale_y;
                            position[2] *= scalar;
                        }
                        RenderSvgLight::Spot {
                            position, target, ..
                        } => {
                            for point in [position, target] {
                                point[0] *= scale_x;
                                point[1] *= scale_y;
                                point[2] *= scalar;
                            }
                        }
                    }
                }
            }
            RenderSvgFilterKind::ColorMatrix(_)
            | RenderSvgFilterKind::Flood(_)
            | RenderSvgFilterKind::Blend { .. }
            | RenderSvgFilterKind::Composite { .. }
            | RenderSvgFilterKind::Merge(_)
            | RenderSvgFilterKind::ComponentTransfer(_)
            | RenderSvgFilterKind::Turbulence { .. }
            | RenderSvgFilterKind::Tile
            | RenderSvgFilterKind::Image { .. } => {}
        }
    }
    graph
}

/// Chromium cc/paint/render_surface_filters.cc's exact matrices.
pub fn css_color_matrix(kind: ColorMatrixKind, amount: f32) -> [f32; 20] {
    let mut matrix = [0.0; 20];
    matrix[18] = 1.0;
    match kind {
        ColorMatrixKind::Grayscale => {
            let remaining = 1.0 - amount;
            matrix[0] = 0.2126 + 0.7874 * remaining;
            matrix[1] = 0.7152 - 0.7152 * remaining;
            matrix[2] = 0.0722 - 0.0722 * remaining;
            matrix[5] = 0.2126 - 0.2126 * remaining;
            matrix[6] = 0.7152 + 0.2848 * remaining;
            matrix[7] = 0.0722 - 0.0722 * remaining;
            matrix[10] = 0.2126 - 0.2126 * remaining;
            matrix[11] = 0.7152 - 0.7152 * remaining;
            matrix[12] = 0.0722 + 0.9278 * remaining;
        }
        ColorMatrixKind::Sepia => {
            let remaining = 1.0 - amount;
            matrix[0] = 0.393 + 0.607 * remaining;
            matrix[1] = 0.769 - 0.769 * remaining;
            matrix[2] = 0.189 - 0.189 * remaining;
            matrix[5] = 0.349 - 0.349 * remaining;
            matrix[6] = 0.686 + 0.314 * remaining;
            matrix[7] = 0.168 - 0.168 * remaining;
            matrix[10] = 0.272 - 0.272 * remaining;
            matrix[11] = 0.534 - 0.534 * remaining;
            matrix[12] = 0.131 + 0.869 * remaining;
        }
        ColorMatrixKind::Saturate => {
            matrix[0] = 0.213 + 0.787 * amount;
            matrix[1] = 0.715 - 0.715 * amount;
            matrix[2] = 0.072 - 0.072 * amount;
            matrix[5] = 0.213 - 0.213 * amount;
            matrix[6] = 0.715 + 0.285 * amount;
            matrix[7] = 0.072 - 0.072 * amount;
            matrix[10] = 0.213 - 0.213 * amount;
            matrix[11] = 0.715 - 0.715 * amount;
            matrix[12] = 0.072 + 0.928 * amount;
        }
        ColorMatrixKind::HueRotate => {
            let radians = amount.to_radians();
            let cos_hue = radians.cos();
            let sin_hue = radians.sin();
            matrix[0] = 0.213 + cos_hue * 0.787 - sin_hue * 0.213;
            matrix[1] = 0.715 - cos_hue * 0.715 - sin_hue * 0.715;
            matrix[2] = 0.072 - cos_hue * 0.072 + sin_hue * 0.928;
            matrix[5] = 0.213 - cos_hue * 0.213 + sin_hue * 0.143;
            matrix[6] = 0.715 + cos_hue * 0.285 + sin_hue * 0.140;
            matrix[7] = 0.072 - cos_hue * 0.072 - sin_hue * 0.283;
            matrix[10] = 0.213 - cos_hue * 0.213 - sin_hue * 0.787;
            matrix[11] = 0.715 - cos_hue * 0.715 + sin_hue * 0.715;
            matrix[12] = 0.072 + cos_hue * 0.928 + sin_hue * 0.072;
        }
        ColorMatrixKind::Invert => {
            matrix[0] = 1.0 - 2.0 * amount;
            matrix[4] = amount;
            matrix[6] = 1.0 - 2.0 * amount;
            matrix[9] = amount;
            matrix[12] = 1.0 - 2.0 * amount;
            matrix[14] = amount;
        }
        ColorMatrixKind::Opacity => {
            matrix[0] = 1.0;
            matrix[6] = 1.0;
            matrix[12] = 1.0;
            matrix[18] = amount;
        }
        ColorMatrixKind::Brightness => {
            matrix[0] = amount;
            matrix[6] = amount;
            matrix[12] = amount;
        }
        ColorMatrixKind::Contrast => {
            matrix[0] = amount;
            matrix[4] = -0.5 * amount + 0.5;
            matrix[6] = amount;
            matrix[9] = -0.5 * amount + 0.5;
            matrix[12] = amount;
            matrix[14] = -0.5 * amount + 0.5;
        }
    }
    matrix
}

/// Apply SkBlurEngine to caller-owned premultiplied RGBA8888 storage.
/// This is the render-pass counterpart of SkCanvas SaveLayerFilter restore.
pub fn blur_rgba(pixels: &mut [u8], width: u32, height: u32, sigma_x: f32, sigma_y: f32) -> bool {
    let Some(mut view) = crate::compat::pixel_view::PixelView::from_rgba(pixels, width, height)
    else {
        return false;
    };
    crate::src::core::SkBlurEngine::blur(&mut view, sigma_x, sigma_y);
    true
}

#[inline]
fn unit_to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round_ties_even() as u8
}

/// Apply one SkColorFilters::Matrix-compatible 4x5 matrix.  N32 color filters
/// evaluate their matrix in unpremultiplied color space and return
/// premultiplied storage; doing the conversion here is essential for opacity
/// and for translucent antialiased edges.
pub fn color_matrix_rgba(pixels: &mut [u8], matrix: &[f32; 20]) -> bool {
    if pixels.len() % 4 != 0 || !matrix.iter().all(|value| value.is_finite()) {
        return false;
    }
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = f32::from(pixel[3]) / 255.0;
        let (red, green, blue) = if alpha == 0.0 {
            (0.0, 0.0, 0.0)
        } else {
            (
                (f32::from(pixel[0]) / 255.0 / alpha).clamp(0.0, 1.0),
                (f32::from(pixel[1]) / 255.0 / alpha).clamp(0.0, 1.0),
                (f32::from(pixel[2]) / 255.0 / alpha).clamp(0.0, 1.0),
            )
        };
        let inputs = [red, green, blue, alpha, 1.0];
        let mut output = [0.0; 4];
        for row in 0..4 {
            let matrix = &matrix[row * 5..row * 5 + 5];
            // SkRasterPipeline's matrix_4x5 stage is a nested mad chain.
            // Preserve that evaluation order/FMA rounding before N32 packing.
            output[row] = inputs[0]
                .mul_add(
                    matrix[0],
                    inputs[1].mul_add(
                        matrix[1],
                        inputs[2].mul_add(matrix[2], inputs[3].mul_add(matrix[3], matrix[4])),
                    ),
                )
                .clamp(0.0, 1.0);
        }
        let output_alpha = output[3];
        pixel[0] = unit_to_byte(output[0] * output_alpha);
        pixel[1] = unit_to_byte(output[1] * output_alpha);
        pixel[2] = unit_to_byte(output[2] * output_alpha);
        pixel[3] = unit_to_byte(output_alpha);
    }
    true
}

#[inline]
fn sample_channel(
    pixels: &[u8],
    width: usize,
    height: usize,
    x: f32,
    y: f32,
    channel: usize,
) -> u8 {
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let tx = x - x.floor();
    let ty = y - y.floor();
    let sample = |x: i64, y: i64| -> f32 {
        if x < 0 || y < 0 || x >= width as i64 || y >= height as i64 {
            0.0
        } else {
            f32::from(pixels[(y as usize * width + x as usize) * 4 + channel])
        }
    };
    let top = sample(x0, y0) * (1.0 - tx) + sample(x0 + 1, y0) * tx;
    let bottom = sample(x0, y0 + 1) * (1.0 - tx) + sample(x0 + 1, y0 + 1) * tx;
    (top * (1.0 - ty) + bottom * ty)
        .clamp(0.0, 255.0)
        .round_ties_even() as u8
}

/// Apply SkImageFilters::DropShadow in kDrawShadowAndForeground mode.  The
/// shadow is generated from the current input alpha, colored in sRGB,
/// Gaussian-blurred, translated, and then placed below the unchanged input.
/// This makes the function composable in CSS list order.
pub fn drop_shadow_rgba(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    sigma_x: f32,
    sigma_y: f32,
    offset_x: f32,
    offset_y: f32,
    color: [f32; 4],
) -> bool {
    let (width, height) = (width as usize, height as usize);
    let Some(expected) = width
        .checked_mul(height)
        .and_then(|value| value.checked_mul(4))
    else {
        return false;
    };
    if pixels.len() != expected
        || ![sigma_x, sigma_y, offset_x, offset_y]
            .iter()
            .all(|value| value.is_finite())
        || sigma_x < 0.0
        || sigma_y < 0.0
        || !color.iter().all(|value| value.is_finite())
    {
        return false;
    }
    let color = color.map(|value| value.clamp(0.0, 1.0));
    // SkDropShadowImageFilter first blurs the source alpha and only then
    // applies the flood color.  Keeping the intermediate alpha-only avoids a
    // premature color quantization before the Gaussian convolution.
    let mut shadow = vec![0u8; expected];
    for (source, output) in pixels.chunks_exact(4).zip(shadow.chunks_exact_mut(4)) {
        output[3] = source[3];
    }
    if (sigma_x > 0.0 || sigma_y > 0.0)
        && !blur_rgba(&mut shadow, width as u32, height as u32, sigma_x, sigma_y)
    {
        return false;
    }
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) * 4;
            let source_alpha = pixels[index + 3];
            let inverse_alpha = 255 - source_alpha;
            let mask_alpha = f32::from(sample_channel(
                &shadow,
                width,
                height,
                x as f32 - offset_x,
                y as f32 - offset_y,
                3,
            )) / 255.0;
            let shadow_alpha = mask_alpha * color[3];
            let under = [
                unit_to_byte(color[0] * shadow_alpha),
                unit_to_byte(color[1] * shadow_alpha),
                unit_to_byte(color[2] * shadow_alpha),
                unit_to_byte(shadow_alpha),
            ];
            for channel in 0..4 {
                let remainder =
                    crate::cpu::mask_blitter::mul_div_255_round(under[channel], inverse_alpha);
                pixels[index + channel] = pixels[index + channel].saturating_add(remainder);
            }
        }
    }
    true
}

pub fn apply_rgba(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    filter: &RenderSurfaceFilter,
) -> bool {
    apply_rgba_at(pixels, width, height, filter, (0, 0))
}

fn affine_inverse(matrix: [f32; 6]) -> Option<[f32; 6]> {
    let determinant = matrix[0] * matrix[3] - matrix[2] * matrix[1];
    if !determinant.is_finite() || determinant.abs() <= f32::EPSILON {
        return None;
    }
    Some([
        matrix[3] / determinant,
        -matrix[1] / determinant,
        -matrix[2] / determinant,
        matrix[0] / determinant,
        (matrix[2] * matrix[5] - matrix[3] * matrix[4]) / determinant,
        (matrix[1] * matrix[4] - matrix[0] * matrix[5]) / determinant,
    ])
}

#[inline]
fn affine_point(matrix: [f32; 6], x: f32, y: f32) -> (f32, f32) {
    (
        matrix[0] * x + matrix[2] * y + matrix[4],
        matrix[1] * x + matrix[3] * y + matrix[5],
    )
}

fn bilinear_rgba(pixels: &[u8], width: usize, height: usize, x: f32, y: f32) -> [u8; 4] {
    let tx = x - 0.5;
    let ty = y - 0.5;
    let left = tx.floor() as i32;
    let top = ty.floor() as i32;
    let fx = tx - left as f32;
    let fy = ty - top as f32;
    let sample = |x: i32, y: i32, channel: usize| -> f32 {
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            0.0
        } else {
            f32::from(pixels[(y as usize * width + x as usize) * 4 + channel])
        }
    };
    std::array::from_fn(|channel| {
        let top_value =
            sample(left, top, channel) * (1.0 - fx) + sample(left + 1, top, channel) * fx;
        let bottom_value =
            sample(left, top + 1, channel) * (1.0 - fx) + sample(left + 1, top + 1, channel) * fx;
        (top_value * (1.0 - fy) + bottom_value * fy)
            .round_ties_even()
            .clamp(0.0, 255.0) as u8
    })
}

fn affine_svg_graph_rgba(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    graph: &RenderSvgFilterGraph,
    filter_to_device: [f32; 6],
    surface_origin: (i32, i32),
) -> bool {
    let (width, height) = (width as usize, height as usize);
    if pixels.len() != width.saturating_mul(height).saturating_mul(4) {
        return false;
    }
    // FilterResult::applyTransform concatenates integer translations without
    // resolving/resampling the image. This includes the identity mapping used
    // by axis-aligned device-space filters: only their region origin changes.
    if filter_to_device[..4] == [1.0, 0.0, 0.0, 1.0]
        && filter_to_device[4].fract() == 0.0
        && filter_to_device[5].fract() == 0.0
    {
        return svg_graph_rgba_at(
            pixels,
            width as u32,
            height as u32,
            graph,
            (
                surface_origin.0 as f32 - filter_to_device[4],
                surface_origin.1 as f32 - filter_to_device[5],
            ),
        );
    }
    let Some(device_to_filter) = affine_inverse(filter_to_device) else {
        return false;
    };
    let device_left = surface_origin.0 as f32;
    let device_top = surface_origin.1 as f32;
    let device_right = device_left + width as f32;
    let device_bottom = device_top + height as f32;
    let corners = [
        affine_point(device_to_filter, device_left, device_top),
        affine_point(device_to_filter, device_right, device_top),
        affine_point(device_to_filter, device_left, device_bottom),
        affine_point(device_to_filter, device_right, device_bottom),
    ];
    let left = corners
        .iter()
        .map(|point| point.0)
        .fold(f32::INFINITY, f32::min)
        .floor() as i32;
    let top = corners
        .iter()
        .map(|point| point.1)
        .fold(f32::INFINITY, f32::min)
        .floor() as i32;
    let right = corners
        .iter()
        .map(|point| point.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32;
    let bottom = corners
        .iter()
        .map(|point| point.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32;
    let Some(filter_width) = usize::try_from(right.saturating_sub(left)).ok() else {
        return false;
    };
    let Some(filter_height) = usize::try_from(bottom.saturating_sub(top)).ok() else {
        return false;
    };
    let Some(filter_len) = filter_width
        .checked_mul(filter_height)
        .and_then(|pixels| pixels.checked_mul(4))
        .filter(|&len| len > 0 && len <= isize::MAX as usize)
    else {
        return false;
    };
    let source = pixels.to_vec();
    let mut local = vec![0u8; filter_len];
    for y in 0..filter_height {
        for x in 0..filter_width {
            let filter = (left as f32 + x as f32 + 0.5, top as f32 + y as f32 + 0.5);
            let device = affine_point(filter_to_device, filter.0, filter.1);
            let sample = bilinear_rgba(
                &source,
                width,
                height,
                device.0 - device_left,
                device.1 - device_top,
            );
            local[(y * filter_width + x) * 4..(y * filter_width + x + 1) * 4]
                .copy_from_slice(&sample);
        }
    }
    let mut local_graph = graph.clone();
    local_graph.region[0] -= left as f32;
    local_graph.region[1] -= top as f32;
    for primitive in &mut local_graph.primitives {
        primitive.region[0] -= left as f32;
        primitive.region[1] -= top as f32;
    }
    if !svg_graph_rgba(
        &mut local,
        filter_width as u32,
        filter_height as u32,
        &local_graph,
    ) {
        return false;
    }
    for y in 0..height {
        for x in 0..width {
            let device = (device_left + x as f32 + 0.5, device_top + y as f32 + 0.5);
            let filter = affine_point(device_to_filter, device.0, device.1);
            let sample = bilinear_rgba(
                &local,
                filter_width,
                filter_height,
                filter.0 - left as f32,
                filter.1 - top as f32,
            );
            pixels[(y * width + x) * 4..(y * width + x + 1) * 4].copy_from_slice(&sample);
        }
    }
    true
}

/// Apply a filter to a cropped render surface. `surface_origin` is required
/// by reflection masks because their private PaintRecord stays in property
/// coordinates while effect surfaces are allocated at tight device bounds.
pub fn apply_rgba_at(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    filter: &RenderSurfaceFilter,
    surface_origin: (i32, i32),
) -> bool {
    match filter {
        RenderSurfaceFilter::Luma => {
            if pixels.len() % 4 != 0 {
                return false;
            }
            // The runtime color filter consumes premultiplied RGB, writes
            // black RGB and its Rec709 luminance as alpha.
            for pixel in pixels.chunks_exact_mut(4) {
                let a = (f32::from(pixel[0]) / 255.).mul_add(
                    0.2126,
                    (f32::from(pixel[1]) / 255.)
                        .mul_add(0.7152, (f32::from(pixel[2]) / 255.) * 0.0722),
                );
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                pixel[3] = unit_to_byte(a.clamp(0., 1.));
            }
            true
        }
        RenderSurfaceFilter::ColorMatrix(matrix) => color_matrix_rgba(pixels, matrix),
        RenderSurfaceFilter::Offset { x, y } => {
            if pixels.len() != width as usize * height as usize * 4 {
                return false;
            }
            let input = pixels.to_vec();
            for py in 0..height as usize {
                for px in 0..width as usize {
                    for channel in 0..4 {
                        pixels[(py * width as usize + px) * 4 + channel] = sample_channel(
                            &input,
                            width as usize,
                            height as usize,
                            px as f32 - *x,
                            py as f32 - *y,
                            channel,
                        );
                    }
                }
            }
            true
        }
        RenderSurfaceFilter::AlphaThreshold {
            region,
            local_to_device,
        } => {
            if pixels.len() != width as usize * height as usize * 4 {
                return false;
            }
            let Some(inverse) = affine_inverse(*local_to_device) else {
                pixels.fill(0);
                return true;
            };
            for py in 0..height as usize {
                for px in 0..width as usize {
                    let point = affine_point(
                        inverse,
                        px as f32 + surface_origin.0 as f32 + 0.5,
                        py as f32 + surface_origin.1 as f32 + 0.5,
                    );
                    if !region.contains_point(point.0.floor() as i32, point.1.floor() as i32) {
                        pixels[(py * width as usize + px) * 4..(py * width as usize + px + 1) * 4]
                            .fill(0);
                    }
                }
            }
            true
        }
        RenderSurfaceFilter::Blur { sigma_x, sigma_y } => {
            blur_rgba(pixels, width, height, *sigma_x, *sigma_y)
        }
        RenderSurfaceFilter::DropShadow {
            sigma_x,
            sigma_y,
            offset_x,
            offset_y,
            color,
        } => drop_shadow_rgba(
            pixels, width, height, *sigma_x, *sigma_y, *offset_x, *offset_y, *color,
        ),
        RenderSurfaceFilter::SvgGraph(graph) => svg_graph_rgba_at(
            pixels,
            width,
            height,
            graph,
            (surface_origin.0 as f32, surface_origin.1 as f32),
        ),
        RenderSurfaceFilter::AffineSvgGraph {
            graph,
            filter_to_device,
        } => affine_svg_graph_rgba(
            pixels,
            width,
            height,
            graph,
            *filter_to_device,
            surface_origin,
        ),
        RenderSurfaceFilter::BoxReflect { matrix, mask } => {
            let mut matrix = *matrix;
            let ox = surface_origin.0 as f32;
            let oy = surface_origin.1 as f32;
            matrix[4] += matrix[0] * ox + matrix[2] * oy - ox;
            matrix[5] += matrix[1] * ox + matrix[3] * oy - oy;
            let mask_alpha = mask.as_ref().map(|mask| {
                crate::compat::canvas::rasterize_reflection_mask_alpha(
                    mask,
                    width,
                    height,
                    surface_origin,
                )
            });
            box_reflect_rgba(pixels, width, height, matrix, mask_alpha.as_deref())
        }
    }
}

/// Build `SrcOver(Matrix(input), input)`, the pixel form of
/// paint_filter_builder::BuildBoxReflectFilter without a mask record.
pub fn box_reflect_rgba(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    matrix: [f32; 6],
    mask_alpha: Option<&[u8]>,
) -> bool {
    let (width, height) = (width as usize, height as usize);
    if pixels.len() != width.saturating_mul(height).saturating_mul(4)
        || !matrix.iter().all(|value| value.is_finite())
    {
        return false;
    }
    let [sx, ky, kx, sy, tx, ty] = matrix;
    let determinant = sx * sy - kx * ky;
    if determinant.abs() <= f32::EPSILON {
        return false;
    }
    let inverse = [
        sy / determinant,
        -ky / determinant,
        -kx / determinant,
        sx / determinant,
        (kx * ty - sy * tx) / determinant,
        (ky * tx - sx * ty) / determinant,
    ];
    if mask_alpha.is_some_and(|mask| mask.len() != width.saturating_mul(height)) {
        return false;
    }
    let source = pixels.to_vec();
    let mut reflected_source = source.clone();
    if let Some(mask) = mask_alpha {
        for (pixel, alpha) in reflected_source.chunks_exact_mut(4).zip(mask) {
            for channel in pixel {
                *channel = crate::cpu::mask_blitter::mul_div_255_round(*channel, *alpha);
            }
        }
    }
    for y in 0..height {
        for x in 0..width {
            // Image-filter sampling maps pixel centers through the inverse
            // local matrix; subtracting half a pixel restores texel space.
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let sample_x = inverse[0] * px + inverse[2] * py + inverse[4] - 0.5;
            let sample_y = inverse[1] * px + inverse[3] * py + inverse[5] - 0.5;
            let reflected = [
                sample_channel(&reflected_source, width, height, sample_x, sample_y, 0),
                sample_channel(&reflected_source, width, height, sample_x, sample_y, 1),
                sample_channel(&reflected_source, width, height, sample_x, sample_y, 2),
                sample_channel(&reflected_source, width, height, sample_x, sample_y, 3),
            ];
            let original = &source[(y * width + x) * 4..(y * width + x + 1) * 4];
            let destination = &mut pixels[(y * width + x) * 4..(y * width + x + 1) * 4];
            // SkImageFilters::Blend receives the reflected image as the
            // background and the unchanged input as the foreground.
            let inverse_alpha = 255 - u16::from(original[3]);
            for channel in 0..4 {
                destination[channel] = (u16::from(original[channel])
                    + (u16::from(reflected[channel]) * inverse_alpha + 127) / 255)
                    .min(255) as u8;
            }
        }
    }
    true
}

fn source_over(output: &mut [u8], source: &[u8]) {
    for (dst, src) in output.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
        let inverse = 255 - u16::from(src[3]);
        for c in 0..4 {
            dst[c] = (u16::from(src[c]) + (u16::from(dst[c]) * inverse + 127) / 255).min(255) as u8;
        }
    }
}
fn solid_buffer(len: usize, color: Option<[f32; 4]>) -> Vec<u8> {
    let mut out = vec![0; len];
    if let Some(c) = color {
        let a = unit_to_byte(c[3]);
        let p = [
            unit_to_byte(c[0] * c[3]),
            unit_to_byte(c[1] * c[3]),
            unit_to_byte(c[2] * c[3]),
            a,
        ];
        for pixel in out.chunks_exact_mut(4) {
            pixel.copy_from_slice(&p)
        }
    }
    out
}
fn convert_space(pixels: &mut [u8], to_linear: bool) {
    for p in pixels.chunks_exact_mut(4) {
        let a = f32::from(p[3]) / 255.0;
        if a == 0.0 {
            continue;
        }
        for c in 0..3 {
            let v = (f32::from(p[c]) / 255.0 / a).clamp(0.0, 1.0);
            let v = if to_linear {
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            } else if v <= 0.0031308 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            p[c] = unit_to_byte(v * a)
        }
    }
}
fn edge_index(value: i64, size: usize, mode: RenderSvgEdgeMode) -> Option<usize> {
    if size == 0 {
        return None;
    }
    match mode {
        RenderSvgEdgeMode::None => (value >= 0 && value < size as i64).then_some(value as usize),
        RenderSvgEdgeMode::Duplicate => Some(value.clamp(0, size as i64 - 1) as usize),
        RenderSvgEdgeMode::Wrap => Some(value.rem_euclid(size as i64) as usize),
    }
}
fn morphology(input: &[u8], w: usize, h: usize, rx: u32, ry: u32, dilate: bool) -> Vec<u8> {
    let mut out = vec![0; input.len()];
    for y in 0..h {
        for x in 0..w {
            let mut value = if dilate { [0u8; 4] } else { [255u8; 4] };
            for sy in y.saturating_sub(ry as usize)..=(y + ry as usize).min(h - 1) {
                for sx in x.saturating_sub(rx as usize)..=(x + rx as usize).min(w - 1) {
                    let i = (sy * w + sx) * 4;
                    for c in 0..4 {
                        value[c] = if dilate {
                            value[c].max(input[i + c])
                        } else {
                            value[c].min(input[i + c])
                        }
                    }
                }
            }
            out[(y * w + x) * 4..(y * w + x + 1) * 4].copy_from_slice(&value)
        }
    }
    out
}
fn transfer_rgba(pixels: &mut [u8], functions: &[RenderSvgTransfer; 4]) {
    for p in pixels.chunks_exact_mut(4) {
        let a = f32::from(p[3]) / 255.0;
        let mut v = [0.0; 4];
        for c in 0..3 {
            v[c] = if a > 0.0 {
                f32::from(p[c]) / 255.0 / a
            } else {
                0.0
            }
        }
        v[3] = a;
        for c in 0..4 {
            v[c] = match &functions[c] {
                RenderSvgTransfer::Identity => v[c],
                RenderSvgTransfer::Table(t) => {
                    if t.is_empty() {
                        v[c]
                    } else if t.len() == 1 {
                        t[0]
                    } else {
                        let x = v[c].clamp(0.0, 1.0) * (t.len() - 1) as f32;
                        let i = x.floor() as usize;
                        t[i] + (t[(i + 1).min(t.len() - 1)] - t[i]) * (x - i as f32)
                    }
                }
                RenderSvgTransfer::Discrete(t) => {
                    if t.is_empty() {
                        v[c]
                    } else {
                        t[((v[c].clamp(0.0, 1.0) * t.len() as f32).floor() as usize)
                            .min(t.len() - 1)]
                    }
                }
                RenderSvgTransfer::Linear { slope, intercept } => slope * v[c] + intercept,
                RenderSvgTransfer::Gamma {
                    amplitude,
                    exponent,
                    offset,
                } => amplitude * v[c].max(0.0).powf(*exponent) + offset,
            }
            .clamp(0.0, 1.0)
        }
        let oa = v[3];
        for c in 0..3 {
            p[c] = unit_to_byte(v[c] * oa)
        }
        p[3] = unit_to_byte(oa)
    }
}
fn convolve(
    input: &[u8],
    w: usize,
    h: usize,
    ox: u32,
    oy: u32,
    kernel: &[f32],
    divisor: f32,
    bias: f32,
    tx: u32,
    ty: u32,
    edge: RenderSvgEdgeMode,
    ux: f32,
    uy: f32,
    preserve_alpha: bool,
) -> Vec<u8> {
    let mut out = vec![0; input.len()];
    for y in 0..h {
        for x in 0..w {
            let mut sum = [0.0; 4];
            for ky in 0..oy {
                for kx in 0..ox {
                    let sx = x as f32 + (kx as i32 - tx as i32) as f32 * ux;
                    let sy = y as f32 + (ky as i32 - ty as i32) as f32 * uy;
                    let Some(sx) = edge_index(sx.round() as i64, w, edge) else {
                        continue;
                    };
                    let Some(sy) = edge_index(sy.round() as i64, h, edge) else {
                        continue;
                    };
                    let i = (sy * w + sx) * 4;
                    let weight = kernel[((oy - 1 - ky) * ox + (ox - 1 - kx)) as usize];
                    for c in 0..4 {
                        sum[c] += f32::from(input[i + c]) / 255.0 * weight
                    }
                }
            }
            let i = (y * w + x) * 4;
            for c in 0..4 {
                out[i + c] = if preserve_alpha && c == 3 {
                    input[i + 3]
                } else {
                    unit_to_byte(sum[c] / divisor + bias)
                }
            }
        }
    }
    out
}
fn turbulence_rgba(
    out: &mut [u8],
    w: usize,
    h: usize,
    mut fx: f32,
    mut fy: f32,
    octaves: u32,
    seed: f32,
    stitch: bool,
    fractal: bool,
) {
    const N: usize = 256;
    const M: i64 = 2_147_483_647;
    let mut state = seed.trunc() as i64;
    if state <= 0 {
        state = -(state % (M - 1)) + 1;
    }
    if state > M - 1 {
        state = M - 1;
    }
    let random = |s: &mut i64| {
        let mut r = 16807 * (*s % 127773) - 2836 * (*s / 127773);
        if r <= 0 {
            r += M;
        }
        *s = r;
        r
    };
    let mut permutation = [0u8; N];
    let mut raw = [[[0i32; 2]; N]; 4];
    for c in 0..4 {
        for i in 0..N {
            permutation[i] = i as u8;
            raw[c][i][0] = (random(&mut state) % (2 * N as i64)) as i32;
            raw[c][i][1] = (random(&mut state) % (2 * N as i64)) as i32;
        }
    }
    for i in (1..N).rev() {
        let j = (random(&mut state) % N as i64) as usize;
        permutation.swap(i, j);
    }
    let mut gradient = [[[0.0f32; 2]; N]; 4];
    for c in 0..4 {
        for i in 0..N {
            let p = permutation[i] as usize;
            let mut gx = (raw[c][p][0] - N as i32) as f32 / N as f32;
            let mut gy = (raw[c][p][1] - N as i32) as f32 / N as f32;
            let len = gx.hypot(gy);
            if len > 0.0 {
                gx /= len;
                gy /= len;
            }
            // SkPerlinNoiseShader stores normalized gradients as two u16s,
            // then the raster pipeline expands them back to [-1, 1].
            let quantize = |value: f32| {
                (((value + 1.0) * 32767.5).round().clamp(0.0, 65535.0) * 2.0 / 65535.0) - 1.0
            };
            gradient[c][i] = [quantize(gx), quantize(gy)];
        }
    }
    if stitch {
        if fx != 0.0 {
            let low = (w as f32 * fx).floor() / w.max(1) as f32;
            let high = (w as f32 * fx).ceil() / w.max(1) as f32;
            fx = if low > 0.0 && fx / low < high / fx {
                low
            } else {
                high
            };
        }
        if fy != 0.0 {
            let low = (h as f32 * fy).floor() / h.max(1) as f32;
            let high = (h as f32 * fy).ceil() / h.max(1) as f32;
            fy = if low > 0.0 && fy / low < high / fy {
                low
            } else {
                high
            };
        }
    }
    let initial = [(w as f32 * fx).round(), (h as f32 * fy).round()];
    for y in 0..h {
        for x in 0..w {
            let mut point = [(x as f32 + 0.5) * fx, (y as f32 + 0.5) * fy];
            let mut stitch_data = initial;
            let mut ratio = 1.0;
            let mut color = [0.0f32; 4];
            for _ in 0..octaves {
                let mut x0 = point[0].floor();
                let mut y0 = point[1].floor();
                let mut x1 = x0 + 1.0;
                let mut y1 = y0 + 1.0;
                let tx = point[0] - x0;
                let ty = point[1] - y0;
                if stitch {
                    if x0 >= stitch_data[0] {
                        x0 -= stitch_data[0]
                    }
                    if x1 >= stitch_data[0] {
                        x1 -= stitch_data[0]
                    }
                    if y0 >= stitch_data[1] {
                        y0 -= stitch_data[1]
                    }
                    if y1 >= stitch_data[1] {
                        y1 -= stitch_data[1]
                    }
                }
                let px0 = permutation[(x0.round() as i32 & 255) as usize] as i32;
                let px1 = permutation[(x1.round() as i32 & 255) as usize] as i32;
                let ids = [
                    ((px0 + y0.round() as i32) & 255) as usize,
                    ((px1 + y0.round() as i32) & 255) as usize,
                    ((px0 + y1.round() as i32) & 255) as usize,
                    ((px1 + y1.round() as i32) & 255) as usize,
                ];
                let sx = tx * tx * (3.0 - 2.0 * tx);
                let sy = ty * ty * (3.0 - 2.0 * ty);
                for c in 0..4 {
                    let dot = |id: usize, dx: f32, dy: f32| {
                        gradient[c][id][0] * dx + gradient[c][id][1] * dy
                    };
                    let n00 = dot(ids[0], tx, ty);
                    let a = n00 + (dot(ids[1], tx - 1.0, ty) - n00) * sx;
                    let n01 = dot(ids[2], tx, ty - 1.0);
                    let b = n01 + (dot(ids[3], tx - 1.0, ty - 1.0) - n01) * sx;
                    let n = a + (b - a) * sy;
                    color[c] += if fractal { n } else { n.abs() } * ratio;
                }
                point[0] *= 2.0;
                point[1] *= 2.0;
                stitch_data[0] *= 2.0;
                stitch_data[1] *= 2.0;
                ratio *= 0.5;
            }
            if fractal {
                for value in &mut color {
                    *value = *value * 0.5 + 0.5;
                }
            }
            for value in &mut color {
                *value = value.clamp(0.0, 1.0);
            }
            let i = (y * w + x) * 4;
            out[i] = unit_to_byte(color[0] * color[3]);
            out[i + 1] = unit_to_byte(color[1] * color[3]);
            out[i + 2] = unit_to_byte(color[2] * color[3]);
            out[i + 3] = unit_to_byte(color[3]);
        }
    }
}
fn blur_edge(
    input: &[u8],
    w: usize,
    h: usize,
    sx: f32,
    sy: f32,
    mode: RenderSvgEdgeMode,
) -> Option<Vec<u8>> {
    if matches!(mode, RenderSvgEdgeMode::None) {
        let mut out = input.to_vec();
        return blur_rgba(&mut out, w as u32, h as u32, sx, sy).then_some(out);
    }
    let mx = (sx * 3.0).ceil() as usize + 1;
    let my = (sy * 3.0).ceil() as usize + 1;
    let pw = w + mx * 2;
    let ph = h + my * 2;
    let mut padded = vec![0; pw.checked_mul(ph)?.checked_mul(4)?];
    for y in 0..ph {
        for x in 0..pw {
            let ix = edge_index(x as i64 - mx as i64, w, mode)?;
            let iy = edge_index(y as i64 - my as i64, h, mode)?;
            padded[(y * pw + x) * 4..(y * pw + x + 1) * 4]
                .copy_from_slice(&input[(iy * w + ix) * 4..(iy * w + ix + 1) * 4])
        }
    }
    if !blur_rgba(&mut padded, pw as u32, ph as u32, sx, sy) {
        return None;
    }
    let mut out = vec![0; input.len()];
    for y in 0..h {
        out[y * w * 4..(y + 1) * w * 4]
            .copy_from_slice(&padded[((y + my) * pw + mx) * 4..((y + my) * pw + mx + w) * 4])
    }
    Some(out)
}
fn region_bounds(width: usize, height: usize, region: [f32; 4]) -> [usize; 4] {
    [
        region[0].floor().clamp(0.0, width as f32) as usize,
        region[1].floor().clamp(0.0, height as f32) as usize,
        (region[0] + region[2]).ceil().clamp(0.0, width as f32) as usize,
        (region[1] + region[3]).ceil().clamp(0.0, height as f32) as usize,
    ]
}
fn clip_region(pixels: &mut [u8], width: usize, height: usize, region: [f32; 4]) {
    let [left, top, right, bottom] = region_bounds(width, height, region);
    if left >= right || top >= bottom {
        pixels.fill(0);
        return;
    }
    let stride = width * 4;
    pixels[..top * stride].fill(0);
    pixels[bottom * stride..].fill(0);
    for row in pixels[top * stride..bottom * stride].chunks_exact_mut(stride) {
        row[..left * 4].fill(0);
        row[right * 4..].fill(0);
    }
}
fn offset_rgba(
    input: &[u8],
    width: usize,
    height: usize,
    x: f32,
    y: f32,
    region: [f32; 4],
) -> Vec<u8> {
    let mut output = vec![0; input.len()];
    let [left, top, right, bottom] = region_bounds(width, height, region);
    if left >= right || top >= bottom || !x.is_finite() || !y.is_finite() {
        return output;
    }
    // SkImageFilterTypes::FilterResult::applyTransform downgrades pixel-aligned
    // translation to nearest sampling. Copy the surviving rows exactly; as in
    // SkImageFilters::Offset, the primitive crop restricts output, never input.
    if x.fract() == 0.0 && y.fract() == 0.0 {
        if x.abs() >= width as f32 || y.abs() >= height as f32 {
            return output;
        }
        let (dx, dy) = (x as i64, y as i64);
        let left = (left as i64).max(dx);
        let right = (right as i64).min(width as i64 + dx);
        let top = (top as i64).max(dy);
        let bottom = (bottom as i64).min(height as i64 + dy);
        if left >= right || top >= bottom {
            return output;
        }
        let count = (right - left) as usize * 4;
        for py in top..bottom {
            let dst = (py as usize * width + left as usize) * 4;
            let src = ((py - dy) as usize * width + (left - dx) as usize) * 4;
            output[dst..dst + count].copy_from_slice(&input[src..src + count]);
        }
    } else {
        for py in top..bottom {
            for px in left..right {
                let i = (py * width + px) * 4;
                for c in 0..4 {
                    output[i + c] =
                        sample_channel(input, width, height, px as f32 - x, py as f32 - y, c);
                }
            }
        }
    }
    output
}
fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let length = value[0]
        .mul_add(value[0], value[1].mul_add(value[1], value[2] * value[2]))
        .sqrt();
    if length > 0.0 && length.is_finite() {
        [value[0] / length, value[1] / length, value[2] / length]
    } else {
        [0.0; 3]
    }
}
fn lighting_rgba(
    input: &[u8],
    width: usize,
    height: usize,
    region: [f32; 4],
    lighting_type: RenderSvgLightingType,
    light: Option<RenderSvgLight>,
    mut color: [f32; 3],
    surface_scale: f32,
    constant: f32,
    shininess: f32,
    linear_rgb: bool,
) -> Vec<u8> {
    let Some(light) = light else {
        return vec![0; input.len()];
    };
    if linear_rgb {
        for channel in &mut color {
            *channel = if *channel <= 0.04045 {
                *channel / 12.92
            } else {
                ((*channel + 0.055) / 1.055).powf(2.4)
            };
        }
    }
    for channel in &mut color {
        *channel *= constant;
    }
    let left = region[0].floor().clamp(0.0, width as f32) as isize;
    let top = region[1].floor().clamp(0.0, height as f32) as isize;
    let right = (region[0] + region[2]).ceil().clamp(0.0, width as f32) as isize - 1;
    let bottom = (region[1] + region[3]).ceil().clamp(0.0, height as f32) as isize - 1;
    let alpha = |x: isize, y: isize| -> f32 {
        if right < left || bottom < top {
            return 0.0;
        }
        let x = x.clamp(left, right) as usize;
        let y = y.clamp(top, bottom) as usize;
        f32::from(input[(y * width + x) * 4 + 3]) / 255.0
    };
    let (light_direction, light_position, spot) = match light {
        RenderSvgLight::Distant { direction } => (normalize3(direction), [0.0; 3], None),
        RenderSvgLight::Point { position } => ([0.0; 3], position, None),
        RenderSvgLight::Spot {
            position,
            target,
            falloff_exponent,
            limiting_cone_angle,
        } => {
            let direction = normalize3([
                target[0] - position[0],
                target[1] - position[1],
                target[2] - position[2],
            ]);
            let angle = if limiting_cone_angle == 0.0 || limiting_cone_angle.abs() > 90.0 {
                90.0
            } else {
                limiting_cone_angle
            };
            (
                direction,
                position,
                Some((falloff_exponent, angle.to_radians().cos())),
            )
        }
    };
    let mut output = vec![0; input.len()];
    for y in 0..height {
        for x in 0..width {
            let x = x as isize;
            let y = y as isize;
            let c0 = [alpha(x - 1, y - 1), alpha(x - 1, y), alpha(x - 1, y + 1)];
            let c2 = [alpha(x + 1, y - 1), alpha(x + 1, y), alpha(x + 1, y + 1)];
            let r0 = [alpha(x - 1, y - 1), alpha(x, y - 1), alpha(x + 1, y - 1)];
            let r2 = [alpha(x - 1, y + 1), alpha(x, y + 1), alpha(x + 1, y + 1)];
            let weighted = |v: [f32; 3]| 0.25 * v[0] + 0.5 * v[1] + 0.25 * v[2];
            let normal = normalize3([
                -surface_scale * (weighted(c2) - weighted(c0)),
                -surface_scale * (weighted(r2) - weighted(r0)),
                1.0,
            ]);
            let z = surface_scale * alpha(x, y);
            let coordinate_x = x as f32 + 0.5;
            let coordinate_y = y as f32 + 0.5;
            let surface_to_light = match light {
                RenderSvgLight::Distant { .. } => light_direction,
                _ => normalize3([
                    light_position[0] - coordinate_x,
                    light_position[1] - coordinate_y,
                    light_position[2] - z,
                ]),
            };
            let mut lit = color;
            if let Some((falloff, cutoff)) = spot {
                let cosine = -(surface_to_light[0] * light_direction[0]
                    + surface_to_light[1] * light_direction[1]
                    + surface_to_light[2] * light_direction[2]);
                let mut attenuation = if cosine < cutoff {
                    0.0
                } else {
                    cosine.powf(falloff)
                };
                if cosine >= cutoff && cosine < cutoff + 0.016 {
                    attenuation *= (cosine - cutoff) / 0.016;
                }
                for channel in &mut lit {
                    *channel *= attenuation;
                }
            }
            let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
            let (coefficient, alpha_out) = match lighting_type {
                RenderSvgLightingType::Diffuse => (dot(normal, surface_to_light), 1.0),
                RenderSvgLightingType::Specular => {
                    let half = normalize3([
                        surface_to_light[0],
                        surface_to_light[1],
                        surface_to_light[2] + 1.0,
                    ]);
                    (dot(normal, half).powf(shininess), -1.0)
                }
            };
            for channel in &mut lit {
                *channel = (*channel * coefficient).clamp(0.0, 1.0);
            }
            let alpha_out = if alpha_out < 0.0 {
                lit[0].max(lit[1]).max(lit[2])
            } else {
                alpha_out
            };
            let index = (y as usize * width + x as usize) * 4;
            output[index] = unit_to_byte(lit[0]);
            output[index + 1] = unit_to_byte(lit[1]);
            output[index + 2] = unit_to_byte(lit[2]);
            output[index + 3] = unit_to_byte(alpha_out);
        }
    }
    output
}
// Graph regions are in the owning filter space; translate them into the
// caller-owned pixel storage, as SkCanvas saveLayer does for its device origin.
fn svg_graph_rgba_at(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    graph: &RenderSvgFilterGraph,
    origin: (f32, f32),
) -> bool {
    let mut graph = graph.clone();
    graph.region[0] -= origin.0;
    graph.region[1] -= origin.1;
    for primitive in &mut graph.primitives {
        primitive.region[0] -= origin.0;
        primitive.region[1] -= origin.1;
    }
    svg_graph_rgba(pixels, width, height, &graph)
}
fn svg_graph_rgba(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    graph: &RenderSvgFilterGraph,
) -> bool {
    let (w, h) = (width as usize, height as usize);
    if pixels.len() != w.saturating_mul(h).saturating_mul(4) {
        return false;
    }
    let source = pixels.to_vec();
    let mut alpha = vec![0u8; source.len()];
    for (s, d) in source.chunks_exact(4).zip(alpha.chunks_exact_mut(4)) {
        d[3] = s[3]
    }
    let mut named: Vec<(String, Vec<u8>)> = Vec::new();
    let mut previous: Option<Vec<u8>> = None;
    let fill = graph
        .fill_paint
        .map(|color| solid_buffer(source.len(), Some(color)));
    let stroke = graph
        .stroke_paint
        .map(|color| solid_buffer(source.len(), Some(color)));
    let lookup =
        |name: &str, previous: &Option<Vec<u8>>, named: &Vec<(String, Vec<u8>)>| -> Vec<u8> {
            match name {
                "SourceGraphic" => source.clone(),
                "SourceAlpha" => alpha.clone(),
                // Fixed Chromium's SVGFilterBuilder has no BackgroundImage /
                // BackgroundAlpha built-ins; GetEffectById therefore applies
                // the ordinary previous-effect/SourceGraphic fallback.
                "BackgroundImage" | "BackgroundAlpha" => {
                    previous.clone().unwrap_or_else(|| source.clone())
                }
                "FillPaint" => fill
                    .clone()
                    .or_else(|| previous.clone())
                    .unwrap_or_else(|| source.clone()),
                "StrokePaint" => stroke
                    .clone()
                    .or_else(|| previous.clone())
                    .unwrap_or_else(|| source.clone()),
                _ => named
                    .iter()
                    .rev()
                    .find(|(n, _)| n == name)
                    .map(|(_, v)| v.clone())
                    .or_else(|| previous.clone())
                    .unwrap_or_else(|| source.clone()),
            }
        };
    for primitive in &graph.primitives {
        let mut out = lookup(&primitive.input, &previous, &named);
        if primitive.linear_rgb {
            convert_space(&mut out, true);
        }
        match &primitive.kind {
            RenderSvgFilterKind::GaussianBlur { x, y, edge_mode } => {
                let Some(blurred) = blur_edge(&out, w, h, *x, *y, *edge_mode) else {
                    return false;
                };
                out = blurred;
            }
            RenderSvgFilterKind::ColorMatrix(matrix) => {
                if !color_matrix_rgba(&mut out, matrix) {
                    return false;
                }
            }
            RenderSvgFilterKind::Offset { x, y } => {
                out = offset_rgba(&out, w, h, *x, *y, primitive.region);
            }
            RenderSvgFilterKind::Flood(color) => {
                let a = unit_to_byte(color[3]);
                let rgba = [
                    unit_to_byte(color[0] * color[3]),
                    unit_to_byte(color[1] * color[3]),
                    unit_to_byte(color[2] * color[3]),
                    a,
                ];
                for p in out.chunks_exact_mut(4) {
                    p.copy_from_slice(&rgba)
                }
            }
            RenderSvgFilterKind::Merge(inputs) => {
                out.fill(0);
                for name in inputs {
                    let input = lookup(name, &previous, &named);
                    source_over(&mut out, &input)
                }
            }
            RenderSvgFilterKind::Blend { input2, mode } => {
                let mut b = lookup(input2, &previous, &named);
                if primitive.linear_rgb {
                    convert_space(&mut b, true)
                }
                for (a, b) in out.chunks_exact_mut(4).zip(b.chunks_exact(4)) {
                    blend_pixel(a, b, *mode)
                }
            }
            RenderSvgFilterKind::Composite {
                input2,
                operator,
                k,
            } => {
                let mut b = lookup(input2, &previous, &named);
                if primitive.linear_rgb {
                    convert_space(&mut b, true)
                }
                for (a, b) in out.chunks_exact_mut(4).zip(b.chunks_exact(4)) {
                    composite_pixel(a, b, *operator, *k)
                }
            }
            RenderSvgFilterKind::ComponentTransfer(functions) => transfer_rgba(&mut out, functions),
            RenderSvgFilterKind::Morphology { dilate, x, y } => {
                out = morphology(&out, w, h, *x, *y, *dilate)
            }
            RenderSvgFilterKind::DisplacementMap {
                input2,
                scale,
                x_channel,
                y_channel,
            } => {
                let mut map = lookup(input2, &previous, &named);
                if primitive.linear_rgb {
                    convert_space(&mut map, true)
                }
                let input = out;
                out = vec![0; input.len()];
                for y in 0..h {
                    for x in 0..w {
                        let i = (y * w + x) * 4;
                        let alpha = f32::from(map[i + 3]) / 255.0;
                        let channel = |channel: usize| {
                            if channel == 3 {
                                alpha
                            } else if alpha > 0.0 {
                                (f32::from(map[i + channel]) / 255.0 / alpha).clamp(0.0, 1.0)
                            } else {
                                0.0
                            }
                        };
                        let dx = (channel(*x_channel) - 0.5) * scale;
                        let dy = (channel(*y_channel) - 0.5) * scale;
                        for c in 0..4 {
                            out[i + c] =
                                sample_channel(&input, w, h, x as f32 + dx, y as f32 + dy, c)
                        }
                    }
                }
            }
            RenderSvgFilterKind::Turbulence {
                base_x,
                base_y,
                octaves,
                seed,
                stitch,
                fractal_noise,
            } => {
                turbulence_rgba(
                    &mut out,
                    w,
                    h,
                    *base_x,
                    *base_y,
                    *octaves,
                    *seed,
                    *stitch,
                    *fractal_noise,
                );
            }
            RenderSvgFilterKind::Tile => {
                let input = std::mem::take(&mut out);
                out = input.clone();
                let mut bounds = None;
                for y in 0..h {
                    for x in 0..w {
                        if input[(y * w + x) * 4 + 3] != 0 {
                            bounds = Some(bounds.map_or(
                                (x, y, x, y),
                                |(l, t, r, b): (usize, usize, usize, usize)| {
                                    (l.min(x), t.min(y), r.max(x), b.max(y))
                                },
                            ))
                        }
                    }
                }
                if let Some((l, t, r, b)) = bounds {
                    let tw = r - l + 1;
                    let th = b - t + 1;
                    out = vec![0; input.len()];
                    for y in 0..h {
                        for x in 0..w {
                            let sx = l + (x as isize - l as isize).rem_euclid(tw as isize) as usize;
                            let sy = t + (y as isize - t as isize).rem_euclid(th as isize) as usize;
                            out[(y * w + x) * 4..(y * w + x + 1) * 4]
                                .copy_from_slice(&input[(sy * w + sx) * 4..(sy * w + sx + 1) * 4])
                        }
                    }
                }
            }
            RenderSvgFilterKind::Image {
                href,
                local_fragment,
                preserve_none,
                width: iw,
                height: ih,
                pixels,
            } => {
                if *local_fragment {
                    out = lookup(href.trim_start_matches('#'), &previous, &named)
                } else {
                    if *iw == 0 || *ih == 0 || pixels.len() != *iw as usize * *ih as usize * 4 {
                        return false;
                    }
                    out = vec![0; w * h * 4];
                    let scale_x = w as f64 / f64::from(*iw);
                    let scale_y = h as f64 / f64::from(*ih);
                    let fit = scale_x.min(scale_y);
                    let draw_w = f64::from(*iw) * fit;
                    let draw_h = f64::from(*ih) * fit;
                    let left = (w as f64 - draw_w) * 0.5;
                    let top = (h as f64 - draw_h) * 0.5;
                    for y in 0..h {
                        for x in 0..w {
                            if !*preserve_none
                                && ((x as f64) < left
                                    || (x as f64) >= left + draw_w
                                    || (y as f64) < top
                                    || (y as f64) >= top + draw_h)
                            {
                                continue;
                            }
                            let nx = if *preserve_none {
                                x as f64 / w.max(1) as f64
                            } else {
                                (x as f64 - left) / draw_w.max(1.0)
                            };
                            let ny = if *preserve_none {
                                y as f64 / h.max(1) as f64
                            } else {
                                (y as f64 - top) / draw_h.max(1.0)
                            };
                            let sx = (nx * f64::from(*iw)).floor().min(f64::from(*iw - 1)) as usize;
                            let sy = (ny * f64::from(*ih)).floor().min(f64::from(*ih - 1)) as usize;
                            out[(y * w + x) * 4..(y * w + x + 1) * 4].copy_from_slice(
                                &pixels[(sy * *iw as usize + sx) * 4
                                    ..(sy * *iw as usize + sx + 1) * 4],
                            )
                        }
                    }
                }
            }
            RenderSvgFilterKind::Convolve {
                order_x,
                order_y,
                kernel,
                divisor,
                bias,
                target_x,
                target_y,
                edge_mode,
                unit_x,
                unit_y,
                preserve_alpha,
            } => {
                out = convolve(
                    &out,
                    w,
                    h,
                    *order_x,
                    *order_y,
                    kernel,
                    *divisor,
                    *bias,
                    *target_x,
                    *target_y,
                    *edge_mode,
                    *unit_x,
                    *unit_y,
                    *preserve_alpha,
                )
            }
            RenderSvgFilterKind::Lighting {
                lighting_type,
                light,
                color,
                surface_scale,
                constant,
                shininess,
            } => {
                out = lighting_rgba(
                    &out,
                    w,
                    h,
                    primitive.region,
                    *lighting_type,
                    *light,
                    *color,
                    *surface_scale,
                    *constant,
                    *shininess,
                    primitive.linear_rgb,
                )
            }
        }
        if primitive.linear_rgb {
            convert_space(&mut out, false);
        }
        clip_region(&mut out, w, h, primitive.region);
        if !primitive.result.is_empty() {
            named.push((primitive.result.clone(), out.clone()))
        }
        previous = Some(out);
    }
    let mut output = previous.unwrap_or(source);
    clip_region(&mut output, w, h, graph.region);
    pixels.copy_from_slice(&output);
    true
}
fn blend_pixel(a: &mut [u8], b: &[u8], mode: RenderSvgBlend) {
    let aa = f32::from(a[3]) / 255.0;
    let ab = f32::from(b[3]) / 255.0;
    let ao = aa + ab - aa * ab;
    let mut result = [0u8; 4];
    for c in 0..3 {
        let ca = if aa > 0.0 {
            f32::from(a[c]) / 255.0 / aa
        } else {
            0.0
        };
        let cb = if ab > 0.0 {
            f32::from(b[c]) / 255.0 / ab
        } else {
            0.0
        };
        let blend = match mode {
            RenderSvgBlend::Normal => ca,
            RenderSvgBlend::Multiply => ca * cb,
            RenderSvgBlend::Screen => ca + cb - ca * cb,
            RenderSvgBlend::Darken => ca.min(cb),
            RenderSvgBlend::Lighten => ca.max(cb),
        };
        result[c] = unit_to_byte(
            ((1.0 - ab) * ca * aa + (1.0 - aa) * cb * ab + aa * ab * blend).clamp(0.0, 1.0),
        );
    }
    result[3] = unit_to_byte(ao);
    a.copy_from_slice(&result)
}
fn composite_pixel(a: &mut [u8], b: &[u8], op: RenderSvgComposite, k: [f32; 4]) {
    let aa = f32::from(a[3]) / 255.0;
    let ab = f32::from(b[3]) / 255.0;
    let mut out = [0u8; 4];
    for c in 0..4 {
        let x = f32::from(a[c]) / 255.0;
        let y = f32::from(b[c]) / 255.0;
        let v = match op {
            RenderSvgComposite::Over => x + y * (1.0 - aa),
            RenderSvgComposite::In => x * ab,
            RenderSvgComposite::Out => x * (1.0 - ab),
            RenderSvgComposite::Atop => x * ab + y * (1.0 - aa),
            RenderSvgComposite::Xor => x * (1.0 - ab) + y * (1.0 - aa),
            RenderSvgComposite::Arithmetic => k[0] * x * y + k[1] * x + k[2] * y + k[3],
        };
        out[c] = unit_to_byte(v)
    }
    a.copy_from_slice(&out)
}

#[cfg(test)]
mod lighting_tests {
    use super::*;

    #[test]
    fn flat_surface_uses_skia_diffuse_and_missing_light_semantics() {
        let primitive = |light| RenderSvgFilterPrimitive {
            input: "SourceGraphic".into(),
            result: String::new(),
            region: [0.0, 0.0, 1.0, 1.0],
            linear_rgb: false,
            kind: RenderSvgFilterKind::Lighting {
                lighting_type: RenderSvgLightingType::Diffuse,
                light,
                color: [1.0; 3],
                surface_scale: 1.0,
                constant: 1.0,
                shininess: 1.0,
            },
        };
        let graph = |light| {
            RenderSurfaceFilter::SvgGraph(RenderSvgFilterGraph {
                primitives: vec![primitive(light)],
                region: [0.0, 0.0, 1.0, 1.0],
                fill_paint: None,
                stroke_paint: None,
            })
        };
        let mut pixel = [0, 0, 0, 255];
        assert!(apply_rgba(
            &mut pixel,
            1,
            1,
            &graph(Some(RenderSvgLight::Distant {
                direction: [0.0, 0.0, 1.0],
            }))
        ));
        assert_eq!(pixel, [255; 4]);
        assert!(apply_rgba(&mut pixel, 1, 1, &graph(None)));
        assert_eq!(pixel, [0; 4]);
    }
}
