#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::fragment_tree::{
    CollapsedTablePaintData, CollapsedTableSection, TablePaintColumn,
};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Display, Offset, PaintImage, Size, TextDirection, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{BackgroundImageLayer, PaintBlendMode};

use crate::background_geometry::{ResolveBackgroundTile, ResolveBackgroundTileSize};
use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::geometry_mapper::MapRectToRoot;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType};
use crate::paint_info::PaintInfo;
use crate::paint_shader_resolver::ResolvePaintShader;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/table_painters.cc:18
type CollapsedData = CollapsedTablePaintData;

// cpp: paint/table_painters.cc:20-23
#[derive(Default)]
struct LogicalSize {
    inline_size: f64,
    block_size: f64,
}

// cpp: paint/table_painters.cc:25-181
#[derive(Clone, Copy)]
struct TableCollapsedEdge<'a> {
    borders: &'a CollapsedData,
    edge_index: usize,
}

impl<'a> TableCollapsedEdge<'a> {
    const INVALID: usize = usize::MAX;

    // cpp: paint/table_painters.cc:27-31
    fn new(borders: &'a CollapsedData, edge_index: usize) -> Self {
        let edge_index = if edge_index >= borders.edges.len() {
            Self::INVALID
        } else {
            edge_index
        };
        Self {
            borders,
            edge_index,
        }
    }

    // cpp: paint/table_painters.cc:33-55
    fn offset_from(source: Self, offset: i32) -> Self {
        let mut result = Self {
            borders: source.borders,
            edge_index: Self::INVALID,
        };
        if !source.Exists() {
            return result;
        }
        if offset < 0 {
            let magnitude = (-offset) as usize;
            result.edge_index = if source.edge_index < magnitude {
                Self::INVALID
            } else {
                source.edge_index - magnitude
            };
        } else {
            let magnitude = offset as usize;
            result.edge_index = source
                .edge_index
                .checked_add(magnitude)
                .unwrap_or(Self::INVALID);
        }
        if result.edge_index >= result.borders.edges.len() {
            result.edge_index = Self::INVALID;
        }
        result
    }

    // cpp: paint/table_painters.cc:57-60
    fn Assign(&mut self, other: Self) {
        self.edge_index = other.edge_index;
    }

    // cpp: paint/table_painters.cc:62
    fn Exists(&self) -> bool {
        self.edge_index != Self::INVALID
    }

    // cpp: paint/table_painters.cc:63-66
    fn CanPaint(&self) -> bool {
        self.Exists()
            && self.borders.edges[self.edge_index].can_paint
            && self.borders.edges[self.edge_index].width > 0.0
    }

    // cpp: paint/table_painters.cc:67-69
    fn BorderWidth(&self) -> f64 {
        if self.Exists() {
            self.borders.edges[self.edge_index].width
        } else {
            0.0
        }
    }

    // cpp: paint/table_painters.cc:70-73
    fn BorderStyle(&self) -> BorderLineStyle {
        if self.Exists() {
            self.borders.edges[self.edge_index].style
        } else {
            BorderLineStyle::kNone
        }
    }

    // cpp: paint/table_painters.cc:74-76
    fn StyleRank(&self) -> u8 {
        if self.Exists() {
            self.borders.edges[self.edge_index].style_rank
        } else {
            0
        }
    }

    // cpp: paint/table_painters.cc:77-79
    fn BorderColor(&self) -> Color {
        if self.Exists() {
            self.borders.edges[self.edge_index].color
        } else {
            Color::default()
        }
    }

    // cpp: paint/table_painters.cc:81-89
    fn CompareBoxOrder(&self, other_edge_index: usize) -> i32 {
        let order = self.borders.edges[self.edge_index].box_order;
        let other_order = self.borders.edges[other_edge_index].box_order;
        if order < other_order {
            return 1;
        }
        if order > other_order {
            return -1;
        }
        0
    }

    // cpp: paint/table_painters.cc:91-94
    fn IsInlineAxis(&self) -> bool {
        self.Exists()
            && self.borders.edges_per_row != 0
            && self.edge_index % self.borders.edges_per_row % 2 != 0
    }

    // cpp: paint/table_painters.cc:95-97
    fn TableColumn(&self) -> usize {
        self.edge_index % self.borders.edges_per_row / 2
    }

    // cpp: paint/table_painters.cc:98
    fn TableRow(&self) -> usize {
        self.edge_index / self.borders.edges_per_row
    }

