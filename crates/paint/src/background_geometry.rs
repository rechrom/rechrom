#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{Offset, PaintImage, Size};
use layoutng_assembly::internal::paint_input::{
    BackgroundImageLayer, BackgroundRepeat, BackgroundRepeatRule, BackgroundSizeMode,
};

use crate::PaintRect;

// cpp: paint/background_geometry.h:7-16
pub struct ResolvedBackgroundTile {
    pub tile_rect: PaintRect,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub rule_x: BackgroundRepeatRule,
    pub rule_y: BackgroundRepeatRule,
    pub tile_scale: Offset,
    pub tile_spacing: Size,
}

// cpp: paint/background_geometry.cc:10-22
fn LegacyRule(repeat: BackgroundRepeat, horizontal: bool) -> BackgroundRepeatRule {
    match repeat {
        BackgroundRepeat::kRepeat => BackgroundRepeatRule::kRepeat,
        BackgroundRepeat::kRepeatX => {
            if horizontal {
                BackgroundRepeatRule::kRepeat
            } else {
                BackgroundRepeatRule::kNoRepeat
            }
        }
        BackgroundRepeat::kRepeatY => {
            if horizontal {
                BackgroundRepeatRule::kNoRepeat
            } else {
                BackgroundRepeatRule::kRepeat
            }
        }
        BackgroundRepeat::kNoRepeat => BackgroundRepeatRule::kNoRepeat,
    }
}

// cpp: paint/background_geometry.cc:24-28
struct AxisGeometry {
    origin: f64,
    spacing: f64,
    repeats: bool,
}

// cpp: paint/background_geometry.cc:30-64
fn ResolveAxis(
    area_origin: f64,
    area_size: f64,
    tile_size: f64,
    position: f64,
    position_offset: f64,
    rule: BackgroundRepeatRule,
) -> AxisGeometry {
    if tile_size <= 0.0 {
        return AxisGeometry {
            origin: area_origin,
            spacing: 0.0,
            repeats: false,
        };
    }
    let mut result = AxisGeometry {
        origin: area_origin + (area_size - tile_size) * position + position_offset,
        spacing: 0.0,
        repeats: false,
    };
    match rule {
        BackgroundRepeatRule::kRepeat | BackgroundRepeatRule::kRound => {
            result.repeats = true;
            if rule == BackgroundRepeatRule::kRound {
                result.origin = area_origin;
            }
        }
        BackgroundRepeatRule::kSpace => {
            let count = (area_size / tile_size).floor();
            if count >= 2.0 {
                result.origin = area_origin;
                result.spacing = (area_size - count * tile_size) / (count - 1.0);
                result.repeats = true;
            }
        }
        BackgroundRepeatRule::kNoRepeat => {}
    }
    result
}

// cpp: paint/background_geometry.h:20-22
// cpp: paint/background_geometry.cc:68-149
pub fn ResolveBackgroundTileSize(
    layer: &BackgroundImageLayer,
    positioning_area: &PaintRect,
    image: Option<&PaintImage>,
) -> Size {
    if !positioning_area.width.is_finite()
        || !positioning_area.height.is_finite()
        || positioning_area.width < 0.0
        || positioning_area.height < 0.0
    {
        panic!("background positioning area is invalid");
    }
    if layer
        .width
        .is_some_and(|value| !value.is_finite() || value < 0.0)
        || layer
            .height
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        || layer
            .width_percentage
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        || layer
            .height_percentage
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        || (layer.width.is_some() && layer.width_percentage.is_some())
        || (layer.height.is_some() && layer.height_percentage.is_some())
    {
        panic!("explicit background size must be finite and non-negative");
    }

    let mut width = positioning_area.width;
    let mut height = positioning_area.height;
    let specified_width = layer.width.or_else(|| {
        layer
            .width_percentage
            .map(|value| value * positioning_area.width)
    });
    let specified_height = layer.height.or_else(|| {
        layer
            .height_percentage
            .map(|value| value * positioning_area.height)
    });
    if let Some(image) = image {
        if image.width == 0
            || image.height == 0
            || !image.resolution_scale.is_finite()
            || image.resolution_scale <= 0.0
        {
            panic!("background image dimensions are invalid");
        }
        let intrinsic_width = f64::from(image.width) / image.resolution_scale;
        let intrinsic_height = f64::from(image.height) / image.resolution_scale;
        width = intrinsic_width;
        height = intrinsic_height;
        if matches!(
            layer.size_mode,
            BackgroundSizeMode::kContain | BackgroundSizeMode::kCover
        ) {
            if positioning_area.width == 0.0 || positioning_area.height == 0.0 {
                width = 0.0;
                height = 0.0;
            } else {
                let x_scale = positioning_area.width / intrinsic_width;
                let y_scale = positioning_area.height / intrinsic_height;
                let scale = if layer.size_mode == BackgroundSizeMode::kContain {
                    x_scale.min(y_scale)
                } else {
                    x_scale.max(y_scale)
                };
                width *= scale;
                height *= scale;
            }
        } else if layer.size_mode == BackgroundSizeMode::kExplicit {
            if let (Some(specified_width), Some(specified_height)) =
                (specified_width, specified_height)
            {
                width = specified_width;
                height = specified_height;
            } else if let Some(specified_width) = specified_width {
                width = specified_width;
                height = intrinsic_height * width / intrinsic_width;
            } else if let Some(specified_height) = specified_height {
                height = specified_height;
                width = intrinsic_width * height / intrinsic_height;
            }
        }
    } else if layer.size_mode == BackgroundSizeMode::kExplicit {
        width = specified_width.unwrap_or(positioning_area.width);
        height = specified_height.unwrap_or(positioning_area.height);
    }
    if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
        panic!("computed background tile size is invalid");
    }
    Size { width, height }
}

