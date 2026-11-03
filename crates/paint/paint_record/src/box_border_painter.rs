#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::background_bleed_avoidance::{
    BackgroundBleedAvoidance, BleedAvoidanceIsClipping,
};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Edges, Offset, PaintPathCommand, PaintPathVerb, Size,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::PaintCornerRadii;

use crate::contoured_border_geometry::{ConicRoundedRectPath, ResolveCornerRadii, RoundedRectPath};
use crate::geometry_mapper::MapRectToRoot;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/box_border_painter.h:25-182
pub struct BoxBorderPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
    border_rect: PaintRect,
}

impl<'n, 'f, 'c, 'o> BoxBorderPainter<'n, 'f, 'c, 'o> {
    // cpp: paint/box_border_painter.h:28-33
    pub fn new(
        node: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
        border_rect: PaintRect,
    ) -> Self {
        Self {
            node,
            context,
            border_rect,
        }
    }

    // cpp: paint/box_border_painter.h:25-31
    // cpp: paint/box_border_painter.cc:1336-1494
    pub fn PaintBorder(&self, phase: PaintPhase, bleed_avoidance: BackgroundBleedAvoidance) {
        self.Paint(phase, bleed_avoidance, false);
    }

    // cpp: paint/box_border_painter.h:34-44
    // cpp: paint/box_border_painter.cc:1374-1494
    pub fn PaintBorderArea(&self, phase: PaintPhase) {
        self.Paint(phase, BackgroundBleedAvoidance::kBackgroundBleedNone, true);
    }

