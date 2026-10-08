//! Eligibility for a direct leaf rounded-clip quad, without a clip RGBA plane.
//! Only planning lives here: real AA coverage and sampling stay in the renderer.
use super::*;
use layoutng_assembly::internal::paint_input::PaintBlendMode;
use paint::paint_property_tree::TransformPaintPropertyNode;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Copy)]
pub(super) struct DirectClipRun {
    /// Index of the existing Scope::Clip in this layer's actual chain.
    pub scope_index: usize,
    /// Existing allocation opening identity, never a synthesized client ID.
    pub opening_run: usize,
}

struct Candidate {
    valid: bool,
    layers: Vec<usize>,
    scope_index: usize,
}

/// Return a direct-clip candidate in exactly the frame's layer order.
/// Each layer's already-rastered records may overlap; the complete run's
/// output tile rectangles must not. Thus each destination pixel has at most one
/// source quad, and multiplying its AA coverage before SrcOver preserves the
/// isolated clip-plane result. Overlapping, nested and mask runs stay isolated.
pub(super) fn eligible_runs(
    plan: &CompositionPlan,
    effects: &[LayerEffectPlan],
    width: u32,
    height: u32,
) -> io::Result<Vec<Option<DirectClipRun>>> {
    if effects.len() != plan.layers.len() {
        return Err(io::Error::other("direct clip effect plan extent changed"));
    }
    let mut active: Vec<(Scope, usize)> = Vec::new();
    let mut seen_runs = HashSet::new();
    let mut candidates: HashMap<usize, Candidate> = HashMap::new();
    for (layer_index, (layer, effect_plan)) in plan.layers.iter().zip(effects).enumerate() {
        // Mirror effect_allocation_plan and actual execution: these layers
        // do not enter/exit any target. Empty Mask layers DO perform DstIn.
        if layer.tiles.is_empty() && !layer.properties.effect.is_mask {
            continue;
        }
        if effect_plan
            .chain
            .iter()
            .any(|scope| matches!(scope, Scope::Effect(effect) if effect.opacity == 0.0))
        {
            continue;
        }
        let common = active
            .iter()
            .zip(&effect_plan.chain)
            .take_while(|((old, _), next)| old.same_node(next))
            .count();
        if active
            .iter()
            .zip(&effect_plan.chain)
            .take(common)
            .any(|((old, _), next)| !old.same_values(next))
        {
            return Err(io::Error::other(
                "direct clip scope changed within opening run",
            ));
        }
        if effect_plan.chain.len() - common != effect_plan.opened_runs.len() {
            return Err(io::Error::other(
                "direct clip allocation opening run changed",
            ));
        }
        active.truncate(common);
        for (scope, &run) in effect_plan
            .chain
            .iter()
            .skip(common)
            .zip(&effect_plan.opened_runs)
        {
            if !seen_runs.insert(run) {
                return Err(io::Error::other(
                    "direct clip allocation reused a closed opening run",
                ));
            }
            active.push((scope.clone(), run));
        }
        let one_clip = active
            .iter()
            .filter(|(scope, _)| matches!(scope, Scope::Clip(_)))
            .count()
            == 1;
        let has_mask = effect_plan.direct_mask.is_some()
            || active
                .iter()
                .any(|(scope, _)| matches!(scope, Scope::DstIn(_)))
            || !normal_effect_chain(&layer.properties.effect);
        for (scope_index, (scope, run)) in active.iter().enumerate() {
            let Scope::Clip(clip) = scope else {
                continue;
            };
            let candidate = candidates.entry(*run).or_insert_with(|| Candidate {
                valid: fast_rounded_shape(clip),
                layers: Vec::new(),
                scope_index,
            });
            // No child Scope at all is admitted, even an opacity-one mask
            // operation with no visible tile. Outer real Effect targets remain
            // intact: only this last leaf Clip surface may be omitted.
            candidate.valid &= one_clip
                && !has_mask
                && scope_index + 1 == active.len()
                && candidate.scope_index == scope_index;
            if !layer.tiles.is_empty() {
                candidate.layers.push(layer_index);
            }
        }
    }
    let mut result = vec![None; plan.layers.len()];
    for (opening_run, candidate) in candidates {
        if !candidate.valid || candidate.layers.is_empty() {
            continue;
        }
        let mut rectangles = Vec::new();
        let mut valid = true;
        for &layer_index in &candidate.layers {
            let layer = &plan.layers[layer_index];
            let composition = layer.composition;
            if composition.white_backing {
                valid = false;
                break;
            }
            let Some(partition) = layer_tile_partition(layer, composition, width, height)? else {
                valid = false;
                break;
            };
            rectangles.extend(partition);
        }
        // PropertyTreeManager's fast leaf RRect may span several layers.
        // Keep our stronger source-over equivalence proof across the complete
        // opening run: no pixel may receive two independently clipped quads.
        if valid
            && !rectangles.is_empty()
            && (candidate.layers.len() == 1 || disjoint_rectangles(&rectangles))
        {
            for layer_index in candidate.layers {
                result[layer_index] = Some(DirectClipRun {
                    scope_index: candidate.scope_index,
                    opening_run,
                });
            }
        }
    }
    Ok(result)
}

