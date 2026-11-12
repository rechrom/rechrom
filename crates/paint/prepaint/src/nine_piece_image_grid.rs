#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use foundation::{EvaluationInput, LayoutUnit};
use layoutng_assembly::fragment_tree::FragmentBoxSides;
use layoutng_assembly::internal::layout_input::{Edges, Offset, Size};
use layoutng_assembly::internal::paint_input::{
    NinePieceImagePaint, NinePieceImageWidth, PaintImageTileRule,
};

use paint_model::PaintRect;

// cpp: paint/nine_piece_image_grid.h:22-34
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NinePiece {
    kTopLeft,
    kBottomLeft,
    kLeft,
    kTopRight,
    kBottomRight,
    kRight,
    kTop,
    kBottom,
    kMiddle,
    kCount,
}

// cpp: paint/nine_piece_image_grid.h:74-91
pub struct NinePieceDrawInfo {
    pub is_drawable: bool,
    pub is_corner_piece: bool,
    pub destination: PaintRect,
    pub source: PaintRect,
    pub tile_scale: Offset,
    pub horizontal_rule: PaintImageTileRule,
    pub vertical_rule: PaintImageTileRule,
}

impl Default for NinePieceDrawInfo {
    fn default() -> Self {
        Self {
            is_drawable: false,
            is_corner_piece: false,
            destination: PaintRect::default(),
            source: PaintRect::default(),
            tile_scale: Offset { x: 1.0, y: 1.0 },
            horizontal_rule: PaintImageTileRule::kStretch,
            vertical_rule: PaintImageTileRule::kStretch,
        }
    }
}

fn IsFiniteNonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn ValidateEdges(edges: &Edges, name: &str) {
    if !IsFiniteNonnegative(edges.top)
        || !IsFiniteNonnegative(edges.right)
        || !IsFiniteNonnegative(edges.bottom)
        || !IsFiniteNonnegative(edges.left)
    {
        panic!("{name} must be finite and nonnegative");
    }
}

fn QuantizeLayoutUnit(value: f64) -> f64 {
    LayoutUnit::from_f32(value as f32).ToDouble()
}

// cpp: paint/nine_piece_image_grid.cc:15-24
fn ComputeEdgeWidth(
    width: &NinePieceImageWidth,
    border_side: f64,
    image_side: f64,
    box_extent: f64,
) -> f64 {
    let resolved = match width {
        NinePieceImageWidth::Number(number) => number * border_side,
        NinePieceImageWidth::Auto => image_side,
        NinePieceImageWidth::Length {
            pixels,
            percentage,
            calculation,
        } => calculation.as_ref().map_or_else(
            || pixels + box_extent * percentage / 100.0,
            |calculation| {
                f64::from(calculation.Evaluate(box_extent as f32, &EvaluationInput::default()))
            },
        ),
    };
    QuantizeLayoutUnit(resolved.max(0.0))
}

fn SnapToPixel(value: f64, scale: f64) -> f64 {
    (value * scale + 0.5).floor() / scale
}

fn FloorToPixel(value: f64, scale: f64) -> f64 {
    (value * scale).floor() / scale
}

// cpp: paint/nine_piece_image_grid.cc:41-75
fn SnapEdgeWidths(edge_widths: &Edges, box_size: Size, pixel_snap_scale: f64) -> Edges {
    let mut snapped = Edges::default();
    let epsilon = f64::from(LayoutUnit::Epsilon());
    if box_size.width - edge_widths.left - edge_widths.right <= epsilon {
        snapped.left = SnapToPixel(edge_widths.left, pixel_snap_scale);
        snapped.right = (box_size.width - snapped.left).max(0.0);
    } else {
        snapped.left = FloorToPixel(edge_widths.left, pixel_snap_scale);
        snapped.right = FloorToPixel(edge_widths.right, pixel_snap_scale);
    }
    if box_size.height - edge_widths.top - edge_widths.bottom <= epsilon {
        snapped.top = SnapToPixel(edge_widths.top, pixel_snap_scale);
        snapped.bottom = (box_size.height - snapped.top).max(0.0);
    } else {
        snapped.top = FloorToPixel(edge_widths.top, pixel_snap_scale);
        snapped.bottom = FloorToPixel(edge_widths.bottom, pixel_snap_scale);
    }
    debug_assert!(snapped.left + snapped.right <= box_size.width + epsilon);
    debug_assert!(snapped.top + snapped.bottom <= box_size.height + epsilon);
    snapped
}