    // cpp: paint/box_border_painter.cc:37-2518
    fn Paint(
        &self,
        phase: PaintPhase,
        bleed_avoidance: BackgroundBleedAvoidance,
        alpha_mask: bool,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &fragment.paint;
        let opaque_mask_color = Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let border_color = |edge: usize| {
            if alpha_mask {
                opaque_mask_color
            } else {
                style.style.border_colors()[edge]
            }
        };
        let styled_border_color = |edge: usize, line_style: BorderLineStyle| {
            StyledBorderColor(edge, line_style, border_color(edge), alpha_mask)
        };
        let border_has_alpha =
            |edge: usize| alpha_mask || style.style.border_colors()[edge].alpha > 0.0;
        let sides = &style.border_sides;
        let widths = [
            if sides.top { style.border.top } else { 0.0 },
            if sides.right { style.border.right } else { 0.0 },
            if sides.bottom {
                style.border.bottom
            } else {
                0.0
            },
            if sides.left { style.border.left } else { 0.0 },
        ];
        let decoration = self.node.PixelSnappedRect(&self.border_rect);
        let x = decoration.x;
        let y = decoration.y;
        let width = decoration.width;
        let height = decoration.height;
        let size = Size { width, height };
        let outer_radii = ResolveCornerRadii(&style.style, sides, size, Edges::default());
        let first_style = style.border_styles[0];
        let same_colors = alpha_mask
            || (style.style.border_colors()[0] == style.style.border_colors()[1]
                && style.style.border_colors()[0] == style.style.border_colors()[2]
                && style.style.border_colors()[0] == style.style.border_colors()[3]);
        let uniform_rounded_double = sides.HasAllSides()
            && outer_radii.HasRadius()
            && widths[0] >= 3.0
            && first_style == BorderLineStyle::kDouble
            && widths[1] == widths[0]
            && widths[2] == widths[0]
            && widths[3] == widths[0]
            && style.border_styles[1] == first_style
            && style.border_styles[2] == first_style
            && style.border_styles[3] == first_style
            && same_colors
            && border_has_alpha(0);
        if uniform_rounded_double {
            let band = widths[0] / 3.0;
            let inset_rect = |inset: f64| PaintRect {
                x: x + inset,
                y: y + inset,
                width: (width - 2.0 * inset).max(0.0),
                height: (height - 2.0 * inset).max(0.0),
            };
            let radii_at = |inset: f64| {
                ResolveCornerRadii(
                    &style.style,
                    sides,
                    size,
                    Edges {
                        top: inset,
                        right: inset,
                        bottom: inset,
                        left: inset,
                    },
                )
            };
            if IsVisible(
                self.node,
                &PaintRect {
                    x,
                    y,
                    width,
                    height,
                },
            ) {
                for (outer_inset, inner_inset) in [(0.0, band), (2.0 * band, widths[0])] {
                    let band_outer_radii = radii_at(outer_inset);
                    let band_inner_radii = radii_at(inner_inset);
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kDrawDoubleRoundedRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: inset_rect(outer_inset),
                            inner_rect: inset_rect(inner_inset),
                            color: border_color(0),
                            line_style: BorderLineStyle::kDouble,
                            corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                                &band_outer_radii,
                            ),
                            inner_corner_radius:
                                crate::contoured_border_geometry::UniformCornerRadius(
                                    &band_inner_radii,
                                ),
                            corner_radii: band_outer_radii,
                            inner_corner_radii: band_inner_radii,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
            }
            return;
        }
        let uniform_rounded_dash = sides.HasAllSides()
            && outer_radii.HasRadius()
            && widths[0] > 0.0
            && matches!(
                first_style,
                BorderLineStyle::kDashed | BorderLineStyle::kDotted
            )
            && widths[1] == widths[0]
            && widths[2] == widths[0]
            && widths[3] == widths[0]
            && style.border_styles[1] == first_style
            && style.border_styles[2] == first_style
            && style.border_styles[3] == first_style
            && same_colors
            && border_has_alpha(0);
        if uniform_rounded_dash {
            let half_width = widths[0] * 0.5;
            let centerline = PaintRect {
                x: x + half_width,
                y: y + half_width,
                width: (width - widths[0]).max(0.0),
                height: (height - widths[0]).max(0.0),
            };
            let center_radii = ResolveCornerRadii(
                &style.style,
                sides,
                size,
                Edges {
                    top: half_width,
                    right: half_width,
                    bottom: half_width,
                    left: half_width,
                },
            );
            if IsVisible(
                self.node,
                &PaintRect {
                    x,
                    y,
                    width,
                    height,
                },
            ) {
                let mut item = DisplayItem {
                    r#type: DisplayItemType::kStrokePath,
                    phase,
                    node_id: fragment.node_id,
                    rect: PaintRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    color: border_color(0),
                    line_style: first_style,
                    stroke_width: widths[0],
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                        &center_radii,
                    ),
                    corner_radii: center_radii,
                    ..Default::default()
                };
                item.path = RoundedRectPath(&centerline, &center_radii);
                if first_style == BorderLineStyle::kDashed {
                    item.dash_intervals = vec![3.0 * widths[0], 3.0 * widths[0]];
                } else {
                    item.dash_intervals = vec![0.0, 2.0 * widths[0]];
                    item.round_cap = true;
                }
                self.context.borrow_mut().Append(item, self.node);
            }
            return;
        }
        let solid_rounded_ring = sides.HasAllSides()
            && outer_radii.HasRadius()
            && style
                .border_styles
                .iter()
                .all(|line| *line == BorderLineStyle::kSolid)
            && same_colors
            && border_has_alpha(0)
            && widths.iter().any(|width| *width > 0.0);
        if solid_rounded_ring {
            let outer = PaintRect {
                x,
                y,
                width,
                height,
            };
            let inner = PaintRect {
                x: x + widths[3],
                y: y + widths[0],
                width: (width - widths[1] - widths[3]).max(0.0),
                height: (height - widths[0] - widths[2]).max(0.0),
            };
            let inner_radii = ResolveCornerRadii(
                &style.style,
                sides,
                size,
                Edges {
                    top: widths[0],
                    right: widths[1],
                    bottom: widths[2],
                    left: widths[3],
                },
            );
            if IsVisible(self.node, &outer) {
                if BleedAvoidanceIsClipping(bleed_avoidance) {
                    let mut item = DisplayItem {
                        r#type: DisplayItemType::kDrawPath,
                        phase,
                        node_id: fragment.node_id,
                        rect: inner,
                        color: border_color(0),
                        line_style: BorderLineStyle::kSolid,
                        corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &inner_radii,
                        ),
                        corner_radii: inner_radii,
                        inverse_winding: true,
                        ..Default::default()
                    };
                    item.path = ConicRoundedRectPath(&inner, &inner_radii);
                    self.context.borrow_mut().Append(item, self.node);
                    return;
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawDoubleRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: outer,
                        inner_rect: inner,
                        color: border_color(0),
                        line_style: BorderLineStyle::kSolid,
                        corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &outer_radii,
                        ),
                        inner_corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &inner_radii,
                        ),
                        corner_radii: outer_radii,
                        inner_corner_radii: inner_radii,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            return;
        }
        let mut rounded_solid_edges = outer_radii.HasRadius();
        for (edge, width) in widths.iter().enumerate() {
            if *width > 0.0 && style.border_styles[edge] != BorderLineStyle::kSolid {
                rounded_solid_edges = false;
            }
        }
        if rounded_solid_edges {
            let outer = PaintRect {
                x,
                y,
                width,
                height,
            };
            let inner = PaintRect {
                x: x + widths[3],
                y: y + widths[0],
                width: (width - widths[1] - widths[3]).max(0.0),
                height: (height - widths[0] - widths[2]).max(0.0),
            };
            let inner_radii = ResolveCornerRadii(
                &style.style,
                sides,
                size,
                Edges {
                    top: widths[0],
                    right: widths[1],
                    bottom: widths[2],
                    left: widths[3],
                },
            );
            const ADJACENT_EDGES: [[usize; 2]; 4] = [[3, 1], [0, 2], [1, 3], [2, 0]];
            if IsVisible(self.node, &outer) {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: outer,
                        corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &outer_radii,
                        ),
                        corner_radii: outer_radii,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipOutRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: inner,
                        corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &inner_radii,
                        ),
                        corner_radii: inner_radii,
                        ..Default::default()
                    },
                    self.node,
                );
                for (edge, width) in widths.iter().enumerate() {
                    if *width <= 0.0 || !border_has_alpha(edge) {
                        continue;
                    }
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kSave,
                            phase,
                            node_id: fragment.node_id,
                            ..Default::default()
                        },
                        self.node,
                    );
                    let corner_clips = BorderEdgeCornerClips(edge, &outer, &inner, &outer_radii);
                    let adjacent_renders = |adjacent: usize| {
                        widths[adjacent] > 0.0
                            && !matches!(
                                style.border_styles[adjacent],
                                BorderLineStyle::kNone | BorderLineStyle::kHidden
                            )
                            && border_has_alpha(adjacent)
                    };
                    let first_adjacent = ADJACENT_EDGES[edge][0];
                    let second_adjacent = ADJACENT_EDGES[edge][1];
                    let first_renders = adjacent_renders(first_adjacent);
                    let second_renders = adjacent_renders(second_adjacent);
                    let mut first_antialias =
                        !first_renders || border_color(edge) != border_color(first_adjacent);
                    let mut second_antialias =
                        !second_renders || border_color(edge) != border_color(second_adjacent);

                    // Chromium treats a missing adjacent edge as completed:
                    // there is no visible seam to antialias or miter against.
                    // Keep the real seam's AA mode here; CombinedBorderEdgeClip
                    // also extends the missing seam to the inner boundary.
                    match (first_renders, second_renders) {
                        (false, true) => first_antialias = second_antialias,
                        (true, false) => second_antialias = first_antialias,
                        _ => {}
                    }
                    if first_antialias == second_antialias {
                        self.context.borrow_mut().Append(
                            BorderEdgeClip(
                                fragment.node_id,
                                phase,
                                &CombinedBorderEdgeClip(
                                    edge,
                                    &outer,
                                    &inner,
                                    &corner_clips,
                                    first_renders,
                                    second_renders,
                                ),
                                first_antialias,
                            ),
                            self.node,
                        );
                    } else {
                        self.context.borrow_mut().Append(
                            BorderEdgeClip(
                                fragment.node_id,
                                phase,
                                &corner_clips[0],
                                first_antialias,
                            ),
                            self.node,
                        );
                        self.context.borrow_mut().Append(
                            BorderEdgeClip(
                                fragment.node_id,
                                phase,
                                &corner_clips[1],
                                second_antialias,
                            ),
                            self.node,
                        );
                    }
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: outer,
                            color: border_color(edge),
                            line_style: BorderLineStyle::kSolid,
                            ..Default::default()
                        },
                        self.node,
                    );
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kRestore,
                            phase,
                            node_id: fragment.node_id,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            return;
        }
        if outer_radii.HasRadius() {
            let outer = PaintRect {
                x,
                y,
                width,
                height,
            };
            let inset_rect = |inset: Edges| PaintRect {
                x: x + inset.left,
                y: y + inset.top,
                width: (width - inset.left - inset.right).max(0.0),
                height: (height - inset.top - inset.bottom).max(0.0),
            };
            let scaled_insets = |scale: f64| Edges {
                top: widths[0] * scale,
                right: widths[1] * scale,
                bottom: widths[2] * scale,
                left: widths[3] * scale,
            };
            let full_insets = scaled_insets(1.0);
            let inner = inset_rect(full_insets);
            let edge_polygons = BorderEdgePolygons(&outer, &inner);
            let half_insets = scaled_insets(0.5);
            let centerline = inset_rect(half_insets);
            let center_radii = ResolveCornerRadii(&style.style, sides, size, half_insets);
            let center_path = RoundedRectPath(&centerline, &center_radii);
            let append_ring = |outer_inset: Edges,
                               inner_inset: Edges,
                               color: Color,
                               line_style: BorderLineStyle| {
                let ring_outer_radii = ResolveCornerRadii(&style.style, sides, size, outer_inset);
                let ring_inner_radii = ResolveCornerRadii(&style.style, sides, size, inner_inset);
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawDoubleRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: inset_rect(outer_inset),
                        inner_rect: inset_rect(inner_inset),
                        color,
                        line_style,
                        corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &ring_outer_radii,
                        ),
                        inner_corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &ring_inner_radii,
                        ),
                        corner_radii: ring_outer_radii,
                        inner_corner_radii: ring_inner_radii,
                        ..Default::default()
                    },
                    self.node,
                );
            };
            if IsVisible(self.node, &outer) {
                for (edge, width) in widths.iter().enumerate() {
                    if *width <= 0.0
                        || matches!(
                            style.border_styles[edge],
                            BorderLineStyle::kNone | BorderLineStyle::kHidden
                        )
                        || !border_has_alpha(edge)
                    {
                        continue;
                    }
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kSave,
                            phase,
                            node_id: fragment.node_id,
                            ..Default::default()
                        },
                        self.node,
                    );
                    self.context.borrow_mut().Append(
                        BorderEdgeClip(fragment.node_id, phase, &edge_polygons[edge], false),
                        self.node,
                    );
                    let line_style = style.border_styles[edge];
                    let color = border_color(edge);
                    if matches!(
                        line_style,
                        BorderLineStyle::kDashed | BorderLineStyle::kDotted
                    ) {
                        let mut item = DisplayItem {
                            r#type: DisplayItemType::kStrokePath,
                            phase,
                            node_id: fragment.node_id,
                            rect: outer,
                            color,
                            line_style,
                            stroke_width: *width,
                            corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                                &center_radii,
                            ),
                            corner_radii: center_radii,
                            ..Default::default()
                        };
                        item.path = center_path.clone();
                        if line_style == BorderLineStyle::kDashed {
                            item.dash_intervals = vec![3.0 * width, 3.0 * width];
                        } else {
                            item.dash_intervals = vec![0.0, 2.0 * width];
                            item.round_cap = true;
                        }
                        self.context.borrow_mut().Append(item, self.node);
                    } else if line_style == BorderLineStyle::kDouble && *width >= 3.0 {
                        append_ring(
                            Edges::default(),
                            scaled_insets(1.0 / 3.0),
                            color,
                            line_style,
                        );
                        append_ring(scaled_insets(2.0 / 3.0), full_insets, color, line_style);
                    } else if matches!(
                        line_style,
                        BorderLineStyle::kGroove | BorderLineStyle::kRidge
                    ) {
                        let outer_style = if line_style == BorderLineStyle::kGroove {
                            BorderLineStyle::kInset
                        } else {
                            BorderLineStyle::kOutset
                        };
                        let inner_style = if line_style == BorderLineStyle::kGroove {
                            BorderLineStyle::kOutset
                        } else {
                            BorderLineStyle::kInset
                        };
                        append_ring(
                            Edges::default(),
                            half_insets,
                            styled_border_color(edge, outer_style),
                            BorderLineStyle::kSolid,
                        );
                        append_ring(
                            half_insets,
                            full_insets,
                            styled_border_color(edge, inner_style),
                            BorderLineStyle::kSolid,
                        );
                    } else {
                        append_ring(
                            Edges::default(),
                            full_insets,
                            styled_border_color(edge, line_style),
                            BorderLineStyle::kSolid,
                        );
                    }
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kRestore,
                            phase,
                            node_id: fragment.node_id,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
            }
            return;
        }
        let rects = [
            PaintRect {
                x,
                y,
                width,
                height: widths[0],
            },
            PaintRect {
                x: x + width - widths[1],
                y,
                width: widths[1],
                height,
            },
            PaintRect {
                x,
                y: y + height - widths[2],
                width,
                height: widths[2],
            },
            PaintRect {
                x,
                y,
                width: widths[3],
                height,
            },
        ];
        let center_lines = [
            PaintRect {
                x,
                y: y + widths[0] / 2.0,
                width,
                height: 0.0,
            },
            PaintRect {
                x: x + width - widths[1] / 2.0,
                y,
                width: 0.0,
                height,
            },
            PaintRect {
                x,
                y: y + height - widths[2] / 2.0,
                width,
                height: 0.0,
            },
            PaintRect {
                x: x + widths[3] / 2.0,
                y,
                width: 0.0,
                height,
            },
        ];
        for (edge, edge_width) in widths.iter().enumerate() {
            if *edge_width <= 0.0
                || matches!(
                    style.border_styles[edge],
                    BorderLineStyle::kNone | BorderLineStyle::kHidden
                )
                || !border_has_alpha(edge)
            {
                continue;
            }
            if !IsVisible(self.node, &rects[edge]) {
                continue;
            }
            let line_style = style.border_styles[edge];
            if matches!(
                line_style,
                BorderLineStyle::kDashed | BorderLineStyle::kDotted
            ) {
                let mut item = DisplayItem {
                    r#type: DisplayItemType::kStrokeLine,
                    phase,
                    node_id: fragment.node_id,
                    rect: center_lines[edge],
                    color: border_color(edge),
                    line_style,
                    stroke_width: *edge_width,
                    ..Default::default()
                };
                if line_style == BorderLineStyle::kDashed {
                    item.dash_intervals = vec![3.0 * edge_width, 3.0 * edge_width];
                } else {
                    item.dash_intervals = vec![0.0, 2.0 * edge_width];
                    item.round_cap = true;
                }
                self.context.borrow_mut().Append(item, self.node);
                continue;
            }
            if line_style == BorderLineStyle::kDouble && *edge_width >= 3.0 {
                let band = edge_width / 3.0;
                let bands = match edge {
                    0 => [
                        PaintRect {
                            x,
                            y,
                            width,
                            height: band,
                        },
                        PaintRect {
                            x,
                            y: y + 2.0 * band,
                            width,
                            height: band,
                        },
                    ],
                    1 => [
                        PaintRect {
                            x: x + width - band,
                            y,
                            width: band,
                            height,
                        },
                        PaintRect {
                            x: x + width - 3.0 * band,
                            y,
                            width: band,
                            height,
                        },
                    ],
                    2 => [
                        PaintRect {
                            x,
                            y: y + height - band,
                            width,
                            height: band,
                        },
                        PaintRect {
                            x,
                            y: y + height - 3.0 * band,
                            width,
                            height: band,
                        },
                    ],
                    _ => [
                        PaintRect {
                            x,
                            y,
                            width: band,
                            height,
                        },
                        PaintRect {
                            x: x + 2.0 * band,
                            y,
                            width: band,
                            height,
                        },
                    ],
                };
                for band_rect in bands {
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: band_rect,
                            color: border_color(edge),
                            line_style,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
                continue;
            }
            if matches!(
                line_style,
                BorderLineStyle::kGroove | BorderLineStyle::kRidge
            ) {
                let (outer_rect, inner_rect) = SplitBorderEdgeRect(rects[edge], edge);
                let outer_style = if line_style == BorderLineStyle::kGroove {
                    BorderLineStyle::kInset
                } else {
                    BorderLineStyle::kOutset
                };
                let inner_style = if line_style == BorderLineStyle::kGroove {
                    BorderLineStyle::kOutset
                } else {
                    BorderLineStyle::kInset
                };
                for (rect, color) in [
                    (outer_rect, styled_border_color(edge, outer_style)),
                    (inner_rect, styled_border_color(edge, inner_style)),
                ] {
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            phase,
                            node_id: fragment.node_id,
                            rect,
                            color,
                            line_style: BorderLineStyle::kSolid,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
                continue;
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: rects[edge],
                    color: styled_border_color(edge, line_style),
                    line_style,
                    stroke_width: *edge_width,
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                        &outer_radii,
                    ),
                    corner_radii: outer_radii,
                    ..Default::default()
                },
                self.node,
            );
        }
        let one_pixel_solid_border = sides.HasAllSides()
            && widths.iter().all(|width| *width == 1.0)
            && style
                .border_styles
                .iter()
                .all(|line| *line == BorderLineStyle::kSolid)
            && (alpha_mask
                || style
                    .style
                    .border_colors()
                    .iter()
                    .all(|color| color.alpha >= 1.0));
        if one_pixel_solid_border {
            let channel = |first: f32, second: f32| {
                let first_byte = (first * 255.0).round() as i32;
                let second_byte = (second * 255.0).round() as i32;
                ((first_byte + second_byte) / 2) as f32 / 255.0
            };
            let average = |a: Color, b: Color| Color {
                red: channel(a.red, b.red),
                green: channel(a.green, b.green),
                blue: channel(a.blue, b.blue),
                alpha: 1.0,
            };
            let corners = [
                PaintRect {
                    x,
                    y,
                    width: 1.0,
                    height: 1.0,
                },
                PaintRect {
                    x: x + width - 1.0,
                    y,
                    width: 1.0,
                    height: 1.0,
                },
                PaintRect {
                    x: x + width - 1.0,
                    y: y + height - 1.0,
                    width: 1.0,
                    height: 1.0,
                },
                PaintRect {
                    x,
                    y: y + height - 1.0,
                    width: 1.0,
                    height: 1.0,
                },
            ];
            const ADJACENT: [[usize; 2]; 4] = [[0, 3], [0, 1], [2, 1], [2, 3]];
            for (corner, corner_rect) in corners.into_iter().enumerate() {
                let first = border_color(ADJACENT[corner][0]);
                let second = border_color(ADJACENT[corner][1]);
                if first == second {
                    continue;
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: corner_rect,
                        color: average(first, second),
                        line_style: BorderLineStyle::kSolid,
                        antialias: false,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }
    }
}

