#![allow(non_snake_case)]

use layoutng_assembly::fragment_tree::{FragmentKind, FragmentNode};
use layoutng_assembly::internal::layout_input::{NodeKind, Position};
use layoutng_assembly::internal::paint_input::PaintBlendMode;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintLayerClassification {
    pub is_paint_layer: bool,
    pub is_stacking_context: bool,
    pub stacking_level: i32,
}

// Keep the fragment-side projection of Blink's PaintLayer/stacking rules in
// paint. Interaction consumes this result to walk the inverse paint order; it
// must not grow its own, subtly different definition of a paint layer.
pub fn ClassifyFragmentPaintLayer(
    fragment: &FragmentNode,
    is_root: bool,
    is_flex_or_grid_item: bool,
) -> PaintLayerClassification {
    let style = fragment.paint.has_source.then_some(&*fragment.paint.style);
    let establishes = fragment.paint.establishes_paint_state;
    let position = if fragment.paint.has_source {
        fragment.paint.position
    } else {
        Position::kStatic
    };
    let positioned_z_index = style.is_some_and(|style| {
        style.z_index.is_some() && (position != Position::kStatic || is_flex_or_grid_item)
    });
    let stacking_level = if positioned_z_index {
        style.unwrap().z_index.unwrap()
    } else {
        0
    };
    let mut is_paint_layer = is_root
        || fragment.paint.source_kind == NodeKind::kSvgRoot
        || (establishes
            && style.is_some_and(|style| {
                position != Position::kStatic
                    || positioned_z_index
                    || style.transform.is_some()
                    || style.will_change_transform
                    || style.opacity != 1.0
                    || !style.filters.is_empty()
                    || style.blend_mode != PaintBlendMode::kNormal
                    || style.isolate_blending
                    || style.clip_path.is_some()
                    || !style.mask_images.is_empty()
            }));
    // A native PaintLayer is authoritative when the bridge exported one.
    // Borrowed line/text fragments never become independent layer owners.
    if fragment.paint.paint_layer_client_id != 0 {
        is_paint_layer = fragment.kind == FragmentKind::kBox
            && establishes
            && fragment.paint.paint_layer_is_self_painting;
    }
    let is_stacking_context = is_root
        || (establishes
            && style.is_some_and(|style| {
                positioned_z_index
                    || position == Position::kFixed
                    || position == Position::kSticky
                    || style.transform.is_some()
                    || style.will_change_transform
                    || style.opacity != 1.0
                    || !style.filters.is_empty()
                    || style.blend_mode != PaintBlendMode::kNormal
                    || style.isolate_blending
                    || style.clip_path.is_some()
                    || !style.mask_images.is_empty()
            }));
    PaintLayerClassification {
        is_paint_layer,
        is_stacking_context,
        stacking_level,
    }
}
