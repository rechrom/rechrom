#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{
    BoxDecorationBreak, Display, Offset, PaintImage, Size,
};
use layoutng_assembly::internal::paint_input::{
    NinePieceImagePaint, PaintImageTileRule, PaintReflectionMaskLayer,
};

use crate::nine_piece_image_grid::{NinePiece, NinePieceDrawInfo, NinePieceImageGrid};
use crate::paint_shader_resolver::ResolvePaintShader;
use crate::pre_paint_tree_walk::PaintTreeNode;
use paint_model::{PaintRect, ResolveDocumentImageContainerKey};

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

// cpp: paint/nine_piece_image_painter.cc:82-86
fn ShouldTile(info: &NinePieceDrawInfo) -> bool {
    !info.is_corner_piece
        && (info.horizontal_rule != PaintImageTileRule::kStretch
            || info.vertical_rule != PaintImageTileRule::kStretch)
}

// cpp: paint/nine_piece_image_painter.h:7-14
pub struct NinePieceImageGeometry;

impl NinePieceImageGeometry {
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
                fragment.paint.resources.as_deref(),
                (!fragment.paint.style.color_scheme_flags_is_normal)
                    .then_some(u8::from(fragment.paint.style.used_dark_color_scheme)),
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
}
