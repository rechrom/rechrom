use super::*;
use layer_tile::{CompositorScrollOffset, FrameConfig, LayerTileEngine};
use paint::paint_engine::PaintArtifact;
use paint::paint_property_tree::PaintPropertyNodeLifecycle;

pub(super) fn fixture() -> FramePlan {
    let bounds = PaintRect {
        x: 0.,
        y: 0.,
        width: 64.,
        height: 64.,
    };
    let mut engine = LayerTileEngine::new(64);
    engine
        .UpdatePending(
            &Arc::new(PaintArtifact::default()),
            FrameConfig {
                viewport: bounds,
                raster_scale: 1.,
                device_scale_factor: 1.,
                layout_zoom_factor: 1.,
                activation_scroll: None,
                frame_time: None,
            },
        )
        .unwrap();
    let mut plan = engine.GetPendingFramePlan().unwrap().clone();
    for id in 1..=2 {
        let mut properties = PropertyTreeState::default();
        if id == 2 {
            let mut transform = (*properties.transform).clone();
            transform.lifecycle = PaintPropertyNodeLifecycle::default();
            transform.id = 2;
            transform.parent = Some(properties.transform.clone());
            let mut scroll = (**transform.scroll.as_ref().unwrap()).clone();
            scroll.lifecycle = PaintPropertyNodeLifecycle::default();
            scroll.id = 2;
            scroll.user_scrollable_vertical = true;
            transform.scroll = Some(Arc::new(scroll));
            properties.transform = Arc::new(transform);
        }
        let mut effect = (*properties.effect).clone();
        effect.lifecycle = PaintPropertyNodeLifecycle::default();
        effect.id = id;
        effect.parent = Some(properties.effect.clone());
        effect.local_transform_space = properties.transform.clone();
        effect.opacity = 0.5;
        properties.effect = Arc::new(effect);
        plan.layers.push(LayerPlan {
            id: LayerId(id),
            properties,
            is_first_layer: false,
            requires_transparent_backing: true,
            chunk_indices: vec![],
            bounds,
            bounds_are_complete: true,
            raster_content_bounds: Some(bounds),
            rect_known_to_be_opaque: PaintRect::default(),
            raster_origin: (0., 0.),
            root_translation: (0., 0.),
            compositor_scroll: None,
            root_clip: None,
            record_indices: vec![],
            interest_tiles: vec![],
            tiles: vec![TilePlacement {
                tile_id: TileId(id),
                generation: 1,
                tile_index: (0, 0),
                tiling_rect: bounds,
                tile_rect: bounds,
                raster_scale: 1.,
                pixel_size: (64, 64),
                ready: true,
            }],
        });
    }
    plan
}
fn child_damage(frame: &CompositorFrame, effect: &EffectPaintPropertyNode) -> DeviceRect {
    frame
        .render_pass_list
        .iter()
        .find(|pass| pass.id == effect_pass_id(effect))
        .unwrap()
        .damage_rect
}
fn empty(rect: DeviceRect) -> bool {
    rect.width == 0 || rect.height == 0
}