fn disjoint_rectangles(rectangles: &[SurfaceBounds]) -> bool {
    // Sweep x, removing right edges before inserting touching left edges.
    // Active y intervals are already disjoint, so only neighbors can overlap.
    // This admits staggered sibling grids without quadratic tile comparisons.
    let mut edges = Vec::with_capacity(rectangles.len() * 2);
    for rect in rectangles {
        edges.push((rect.left, true, rect.top, rect.bottom));
        edges.push((rect.right, false, rect.top, rect.bottom));
    }
    edges.sort_unstable();
    let mut active = BTreeMap::new();
    for (_, entering, top, bottom) in edges {
        if entering {
            if active
                .range(..=top)
                .next_back()
                .is_some_and(|(_, &end)| end > top)
                || active
                    .range(top..)
                    .next()
                    .is_some_and(|(&start, _)| start < bottom)
            {
                return false;
            }
            active.insert(top, bottom);
        } else {
            active.remove(&top);
        }
    }
    true
}

fn normal_effect_chain(effect: &Arc<EffectPaintPropertyNode>) -> bool {
    let mut node = Some(effect);
    while let Some(effect) = node {
        if effect.is_mask
            || effect.opacity == 0.0
            || effect.blend_mode != PaintBlendMode::kNormal
            || !translation_space(&effect.local_transform_space)
        {
            return false;
        }
        node = effect.parent.as_ref();
    }
    true
}

fn translation_space(space: &Arc<TransformPaintPropertyNode>) -> bool {
    let identity = layoutng_assembly::internal::layout_input::TransformMatrix::default();
    let mut node = Some(space);
    while let Some(transform) = node {
        if transform.origin != [0.0; 3]
            || transform
                .matrix
                .values
                .iter()
                .enumerate()
                .any(|(index, &value)| {
                    !value.is_finite()
                        || (!matches!(index, 12 | 13) && value != identity.values[index])
                })
        {
            return false;
        }
        node = transform.parent.as_ref();
    }
    true
}

fn fast_rounded_shape(clip: &ClipPaintPropertyNode) -> bool {
    // PropertyTreeManager::ShaderBasedRRect: no path/axis-misalignment; circular
    // corners only, and on Mac all four radii must match for CALayer overlays.
    // Use the Mac restriction on every backend for this initial narrow path.
    // SynthesizeCcEffectsForClipsIfNeeded keeps nested rounded ancestors on a
    // surface; SoftwareRenderer::SetClipRRect then clips the actual leaf quad
    // directly with AA=true, without creating a saveLayer.
    if !clip.clip_path.is_empty()
        || clip.pixel_moving_filter.is_some()
        || !translation_space(&clip.local_transform_space)
    {
        return false;
    }
    let Some(rect) = clip.rect else {
        return false;
    };
    if ![
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        rect.x + rect.width,
        rect.y + rect.height,
    ]
    .iter()
    .all(|value| value.is_finite())
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return false;
    }
    let radii = clip.radii;
    let radius = radii.top_left;
    radius.x.is_finite()
        && radius.y.is_finite()
        && radius.x > 0.0
        && radius.x == radius.y
        && radius == radii.top_right
        && radius == radii.bottom_right
        && radius == radii.bottom_left
}