    // cpp: paint/table_painters.cc:102-126
    fn CompareForPaint(lhs: Self, rhs: Self) -> i32 {
        if lhs.edge_index == rhs.edge_index {
            return 0;
        }
        let lhs_paints = lhs.CanPaint();
        let rhs_paints = rhs.CanPaint();
        if lhs_paints && rhs_paints {
            if lhs.BorderWidth() > rhs.BorderWidth() {
                return 1;
            }
            if lhs.BorderWidth() < rhs.BorderWidth() {
                return -1;
            }
            if lhs.StyleRank() == rhs.StyleRank() {
                return lhs.CompareBoxOrder(rhs.edge_index);
            }
            const HIDDEN_RANK: u8 = 1;
            if rhs.StyleRank() == HIDDEN_RANK {
                return 1;
            }
            if lhs.StyleRank() == HIDDEN_RANK {
                return -1;
            }
            return if lhs.StyleRank() > rhs.StyleRank() {
                1
            } else {
                -1
            };
        }
        if !lhs_paints && !rhs_paints {
            return 0;
        }
        if lhs_paints {
            1
        } else {
            -1
        }
    }

    // cpp: paint/table_painters.cc:128-130
    fn EdgeBeforeStartIntersection(&self) -> Self {
        Self::offset_from(*self, if self.IsInlineAxis() { -2 } else { -1 })
    }

    // cpp: paint/table_painters.cc:131-133
    fn EdgeAfterStartIntersection(&self) -> Self {
        Self::offset_from(*self, if self.IsInlineAxis() { 0 } else { 1 })
    }

    // cpp: paint/table_painters.cc:134-139
    fn EdgeOverStartIntersection(&self) -> Self {
        Self::offset_from(
            *self,
            if self.IsInlineAxis() {
                -((self.borders.edges_per_row + 1) as i32)
            } else {
                -(self.borders.edges_per_row as i32)
            },
        )
    }

    // cpp: paint/table_painters.cc:140-142
    fn EdgeUnderStartIntersection(&self) -> Self {
        Self::offset_from(*self, if self.IsInlineAxis() { -1 } else { 0 })
    }

    // cpp: paint/table_painters.cc:143-148
    fn EdgeBeforeEndIntersection(&self) -> Self {
        Self::offset_from(
            *self,
            if self.IsInlineAxis() {
                0
            } else {
                (self.borders.edges_per_row - 1) as i32
            },
        )
    }

    // cpp: paint/table_painters.cc:149-154
    fn EdgeAfterEndIntersection(&self) -> Self {
        Self::offset_from(
            *self,
            if self.IsInlineAxis() {
                2
            } else {
                (self.borders.edges_per_row + 1) as i32
            },
        )
    }

    // cpp: paint/table_painters.cc:155-160
    fn EdgeOverEndIntersection(&self) -> Self {
        Self::offset_from(
            *self,
            if self.IsInlineAxis() {
                -((self.borders.edges_per_row - 1) as i32)
            } else {
                0
            },
        )
    }

    // cpp: paint/table_painters.cc:161-166
    fn EdgeUnderEndIntersection(&self) -> Self {
        Self::offset_from(
            *self,
            if self.IsInlineAxis() {
                1
            } else {
                self.borders.edges_per_row as i32
            },
        )
    }

    // cpp: paint/table_painters.cc:167-169
    fn EmptyEdge(&self) -> Self {
        Self::new(self.borders, Self::INVALID)
    }

    // cpp: paint/table_painters.cc:171-175
    fn Advance(&mut self) {
        if self.Exists() {
            self.edge_index += 1;
            if self.edge_index >= self.borders.edges.len() {
                self.edge_index = Self::INVALID;
            }
        }
    }
}

