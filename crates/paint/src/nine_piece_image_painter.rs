#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::{
    BoxDecorationBreak, Display, Offset, PaintImage, Size, TransformMatrix,
};
use layoutng_assembly::internal::paint_input::{
    NinePieceImagePaint, PaintImageTileRule, PaintReflectionMaskLayer, PaintShader,
};

use crate::geometry_mapper::{MapRectToRoot, MultiplyTransforms, TranslationTransform};
use crate::nine_piece_image_grid::{NinePiece, NinePieceDrawInfo, NinePieceImageGrid};
use crate::paint_context::PaintContext;
use crate::paint_engine::{
    DisplayItem, DisplayItemType, PaintPhase, ResolveDocumentImageContainerKey,
};
use crate::paint_shader_resolver::ResolvePaintShader;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/nine_piece_image_painter.cc:14-18
struct TileParameters {
    scale_factor: f64,
    phase: f64,
    spacing: f64,
}

impl Default for TileParameters {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            phase: 0.0,
            spacing: 0.0,
        }
    }
}

// cpp: paint/nine_piece_image_painter.cc:20-48
fn ComputeTileParameters(
    rule: PaintImageTileRule,
    destination: f64,
    source: f64,
) -> Option<TileParameters> {
    if !destination.is_finite() || !source.is_finite() || destination <= 0.0 || source <= 0.0 {
        panic!("nine-piece tile extent is invalid");
    }
    match rule {
        PaintImageTileRule::kRound => {
            let repetitions = (destination / source).round().max(1.0);
            Some(TileParameters {
                scale_factor: destination / (source * repetitions),
                phase: 0.0,
                spacing: 0.0,
            })
        }
        PaintImageTileRule::kRepeat => Some(TileParameters {
            scale_factor: 1.0,
            phase: (destination - source) / 2.0,
            spacing: 0.0,
        }),
        PaintImageTileRule::kSpace => {
            let repetitions = (destination / source).floor();
            if repetitions < 1.0 {
                return None;
            }
            let spacing = (destination - source * repetitions) / (repetitions + 1.0);
            Some(TileParameters {
                scale_factor: 1.0,
                phase: spacing,
                spacing,
            })
        }
        PaintImageTileRule::kStretch => Some(TileParameters::default()),
    }
}

// cpp: paint/nine_piece_image_painter.cc:50-68
fn FindImage<'a>(node: &'a PaintTreeNode<'_>, id: u64) -> Option<&'a PaintImage> {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let Some(resources) = fragment.paint.resources.as_ref() else {
        return None;
    };
    let found = resources.images.iter().find(|image| image.id == id)?;
    if found.width == 0
        || found.height == 0
        || !found.resolution_scale.is_finite()
        || found.resolution_scale <= 0.0
        || found
            .BitmapPixels()
            .is_some_and(|pixels| pixels.len() != found.width as usize * found.height as usize * 4)
    {
        panic!("border image resource is invalid");
    }
    Some(found)
}

// cpp: paint/nine_piece_image_painter.cc:70-73
fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

// cpp: paint/nine_piece_image_painter.cc:75-80
fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(cull_rect) = node.cull_rect else {
        return true;
    };
    let mapped = MapRectToRoot(*rect, &node.transforms);
    mapped.is_none_or(|mapped| Intersects(&mapped, &cull_rect))
}

// cpp: paint/nine_piece_image_painter.cc:82-86
fn ShouldTile(info: &NinePieceDrawInfo) -> bool {
    !info.is_corner_piece
        && (info.horizontal_rule != PaintImageTileRule::kStretch
            || info.vertical_rule != PaintImageTileRule::kStretch)
}

fn ScaleTransform(x: f64, y: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    output.values[0] = x;
    output.values[5] = y;
    output
}

