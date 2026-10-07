#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use layoutng_assembly::internal::layout_input::{Edges, Offset, Size};
use layoutng_assembly::internal::paint_input::{NinePieceImagePaint, PaintImageTileRule};

use crate::PaintRect;

// cpp: paint/nine_piece_image_grid.h:7-18
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

// cpp: paint/nine_piece_image_grid.h:20-28
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

// cpp: paint/nine_piece_image_grid.cc:11-13
fn IsFiniteNonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

// cpp: paint/nine_piece_image_grid.cc:15-22
fn ValidateEdges(edges: &Edges, name: &str) {
    if !IsFiniteNonnegative(edges.top)
        || !IsFiniteNonnegative(edges.right)
        || !IsFiniteNonnegative(edges.bottom)
        || !IsFiniteNonnegative(edges.left)
    {
        panic!("{name} must be finite and nonnegative");
    }
}

// cpp: paint/nine_piece_image_grid.cc:25-32
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

// cpp: paint/nine_piece_image_grid.cc:35-37
fn ImageRect(size: Size) -> PaintRect {
    PaintRect {
        x: 0.0,
        y: 0.0,
        width: size.width,
        height: size.height,
    }
}

// cpp: paint/nine_piece_image_grid.h:41-46
#[derive(Default)]
struct Edge {
    slice: f64,
    width: f64,
}

impl Edge {
    // cpp: paint/nine_piece_image_grid.h:44
    fn IsDrawable(&self) -> bool {
        self.slice > 0.0 && self.width > 0.0
    }

    // cpp: paint/nine_piece_image_grid.h:45
    fn Scale(&self) -> f64 {
        if self.IsDrawable() {
            self.width / self.slice
        } else {
            1.0
        }
    }
}

// cpp: paint/nine_piece_image_grid.h:32-61
pub struct NinePieceImageGrid {
    area: PaintRect,
    image_size: Size,
    horizontal_rule: PaintImageTileRule,
    vertical_rule: PaintImageTileRule,
    fill: bool,
    top: Edge,
    right: Edge,
    bottom: Edge,
    left: Edge,
}

impl NinePieceImageGrid {
    // cpp: paint/nine_piece_image_grid.h:34-36
    // cpp: paint/nine_piece_image_grid.cc:41-84
    pub fn new(
        input: &NinePieceImagePaint,
        image_size: Size,
        border_image_area: PaintRect,
    ) -> Self {
        let mut grid = Self {
            area: border_image_area,
            image_size,
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
        ValidateEdges(&input.widths, "nine-piece widths");
        ValidateEdges(&input.outsets, "nine-piece outsets");
        grid.top.slice = input.slices.top.min(image_size.height);
        grid.right.slice = input.slices.right.min(image_size.width);
        grid.bottom.slice = input.slices.bottom.min(image_size.height);
        grid.left.slice = input.slices.left.min(image_size.width);
        grid.top.width = input.widths.top;
        grid.right.width = input.widths.right;
        grid.bottom.width = input.widths.bottom;
        grid.left.width = input.widths.left;
        let horizontal_sum = grid.left.width + grid.right.width;
        let vertical_sum = grid.top.width + grid.bottom.width;
        let mut scale = 1.0_f64;
        if horizontal_sum > 0.0 {
            scale = scale.min(grid.area.width / horizontal_sum);
        }
        if vertical_sum > 0.0 {
            scale = scale.min(grid.area.height / vertical_sum);
        }
        if scale < 1.0 {
            grid.top.width *= scale;
            grid.right.width *= scale;
            grid.bottom.width *= scale;
            grid.left.width *= scale;
        }
        grid
    }

    // cpp: paint/nine_piece_image_grid.h:48
    // cpp: paint/nine_piece_image_grid.cc:86-119
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

    // cpp: paint/nine_piece_image_grid.h:49
    // cpp: paint/nine_piece_image_grid.cc:121-174
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

    // cpp: paint/nine_piece_image_grid.h:50
    // cpp: paint/nine_piece_image_grid.cc:176-201
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
                1.0
            },
            y: if self.left.IsDrawable() {
                self.left.Scale()
            } else if self.right.IsDrawable() {
                self.right.Scale()
            } else {
                1.0
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

    // cpp: paint/nine_piece_image_grid.h:38
    // cpp: paint/nine_piece_image_grid.cc:203-215
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