// cpp: paint/background_geometry.h:24-28
// cpp: paint/background_geometry.cc:151-226
pub fn ResolveBackgroundTile(
    layer: &BackgroundImageLayer,
    positioning_area: &PaintRect,
    mut tile_size: Size,
) -> ResolvedBackgroundTile {
    if !positioning_area.x.is_finite()
        || !positioning_area.y.is_finite()
        || !positioning_area.width.is_finite()
        || !positioning_area.height.is_finite()
        || positioning_area.width < 0.0
        || positioning_area.height < 0.0
        || !tile_size.width.is_finite()
        || !tile_size.height.is_finite()
        || tile_size.width < 0.0
        || tile_size.height < 0.0
        || !layer.position.x.is_finite()
        || !layer.position.y.is_finite()
        || !layer.position_offset.x.is_finite()
        || !layer.position_offset.y.is_finite()
    {
        panic!("background repeat geometry is invalid");
    }

    let rule_x = layer
        .repeat_rule_x
        .unwrap_or(LegacyRule(layer.repeat, true));
    let rule_y = layer
        .repeat_rule_y
        .unwrap_or(LegacyRule(layer.repeat, false));
    let original_size = tile_size;
    let auto_width = layer.size_mode == BackgroundSizeMode::kAuto
        || (layer.size_mode == BackgroundSizeMode::kExplicit
            && layer.width.is_none()
            && layer.width_percentage.is_none());
    let auto_height = layer.size_mode == BackgroundSizeMode::kAuto
        || (layer.size_mode == BackgroundSizeMode::kExplicit
            && layer.height.is_none()
            && layer.height_percentage.is_none());
    if rule_x == BackgroundRepeatRule::kRound
        && positioning_area.width > 0.0
        && tile_size.width > 0.0
    {
        let count = (positioning_area.width / tile_size.width).round().max(1.0);
        let rounded_width = positioning_area.width / count;
        if rule_y != BackgroundRepeatRule::kRound && auto_height {
            tile_size.height *= rounded_width / tile_size.width;
        }
        tile_size.width = rounded_width;
    }
    if rule_y == BackgroundRepeatRule::kRound
        && positioning_area.height > 0.0
        && tile_size.height > 0.0
    {
        let count = (positioning_area.height / tile_size.height)
            .round()
            .max(1.0);
        let rounded_height = positioning_area.height / count;
        if rule_x != BackgroundRepeatRule::kRound && auto_width {
            tile_size.width *= rounded_height / tile_size.height;
        }
        tile_size.height = rounded_height;
    }
    if tile_size.width < 0.0
        || tile_size.height < 0.0
        || !tile_size.width.is_finite()
        || !tile_size.height.is_finite()
    {
        panic!("rounded background tile size is invalid");
    }
    let x = ResolveAxis(
        positioning_area.x,
        positioning_area.width,
        tile_size.width,
        layer.position.x,
        layer.position_offset.x,
        rule_x,
    );
    let y = ResolveAxis(
        positioning_area.y,
        positioning_area.height,
        tile_size.height,
        layer.position.y,
        layer.position_offset.y,
        rule_y,
    );
    ResolvedBackgroundTile {
        tile_rect: PaintRect {
            x: x.origin,
            y: y.origin,
            width: tile_size.width,
            height: tile_size.height,
        },
        repeat_x: x.repeats,
        repeat_y: y.repeats,
        rule_x,
        rule_y,
        tile_scale: Offset {
            x: if original_size.width > 0.0 {
                tile_size.width / original_size.width
            } else {
                0.0
            },
            y: if original_size.height > 0.0 {
                tile_size.height / original_size.height
            } else {
                0.0
            },
        },
        tile_spacing: Size {
            width: x.spacing,
            height: y.spacing,
        },
    }
}