// cpp: paint/table_painters.cc:183-251
fn ComputeEdgeJoints(
    edge: TableCollapsedEdge<'_>,
    over_fragmentation_boundary: bool,
    under_fragmentation_boundary: bool,
    start_joint: &mut LogicalSize,
    end_joint: &mut LogicalSize,
    start_wins: &mut bool,
    end_wins: &mut bool,
) {
    *start_wins = false;
    *end_wins = false;
    let mut before = edge.EdgeBeforeStartIntersection();
    let mut after = edge.EdgeAfterStartIntersection();
    let mut over = if over_fragmentation_boundary {
        edge.EmptyEdge()
    } else {
        edge.EdgeOverStartIntersection()
    };
    let mut under = if under_fragmentation_boundary && edge.IsInlineAxis() {
        edge.EmptyEdge()
    } else {
        edge.EdgeUnderStartIntersection()
    };
    let mut inline_compare = TableCollapsedEdge::CompareForPaint(before, after);
    start_joint.block_size = if inline_compare == 1 {
        before.BorderWidth()
    } else {
        after.BorderWidth()
    };
    if over_fragmentation_boundary || (under_fragmentation_boundary && edge.IsInlineAxis()) {
        start_joint.block_size = 0.0;
    }
    let mut block_compare = TableCollapsedEdge::CompareForPaint(over, under);
    start_joint.inline_size = if block_compare == 1 {
        over.BorderWidth()
    } else {
        under.BorderWidth()
    };
    let mut inline_vs_block = TableCollapsedEdge::CompareForPaint(
        if inline_compare == 1 { before } else { after },
        if block_compare == 1 { over } else { under },
    );
    if edge.IsInlineAxis() {
        if inline_vs_block != -1 && inline_compare != 1 {
            *start_wins = true;
        }
    } else if inline_vs_block != 1 && block_compare != 1 {
        *start_wins = true;
    }
    before.Assign(edge.EdgeBeforeEndIntersection());
    after.Assign(edge.EdgeAfterEndIntersection());
    over.Assign(if over_fragmentation_boundary && edge.IsInlineAxis() {
        edge.EmptyEdge()
    } else {
        edge.EdgeOverEndIntersection()
    });
    under.Assign(if under_fragmentation_boundary {
        edge.EmptyEdge()
    } else {
        edge.EdgeUnderEndIntersection()
    });
    inline_compare = TableCollapsedEdge::CompareForPaint(before, after);
    end_joint.block_size = if inline_compare == 1 {
        before.BorderWidth()
    } else {
        after.BorderWidth()
    };
    if (over_fragmentation_boundary && edge.IsInlineAxis()) || under_fragmentation_boundary {
        end_joint.block_size = 0.0;
    }
    block_compare = TableCollapsedEdge::CompareForPaint(over, under);
    end_joint.inline_size = if block_compare == 1 {
        over.BorderWidth()
    } else {
        under.BorderWidth()
    };
    inline_vs_block = TableCollapsedEdge::CompareForPaint(
        if inline_compare == 1 { before } else { after },
        if block_compare == 1 { over } else { under },
    );
    if edge.IsInlineAxis() {
        if inline_vs_block != -1 && inline_compare != -1 {
            *end_wins = true;
        }
    } else if inline_vs_block != 1 && block_compare != -1 {
        *end_wins = true;
    }
}

// cpp: paint/table_painters.cc:253-278
fn ToPhysical(
    inline_start: f64,
    block_start: f64,
    inline_size: f64,
    block_size: f64,
    section: &CollapsedTableSection,
    writing_mode: WritingMode,
    direction: TextDirection,
) -> PaintRect {
    let ltr = direction == TextDirection::kLtr;
    match writing_mode {
        WritingMode::kHorizontalTb => PaintRect {
            x: if ltr {
                inline_start
            } else {
                section.size.width - inline_start - inline_size
            },
            y: block_start,
            width: inline_size,
            height: block_size,
        },
        WritingMode::kVerticalRl => PaintRect {
            x: section.size.width - block_start - block_size,
            y: if ltr {
                inline_start
            } else {
                section.size.height - inline_start - inline_size
            },
            width: block_size,
            height: inline_size,
        },
        WritingMode::kVerticalLr => PaintRect {
            x: block_start,
            y: if ltr {
                inline_start
            } else {
                section.size.height - inline_start - inline_size
            },
            width: block_size,
            height: inline_size,
        },
    }
}

// cpp: paint/table_painters.cc:280-283
fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

// cpp: paint/table_painters.cc:285-290
fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(cull_rect) = node.cull_rect else {
        return true;
    };
    let mapped = MapRectToRoot(*rect, &node.transforms);
    mapped.is_none_or(|mapped| Intersects(&mapped, &cull_rect))
}