// cpp: paint/box_border_painter.cc:612-615
fn DarkenBoxSide(edge: usize, style: BorderLineStyle) -> bool {
    let is_top_or_left = matches!(edge, 0 | 3);
    is_top_or_left == (style == BorderLineStyle::kInset)
}

// cpp: platform/graphics/color.cc:1133-1176
fn LightColor(color: Color) -> Color {
    let maximum = color.red.max(color.green).max(color.blue).clamp(0.0, 1.0);
    if maximum == 0.0 {
        return Color {
            red: 84.0 / 255.0,
            green: 84.0 / 255.0,
            blue: 84.0 / 255.0,
            alpha: color.alpha,
        };
    }
    let multiplier = (maximum + 0.33).min(1.0) / maximum;
    Color {
        red: QuantizeColorChannel(color.red * multiplier),
        green: QuantizeColorChannel(color.green * multiplier),
        blue: QuantizeColorChannel(color.blue * multiplier),
        alpha: color.alpha,
    }
}

// cpp: platform/graphics/color.cc:1160-1176
fn DarkColor(color: Color) -> Color {
    if color.red == 1.0 && color.green == 1.0 && color.blue == 1.0 && color.alpha == 1.0 {
        return Color {
            red: 171.0 / 255.0,
            green: 171.0 / 255.0,
            blue: 171.0 / 255.0,
            alpha: 1.0,
        };
    }
    let maximum = color.red.max(color.green).max(color.blue).clamp(0.0, 1.0);
    let multiplier = if maximum == 0.0 {
        0.0
    } else {
        ((maximum - 0.33) / maximum).max(0.0)
    };
    Color {
        red: QuantizeColorChannel(color.red * multiplier),
        green: QuantizeColorChannel(color.green * multiplier),
        blue: QuantizeColorChannel(color.blue * multiplier),
        alpha: color.alpha,
    }
}