// cpp: paint/nine_piece_image_grid.cc:159-191
fn Subrect(rect: &PaintRect, offset_x: f64, offset_y: f64, width: f64, height: f64) -> PaintRect {
    let base_x = if offset_x < 0.0 {
        rect.x + rect.width
    } else {
        rect.x
    };
    let base_y = if offset_y < 0.0 {
        rect.y + rect.height
    } else {
        rect.y
    };
    PaintRect {
        x: base_x + offset_x,
        y: base_y + offset_y,
        width,
        height,
    }
}

fn ImageRect(size: Size) -> PaintRect {
    PaintRect {
        x: 0.0,
        y: 0.0,
        width: size.width,
        height: size.height,
    }
}

// cpp: paint/nine_piece_image_grid.h:93-99
#[derive(Default)]
struct Edge {
    slice: f64,
    width: f64,
}

impl Edge {
    // cpp: paint/nine_piece_image_grid.h:95
    fn IsDrawable(&self) -> bool {
        self.slice > 0.0 && self.width > 0.0
    }

    // cpp: paint/nine_piece_image_grid.h:96
    fn Scale(&self) -> f64 {
        if self.IsDrawable() {
            self.width / self.slice
        } else {
            1.0
        }
    }
}

// cpp: paint/nine_piece_image_grid.h:62-117
pub struct NinePieceImageGrid {
    area: PaintRect,
    image_size: Size,
    zoom: f64,
    horizontal_rule: PaintImageTileRule,
    vertical_rule: PaintImageTileRule,
    fill: bool,
    top: Edge,
    right: Edge,
    bottom: Edge,
    left: Edge,
}