// cpp: paint/table_painters.cc:292-309
fn FindImage<'a>(node: &'a PaintTreeNode<'_>, id: u64) -> &'a PaintImage {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let Some(resources) = fragment.paint.resources.as_ref() else {
        panic!("table column image has no PaintResources");
    };
    let Some(found) = resources.images.iter().find(|image| image.id == id) else {
        panic!("table column image resource is missing");
    };
    if found.width == 0
        || found.height == 0
        || found.resolution_scale <= 0.0
        || !found.resolution_scale.is_finite()
        || found.rgba8.len() != found.width as usize * found.height as usize * 4
    {
        panic!("table column image resource is invalid");
    }
    found
}

// cpp: paint/table_painters.cc:311-431
fn PaintColumnStyle(
    table_node: &PaintTreeNode<'_>,
    context: &RefCell<PaintContext<'_>>,
    info: &PaintInfo<'_, '_>,
    column: &TablePaintColumn,
    positioning: &PaintRect,
    cell_clip: &PaintRect,
) {
    let style = &*column.style;
    if !style.visible
        || (style.background_color.alpha <= 0.0 && style.background_images.is_empty())
        || !IsVisible(table_node, cell_clip)
    {
        return;
    }
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kSave,
            phase: info.phase,
            node_id: column.node_id,
            ..Default::default()
        },
        table_node,
    );
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kClipRect,
            phase: info.phase,
            node_id: column.node_id,
            rect: *cell_clip,
            ..Default::default()
        },
        table_node,
    );
    let has_blended_layer = style
        .background_images
        .iter()
        .any(|layer: &BackgroundImageLayer| layer.blend_mode != PaintBlendMode::kNormal);
    if has_blended_layer {
        context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kSaveLayerBlend,
                phase: info.phase,
                node_id: column.node_id,
                blend_mode: PaintBlendMode::kNormal,
                ..Default::default()
            },
            table_node,
        );
    }
    if style.background_color.alpha > 0.0 {
        context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                phase: info.phase,
                node_id: column.node_id,
                rect: *cell_clip,
                color: style.background_color,
                ..Default::default()
            },
            table_node,
        );
    }
    for layer in style.background_images.iter().rev() {
        if layer.shader.is_none() && layer.resource_id == 0 {
            continue;
        }
        let image = if layer.shader.is_some() {
            None
        } else {
            Some(FindImage(table_node, layer.resource_id))
        };
        let tile_size = ResolveBackgroundTileSize(layer, positioning, image);
        let resolved = ResolveBackgroundTile(layer, positioning, tile_size);
        let tile = resolved.tile_rect;
        let repeat_x = resolved.repeat_x;
        let repeat_y = resolved.repeat_y;
        let bitmap_tile_scale = if let Some(image) = image {
            Offset {
                x: tile.width / image.width as f64,
                y: tile.height / image.height as f64,
            }
        } else {
            resolved.tile_scale
        };
        if let Some(shader) = layer.shader.as_ref() {
            let mut item = DisplayItem {
                r#type: if repeat_x || repeat_y {
                    DisplayItemType::kDrawTiledGradient
                } else {
                    DisplayItemType::kDrawGradientRect
                },
                phase: info.phase,
                node_id: column.node_id,
                rect: if repeat_x || repeat_y {
                    *cell_clip
                } else {
                    tile
                },
                tile_rect: tile,
                repeat_x,
                repeat_y,
                background_repeat_x: resolved.rule_x,
                background_repeat_y: resolved.rule_y,
                blend_mode: layer.blend_mode,
                tile_scale: resolved.tile_scale,
                tile_spacing: resolved.tile_spacing,
                ..Default::default()
            };
            item.paint_shader = Some(ResolvePaintShader(
                shader,
                table_node
                    .fragment
                    .as_deref()
                    .expect("table node has a fragment")
                    .paint
                    .resources
                    .as_deref(),
                Offset {
                    x: tile.x,
                    y: tile.y,
                },
                Some(Size {
                    width: tile.width,
                    height: tile.height,
                }),
            ));
            context.borrow_mut().Append(item, table_node);
        } else if repeat_x || repeat_y {
            let image = image.expect("bitmap column background requires image");
            context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawTiledImage,
                    phase: info.phase,
                    node_id: column.node_id,
                    rect: *cell_clip,
                    source_rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: image.width as f64,
                        height: image.height as f64,
                    },
                    tile_rect: tile,
                    resource_id: image.id,
                    repeat_x,
                    repeat_y,
                    background_repeat_x: resolved.rule_x,
                    background_repeat_y: resolved.rule_y,
                    blend_mode: layer.blend_mode,
                    tile_scale: bitmap_tile_scale,
                    tile_spacing: resolved.tile_spacing,
                    ..Default::default()
                },
                table_node,
            );
        } else {
            let image = image.expect("bitmap column background requires image");
            context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawImageRect,
                    phase: info.phase,
                    node_id: column.node_id,
                    rect: tile,
                    source_rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: image.width as f64,
                        height: image.height as f64,
                    },
                    resource_id: image.id,
                    background_repeat_x: resolved.rule_x,
                    background_repeat_y: resolved.rule_y,
                    blend_mode: layer.blend_mode,
                    tile_scale: bitmap_tile_scale,
                    tile_spacing: resolved.tile_spacing,
                    ..Default::default()
                },
                table_node,
            );
        }
    }
    if has_blended_layer {
        context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: info.phase,
                node_id: column.node_id,
                ..Default::default()
            },
            table_node,
        );
    }
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kRestore,
            phase: info.phase,
            node_id: column.node_id,
            ..Default::default()
        },
        table_node,
    );
}

