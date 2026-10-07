//! The initial static block slice of //src/paint. Unsupported paint features
//! fail explicitly until their source painters are translated.

use layoutng_assembly::fragment_tree::{FragmentKind, FragmentNode, PaintGlyphRun, PaintResources};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, FloatSide, Offset, Overflow, Position,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{BackgroundBox, PaintBlendMode, PaintCornerRadii};
use std::sync::Arc;

pub mod background_geometry;
pub mod border_shape_utils;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod box_fragment_painter;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod caret_display_item_client;
#[cfg(feature = "translation_in_progress")]
mod client_rects;
pub mod display_item_id;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod drawing_recorder;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod fieldset_painter;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod frame_set_painter;
pub mod geometry_mapper;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod mathml_painter;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod nine_piece_image_grid;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod nine_piece_image_painter;
pub mod paint_chunker;
pub(crate) mod paint_context;
pub mod paint_engine;
pub(crate) mod paint_info;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod paint_layer_painter;
pub mod paint_property_tree;
pub(crate) mod paint_shader_resolver;
pub(crate) mod pre_paint_tree_walk;
#[cfg(feature = "translation_in_progress")]
pub mod scroll_paint_update;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod scrollable_area_painter;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod svg_shape_painter;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod table_painters;
#[cfg(feature = "translation_in_progress")]
pub(crate) mod theme_painter;

pub use paint_engine::PaintRect;
#[cfg(feature = "translation_in_progress")]
pub use paint_engine::{PaintArtifact, PaintEngine};

#[derive(Clone)]
pub struct DisplayItem {
    pub node_id: u64,
    pub fragment_instance_id: u64,
    pub rect: PaintRect,
    pub color: Color,
    pub glyph_run: Option<PaintGlyphRun>,
    pub text_blob_origin: Offset,
}

#[derive(Clone, Default)]
pub struct DisplayItemList {
    pub items: Vec<DisplayItem>,
    pub resources: Option<Arc<PaintResources>>,
}

fn snapped(rect: PaintRect) -> PaintRect {
    let left = (rect.x + 0.5).floor();
    let top = (rect.y + 0.5).floor();
    let right = (rect.x + rect.width + 0.5).floor();
    let bottom = (rect.y + rect.height + 0.5).floor();
    PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

fn append(items: &mut Vec<DisplayItem>, node: &FragmentNode, rect: PaintRect, color: Color) {
    if color.alpha <= 0.0 {
        return;
    }
    assert_eq!(
        color.alpha, 1.0,
        "translucent block paint requires compositing"
    );
    items.push(DisplayItem {
        node_id: node.node_id,
        fragment_instance_id: node.fragment_instance_id,
        rect: snapped(rect),
        color,
        glyph_run: None,
        text_blob_origin: Offset::default(),
    });
}

fn walk(node: &FragmentNode, parent: Offset, items: &mut Vec<DisplayItem>) {
    if node.paint.hidden || !node.paint.style.visible {
        return;
    }
    let style = &*node.paint.style;
    let x = parent.x + node.offset.x;
    let y = parent.y + node.offset.y;
    if node.kind == FragmentKind::kLine {
        for child in &node.children {
            walk(child, Offset { x, y }, items);
        }
        return;
    }
    if matches!(
        node.kind,
        FragmentKind::kText | FragmentKind::kGeneratedText
    ) {
        let snap_delta = node.paint.text_line_top_offset.map_or(0.0, |line_offset| {
            let line_top = y + line_offset;
            line_top.round() - line_top
        });
        for run in &node.paint.glyph_runs {
            if run.glyphs.is_empty() {
                continue;
            }
            items.push(DisplayItem {
                node_id: node.node_id,
                fragment_instance_id: node.fragment_instance_id,
                rect: PaintRect {
                    x,
                    y,
                    width: node.size.width,
                    height: node.size.height,
                },
                color: style.color,
                glyph_run: Some(run.clone()),
                text_blob_origin: Offset {
                    x,
                    y: y + snap_delta + run.baseline,
                },
            });
        }
        return;
    }
    assert!(
        style.background_images.is_empty()
            && style.border_image.is_none()
            && style.box_shadows.is_empty()
            && style.border_radii.is_none()
            && style.transform.is_none()
            && style.mask_images.is_empty()
            && style.filters.is_empty(),
        "advanced box paint is not installed"
    );
    assert_eq!(style.background_clip, BackgroundBox::kBorderBox);
    assert_eq!(style.blend_mode, PaintBlendMode::kNormal);
    assert_eq!(style.opacity, 1.0, "opacity paint is not installed");
    assert_eq!(
        style.border_radius, 0.0,
        "rounded borders are not installed"
    );
    assert_eq!(
        style.border_radii_percentages,
        PaintCornerRadii::default(),
        "rounded borders are not installed"
    );
    assert!(
        style.z_index.is_none() && !style.isolate_blending,
        "stacking paint is not installed"
    );
    assert_eq!(
        node.paint.position,
        Position::kStatic,
        "positioned paint is not installed"
    );
    assert_eq!(
        node.paint.floating,
        FloatSide::kNone,
        "float paint is not installed"
    );
    assert_eq!(
        node.paint.scroll_offset,
        Offset::default(),
        "scroll paint is not installed"
    );
    assert!(
        node.paint.border_sides.HasAllSides(),
        "fragmented border paint is not installed"
    );
    assert_eq!(
        node.paint.overflow_x,
        Overflow::kVisible,
        "overflow clip paint is not installed"
    );
    assert_eq!(
        node.paint.overflow_y,
        Overflow::kVisible,
        "overflow clip paint is not installed"
    );
    let outer = PaintRect {
        x,
        y,
        width: node.size.width,
        height: node.size.height,
    };
    if node.paint.establishes_paint_state {
        append(items, node, outer, style.background_color);
        let border = node.paint.border;
        let widths = [border.top, border.right, border.bottom, border.left];
        let mut first_border_color = None;
        for (side, width) in widths.iter().enumerate() {
            if *width > 0.0 && style.border_colors[side].alpha > 0.0 {
                if let Some(color) = first_border_color {
                    assert_eq!(
                        style.border_colors[side], color,
                        "mixed-color border joins are not installed"
                    );
                } else {
                    first_border_color = Some(style.border_colors[side]);
                }
            }
        }
        for (side, width) in widths.into_iter().enumerate() {
            if width <= 0.0 {
                continue;
            }
            assert_eq!(
                node.paint.border_styles[side],
                BorderLineStyle::kSolid,
                "non-solid border paint is not installed"
            );
            let rect = match side {
                0 => PaintRect {
                    x,
                    y,
                    width: outer.width,
                    height: width,
                },
                1 => PaintRect {
                    x: x + outer.width - width,
                    y,
                    width,
                    height: outer.height,
                },
                2 => PaintRect {
                    x,
                    y: y + outer.height - width,
                    width: outer.width,
                    height: width,
                },
                _ => PaintRect {
                    x,
                    y,
                    width,
                    height: outer.height,
                },
            };
            append(items, node, rect, style.border_colors[side]);
        }
    }
    for child in &node.children {
        walk(child, Offset { x, y }, items);
    }
}

#[allow(non_snake_case)]
pub fn Paint(root: &FragmentNode) -> DisplayItemList {
    let mut result = DisplayItemList {
        items: Vec::new(),
        resources: root.paint.resources.clone(),
    };
    walk(root, Offset::default(), &mut result.items);
    result
}
