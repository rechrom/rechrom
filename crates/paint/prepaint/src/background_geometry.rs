#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{Offset, PaintImage, Size};
use layoutng_assembly::internal::paint_input::{
    BackgroundImageLayer, BackgroundRepeat, BackgroundRepeatRule, BackgroundSizeMode,
};

use paint_model::PaintRect;

pub struct ResolvedBackgroundTile {
    pub tile_rect: PaintRect,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub rule_x: BackgroundRepeatRule,
    pub rule_y: BackgroundRepeatRule,
    pub tile_scale: Offset,
    pub tile_spacing: Size,
}

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

struct AxisGeometry {
    origin: f64,
    spacing: f64,
    repeats: bool,
}

fn ResolveAxis(
    area_origin: f64,
    area_size: f64,
    tile_size: f64,
    position_available: f64,
    position: f64,
    position_offset: f64,
    calculation: Option<&std::sync::Arc<foundation::CalculationValue>>,
    from_far_edge: bool,
    rule: BackgroundRepeatRule,
) -> AxisGeometry {
    if tile_size <= 0.0 {
        return AxisGeometry {
            origin: area_origin,
            spacing: 0.0,
            repeats: false,
        };
    }
    let resolved_position =
        calculation.map_or(position_available * position + position_offset, |value| {
            let edge_relative = value.Evaluate(
                position_available as f32,
                &foundation::EvaluationInput::default(),
            ) as f64;
            if from_far_edge {
                position_available - edge_relative
            } else {
                edge_relative
            }
        });
    let mut result = AxisGeometry {
        origin: area_origin + resolved_position,
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

// cpp: paint/background_image_geometry.cc:45-61
fn ResolveClampedWidthForRatio(height: f64, natural_size: Size) -> f64 {
    let width = height * natural_size.width / natural_size.height;
    if natural_size.width >= 1.0 && width < 1.0 {
        1.0
    } else {
        width
    }
}

// cpp: paint/background_image_geometry.cc:45-61
fn ResolveClampedHeightForRatio(width: f64, natural_size: Size) -> f64 {
    let height = width * natural_size.height / natural_size.width;
    if natural_size.height >= 1.0 && height < 1.0 {
        1.0
    } else {
        height
    }
}

fn PixelSnappedSize(rect: &PaintRect) -> Size {
    // LayoutUnit::SnapSizeToPixel preserves a non-zero size once the fixed
    // point magnitude exceeds four raw units (4 / 64 CSS px).
    let snap = |size: f64, origin: f64| {
        let result = (origin.fract() + size).round() - origin.fract().round();
        if result == 0.0 && size.abs() > 4.0 / 64.0 {
            size.signum()
        } else {
            result
        }
    };
    Size {
        width: snap(rect.width, rect.x).max(0.0),
        height: snap(rect.height, rect.y).max(0.0),
    }
}

pub fn ResolveBackgroundTileSize(
    layer: &BackgroundImageLayer,
    positioning_area: &PaintRect,
    image: Option<&PaintImage>,
    image_orientation: image_resource::ImageOrientation,
    effective_zoom: f64,
) -> Size {
    if !positioning_area.width.is_finite()
        || !positioning_area.height.is_finite()
        || positioning_area.width < 0.0
        || positioning_area.height < 0.0
    {
        panic!("background positioning area is invalid");
    }
    if layer.width.is_some_and(|value| {
        !value.is_finite() || (value < 0.0 && layer.width_percentage.is_none())
    }) || layer.height.is_some_and(|value| {
        !value.is_finite() || (value < 0.0 && layer.height_percentage.is_none())
    }) || layer
        .width_percentage
        .is_some_and(|value| !value.is_finite() || (value < 0.0 && layer.width.is_none()))
        || layer
            .height_percentage
            .is_some_and(|value| !value.is_finite() || (value < 0.0 && layer.height.is_none()))
    {
        panic!("explicit background size must be finite and non-negative");
    }

    let snapped_positioning_size = PixelSnappedSize(positioning_area);
    let mut width = positioning_area.width;
    let mut height = positioning_area.height;
    // Fixed and percentage components are additive for calculated lengths.
    // Plain fixed/percentage inputs continue to have just one populated term.
    let specified_width = layer.width_calculation.as_ref().map_or_else(
        || {
            (layer.width.is_some() || layer.width_percentage.is_some()).then(|| {
                layer.width.unwrap_or(0.0)
                    + layer.width_percentage.unwrap_or(0.0) * positioning_area.width
            })
        },
        |value| {
            Some(value.Evaluate(
                positioning_area.width as f32,
                &foundation::EvaluationInput::default(),
            ) as f64)
        },
    );
    let specified_height = layer.height_calculation.as_ref().map_or_else(
        || {
            (layer.height.is_some() || layer.height_percentage.is_some()).then(|| {
                layer.height.unwrap_or(0.0)
                    + layer.height_percentage.unwrap_or(0.0) * positioning_area.height
            })
        },
        |value| {
            Some(value.Evaluate(
                positioning_area.height as f32,
                &foundation::EvaluationInput::default(),
            ) as f64)
        },
    );
    if !effective_zoom.is_finite() || effective_zoom <= 0.0 {
        panic!("background image effective zoom must be finite and positive");
    }
    if let Some(image) = image {
        if image.width == 0
            || image.height == 0
            || !layer.image_resolution_scale.is_finite()
            || layer.image_resolution_scale <= 0.0
        {
            panic!("background image dimensions are invalid");
        }
        // Background geometry is recorded in LayoutNG's zoomed coordinate
        // space. Blink asks StyleImage::ImageSize(style.EffectiveZoom(), ...)
        // before resolving auto/round/space geometry. Keep the decoded pixel
        // density separate: resolution_scale converts resource pixels to CSS
        // pixels, then effective_zoom converts those CSS pixels to this
        // artifact's coordinates.
        let (oriented_width, oriented_height) =
            image_orientation.oriented_size(f64::from(image.width), f64::from(image.height));
        let intrinsic_width = oriented_width / layer.image_resolution_scale * effective_zoom;
        let intrinsic_height = oriented_height / layer.image_resolution_scale * effective_zoom;
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
                // Chromium deliberately sizes contain/cover against the
                // pixel-snapped positioning area so the selected dimension
                // reaches the actual painted edge.
                let x_scale = snapped_positioning_size.width / intrinsic_width;
                let y_scale = snapped_positioning_size.height / intrinsic_height;
                let scale = if layer.size_mode == BackgroundSizeMode::kContain {
                    x_scale.min(y_scale)
                } else {
                    x_scale.max(y_scale)
                };
                width *= scale;
                height *= scale;
                if layer.size_mode == BackgroundSizeMode::kContain {
                    if width != snapped_positioning_size.width {
                        width = width.round().max(1.0);
                    }
                    if height != snapped_positioning_size.height {
                        height = height.round().max(1.0);
                    }
                } else {
                    if width != snapped_positioning_size.width {
                        width = width.max(1.0);
                    }
                    if height != snapped_positioning_size.height {
                        height = height.max(1.0);
                    }
                }
            }
        } else if layer.size_mode == BackgroundSizeMode::kExplicit {
            if let (Some(specified_width), Some(specified_height)) =
                (specified_width, specified_height)
            {
                width = specified_width;
                height = specified_height;
            } else if let Some(specified_width) = specified_width {
                width = specified_width;
                height = ResolveClampedHeightForRatio(
                    width,
                    Size {
                        width: intrinsic_width,
                        height: intrinsic_height,
                    },
                );
            } else if let Some(specified_height) = specified_height {
                height = specified_height;
                width = ResolveClampedWidthForRatio(
                    height,
                    Size {
                        width: intrinsic_width,
                        height: intrinsic_height,
                    },
                );
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
    let snapped_positioning_size = PixelSnappedSize(positioning_area);
    // Chromium computes both available extents before either round axis mutates
    // the tile. Repeat/no-repeat resolve against the unsnapped extent; round
    // resolves against the snapped extent. An orthogonal auto-size change does
    // not rewrite either position reference.
    let original_available_width = positioning_area.width - tile_size.width;
    let original_available_height = positioning_area.height - tile_size.height;
    let snapped_available_width = snapped_positioning_size.width - tile_size.width;
    let snapped_available_height = snapped_positioning_size.height - tile_size.height;
    let auto_width = layer.size_mode == BackgroundSizeMode::kAuto
        || (layer.size_mode == BackgroundSizeMode::kExplicit
            && layer.width.is_none()
            && layer.width_percentage.is_none()
            && layer.width_calculation.is_none());
    let auto_height = layer.size_mode == BackgroundSizeMode::kAuto
        || (layer.size_mode == BackgroundSizeMode::kExplicit
            && layer.height.is_none()
            && layer.height_percentage.is_none()
            && layer.height_calculation.is_none());
    if rule_x == BackgroundRepeatRule::kRound
        && snapped_positioning_size.width > 0.0
        && tile_size.width > 0.0
    {
        let count = (snapped_positioning_size.width / tile_size.width)
            .round()
            .max(1.0);
        let rounded_width = snapped_positioning_size.width / count;
        if rule_y != BackgroundRepeatRule::kRound && auto_height {
            tile_size.height = ResolveClampedHeightForRatio(rounded_width, tile_size);
        }
        tile_size.width = rounded_width;
    }
    if rule_y == BackgroundRepeatRule::kRound
        && snapped_positioning_size.height > 0.0
        && tile_size.height > 0.0
    {
        let count = (snapped_positioning_size.height / tile_size.height)
            .round()
            .max(1.0);
        let rounded_height = snapped_positioning_size.height / count;
        if rule_x != BackgroundRepeatRule::kRound && auto_width {
            tile_size.width = ResolveClampedWidthForRatio(rounded_height, tile_size);
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
        snapped_positioning_size.width,
        tile_size.width,
        if rule_x == BackgroundRepeatRule::kRound {
            snapped_available_width
        } else {
            original_available_width
        },
        layer.position.x,
        layer.position_offset.x,
        layer.position_x_calculation.as_ref(),
        layer.position_x_from_right,
        rule_x,
    );
    let y = ResolveAxis(
        positioning_area.y,
        snapped_positioning_size.height,
        tile_size.height,
        if rule_y == BackgroundRepeatRule::kRound {
            snapped_available_height
        } else {
            original_available_height
        },
        layer.position.y,
        layer.position_offset.y,
        layer.position_y_calculation.as_ref(),
        layer.position_y_from_bottom,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intrinsic_tile_size_uses_the_effective_image_orientation() {
        let image = PaintImage {
            width: 40,
            height: 20,
            resolution_scale: 2.0,
            ..PaintImage::default()
        };
        let area = PaintRect {
            width: 100.0,
            height: 100.0,
            ..PaintRect::default()
        };
        let layer = BackgroundImageLayer {
            image_resolution_scale: 2.0,
            ..BackgroundImageLayer::default()
        };

        assert_eq!(
            ResolveBackgroundTileSize(
                &layer,
                &area,
                Some(&image),
                image_resource::ImageOrientation::kDefault,
                1.0,
            ),
            Size {
                width: 20.0,
                height: 10.0,
            }
        );
        assert_eq!(
            ResolveBackgroundTileSize(
                &layer,
                &area,
                Some(&image),
                image_resource::ImageOrientation::kRotate90,
                1.0,
            ),
            Size {
                width: 10.0,
                height: 20.0,
            }
        );
    }
}