// cpp: paint/table_painters.cc:433-497
fn DrawEdge(
    node: &PaintTreeNode<'_>,
    context: &RefCell<PaintContext<'_>>,
    info: &PaintInfo<'_, '_>,
    edge: TableCollapsedEdge<'_>,
    rect: PaintRect,
) {
    if rect.width <= 0.0
        || rect.height <= 0.0
        || edge.BorderColor().alpha <= 0.0
        || !IsVisible(node, &rect)
    {
        return;
    }
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let horizontal =
        edge.IsInlineAxis() == (fragment.paint.writing_mode == WritingMode::kHorizontalTb);
    let style = edge.BorderStyle();
    let thickness = if horizontal { rect.height } else { rect.width };
    if matches!(style, BorderLineStyle::kDashed | BorderLineStyle::kDotted) {
        let line = if horizontal {
            PaintRect {
                x: rect.x,
                y: rect.y + rect.height / 2.0,
                width: rect.width,
                height: 0.0,
            }
        } else {
            PaintRect {
                x: rect.x + rect.width / 2.0,
                y: rect.y,
                width: 0.0,
                height: rect.height,
            }
        };
        let mut item = DisplayItem {
            r#type: DisplayItemType::kStrokeLine,
            phase: info.phase,
            node_id: fragment.node_id,
            rect: line,
            color: edge.BorderColor(),
            line_style: style,
            stroke_width: thickness,
            ..Default::default()
        };
        if style == BorderLineStyle::kDashed {
            item.dash_intervals = vec![3.0 * thickness, 3.0 * thickness];
        } else {
            item.dash_intervals = vec![0.0, 2.0 * thickness];
            item.round_cap = true;
        }
        context.borrow_mut().Append(item, node);
        return;
    }
    if style == BorderLineStyle::kDouble && thickness >= 3.0 {
        let band = thickness / 3.0;
        let bands = if horizontal {
            [
                PaintRect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: band,
                },
                PaintRect {
                    x: rect.x,
                    y: rect.y + 2.0 * band,
                    width: rect.width,
                    height: band,
                },
            ]
        } else {
            [
                PaintRect {
                    x: rect.x,
                    y: rect.y,
                    width: band,
                    height: rect.height,
                },
                PaintRect {
                    x: rect.x + 2.0 * band,
                    y: rect.y,
                    width: band,
                    height: rect.height,
                },
            ]
        };
        for band_rect in bands {
            context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    rect: band_rect,
                    color: edge.BorderColor(),
                    line_style: style,
                    stroke_width: thickness,
                    ..Default::default()
                },
                node,
            );
        }
        return;
    }
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            phase: info.phase,
            node_id: fragment.node_id,
            rect,
            color: edge.BorderColor(),
            line_style: style,
            stroke_width: thickness,
            ..Default::default()
        },
        node,
    );
}

// cpp: paint/table_painters.h:10-22
pub struct TablePainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> TablePainter<'n, 'f, 'c, 'o> {
    // cpp: paint/table_painters.h:12-13
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // cpp: paint/table_painters.h:15
    // cpp: paint/table_painters.cc:501-511
    pub fn DecorationRect(node: &PaintTreeNode<'_>) -> PaintRect {
        let fragment = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(data) = fragment.paint.table.as_ref() else {
            return PaintRect {
                x: node.paint_offset.x,
                y: node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            };
        };
        PaintRect {
            x: node.paint_offset.x + data.grid_offset.x,
            y: node.paint_offset.y + data.grid_offset.y,
            width: data.grid_size.width,
            height: data.grid_size.height,
        }
    }