fn QuantizeColorChannel(channel: f32) -> f32 {
    (channel.clamp(0.0, 1.0) * 255.0).round() / 255.0
}

fn LinearSrgbChannel(channel: f32) -> f32 {
    let channel = channel.clamp(0.0, 1.0);
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn RelativeLuminance(color: Color) -> f32 {
    0.2126 * LinearSrgbChannel(color.red)
        + 0.7152 * LinearSrgbChannel(color.green)
        + 0.0722 * LinearSrgbChannel(color.blue)
}

// TableDefaultBorderColorCurrentColor is stable at this Chromium revision.
// cpp: paint/box_border_painter.cc:617-635
fn CalculateInsetOutsetColor(is_darken: bool, color: Color) -> Color {
    const BASE_DARK_COLOR_LUMINANCE: f32 = 0.014443844;
    const BASE_LIGHT_COLOR_LUMINANCE: f32 = 0.83077;
    let luminance = RelativeLuminance(color);
    if luminance <= BASE_DARK_COLOR_LUMINANCE {
        return if is_darken {
            LightColor(color)
        } else {
            LightColor(LightColor(color))
        };
    }
    if is_darken {
        return DarkColor(color);
    }
    if luminance > BASE_LIGHT_COLOR_LUMINANCE {
        color
    } else {
        LightColor(color)
    }
}

fn StyledBorderColor(edge: usize, style: BorderLineStyle, color: Color, alpha_mask: bool) -> Color {
    if alpha_mask {
        return color;
    }
    if matches!(style, BorderLineStyle::kInset | BorderLineStyle::kOutset) {
        CalculateInsetOutsetColor(DarkenBoxSide(edge, style), color)
    } else {
        color
    }
}

// cpp: paint/box_border_painter.cc:773-843
fn SplitBorderEdgeRect(rect: PaintRect, edge: usize) -> (PaintRect, PaintRect) {
    if matches!(edge, 0 | 2) {
        let first_height = (rect.height * 0.5).ceil();
        let first = PaintRect {
            height: first_height,
            ..rect
        };
        let second = PaintRect {
            y: rect.y + first_height,
            height: (rect.height - first_height).max(0.0),
            ..rect
        };
        if edge == 0 {
            (first, second)
        } else {
            (second, first)
        }
    } else {
        let first_width = (rect.width * 0.5).ceil();
        let first = PaintRect {
            width: first_width,
            ..rect
        };
        let second = PaintRect {
            x: rect.x + first_width,
            width: (rect.width - first_width).max(0.0),
            ..rect
        };
        if edge == 3 {
            (first, second)
        } else {
            (second, first)
        }
    }
}

fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(self_cull_rect) = node.self_cull_rect else {
        return true;
    };
    let mapped = MapRectToRoot(*rect, &node.transforms);
    mapped.is_none_or(|mapped| Intersects(&mapped, &self_cull_rect))
}