/// Paint one source-image subrect through the same affine crop mapping used
/// by DrawImageRect. Keeping the resolved gradient in source coordinates is
/// important: each border-image piece is a crop of one generated image, not a
/// newly resolved gradient whose geometry follows the destination piece.
fn AppendGradientPiece(
    node: &PaintTreeNode<'_>,
    context: &RefCell<PaintContext<'_>>,
    phase: PaintPhase,
    source: PaintRect,
    destination: PaintRect,
    shader: &PaintShader,
) {
    if source.width <= 0.0
        || source.height <= 0.0
        || destination.width <= 0.0
        || destination.height <= 0.0
    {
        return;
    }
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let scale = ScaleTransform(
        destination.width / source.width,
        destination.height / source.height,
    );
    let transform = MultiplyTransforms(
        &TranslationTransform(destination.x, destination.y),
        &MultiplyTransforms(&scale, &TranslationTransform(-source.x, -source.y)),
    );
    let mut output = context.borrow_mut();
    output.Append(
        DisplayItem {
            r#type: DisplayItemType::kSave,
            phase,
            node_id: fragment.node_id,
            ..Default::default()
        },
        node,
    );
    output.Append(
        DisplayItem {
            r#type: DisplayItemType::kConcat,
            phase,
            node_id: fragment.node_id,
            transform,
            ..Default::default()
        },
        node,
    );
    output.Append(
        DisplayItem {
            r#type: DisplayItemType::kDrawGradientRect,
            phase,
            node_id: fragment.node_id,
            rect: source,
            tile_rect: source,
            paint_shader: Some(shader.clone()),
            ..Default::default()
        },
        node,
    );
    output.Append(
        DisplayItem {
            r#type: DisplayItemType::kRestore,
            phase,
            node_id: fragment.node_id,
            ..Default::default()
        },
        node,
    );
}

fn AppendGradientTiledPiece(
    node: &PaintTreeNode<'_>,
    context: &RefCell<PaintContext<'_>>,
    phase: PaintPhase,
    info: &NinePieceDrawInfo,
    shader: &PaintShader,
) {
    let horizontal = ComputeTileParameters(
        info.horizontal_rule,
        info.destination.width,
        info.source.width * info.tile_scale.x,
    );
    let vertical = ComputeTileParameters(
        info.vertical_rule,
        info.destination.height,
        info.source.height * info.tile_scale.y,
    );
    let (Some(horizontal), Some(vertical)) = (horizontal, vertical) else {
        return;
    };
    let tile_width = info.source.width * info.tile_scale.x * horizontal.scale_factor;
    let tile_height = info.source.height * info.tile_scale.y * vertical.scale_factor;
    let step_x = tile_width + horizontal.spacing;
    let step_y = tile_height + vertical.spacing;
    if tile_width <= 0.0 || tile_height <= 0.0 || step_x <= 0.0 || step_y <= 0.0 {
        return;
    }
    let nominal_x = info.destination.x + horizontal.phase;
    let nominal_y = info.destination.y + vertical.phase;
    let first_x = if info.horizontal_rule != PaintImageTileRule::kStretch {
        nominal_x - ((nominal_x - info.destination.x) / step_x).ceil() * step_x
    } else {
        nominal_x
    };
    let first_y = if info.vertical_rule != PaintImageTileRule::kStretch {
        nominal_y - ((nominal_y - info.destination.y) / step_y).ceil() * step_y
    } else {
        nominal_y
    };
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kSave,
            phase,
            node_id: fragment.node_id,
            ..Default::default()
        },
        node,
    );
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kClipRect,
            phase,
            node_id: fragment.node_id,
            rect: info.destination,
            antialias: false,
            ..Default::default()
        },
        node,
    );
    let mut y = first_y;
    let mut tile_count = 0_usize;
    while y < info.destination.y + info.destination.height && tile_count < 10_000 {
        let mut x = first_x;
        while x < info.destination.x + info.destination.width && tile_count < 10_000 {
            AppendGradientPiece(
                node,
                context,
                phase,
                info.source,
                PaintRect {
                    x,
                    y,
                    width: tile_width,
                    height: tile_height,
                },
                shader,
            );
            tile_count += 1;
            x += if info.horizontal_rule != PaintImageTileRule::kStretch {
                step_x
            } else {
                info.destination.width + step_x
            };
        }
        y += if info.vertical_rule != PaintImageTileRule::kStretch {
            step_y
        } else {
            info.destination.height + step_y
        };
    }
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kRestore,
            phase,
            node_id: fragment.node_id,
            ..Default::default()
        },
        node,
    );
}

// cpp: paint/nine_piece_image_painter.h:7-14
pub struct NinePieceImagePainter;

impl NinePieceImagePainter {
    pub fn ReferenceRect(node: &PaintTreeNode<'_>) -> PaintRect {
        let fragment = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let mut decoration_area = PaintRect {
            x: node.paint_offset.x,
            y: node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        };
        if fragment.paint.display == Display::kInline
            && fragment.paint.box_decoration_break == BoxDecorationBreak::kSlice
            && fragment.paint.stitched_decoration.is_some()
        {
            let stitched = fragment.paint.stitched_decoration.as_ref().unwrap();
            decoration_area = PaintRect {
                x: node.paint_offset.x + stitched.fragment_origin.x - stitched.fragment_offset.x,
                y: node.paint_offset.y + stitched.fragment_origin.y - stitched.fragment_offset.y,
                width: stitched.size.width,
                height: stitched.size.height,
            };
        }
        decoration_area
    }