    // cpp: paint/table_painters.h:16
    // cpp: paint/table_painters.cc:513-568
    pub fn PaintColumnBackgrounds(&self, info: &PaintInfo<'c, 'o>) {
        fn collect_cells<'n, 'f>(
            parent: &'n PaintTreeNode<'f>,
            cells: &mut Vec<&'n PaintTreeNode<'f>>,
        ) {
            for child_owner in &parent.children {
                let child = child_owner.as_ref();
                if child
                    .fragment
                    .as_deref()
                    .expect("table child has a fragment")
                    .paint
                    .display
                    == Display::kTableCell
                {
                    cells.push(child);
                } else {
                    collect_cells(child, cells);
                }
            }
        }
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(data) = fragment.paint.table.as_ref() else {
            return;
        };
        if data.columns.is_empty() {
            return;
        }
        let columns_rect = PaintRect {
            x: self.node.paint_offset.x + data.columns_offset.x,
            y: self.node.paint_offset.y + data.columns_offset.y,
            width: data.columns_size.width,
            height: data.columns_size.height,
        };
        if columns_rect.width <= 0.0 || columns_rect.height <= 0.0 {
            return;
        }
        let mut cells = Vec::new();
        collect_cells(self.node, &mut cells);
        let converter_space = CollapsedTableSection {
            size: data.columns_size,
            ..Default::default()
        };
        let logical_block_size = if fragment.paint.writing_mode == WritingMode::kHorizontalTb {
            columns_rect.height
        } else {
            columns_rect.width
        };
        for column in &data.columns {
            if column.span == 0 || column.inline_size <= 0.0 {
                continue;
            }
            let mut positioning = ToPhysical(
                column.inline_offset,
                0.0,
                column.inline_size,
                logical_block_size,
                &converter_space,
                fragment.paint.writing_mode,
                fragment.paint.direction,
            );
            positioning.x += columns_rect.x;
            positioning.y += columns_rect.y;
            for cell in &cells {
                let cell_fragment = cell.fragment.as_deref().expect("table cell has a fragment");
                let Some(cell_column) = cell_fragment.paint.table_cell_column else {
                    continue;
                };
                if cell_column < column.start_column
                    || cell_column >= column.start_column.wrapping_add(column.span)
                {
                    continue;
                }
                let cell_clip = PaintRect {
                    x: cell.paint_offset.x,
                    y: cell.paint_offset.y,
                    width: cell_fragment.size.width,
                    height: cell_fragment.size.height,
                };
                PaintColumnStyle(
                    self.node,
                    self.context,
                    info,
                    column,
                    &positioning,
                    &cell_clip,
                );
            }
        }
    }