// cpp: paint/box_border_painter.cc:990-996
fn LineIntersection(a: Offset, b: Offset, c: Offset, d: Offset) -> Offset {
    let ab_x = a.x - b.x;
    let ab_y = a.y - b.y;
    let cd_x = c.x - d.x;
    let cd_y = c.y - d.y;
    let denominator = ab_x * cd_y - ab_y * cd_x;
    if denominator.abs() < 1e-12 {
        return b;
    }
    let determinant_ab = a.x * b.y - a.y * b.x;
    let determinant_cd = c.x * d.y - c.y * d.x;
    Offset {
        x: (determinant_ab * cd_x - ab_x * determinant_cd) / denominator,
        y: (determinant_ab * cd_y - ab_y * determinant_cd) / denominator,
    }
}

// cpp: paint/box_border_painter.cc:1965-2465
pub(crate) fn RoundedBorderSideClips(
    outer: &PaintRect,
    inner: &PaintRect,
    inner_radii: &PaintCornerRadii,
) -> [Vec<PaintPathCommand>; 4] {
    let outer_right = outer.x + outer.width;
    let outer_bottom = outer.y + outer.height;
    let inner_right = inner.x + inner.width;
    let inner_bottom = inner.y + inner.height;
    let outer_corners = [
        Offset {
            x: outer.x,
            y: outer.y,
        },
        Offset {
            x: outer_right,
            y: outer.y,
        },
        Offset {
            x: outer_right,
            y: outer_bottom,
        },
        Offset {
            x: outer.x,
            y: outer_bottom,
        },
    ];
    let inner_corners = [
        Offset {
            x: inner.x,
            y: inner.y,
        },
        Offset {
            x: inner_right,
            y: inner.y,
        },
        Offset {
            x: inner_right,
            y: inner_bottom,
        },
        Offset {
            x: inner.x,
            y: inner_bottom,
        },
    ];
    let miter = [
        LineIntersection(
            outer_corners[0],
            inner_corners[0],
            Offset {
                x: inner.x + inner_radii.top_left.x,
                y: inner.y,
            },
            Offset {
                x: inner.x,
                y: inner.y + inner_radii.top_left.y,
            },
        ),
        LineIntersection(
            outer_corners[1],
            inner_corners[1],
            Offset {
                x: inner_right - inner_radii.top_right.x,
                y: inner.y,
            },
            Offset {
                x: inner_right,
                y: inner.y + inner_radii.top_right.y,
            },
        ),
        LineIntersection(
            outer_corners[2],
            inner_corners[2],
            Offset {
                x: inner_right,
                y: inner_bottom - inner_radii.bottom_right.y,
            },
            Offset {
                x: inner_right - inner_radii.bottom_right.x,
                y: inner_bottom,
            },
        ),
        LineIntersection(
            outer_corners[3],
            inner_corners[3],
            Offset {
                x: inner.x + inner_radii.bottom_left.x,
                y: inner_bottom,
            },
            Offset {
                x: inner.x,
                y: inner_bottom - inner_radii.bottom_left.y,
            },
        ),
    ];
    let path = |points: [Offset; 4]| {
        vec![
            PaintPathCommand {
                verb: PaintPathVerb::kMoveTo,
                point: points[0],
                ..Default::default()
            },
            PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: points[1],
                ..Default::default()
            },
            PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: points[2],
                ..Default::default()
            },
            PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: points[3],
                ..Default::default()
            },
            PaintPathCommand {
                verb: PaintPathVerb::kClose,
                ..Default::default()
            },
        ]
    };
    [
        path([outer_corners[0], miter[0], miter[1], outer_corners[1]]),
        path([outer_corners[1], miter[1], miter[2], outer_corners[2]]),
        path([outer_corners[2], miter[2], miter[3], outer_corners[3]]),
        path([outer_corners[3], miter[3], miter[0], outer_corners[0]]),
    ]
}

