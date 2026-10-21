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
    pub image_interpolation_quality: crate::commands::ImageInterpolationQuality,
    pub paint_shader: Option<crate::commands::PaintShader>,
    pub clip_rect: crate::commands::PaintRect,
    pub source_rect: crate::commands::PaintRect,
    pub tile_rect: crate::commands::PaintRect,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub tile_spacing: crate::commands::Size,
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
