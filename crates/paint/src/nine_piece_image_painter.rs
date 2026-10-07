#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::{
    BoxDecorationBreak, Display, Offset, PaintImage, Size,
};
use layoutng_assembly::internal::paint_input::PaintImageTileRule;

use crate::geometry_mapper::MapRectToRoot;
use crate::nine_piece_image_grid::{NinePiece, NinePieceDrawInfo, NinePieceImageGrid};
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};
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
fn FindImage<'a>(node: &'a PaintTreeNode<'_>, id: u64) -> &'a PaintImage {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let Some(resources) = fragment.paint.resources.as_ref() else {
        panic!("border image has no PaintResources");
    };
    let Some(found) = resources.images.iter().find(|image| image.id == id) else {
        panic!("border image resource id is missing");
    };
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
    found
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

// cpp: paint/nine_piece_image_painter.h:7-14
pub struct NinePieceImagePainter;

impl NinePieceImagePainter {
    // cpp: paint/nine_piece_image_painter.h:10-13
    // cpp: paint/nine_piece_image_painter.cc:90-211
    pub fn Paint(
        node: &PaintTreeNode<'_>,
        context: &RefCell<PaintContext<'_>>,
        phase: PaintPhase,
    ) -> bool {
        let fragment = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(border_image) = fragment.paint.style.border_image.as_ref() else {
            return false;
        };
        let image = FindImage(node, border_image.resource_id);
        let outsets = &border_image.outsets;
        let sliced_inline_strip = fragment.paint.display == Display::kInline
            && fragment.paint.box_decoration_break == BoxDecorationBreak::kSlice
            && fragment.paint.stitched_decoration.is_some();
        let mut decoration_area = PaintRect {
            x: node.paint_offset.x,
            y: node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        };
        if sliced_inline_strip {
            let stitched = fragment.paint.stitched_decoration.as_ref().unwrap();
            decoration_area = PaintRect {
                x: node.paint_offset.x + stitched.fragment_origin.x - stitched.fragment_offset.x,
                y: node.paint_offset.y + stitched.fragment_origin.y - stitched.fragment_offset.y,
                width: stitched.size.width,
                height: stitched.size.height,
            };
        }
        let area = PaintRect {
            x: decoration_area.x - outsets.left,
            y: decoration_area.y - outsets.top,
            width: decoration_area.width + outsets.left + outsets.right,
            height: decoration_area.height + outsets.top + outsets.bottom,
        };
        let mut resolved = *border_image;
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
        let grid = NinePieceImageGrid::new(
            &resolved,
            Size {
                width: image.width as f64,
                height: image.height as f64,
            },
            area,
        );
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
            if !ShouldTile(&info) {
                context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawImageRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: info.destination,
                        source_rect: info.source,
                        resource_id: image.id,
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