// cpp: paint/box_border_painter.cc:2038-2465
fn BorderEdgePolygons(outer: &PaintRect, inner: &PaintRect) -> [[Offset; 4]; 4] {
    let right = outer.x + outer.width;
    let bottom = outer.y + outer.height;
    let inner_right = inner.x + inner.width;
    let inner_bottom = inner.y + inner.height;
    let point = |x, y| Offset { x, y };
    [
        [
            point(outer.x, outer.y),
            point(right, outer.y),
            point(inner_right, inner.y),
            point(inner.x, inner.y),
        ],
        [
            point(right, outer.y),
            point(right, bottom),
            point(inner_right, inner_bottom),
            point(inner_right, inner.y),
        ],
        [
            point(right, bottom),
            point(outer.x, bottom),
            point(inner.x, inner_bottom),
            point(inner_right, inner_bottom),
        ],
        [
            point(outer.x, bottom),
            point(outer.x, outer.y),
            point(inner.x, inner.y),
            point(inner.x, inner_bottom),
        ],
    ]
}

// cpp: paint/box_border_painter.cc:2038-2465
fn BorderEdgeClip(
    node_id: u64,
    phase: PaintPhase,
    polygon: &[Offset; 4],
    antialias: bool,
) -> DisplayItem {
    let mut clip = DisplayItem {
        r#type: DisplayItemType::kClipPath,
        phase,
        node_id,
        antialias,
        ..Default::default()
    };
    clip.path = [
        PaintPathVerb::kMoveTo,
        PaintPathVerb::kLineTo,
        PaintPathVerb::kLineTo,
        PaintPathVerb::kLineTo,
    ]
    .into_iter()
    .zip(*polygon)
    .map(|(verb, point)| PaintPathCommand {
        verb,
        point,
        ..Default::default()
    })
    .collect();
    clip
}

