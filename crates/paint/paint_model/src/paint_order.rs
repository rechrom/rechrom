#![allow(non_snake_case)]

use layoutng_assembly::fragment_tree::{FragmentKind, FragmentPaintData};
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
    fragment: &FragmentPaintData,
    is_root: bool,
    is_flex_or_grid_item: bool,
) -> PaintLayerClassification {
    // The native Layout owner decides both presence and absence. SVG children
    // deliberately have no CSS PaintLayer; their transforms/effects are scoped
    // in SVGContainerPainter's sibling-order walk. Treating zero client id as
    // unknown promoted transformed SVG siblings above later untransformed ones.
    // The existing mask recorder still requires a synthetic layer scope and
    // promotes it in PrePaint after classification. Until mask effect scopes
    // are independent of CSS layers, retain that adapter for native no-layer
    // fragments here too so Interaction keeps the same inverse paint order.
    let requires_mask_recording_scope = fragment.paint.has_source
        && (!fragment.paint.style.mask_images.is_empty()
            || fragment.paint.style.mask_box_image.is_some());
    if fragment.paint.paint_layer_state_is_native
        && (fragment.paint.paint_layer_client_id != 0 || !requires_mask_recording_scope)
    {
        let is_paint_layer = fragment.kind == FragmentKind::kBox
            && fragment.paint.establishes_paint_state
            && fragment.paint.paint_layer_client_id != 0
            && fragment.paint.paint_layer_is_self_painting;
        return PaintLayerClassification {
            is_paint_layer,
            is_stacking_context: is_paint_layer
                && (is_root || fragment.paint.paint_layer_is_stacking_context),
            stacking_level: if is_paint_layer {
                fragment.paint.paint_layer_stacking_level
            } else {
                0
            },
        };
    }
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
    let mut stacking_level = if positioned_z_index {
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
                    || !style.backdrop_filters.is_empty()
                    || style.backdrop_filter_has_reference
                    || style.blend_mode != PaintBlendMode::kNormal
                    || style.isolate_blending
                    || style.clip_path.is_some()
                    || !style.mask_images.is_empty()
                    || style.mask_box_image.is_some()
            }));
    // A native PaintLayer is authoritative when the bridge exported one.
    // Borrowed line/text fragments never become independent layer owners.
    if fragment.paint.paint_layer_client_id != 0 {
        is_paint_layer = fragment.kind == FragmentKind::kBox
            && establishes
            && fragment.paint.paint_layer_is_self_painting;
    }
    let mut is_stacking_context = is_root
        || (establishes
            && style.is_some_and(|style| {
                positioned_z_index
                    || position == Position::kFixed
                    || position == Position::kSticky
                    || style.transform.is_some()
                    || style.will_change_transform
                    || style.opacity != 1.0
                    || !style.filters.is_empty()
                    || !style.backdrop_filters.is_empty()
                    || style.backdrop_filter_has_reference
                    || style.blend_mode != PaintBlendMode::kNormal
                    || style.isolate_blending
                    || style.clip_path.is_some()
                    || !style.mask_images.is_empty()
                    || style.mask_box_image.is_some()
            }));
    if fragment.paint.paint_layer_client_id != 0 && is_paint_layer {
        is_stacking_context = is_root || fragment.paint.paint_layer_is_stacking_context;
        stacking_level = fragment.paint.paint_layer_stacking_level;
    }
    PaintLayerClassification {
        is_paint_layer,
        is_stacking_context,
        stacking_level,
    }
}

#[cfg(test)]
mod native_layer_ownership_tests {
    use super::*;

    #[test]
    fn known_absence_and_presence_do_not_use_standalone_style_fallback() {
        let mut fragment = FragmentPaintData::default();
        fragment.kind = FragmentKind::kBox;
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.style.transform = Some(Default::default());
        assert!(ClassifyFragmentPaintLayer(&fragment, false, false).is_paint_layer);

        fragment.paint.paint_layer_state_is_native = true;
        assert_eq!(
            ClassifyFragmentPaintLayer(&fragment, false, false),
            PaintLayerClassification::default()
        );

        fragment.paint.style.mask_images.push(Default::default());
        let mask = ClassifyFragmentPaintLayer(&fragment, false, false);
        assert!(mask.is_paint_layer && mask.is_stacking_context);
        fragment.paint.style.mask_images.clear();
        assert_eq!(
            ClassifyFragmentPaintLayer(&fragment, false, false),
            PaintLayerClassification::default()
        );

        fragment.paint.paint_layer_client_id = 91;
        fragment.paint.paint_layer_is_self_painting = true;
        fragment.paint.paint_layer_is_stacking_context = true;
        fragment.paint.paint_layer_stacking_level = -2;
        assert_eq!(
            ClassifyFragmentPaintLayer(&fragment, false, false),
            PaintLayerClassification {
                is_paint_layer: true,
                is_stacking_context: true,
                stacking_level: -2,
            }
        );
        fragment.paint.paint_layer_is_self_painting = false;
        assert_eq!(
            ClassifyFragmentPaintLayer(&fragment, false, false),
            PaintLayerClassification::default()
        );
    }
}