#[test]
fn contributing_inputs_retain_only_unchanged_children_and_successful_build_history() {
    let mut plan = fixture();
    let mut builder = FrameBuilder::default();
    let fixed = plan.layers[0].properties.effect.clone();
    let moving = plan.layers[1].properties.effect.clone();
    let first = builder.BuildFrame(&plan).unwrap();
    assert_eq!(first.render_pass_list.len(), 3);
    assert_eq!(first.metadata.frame_token, 1);
    assert!(!empty(child_damage(&first, &fixed)));
    let unchanged = builder.BuildFrame(&plan).unwrap();
    assert!(unchanged
        .render_pass_list
        .iter()
        .all(|pass| empty(pass.damage_rect)));
    assert_eq!(unchanged.metadata.frame_token, 2);
    assert_eq!(
        unchanged.metadata.source_frame_id,
        first.metadata.source_frame_id
    );
    plan.layers[1].compositor_scroll = Some(CompositorScrollOffset {
        scroll_node_id: 2,
        translation_x: 0.,
        translation_y: -3.,
    });
    let scrolled = builder.BuildFrame(&plan).unwrap();
    assert!(empty(child_damage(&scrolled, &fixed)));
    assert!(!empty(child_damage(&scrolled, &moving)));
    let moving_pass = scrolled.render_pass_list.iter().find(|pass| pass.id == effect_pass_id(&moving)).unwrap();
    assert_eq!(moving_pass.output_rect, DeviceRect::new(0, 0, 64, 61));
    assert_eq!(moving_pass.damage_rect, moving_pass.output_rect);
    assert_ne!(
        scrolled.render_pass_list[1].shared_quad_state_list[0]
            .composition
            .transform
            .ty,
        0.
    );
    let token = scrolled.metadata.frame_token;
    plan.layers[1].tiles[0].ready = false;
    assert!(builder.BuildFrame(&plan).is_err());
    plan.layers[1].tiles[0].ready = true;
    let retried = builder.BuildFrame(&plan).unwrap();
    assert_eq!(retried.metadata.frame_token, token + 1);
    assert!(retried
        .render_pass_list
        .iter()
        .all(|pass| empty(pass.damage_rect)));
    plan.layers[1].tiles[0].generation += 1;
    let changed = builder.BuildFrame(&plan).unwrap();
    assert!(empty(child_damage(&changed, &fixed)));
    assert!(!empty(child_damage(&changed, &moving)));
    // Separate raster owners may reuse all numeric tile/layer identities.
    plan.layer_tree_id = layer_tile::LayerTreeId(plan.layer_tree_id.0 + 1000);
    assert!(builder
        .BuildFrame(&plan)
        .unwrap()
        .render_pass_list
        .iter()
        .all(|pass| !empty(pass.damage_rect)));
    plan.config.raster_scale = 2.;
    assert!(builder
        .BuildFrame(&plan)
        .unwrap()
        .render_pass_list
        .iter()
        .all(|pass| !empty(pass.damage_rect)));
    plan.config.device_scale_factor = 2.;
    assert!(builder
        .BuildFrame(&plan)
        .unwrap()
        .render_pass_list
        .iter()
        .all(|pass| !empty(pass.damage_rect)));
    let removed = plan.layers.pop().unwrap();
    builder.BuildFrame(&plan).unwrap();
    plan.layers.push(removed);
    assert!(!empty(child_damage(
        &builder.BuildFrame(&plan).unwrap(),
        &moving
    )));
    let previous = builder.submission_sequence;
    builder.Invalidate();
    let invalidated = builder.BuildFrame(&plan).unwrap();
    assert_eq!(invalidated.metadata.frame_token, previous + 1);
    assert!(invalidated
        .render_pass_list
        .iter()
        .all(|pass| !empty(pass.damage_rect)));
    let tiles = std::mem::take(&mut plan.layers[1].tiles);
    let offscreen = builder.BuildFrame(&plan).unwrap();
    assert!(offscreen.render_pass_list.iter().all(|pass| pass.id != effect_pass_id(&moving)),
        "empty effect output must not become an unconstrained render-pass quad");
    plan.layers[1].tiles = tiles;
    assert!(builder.BuildFrame(&plan).unwrap().render_pass_list.iter()
        .any(|pass| pass.id == effect_pass_id(&moving)), "returning content recreates its surface");
}

#[test]
fn nested_order_effect_and_descendant_content_are_contributing_inputs() {
    let mut plan = fixture();
    let parent = plan.layers[0].properties.effect.clone();
    let mut nested = (*plan.layers[1].properties.effect).clone();
    nested.parent = Some(parent.clone());
    plan.layers[1].properties.effect = Arc::new(nested);
    let child = plan.layers[1].properties.effect.clone();
    let mut extra = plan.layers[1].clone();
    extra.id = LayerId(3);
    extra.tiles[0].tile_id = TileId(3);
    plan.layers.push(extra);
    let mut builder = FrameBuilder::default();
    builder.BuildFrame(&plan).unwrap();
    assert!(builder
        .BuildFrame(&plan)
        .unwrap()
        .render_pass_list
        .iter()
        .all(|pass| empty(pass.damage_rect)));
    plan.layers[2].tiles[0].generation += 1;
    let changed = builder.BuildFrame(&plan).unwrap();
    assert!(!empty(child_damage(&changed, &child)));
    assert!(!empty(child_damage(&changed, &parent)));
    plan.layers.swap(1, 2);
    assert!(builder
        .BuildFrame(&plan)
        .unwrap()
        .render_pass_list
        .iter()
        .all(|pass| !empty(pass.damage_rect)));
    // Keep lifecycle identity stable while changing effect values, including
    // mask/filter state: comparing PropertyTreeState identity alone is unsafe.
    let mut effect = (*child).clone();
    effect.opacity = 0.25;
    effect.filters.push(
        layoutng_assembly::internal::paint_input::PaintFilterOperation {
            r#type: layoutng_assembly::internal::paint_input::PaintFilterType::kBlur,
            amount: 1.,
            ..Default::default()
        },
    );
    let effect = Arc::new(effect);
    plan.layers[1].properties.effect = effect.clone();
    plan.layers[2].properties.effect = effect;
    assert!(builder
        .BuildFrame(&plan)
        .unwrap()
        .render_pass_list
        .iter()
        .all(|pass| !empty(pass.damage_rect)));
}