impl NinePieceImageGrid {
    // cpp: paint/nine_piece_image_grid.h:66-72
    // cpp: paint/nine_piece_image_grid.cc:79-157
    pub fn new(
        input: &NinePieceImagePaint,
        image_size: Size,
        zoom: f64,
        border_image_area: PaintRect,
        sides_to_include: FragmentBoxSides,
        pixel_snap_scale: f64,
    ) -> Self {
        let mut grid = Self {
            area: border_image_area,
            image_size,
            zoom,
            horizontal_rule: input.horizontal_rule,
            vertical_rule: input.vertical_rule,
            fill: input.fill,
            top: Edge::default(),
            right: Edge::default(),
            bottom: Edge::default(),
            left: Edge::default(),
        };
        if !image_size.width.is_finite()
            || !image_size.height.is_finite()
            || image_size.width <= 0.0
            || image_size.height <= 0.0
            || !grid.zoom.is_finite()
            || grid.zoom <= 0.0
            || !input.image_resolution_scale.is_finite()
            || input.image_resolution_scale <= 0.0
            || !pixel_snap_scale.is_finite()
            || pixel_snap_scale <= 0.0
            || !grid.area.x.is_finite()
            || !grid.area.y.is_finite()
            || !grid.area.width.is_finite()
            || !grid.area.height.is_finite()
            || grid.area.width < 0.0
            || grid.area.height < 0.0
        {
            panic!("nine-piece image geometry is invalid");
        }
        ValidateEdges(&input.slices, "nine-piece slices");
        ValidateEdges(&input.border_widths, "nine-piece border widths");
        ValidateEdges(&input.outsets, "nine-piece outsets");
        grid.top.slice = input.slices.top.min(image_size.height);
        grid.right.slice = input.slices.right.min(image_size.width);
        grid.bottom.slice = input.slices.bottom.min(image_size.height);
        grid.left.slice = input.slices.left.min(image_size.width);
        let auto_x = zoom / input.image_resolution_scale;
        let auto_y = zoom / input.image_resolution_scale;
        let mut resolved_widths = Edges::default();
        if sides_to_include.top {
            resolved_widths.top = ComputeEdgeWidth(
                &input.widths.top,
                input.border_widths.top,
                grid.top.slice * auto_y,
                grid.area.height,
            );
        }
        if sides_to_include.right {
            resolved_widths.right = ComputeEdgeWidth(
                &input.widths.right,
                input.border_widths.right,
                grid.right.slice * auto_x,
                grid.area.width,
            );
        }
        if sides_to_include.bottom {
            resolved_widths.bottom = ComputeEdgeWidth(
                &input.widths.bottom,
                input.border_widths.bottom,
                grid.bottom.slice * auto_y,
                grid.area.height,
            );
        }
        if sides_to_include.left {
            resolved_widths.left = ComputeEdgeWidth(
                &input.widths.left,
                input.border_widths.left,
                grid.left.slice * auto_x,
                grid.area.width,
            );
        }
        let horizontal_sum = resolved_widths.left + resolved_widths.right;
        let vertical_sum = resolved_widths.top + resolved_widths.bottom;
        let mut scale = 1.0_f64;
        if horizontal_sum > 0.0 {
            scale = scale.min(grid.area.width / horizontal_sum);
        }
        if vertical_sum > 0.0 {
            scale = scale.min(grid.area.height / vertical_sum);
        }
        if scale < 1.0 {
            resolved_widths.top = QuantizeLayoutUnit(resolved_widths.top * scale);
            resolved_widths.right = QuantizeLayoutUnit(resolved_widths.right * scale);
            resolved_widths.bottom = QuantizeLayoutUnit(resolved_widths.bottom * scale);
            resolved_widths.left = QuantizeLayoutUnit(resolved_widths.left * scale);
        }
        let snapped_widths = SnapEdgeWidths(
            &resolved_widths,
            Size {
                width: grid.area.width,
                height: grid.area.height,
            },
            pixel_snap_scale,
        );
        grid.top.width = snapped_widths.top;
        grid.right.width = snapped_widths.right;
        grid.bottom.width = snapped_widths.bottom;
        grid.left.width = snapped_widths.left;
        grid
    }

    // cpp: paint/nine_piece_image_grid.h:102
    // cpp: paint/nine_piece_image_grid.cc:193-238
    fn SetCorner(&self, output: &mut NinePieceDrawInfo, piece: NinePiece) {
        let image = ImageRect(self.image_size);
        match piece {
            NinePiece::kTopLeft => {
                output.is_drawable = self.top.IsDrawable() && self.left.IsDrawable();
                output.source = Subrect(&image, 0.0, 0.0, self.left.slice, self.top.slice);
                output.destination = Subrect(&self.area, 0.0, 0.0, self.left.width, self.top.width);
            }
            NinePiece::kBottomLeft => {
                output.is_drawable = self.bottom.IsDrawable() && self.left.IsDrawable();
                output.source = Subrect(
                    &image,
                    0.0,
                    -self.bottom.slice,
                    self.left.slice,
                    self.bottom.slice,
                );
                output.destination = Subrect(
                    &self.area,
                    0.0,
                    -self.bottom.width,
                    self.left.width,
                    self.bottom.width,
                );
            }
            NinePiece::kTopRight => {
                output.is_drawable = self.top.IsDrawable() && self.right.IsDrawable();
                output.source = Subrect(
                    &image,
                    -self.right.slice,
                    0.0,
                    self.right.slice,
                    self.top.slice,
                );
                output.destination = Subrect(
                    &self.area,
                    -self.right.width,
                    0.0,
                    self.right.width,
                    self.top.width,
                );
            }
            NinePiece::kBottomRight => {
                output.is_drawable = self.bottom.IsDrawable() && self.right.IsDrawable();
                output.source = Subrect(
                    &image,
                    -self.right.slice,
                    -self.bottom.slice,
                    self.right.slice,
                    self.bottom.slice,
                );
                output.destination = Subrect(
                    &self.area,
                    -self.right.width,
                    -self.bottom.width,
                    self.right.width,
                    self.bottom.width,
                );
            }
            _ => panic!("requested nine-piece corner is not a corner"),
        }
    }

