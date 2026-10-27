use layoutng_assembly::fragment_tree::PaintResources;
use layoutng_assembly::internal::layout_input::{Offset, Size};
use layoutng_assembly::internal::paint_input::{
    BackgroundRepeatRule, InterpolationQuality, PaintCornerRadii, PaintMaskComposite,
    PaintMaskMode, PaintShader,
};

use crate::PaintRect;

// cpp: paint/paint_engine.h:82-100
#[derive(Clone, PartialEq)]
pub struct DisplayMaskLayer {
    pub source_error: Option<String>,
    pub resource_id: u64,
    pub image_interpolation_quality: InterpolationQuality,
    pub document_image_container_key: Option<image_resource::ContainerKey>,
    pub paint_shader: Option<PaintShader>,
    pub clip_rect: PaintRect,
    pub clip_radii: PaintCornerRadii,
    pub source_rect: PaintRect,
    pub tile_rect: PaintRect,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub repeat_rule_x: BackgroundRepeatRule,
    pub repeat_rule_y: BackgroundRepeatRule,
    pub tile_scale: Offset,
    pub tile_spacing: Size,
    pub mode: PaintMaskMode,
    pub composite: PaintMaskComposite,
}

impl Default for DisplayMaskLayer {
    fn default() -> Self {
        Self {
            source_error: None,
            resource_id: 0,
            image_interpolation_quality: InterpolationQuality::kInterpolationMedium,
            document_image_container_key: None,
            paint_shader: None,
            clip_rect: PaintRect::default(),
            clip_radii: PaintCornerRadii::default(),
            source_rect: PaintRect::default(),
            tile_rect: PaintRect::default(),
            repeat_x: false,
            repeat_y: false,
            repeat_rule_x: BackgroundRepeatRule::kNoRepeat,
            repeat_rule_y: BackgroundRepeatRule::kNoRepeat,
            tile_scale: Offset { x: 1.0, y: 1.0 },
            tile_spacing: Size::default(),
            mode: PaintMaskMode::kAlpha,
            composite: PaintMaskComposite::kAdd,
        }
    }
}

pub fn ResolveDocumentImageContainerKey(
    image: &layoutng_assembly::internal::layout_input::PaintImage,
    target_size: Size,
    effective_zoom: f64,
    resources: Option<&PaintResources>,
    embedding_color_scheme: Option<u8>,
) -> Option<image_resource::ContainerKey> {
    let image_resource::PaintImageContent::Document(content) = &image.content else {
        return None;
    };
    let zoom = effective_zoom.max(f64::MIN_POSITIVE);
    let width = (target_size.width / zoom).round().max(1.0) as u32;
    let height = (target_size.height / zoom).round().max(1.0) as u32;
    let mut key = image_resource::ContainerKey::new(
        width,
        height,
        zoom,
        resources.map_or(1.0, |resources| resources.device_pixel_ratio),
    );
    // StyleFetchedImage::GetImage / StyleEngine::ResolveColorSchemeForEmbedding:
    // normal uses the embedding document preference; an authored color-scheme
    // uses the computed used scheme. Resolve before recording resource identity,
    // so Page publishes exactly the same key consumed by raster.
    key.preferred_color_scheme = embedding_color_scheme
        .unwrap_or_else(|| resources.map_or(0, |resources| resources.preferred_color_scheme));
    key.preferred_color_scheme_is_explicit = true;
    if let Some(current) = &image.document_container_key {
        key.view_specification = current.view_specification.clone();
    } else if let Some(initial) = content.variants().first() {
        key.view_specification = initial.container_key.view_specification.clone();
    }
    Some(key)
}
