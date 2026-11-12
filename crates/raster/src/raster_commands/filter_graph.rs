//! Backend-neutral reflection-mask and SVG-filter graph values.

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
    pub image_interpolation_quality: crate::raster_commands::commands::ImageInterpolationQuality,
    pub dynamic_range_limit: crate::raster_commands::commands::DynamicRangeLimit,
    pub paint_shader: Option<crate::raster_commands::commands::PaintShader>,
    pub clip_rect: crate::raster_commands::commands::PaintRect,
    pub source_rect: crate::raster_commands::commands::PaintRect,
    pub tile_rect: crate::raster_commands::commands::PaintRect,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub tile_spacing: crate::raster_commands::commands::Size,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderReflectionMask {
    pub layers: Vec<RenderReflectionMaskLayer>,
    /// A retained conversion failure, rather than silently replacing an
    /// unsupported private PaintRecord with a transparent mask.
    pub error: Option<String>,
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
    pub origin_tainted: bool,
    pub kind: RenderSvgFilterKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderSvgPreserveAspectRatioAlign {
    None,
    XMinYMin,
    XMidYMin,
    XMaxYMin,
    XMinYMid,
    XMidYMid,
    XMaxYMid,
    XMinYMax,
    XMidYMax,
    XMaxYMax,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderSvgMeetOrSlice {
    Meet,
    Slice,
}
#[derive(Clone, Debug, PartialEq)]
pub enum RenderSvgFilterKind {
    GaussianBlur {
        x: f32,
        y: f32,
    },
    ColorMatrix([f32; 20]),
    Offset {
        x: f32,
        y: f32,
    },
    DropShadow {
        sigma_x: f32,
        sigma_y: f32,
        dx: f32,
        dy: f32,
        color: [f32; 4],
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
        x: f32,
        y: f32,
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
        octaves: i32,
        seed: f32,
        stitch: bool,
        fractal_noise: bool,
    },
    Tile {
        source_region: [f32; 4],
    },
    Image {
        href: String,
        local_subtree: bool,
        align: RenderSvgPreserveAspectRatioAlign,
        meet_or_slice: RenderSvgMeetOrSlice,
        width: u32,
        height: u32,
        orientation: crate::raster_commands::commands::ImageOrientation,
        pixels: std::sync::Arc<Vec<u8>>,
    },
    Convolve {
        order_x: u32,
        order_y: u32,
        kernel: Vec<f32>,
        divisor: f32,
        bias: f32,
        target_x: i32,
        target_y: i32,
        edge_mode: RenderSvgEdgeMode,
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
    Overlay,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderSvgComposite {
    Over,
    In,
    Out,
    Atop,
    Xor,
    Arithmetic,
    Lighter,
}