    // cpp: paint/nine_piece_image_grid.h:103
    // cpp: paint/nine_piece_image_grid.cc:240-317
    fn SetEdge(&self, output: &mut NinePieceDrawInfo, piece: NinePiece) {
        let image = ImageRect(self.image_size);
        let source_width = self.image_size.width - self.left.slice - self.right.slice;
        let source_height = self.image_size.height - self.top.slice - self.bottom.slice;
        let destination_width = self.area.width - self.left.width - self.right.width;
        let destination_height = self.area.height - self.top.width - self.bottom.width;
        match piece {
            NinePiece::kLeft => {
                output.is_drawable =
                    self.left.IsDrawable() && source_height > 0.0 && destination_height > 0.0;
                output.source =
                    Subrect(&image, 0.0, self.top.slice, self.left.slice, source_height);
                output.destination = Subrect(
                    &self.area,
                    0.0,
                    self.top.width,
                    self.left.width,
                    destination_height,
                );
                output.tile_scale = Offset {
                    x: self.left.Scale(),
                    y: self.left.Scale(),
                };
                output.vertical_rule = self.vertical_rule;
            }
            NinePiece::kRight => {
                output.is_drawable =
                    self.right.IsDrawable() && source_height > 0.0 && destination_height > 0.0;
                output.source = Subrect(
                    &image,
                    -self.right.slice,
                    self.top.slice,
                    self.right.slice,
                    source_height,
                );
                output.destination = Subrect(
                    &self.area,
                    -self.right.width,
                    self.top.width,
                    self.right.width,
                    destination_height,
                );
                output.tile_scale = Offset {
                    x: self.right.Scale(),
                    y: self.right.Scale(),
                };
                output.vertical_rule = self.vertical_rule;
            }
            NinePiece::kTop => {
                output.is_drawable =
                    self.top.IsDrawable() && source_width > 0.0 && destination_width > 0.0;
                output.source = Subrect(&image, self.left.slice, 0.0, source_width, self.top.slice);
                output.destination = Subrect(
                    &self.area,
                    self.left.width,
                    0.0,
                    destination_width,
                    self.top.width,
                );
                output.tile_scale = Offset {
                    x: self.top.Scale(),
                    y: self.top.Scale(),
                };
                output.horizontal_rule = self.horizontal_rule;
            }
            NinePiece::kBottom => {
                output.is_drawable =
                    self.bottom.IsDrawable() && source_width > 0.0 && destination_width > 0.0;
                output.source = Subrect(
                    &image,
                    self.left.slice,
                    -self.bottom.slice,
                    source_width,
                    self.bottom.slice,
                );
                output.destination = Subrect(
                    &self.area,
                    self.left.width,
                    -self.bottom.width,
                    destination_width,
                    self.bottom.width,
                );
                output.tile_scale = Offset {
                    x: self.bottom.Scale(),
                    y: self.bottom.Scale(),
                };
                output.horizontal_rule = self.horizontal_rule;
            }
            _ => panic!("requested nine-piece edge is not an edge"),
        }
    }