    pub fn Bounds(node: &PaintTreeNode<'_>, image: &NinePieceImagePaint) -> PaintRect {
        let decoration_area = Self::ReferenceRect(node);
        PaintRect {
            x: decoration_area.x - image.outsets.left,
            y: decoration_area.y - image.outsets.top,
            width: decoration_area.width + image.outsets.left + image.outsets.right,
            height: decoration_area.height + image.outsets.top + image.outsets.bottom,
        }
    }

    pub fn ReflectionReferenceRect(node: &PaintTreeNode<'_>) -> PaintRect {
        let fragment = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        fragment.paint.stitched_decoration.as_ref().map_or(
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: fragment.size.width,
                height: fragment.size.height,
            },
            |stitched| PaintRect {
                x: 0.0,
                y: 0.0,
                width: stitched.size.width,
                height: stitched.size.height,
            },
        )
    }

    fn ReflectionBounds(node: &PaintTreeNode<'_>, image: &NinePieceImagePaint) -> PaintRect {
        let reference = Self::ReflectionReferenceRect(node);
        PaintRect {
            x: reference.x - image.outsets.left,
            y: reference.y - image.outsets.top,
            width: reference.width + image.outsets.left + image.outsets.right,
            height: reference.height + image.outsets.top + image.outsets.bottom,
        }
    }

    /// Lower the private PaintRecord used by BoxReflection into immutable
    /// nine-piece source/tile draws. Unlike CSS mask layers this is always an
    /// alpha mask and is consumed only by the reflected input branch.
    pub fn ResolveReflectionMask(
        node: &PaintTreeNode<'_>,
        image: &NinePieceImagePaint,
    ) -> Vec<PaintReflectionMaskLayer> {
        let fragment = node.fragment.as_deref().expect("reflection mask fragment");
        let fetched = if image.shader.is_none() {
            FindImage(node, image.resource_id)
        } else {
            None
        };
        if fetched.is_none() && image.shader.is_none() {
            // A pending mask image produces an empty PaintRecord, therefore
            // the reflected SrcIn branch is transparent.
            return Vec::new();
        }
        let source_size = fetched.map_or(image.image_size, |image| Size {
            width: f64::from(image.width),
            height: f64::from(image.height),
        });
        if source_size.width <= 0.0
            || source_size.height <= 0.0
            || !source_size.width.is_finite()
            || !source_size.height.is_finite()
        {
            return Vec::new();
        }
        let effective_zoom = f64::from(fragment.paint.effective_zoom);
        let document_image_container_key = fetched.and_then(|resource| {
            ResolveDocumentImageContainerKey(
                resource,
                Size {
                    width: f64::from(resource.width) / image.image_resolution_scale
                        * effective_zoom,
                    height: f64::from(resource.height) / image.image_resolution_scale
                        * effective_zoom,
                },
                effective_zoom,
                fragment
                    .paint
                    .resources
                    .as_ref()
                    .map_or(1.0, |resources| resources.device_pixel_ratio),
                &fragment.paint.style,
            )
        });
        let shader = image.shader.as_ref().map(|shader| {
            ResolvePaintShader(
                shader,
                fragment.paint.resources.as_deref(),
                Offset::default(),
                Some(source_size),
            )
        });
        let grid = NinePieceImageGrid::new(
            image,
            source_size,
            effective_zoom,
            Self::ReflectionBounds(node, image),
        );
        const PIECES: [NinePiece; 9] = [
            NinePiece::kTopLeft,
            NinePiece::kBottomLeft,
            NinePiece::kLeft,
            NinePiece::kTopRight,
            NinePiece::kBottomRight,
            NinePiece::kRight,
            NinePiece::kTop,
            NinePiece::kBottom,
            NinePiece::kMiddle,
        ];
        PIECES
            .into_iter()
            .filter_map(|piece| {
                let info = grid.GetDrawInfo(piece);
                if !info.is_drawable {
                    return None;
                }
                let tiled = ShouldTile(&info);
                let horizontal = tiled
                    .then(|| {
                        ComputeTileParameters(
                            info.horizontal_rule,
                            info.destination.width,
                            info.source.width * info.tile_scale.x,
                        )
                    })
                    .flatten();
                let vertical = tiled
                    .then(|| {
                        ComputeTileParameters(
                            info.vertical_rule,
                            info.destination.height,
                            info.source.height * info.tile_scale.y,
                        )
                    })
                    .flatten();
                if tiled && (horizontal.is_none() || vertical.is_none()) {
                    return None;
                }
                Some(PaintReflectionMaskLayer {
                    image: fetched.cloned(),
                    document_image_container_key: document_image_container_key.clone(),
                    paint_shader: shader.clone(),
                    image_interpolation_quality: fragment.paint.style.image_interpolation_quality,
                    clip_origin: Offset {
                        x: info.destination.x,
                        y: info.destination.y,
                    },
                    clip_size: Size {
                        width: info.destination.width,
                        height: info.destination.height,
                    },
                    source_origin: Offset {
                        x: info.source.x,
                        y: info.source.y,
                    },
                    source_size: Size {
                        width: info.source.width,
                        height: info.source.height,
                    },
                    tile_origin: Offset {
                        x: info.destination.x + horizontal.as_ref().map_or(0.0, |p| p.phase),
                        y: info.destination.y + vertical.as_ref().map_or(0.0, |p| p.phase),
                    },
                    tile_size: Size {
                        width: horizontal.as_ref().map_or(info.destination.width, |p| {
                            info.source.width * info.tile_scale.x * p.scale_factor
                        }),
                        height: vertical.as_ref().map_or(info.destination.height, |p| {
                            info.source.height * info.tile_scale.y * p.scale_factor
                        }),
                    },
                    repeat_x: tiled && info.horizontal_rule != PaintImageTileRule::kStretch,
                    repeat_y: tiled && info.vertical_rule != PaintImageTileRule::kStretch,
                    tile_spacing: Size {
                        width: horizontal.as_ref().map_or(0.0, |p| p.spacing),
                        height: vertical.as_ref().map_or(0.0, |p| p.spacing),
                    },
                })
            })
            .collect()
    }

    // cpp: paint/nine_piece_image_painter.h:10-13
    // cpp: paint/nine_piece_image_painter.cc:90-211
    pub fn Paint(
        node: &PaintTreeNode<'_>,
        context: &RefCell<PaintContext<'_>>,
        phase: PaintPhase,
    ) -> bool {
        let Some(border_image) = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment")
            .paint
            .style
            .border_image
            .as_ref()
        else {
            return false;
        };
        Self::PaintImage(node, context, phase, border_image)
    }

    pub fn PaintMaskBox(
        node: &PaintTreeNode<'_>,
        context: &RefCell<PaintContext<'_>>,
        phase: PaintPhase,
    ) -> bool {
        let Some(mask_box_image) = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment")
            .paint
            .style
            .mask_box_image
            .as_ref()
        else {
            return false;
        };
        Self::PaintImage(node, context, phase, mask_box_image)
    }

    fn PaintImage(
        node: &PaintTreeNode<'_>,
        context: &RefCell<PaintContext<'_>>,
        phase: PaintPhase,
        border_image: &NinePieceImagePaint,
    ) -> bool {
        let fragment = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        // A specified border image suppresses ordinary border painting while
        // its resource is pending. Blink returns true from Paint() in this
        // state and records no nine-piece draw operations.
        let image = if border_image.shader.is_none() {
            FindImage(node, border_image.resource_id)
        } else {
            None
        };
        if image.is_none() && border_image.shader.is_none() {
            return true;
        }
        let outsets = &border_image.outsets;
        let sliced_inline_strip = fragment.paint.display == Display::kInline
            && fragment.paint.box_decoration_break == BoxDecorationBreak::kSlice
            && fragment.paint.stitched_decoration.is_some();
        let area = Self::Bounds(node, border_image);
        let source_size = if border_image.shader.is_some() {
            border_image.image_size
        } else {
            let image = image.expect("fetched border image disappeared");
            Size {
                width: f64::from(image.width),
                height: f64::from(image.height),
            }
        };
        if !source_size.width.is_finite()
            || !source_size.height.is_finite()
            || source_size.width <= 0.0
            || source_size.height <= 0.0
        {
            return true;
        }
        let effective_zoom = f64::from(fragment.paint.effective_zoom);
        let image_resolution_scale = border_image.image_resolution_scale;
        if !image_resolution_scale.is_finite() || image_resolution_scale <= 0.0 {
            panic!("border image resolution scale must be finite and positive");
        }
        let document_image_container_key = image.and_then(|image| {
            ResolveDocumentImageContainerKey(
                image,
                Size {
                    width: f64::from(image.width) / image_resolution_scale * effective_zoom,
                    height: f64::from(image.height) / image_resolution_scale * effective_zoom,
                },
                effective_zoom,
                fragment
                    .paint
                    .resources
                    .as_ref()
                    .map_or(1.0, |resources| resources.device_pixel_ratio),
                &fragment.paint.style,
            )
        });
        let generated_shader = border_image.shader.as_ref().map(|shader| {
            ResolvePaintShader(
                shader,
                fragment.paint.resources.as_deref(),
                Offset::default(),
                Some(source_size),
            )
        });
        let mut resolved = border_image.clone();
        if !sliced_inline_strip {
            if !fragment.paint.border_sides.top {
                resolved.widths.top = 0.0;
            }
            if !fragment.paint.border_sides.right {
                resolved.widths.right = 0.0;
            }
            if !fragment.paint.border_sides.bottom {
                resolved.widths.bottom = 0.0;
            }
            if !fragment.paint.border_sides.left {
                resolved.widths.left = 0.0;
            }
        }
        let grid = NinePieceImageGrid::new(&resolved, source_size, effective_zoom, area);
        if sliced_inline_strip {
            let sides = &fragment.paint.border_sides;
            let clip = PaintRect {
                x: node.paint_offset.x - if sides.left { outsets.left } else { 0.0 },
                y: node.paint_offset.y - if sides.top { outsets.top } else { 0.0 },
                width: fragment.size.width
                    + if sides.left { outsets.left } else { 0.0 }
                    + if sides.right { outsets.right } else { 0.0 },
                height: fragment.size.height
                    + if sides.top { outsets.top } else { 0.0 }
                    + if sides.bottom { outsets.bottom } else { 0.0 },
            };
            context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: clip,
                    ..Default::default()
                },
                node,
            );
        }
        // The fixed array preserves the C++ uint8_t enum order from kTopLeft through kMiddle.
        const PIECES: [NinePiece; 9] = [
            NinePiece::kTopLeft,
            NinePiece::kBottomLeft,
            NinePiece::kLeft,
            NinePiece::kTopRight,
            NinePiece::kBottomRight,
            NinePiece::kRight,
            NinePiece::kTop,
            NinePiece::kBottom,
            NinePiece::kMiddle,
        ];
        for piece in PIECES {
            let info = grid.GetDrawInfo(piece);
            if !info.is_drawable || !IsVisible(node, &info.destination) {
                continue;
            }
            if let Some(shader) = generated_shader.as_ref() {
                if ShouldTile(&info) {
                    AppendGradientTiledPiece(node, context, phase, &info, shader);
                } else {
                    AppendGradientPiece(
                        node,
                        context,
                        phase,
                        info.source,
                        info.destination,
                        shader,
                    );
                }
                continue;
            }
            let image = image.expect("border image has neither bitmap nor generated shader");
            if !ShouldTile(&info) {
                context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawImageRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: info.destination,
                        source_rect: info.source,
                        resource_id: image.id,
                        image_interpolation_quality: fragment
                            .paint
                            .style
                            .image_interpolation_quality,
                        document_image_container_key: document_image_container_key.clone(),
                        tile_rule_x: info.horizontal_rule,
                        tile_rule_y: info.vertical_rule,
                        ..Default::default()
                    },
                    node,
                );
                continue;
            }
            let horizontal = ComputeTileParameters(
                info.horizontal_rule,
                info.destination.width,
                info.source.width * info.tile_scale.x,
            );
            let vertical = ComputeTileParameters(
                info.vertical_rule,
                info.destination.height,
                info.source.height * info.tile_scale.y,
            );
            let (Some(horizontal), Some(vertical)) = (horizontal, vertical) else {
                continue;
            };
            let scale = Offset {
                x: info.tile_scale.x * horizontal.scale_factor,
                y: info.tile_scale.y * vertical.scale_factor,
            };
            let phase_offset = Offset {
                x: info.destination.x + horizontal.phase - info.source.x * scale.x,
                y: info.destination.y + vertical.phase - info.source.y * scale.y,
            };
            context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawTiledImage,
                    phase,
                    node_id: fragment.node_id,
                    rect: info.destination,
                    source_rect: info.source,
                    resource_id: image.id,
                    image_interpolation_quality: fragment.paint.style.image_interpolation_quality,
                    document_image_container_key: document_image_container_key.clone(),
                    repeat_x: info.horizontal_rule != PaintImageTileRule::kStretch,
                    repeat_y: info.vertical_rule != PaintImageTileRule::kStretch,
                    tile_rule_x: info.horizontal_rule,
                    tile_rule_y: info.vertical_rule,
                    tile_scale: scale,
                    tile_phase: phase_offset,
                    tile_spacing: Size {
                        width: horizontal.spacing,
                        height: vertical.spacing,
                    },
                    ..Default::default()
                },
                node,
            );
        }
        if sliced_inline_strip {
            context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        true
    }
}