// cpp: paint/box_border_painter.cc:1965-2465
fn BorderEdgeCornerClips(
    edge: usize,
    outer: &PaintRect,
    inner: &PaintRect,
    radii: &PaintCornerRadii,
) -> [[Offset; 4]; 2] {
    let l = outer.x;
    let t = outer.y;
    let r = outer.x + outer.width;
    let b = outer.y + outer.height;
    let il = inner.x;
    let it = inner.y;
    let ir = inner.x + inner.width;
    let ib = inner.y + inner.height;
    let top = inner.y - outer.y;
    let right = r - ir;
    let bottom = b - ib;
    let left = inner.x - outer.x;
    const E: f64 = 0.1;
    let point = |x, y| Offset { x, y };
    match edge {
        0 => {
            let ly = t + (radii.top_left.y + top) * 0.5;
            let ry = t + (radii.top_right.y + top) * 0.5;
            let lx = if top > 0.0 {
                l + left * (ly - t) / top
            } else {
                il
            };
            let rx = if top > 0.0 {
                r - right * (ry - t) / top
            } else {
                ir
            };
            [
                [
                    point(l - E, t),
                    point(lx - E, ly),
                    point(r, ly),
                    point(r, t),
                ],
                [
                    point(l, t),
                    point(l, ry),
                    point(rx + E, ry),
                    point(r + E, t),
                ],
            ]
        }
        1 => {
            let tx = r - (radii.top_right.x + right) * 0.5;
            let bx = r - (radii.bottom_right.x + right) * 0.5;
            let ty = (if right > 0.0 {
                t + top * (r - tx) / right
            } else {
                it
            }) - E;
            let by = (if right > 0.0 {
                b - bottom * (r - bx) / right
            } else {
                ib
            }) + E;
            [
                [point(r, t - E), point(tx, ty), point(tx, b), point(r, b)],
                [point(r, t), point(bx, t), point(bx, by), point(r, b + E)],
            ]
        }
        2 => {
            let ry = b - (radii.bottom_right.y + bottom) * 0.5;
            let ly = b - (radii.bottom_left.y + bottom) * 0.5;
            let rx = if bottom > 0.0 {
                r - right * (b - ry) / bottom
            } else {
                ir
            };
            let lx = if bottom > 0.0 {
                l + left * (b - ly) / bottom
            } else {
                il
            };
            [
                [
                    point(r + E, b),
                    point(rx + E, ry),
                    point(l, ry),
                    point(l, b),
                ],
                [point(r, b), point(r, ly), point(lx, ly), point(l - E, b)],
            ]
        }
        _ => {
            let bx = l
                + (radii.bottom_left.x * left * left / (left * left + bottom * bottom).max(1e-9))
                    .max((radii.bottom_left.x + left) * 0.5);
            let tx = l
                + (radii.top_left.x * left * left / (left * left + top * top).max(1e-9))
                    .max((radii.top_left.x + left) * 0.5);
            let by = (if left > 0.0 {
                b - bottom * (bx - l) / left
            } else {
                ib
            }) + E;
            let ty = (if left > 0.0 {
                t + top * (tx - l) / left
            } else {
                it
            }) - E;
            [
                [point(l, b + E), point(bx, by), point(bx, t), point(l, t)],
                [point(l, b), point(tx, b), point(tx, ty), point(l, t - E)],
            ]
        }
    }
}