#[cfg(test)]
mod test {
    use super::*;
    use layer_tile::{LayerId, LayerPlan};
    use paint::paint_engine::{
        DisplayItem, DisplayItemType, PaintArtifact, PaintChunk, RecordedDisplayItem,
        RecordedDisplayItemKind,
    };
    use paint::paint_property_tree::PropertyTreeState;

    #[test]
    fn disjoint_sibling_layers_share_direct_clip_and_match_isolated_source() {
        let (width, height) = (64u32, 44u32);
        let root = PropertyTreeState::default();
        let viewport = PaintRect {
            x: 0.0,
            y: 0.0,
            width: width as f64,
            height: height as f64,
        };
        let artifact = Arc::new(PaintArtifact {
            items: vec![DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                rect: viewport,
                ..Default::default()
            }]
            .into(),
            display_items: vec![RecordedDisplayItem {
                kind: RecordedDisplayItemKind::Drawing,
                id: Default::default(),
                visual_rect: viewport,
                visual_rect_is_accurate: true,
                draws_content: true,
                raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
                record_begin: 0,
                record_end: 1,
                scroll_translation: None,
            }],
            chunks: vec![PaintChunk {
                end_index: 1,
                bounds: viewport,
                drawable_bounds: viewport,
                ..Default::default()
            }],
            ..Default::default()
        });
        let mut engine = layer_tile::LayerTileEngine::default();
        let update = engine
            .UpdatePending(
                &artifact,
                layer_tile::FrameConfig {
                    viewport,
                    raster_scale: 1.0,
                    activation_scroll: None,
                    frame_time: None,
                },
            )
            .unwrap();
        let mut plan = update.frame_plan;
        for tile in plan
            .layers
            .iter_mut()
            .flat_map(|layer| layer.tiles.iter_mut())
        {
            tile.ready = true;
        }
        let clip = Arc::new(ClipPaintPropertyNode {
            lifecycle: Default::default(),
            id: 1,
            parent: Some(root.clip.clone()),
            local_transform_space: root.transform.clone(),
            rect: Some(PaintRect {
                x: 1.375,
                y: 1.625,
                width: 50.25,
                height: 33.5,
            }),
            radii: paint::border_shape_utils::UniformCornerRadii(9.75),
            clip_path: vec![],
            clip_path_even_odd: false,
            pixel_moving_filter: None,
        });
        let effect = Arc::new(EffectPaintPropertyNode {
            lifecycle: Default::default(),
            id: 2,
            parent: Some(root.effect.clone()),
            opacity: 0.5,
            ..(*root.effect).clone()
        });
        let properties = PropertyTreeState {
            clip: clip.clone(),
            effect,
            ..root
        };
        let layer = |id, rect| LayerPlan {
            id: LayerId(id),
            is_first_layer: false,
            requires_transparent_backing: true,
            chunk_indices: vec![],
            properties: properties.clone(),
            bounds: rect,
            bounds_are_complete: true,
            raster_content_bounds: Some(rect),
            rect_known_to_be_opaque: PaintRect::default(),
            raster_origin: (rect.x, rect.y),
            root_translation: (0.0, 0.0),
            compositor_scroll: None,
            root_clip: Some(viewport),
            record_indices: vec![],
            interest_tiles: vec![],
            tiles: vec![TilePlacement {
                tile_id: TileId(id),
                generation: 1,
                tile_index: (0, 0),
                tiling_rect: rect,
                tile_rect: rect,
                raster_scale: 1.0,
                pixel_size: (rect.width as u32, rect.height as u32),
                ready: true,
            }],
        };
        // Staggered sibling grids, including real AA corner coverage.
        plan.layers = vec![
            layer(
                1,
                PaintRect {
                    x: 3.0,
                    y: 4.0,
                    width: 20.0,
                    height: 27.0,
                },
            ),
            layer(
                2,
                PaintRect {
                    x: 28.0,
                    y: 6.0,
                    width: 22.0,
                    height: 25.0,
                },
            ),
        ];
        let replay = LayerReplay::new(&plan).unwrap();
        let (effects, _) = effect_allocation_plan(&plan, &replay, width, height, 1.0).unwrap();
        let candidates = eligible_runs(&plan, &effects, &replay, width, height).unwrap();
        assert!(candidates.iter().all(Option::is_some));
        assert_eq!(
            candidates[0].unwrap().opening_run,
            candidates[1].unwrap().opening_run
        );
        assert_eq!(
            candidates[0].unwrap().scope_index,
            1,
            "parent opacity surface stays outside leaf clip"
        );
        let coverage = ClipMaskCache::default()
            .coverage(&[clip], width, height, 1.0, None)
            .unwrap();
        assert!(coverage
            .pixels
            .iter()
            .any(|&alpha| alpha > 0 && alpha < 255));
        let view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        let mut source = vec![0u32; width as usize * height as usize];
        let mut actual = vec![0x70402010u32; source.len()];
        let mut expected = actual.clone();
        let mut touched = None;
        for (index, layer) in plan.layers.iter().enumerate() {
            let tile = &layer.tiles[0];
            let rgba: Vec<u8> = (0..tile.pixel_size.0 * tile.pixel_size.1)
                .flat_map(|i| {
                    let alpha = if i % 13 == 0 {
                        0
                    } else {
                        95 + index as u8 * 75
                    };
                    [alpha / 4, alpha / 2, alpha / 3, alpha]
                })
                .collect();
            let pixels = CachedTile {
                raster_frame: 0,
                generation: 1,
                pixel_size: tile.pixel_size,
                raster_scale: 1.0,
                white_backing: false,
                row_support: tile_row_support(&rgba, tile.pixel_size.0 as usize, false),
                rgba,
                bgra: Vec::new(),
            };
            let composition = replay.composition(layer.id).unwrap();
            union_bounds(
                &mut touched,
                compose_tile(
                    &mut source,
                    width,
                    height,
                    view,
                    skia::PixelFormat::Rgba8888,
                    tile,
                    composition,
                    &pixels,
                    [None; 3],
                    None,
                )
                .unwrap(),
            );
            layer_clip_quad::compose_clipped_tile(
                &mut actual,
                width,
                height,
                view,
                skia::PixelFormat::Rgba8888,
                tile,
                composition,
                &pixels,
                [None; 3],
                None,
                &coverage,
            )
            .unwrap();
        }
        composite_effect_surface(
            &mut expected,
            &source,
            touched,
            view,
            view,
            skia::PixelFormat::Rgba8888,
            1.0,
            Some(&coverage),
        );
        assert_eq!(
            actual, expected,
            "disjoint siblings retain isolated N32 clip/source-over order"
        );
        drop(replay);
        // Overlapping quads must keep their isolated clip surface even if
        // some source pixels happen to be transparent in this generation.
        plan.layers[1] = layer(
            2,
            PaintRect {
                x: 15.0,
                y: 6.0,
                width: 22.0,
                height: 25.0,
            },
        );
        let replay = LayerReplay::new(&plan).unwrap();
        let (effects, _) = effect_allocation_plan(&plan, &replay, width, height, 1.0).unwrap();
        assert!(eligible_runs(&plan, &effects, &replay, width, height)
            .unwrap()
            .iter()
            .all(Option::is_none));
    }
}