    // cpp: paint/table_painters.h:17
    // cpp: paint/table_painters.cc:570-708
    pub fn PaintCollapsedBorders(&self, info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(borders) = fragment.paint.collapsed_table.as_ref() else {
            return;
        };
        if borders.edges_per_row == 0
            || borders.edges.is_empty()
            || borders.columns.is_empty()
            || borders.sections.is_empty()
        {
            return;
        }
        // TablePainter::PaintCollapsedBorders (.cc:532): one LayoutTable
        // drawing for the phase, containing all edges and section fragments.
        // The specialized kTableCollapsedBorders type is not used here.
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::PaintPhaseToDrawingType(info.phase),
            Self::DecorationRect(self.node),
        );
        let total_row_count = borders.edges.len() / borders.edges_per_row;
        let mut previous_painted_row: Option<usize> = None;
        for (section_index, section) in borders.sections.iter().enumerate() {
            if section.row_offsets.is_empty()
                || section.start_row > usize::MAX / borders.edges_per_row
            {
                continue;
            }
            let start_edge_index = section.start_row * borders.edges_per_row;
            let has_previous_fragmentainer = section_index == 0 && section.start_row > 0;
            let has_next_fragmentainer = section_index + 1 == borders.sections.len()
                && section.start_row.wrapping_add(section.row_offsets.len()) < total_row_count;
            let mut edge = TableCollapsedEdge::new(borders, start_edge_index);
            while edge.Exists() {
                let table_row = edge.TableRow();
                if table_row < section.start_row {
                    edge.Advance();
                    continue;
                }
                let fragment_row = table_row - section.start_row;
                if fragment_row >= section.row_offsets.len() {
                    previous_painted_row = if section.end_row_fragmented || table_row == 0 {
                        None
                    } else {
                        Some(table_row - 1)
                    };
                    break;
                }
                if !edge.CanPaint() {
                    edge.Advance();
                    continue;
                }
                let column = edge.TableColumn();
                if column >= borders.columns.len() {
                    edge.Advance();
                    continue;
                }
                let is_start_row = fragment_row == 0;
                let start_fragmented = is_start_row && section.start_row_fragmented;
                let start_at_boundary = is_start_row && has_previous_fragmentainer;
                let row_start = section.row_offsets[fragment_row];
                let column_start = borders.columns[column];
                let mut inline_start;
                let mut block_start;
                let mut inline_size;
                let mut block_size;
                if edge.IsInlineAxis() {
                    if column + 1 >= borders.columns.len() {
                        edge.Advance();
                        continue;
                    }
                    if previous_painted_row == Some(table_row) {
                        edge.Advance();
                        continue;
                    }
                    let is_end_row = fragment_row == section.row_offsets.len() - 1;
                    let end_fragmented = is_end_row && section.end_row_fragmented;
                    let end_at_boundary = is_end_row && has_next_fragmentainer;
                    if start_fragmented || end_fragmented {
                        edge.Advance();
                        continue;
                    }
                    inline_start = column_start;
                    inline_size = borders.columns[column + 1] - column_start;
                    block_start = if start_at_boundary {
                        row_start
                    } else {
                        row_start - edge.BorderWidth() / 2.0
                    };
                    block_size = if start_at_boundary || end_at_boundary {
                        edge.BorderWidth() / 2.0
                    } else {
                        edge.BorderWidth()
                    };
                    let mut start_joint = LogicalSize::default();
                    let mut end_joint = LogicalSize::default();
                    let mut start_wins = false;
                    let mut end_wins = false;
                    ComputeEdgeJoints(
                        edge,
                        start_at_boundary,
                        end_at_boundary,
                        &mut start_joint,
                        &mut end_joint,
                        &mut start_wins,
                        &mut end_wins,
                    );
                    if start_wins {
                        inline_start -= start_joint.inline_size / 2.0;
                        inline_size += start_joint.inline_size / 2.0;
                    } else {
                        inline_start += start_joint.inline_size / 2.0;
                        inline_size -= start_joint.inline_size / 2.0;
                    }
                    inline_size +=
                        (if end_wins { 1.0 } else { -1.0 }) * end_joint.inline_size / 2.0;
                } else {
                    if fragment_row + 1 >= section.row_offsets.len() {
                        edge.Advance();
                        continue;
                    }
                    let is_end_row = fragment_row + 1 == section.row_offsets.len() - 1;
                    let end_fragmented = is_end_row && section.end_row_fragmented;
                    let end_at_boundary = is_end_row && has_next_fragmentainer;
                    block_start = row_start;
                    block_size = section.row_offsets[fragment_row + 1] - row_start;
                    inline_start = column_start - edge.BorderWidth() / 2.0;
                    inline_size = edge.BorderWidth();
                    let mut start_joint = LogicalSize::default();
                    let mut end_joint = LogicalSize::default();
                    let mut start_wins = false;
                    let mut end_wins = false;
                    ComputeEdgeJoints(
                        edge,
                        start_at_boundary,
                        end_at_boundary,
                        &mut start_joint,
                        &mut end_joint,
                        &mut start_wins,
                        &mut end_wins,
                    );
                    if !start_fragmented {
                        if start_wins {
                            block_start -= start_joint.block_size / 2.0;
                            block_size += start_joint.block_size / 2.0;
                        } else {
                            block_start += start_joint.block_size / 2.0;
                            block_size -= start_joint.block_size / 2.0;
                        }
                    }
                    if !end_fragmented {
                        block_size +=
                            (if end_wins { 1.0 } else { -1.0 }) * end_joint.block_size / 2.0;
                    }
                }
                let mut rect = ToPhysical(
                    inline_start,
                    block_start,
                    inline_size,
                    block_size,
                    section,
                    fragment.paint.writing_mode,
                    fragment.paint.direction,
                );
                rect.x += self.node.paint_offset.x + section.offset.x;
                rect.y += self.node.paint_offset.y + section.offset.y;
                DrawEdge(self.node, self.context, info, edge, rect);
                edge.Advance();
            }
        }
    }
}