// cpp: paint/box_border_painter.cc:1965-2465
fn CombinedBorderEdgeClip(
    edge: usize,
    outer: &PaintRect,
    inner: &PaintRect,
    clips: &[[Offset; 4]; 2],
    first_adjacent_renders: bool,
    second_adjacent_renders: bool,
) -> [Offset; 4] {
    let l = outer.x;
    let t = outer.y;
    let r = outer.x + outer.width;
    let b = outer.y + outer.height;
    let il = inner.x;
    let it = inner.y;
    let ir = inner.x + inner.width;
    let ib = inner.y + inner.height;
    const E: f64 = 0.1;
    let point = |x, y| Offset { x, y };
    match edge {
        0 => [
            point(l, t),
            if first_adjacent_renders {
                point(clips[0][1].x + E, clips[0][1].y)
            } else {
                point(l, it)
            },
            if second_adjacent_renders {
                point(clips[1][2].x - E, clips[1][2].y)
            } else {
                point(r, it)
            },
            point(r, t),
        ],
        1 => [
            point(r, t),
            if first_adjacent_renders {
                point(clips[0][1].x, clips[0][1].y + E)
            } else {
                point(ir, t)
            },
            if second_adjacent_renders {
                point(clips[1][2].x, clips[1][2].y - E)
            } else {
                point(ir, b)
            },
            point(r, b),
        ],
        2 => [
            point(r, b),
            if first_adjacent_renders {
                point(clips[0][1].x - E, clips[0][1].y)
            } else {
                point(r, ib)
            },
            if second_adjacent_renders {
                point(clips[1][2].x + E, clips[1][2].y)
            } else {
                point(l, ib)
            },
            point(l, b),
        ],
        _ => [
            point(l, b),
            if first_adjacent_renders {
                point(clips[0][1].x, clips[0][1].y - E)
            } else {
                point(il, b)
            },
            if second_adjacent_renders {
                point(clips[1][2].x, clips[1][2].y + E)
            } else {
                point(il, t)
            },
            point(l, t),
        ],
    }
}