    // cpp: paint/nine_piece_image_grid.h:104
    // cpp: paint/nine_piece_image_grid.cc:319-366
    fn SetMiddle(&self, output: &mut NinePieceDrawInfo) {
        let source_width = self.image_size.width - self.left.slice - self.right.slice;
        let source_height = self.image_size.height - self.top.slice - self.bottom.slice;
        let destination_width = self.area.width - self.left.width - self.right.width;
        let destination_height = self.area.height - self.top.width - self.bottom.width;
        output.is_drawable = self.fill
            && source_width > 0.0
            && source_height > 0.0
            && destination_width > 0.0
            && destination_height > 0.0;
        if !output.is_drawable {
            return;
        }
        output.source = PaintRect {
            x: self.left.slice,
            y: self.top.slice,
            width: source_width,
            height: source_height,
        };
        output.destination = PaintRect {
            x: self.area.x + self.left.width,
            y: self.area.y + self.top.width,
            width: destination_width,
            height: destination_height,
        };
        output.tile_scale = Offset {
            x: if self.top.IsDrawable() {
                self.top.Scale()
            } else if self.bottom.IsDrawable() {
                self.bottom.Scale()
            } else {
                self.zoom
            },
            y: if self.left.IsDrawable() {
                self.left.Scale()
            } else if self.right.IsDrawable() {
                self.right.Scale()
            } else {
                self.zoom
            },
        };
        if self.horizontal_rule == PaintImageTileRule::kStretch {
            output.tile_scale.x = destination_width / source_width;
        }
        if self.vertical_rule == PaintImageTileRule::kStretch {
            output.tile_scale.y = destination_height / source_height;
        }
        output.horizontal_rule = self.horizontal_rule;
        output.vertical_rule = self.vertical_rule;
    }

    // cpp: paint/nine_piece_image_grid.h:91
    // cpp: paint/nine_piece_image_grid.cc:368-383
    pub fn GetDrawInfo(&self, piece: NinePiece) -> NinePieceDrawInfo {
        let mut output = NinePieceDrawInfo::default();
        output.is_corner_piece = matches!(
            piece,
            NinePiece::kTopLeft
                | NinePiece::kBottomLeft
                | NinePiece::kTopRight
                | NinePiece::kBottomRight
        );
        if output.is_corner_piece {
            self.SetCorner(&mut output, piece);
        } else if piece == NinePiece::kMiddle {
            self.SetMiddle(&mut output);
        } else if piece != NinePiece::kCount {
            self.SetEdge(&mut output, piece);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::internal::paint_input::NinePieceImageWidths;

    fn Slices(value: f64) -> Edges {
        Edges {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    #[test]
    fn ResolvesPercentWidthsAgainstSnappedAreaBeforeFlooring() {
        let percent = || NinePieceImageWidth::Length {
            pixels: 0.0,
            percentage: 10.0,
            calculation: None,
        };
        let input = NinePieceImagePaint {
            image_resolution_scale: 1.0,
            slices: Slices(10.0),
            widths: NinePieceImageWidths {
                top: percent(),
                right: percent(),
                bottom: percent(),
                left: percent(),
            },
            ..Default::default()
        };
        let grid = NinePieceImageGrid::new(
            &input,
            Size {
                width: 100.0,
                height: 100.0,
            },
            1.0,
            PaintRect {
                width: 101.0,
                height: 51.0,
                ..Default::default()
            },
            FragmentBoxSides::default(),
            1.0,
        );
        assert_eq!(grid.left.width, 10.0);
        assert_eq!(grid.right.width, 10.0);
        assert_eq!(grid.top.width, 5.0);
        assert_eq!(grid.bottom.width, 5.0);
    }

    #[test]
    fn GivesAbuttingRoundingPixelToStartingEdge() {
        let input = NinePieceImagePaint {
            image_resolution_scale: 1.0,
            slices: Slices(4.0),
            widths: NinePieceImageWidths {
                top: NinePieceImageWidth::Auto,
                right: NinePieceImageWidth::Auto,
                bottom: NinePieceImageWidth::Auto,
                left: NinePieceImageWidth::Auto,
            },
            ..Default::default()
        };
        let make_grid = |pixel_snap_scale| {
            NinePieceImageGrid::new(
                &input,
                Size {
                    width: 10.0,
                    height: 10.0,
                },
                1.0,
                PaintRect {
                    width: 5.0,
                    height: 5.0,
                    ..Default::default()
                },
                FragmentBoxSides::default(),
                pixel_snap_scale,
            )
        };
        let css_pixel_grid = make_grid(1.0);
        assert_eq!(css_pixel_grid.left.width, 3.0);
        assert_eq!(css_pixel_grid.right.width, 2.0);
        let high_density_grid = make_grid(2.0);
        assert_eq!(high_density_grid.left.width, 2.5);
        assert_eq!(high_density_grid.right.width, 2.5);
    }
}
