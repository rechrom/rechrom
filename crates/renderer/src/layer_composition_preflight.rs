//! Validate a root render pass before beginning writes to the borrowed output.
//! The subsequent draw consumes these immutable plans and resident tile pixels.
use super::*;

struct ActiveScope {
    scope: Scope,
    direct_dst_in: bool,
    direct_clip: bool,
    run: usize,
}

fn close_scope(active: &mut Vec<ActiveScope>) -> io::Result<ActiveScope> {
    let group = active.pop().expect("active preflight scope");
    if !group.direct_dst_in && !group.direct_clip {
        if let Scope::DstIn(effect) = &group.scope {
            let parent = active
                .last()
                .ok_or_else(|| io::Error::other("mask DstIn has no isolated content surface"))?;
            let expected = effect
                .parent
                .as_ref()
                .ok_or_else(|| io::Error::other("mask effect has no content parent"))?;
            if !matches!(&parent.scope, Scope::Effect(actual)
                if actual.lifecycle.same_node(&expected.lifecycle))
            {
                return Err(io::Error::other(
                    "mask DstIn destination differs from its content effect",
                ));
            }
        }
    }
    Ok(group)
}

pub(super) struct PreparedComposition {
    pub coverage: Vec<Option<Arc<MaskCoverage>>>,
    pub clip_time: std::time::Duration,
}

pub(super) fn prepare(
    plan: &FramePlan,
    effects: &[LayerEffectPlan],
    bounds: &[Option<SurfaceBounds>],
    direct_clips: &[Option<layer_direct_clip::DirectClipRun>],
    visible: &[Vec<Option<SurfaceBounds>>],
    replay: &LayerReplay<'_>,
    pixels: &HashMap<TileId, CachedTile>,
    clips: &mut ClipMaskCache,
    width: u32,
    height: u32,
    scale: f64,
    timing: bool,
) -> io::Result<PreparedComposition> {
    if effects.len() != plan.layers.len()
        || direct_clips.len() != plan.layers.len()
        || visible.len() != plan.layers.len()
    {
        return Err(io::Error::other("composition layer plan extent changed"));
    }
    let mut prepared = PreparedComposition {
        coverage: vec![None; bounds.len()],
        clip_time: std::time::Duration::ZERO,
    };
    let mut active: Vec<ActiveScope> = Vec::new();
    let mut closed_effects: Vec<Arc<EffectPaintPropertyNode>> = Vec::new();
    for (((layer, effect), direct_clip), visible) in plan
        .layers
        .iter()
        .zip(effects)
        .zip(direct_clips)
        .zip(visible)
    {
        if layer.tiles.is_empty() && !layer.properties.effect.is_mask {
            continue;
        }
        let common = active
            .iter()
            .zip(&effect.chain)
            .take_while(|(old, wanted)| old.scope.same_node(wanted))
            .count();
        if active
            .iter()
            .zip(&effect.chain)
            .take(common)
            .any(|(old, wanted)| !old.scope.same_values(wanted))
        {
            return Err(io::Error::other(
                "composition scope changed values within one paint plan",
            ));
        }
        if effect
            .chain
            .iter()
            .any(|scope| matches!(scope, Scope::Effect(effect) if effect.opacity == 0.0))
        {
            continue;
        }
        if effect.chain.len() - common != effect.opened_runs.len() {
            return Err(io::Error::other("effect allocation scope run changed"));
        }
        while active.len() > common {
            let group = close_scope(&mut active)?;
            if let Scope::Effect(effect) | Scope::DstIn(effect) = group.scope {
                closed_effects.push(effect);
            }
        }
        for (scope, &run) in effect.chain.iter().skip(common).zip(&effect.opened_runs) {
            let bounds = bounds
                .get(run)
                .ok_or_else(|| io::Error::other("effect allocation opening run changed"))?
                .unwrap_or(SurfaceBounds {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                });
            if bounds.left > bounds.right
                || bounds.top > bounds.bottom
                || bounds.right > width as usize
                || bounds.bottom > height as usize
                || (bounds.right - bounds.left)
                    .checked_mul(bounds.bottom - bounds.top)
                    .is_none()
            {
                return Err(io::Error::other("invalid effect surface dimensions"));
            }
            match scope {
                Scope::Effect(effect) | Scope::DstIn(effect) => {
                    if closed_effects
                        .iter()
                        .any(|old| old.lifecycle.same_node(&effect.lifecycle))
                    {
                        return Err(io::Error::new(
                            io::ErrorKind::Unsupported,
                            "non-contiguous-opacity-effect-group",
                        ));
                    }
                }
                Scope::Clip(clip) if bounds.left < bounds.right && bounds.top < bounds.bottom => {
                    let started = timing.then(std::time::Instant::now);
                    let coverage = clips.coverage(
                        std::slice::from_ref(clip),
                        width,
                        height,
                        scale,
                        layer.compositor_scroll,
                    )?;
                    if coverage
                        .stride
                        .checked_mul(coverage.rows.len())
                        .is_none_or(|len| len != coverage.pixels.len())
                    {
                        return Err(io::Error::other("invalid direct clip coverage extent"));
                    }
                    if let Some(started) = started {
                        prepared.clip_time += started.elapsed();
                    }
                    prepared.coverage[run] = Some(coverage);
                }
                Scope::Clip(_) => {}
            }
            active.push(ActiveScope {
                scope: scope.clone(),
                run,
                direct_dst_in: effect.direct_mask.is_some() && matches!(scope, Scope::DstIn(_)),
                direct_clip: direct_clip.is_some_and(|candidate| candidate.opening_run == run),
            });
        }
        if let Some(candidate) = direct_clip {
            if candidate.scope_index + 1 != active.len()
                || !active.last().is_some_and(|g| g.direct_clip)
            {
                return Err(io::Error::other("direct rounded clip target changed"));
            }
        }
        if effect.direct_mask.is_some() {
            let index = active
                .len()
                .checked_sub(2)
                .ok_or_else(|| io::Error::other("direct mask has no isolated content surface"))?;
            let expected = layer
                .properties
                .effect
                .parent
                .as_ref()
                .ok_or_else(|| io::Error::other("direct mask has no content parent"))?;
            if !active.last().is_some_and(|group| group.direct_dst_in)
                || !matches!(&active[index].scope, Scope::Effect(parent)
                    if parent.lifecycle.same_node(&expected.lifecycle))
            {
                return Err(io::Error::other(
                    "direct mask destination differs from its content effect",
                ));
            }
        }
        if visible.len() != layer.tiles.len() {
            return Err(io::Error::other("composition tile plan extent changed"));
        }
        let composition = replay
            .composition(layer.id)
            .ok_or_else(|| io::Error::other("paint layer has no composition"))?;
        for (tile, visible) in layer.tiles.iter().zip(visible) {
            let pixels = pixels
                .get(&tile.tile_id)
                .ok_or_else(|| io::Error::other("ready tile has no resident pixels"))?;
            if !pixels.matches(tile, composition) {
                return Err(io::Error::other(
                    "tile generation or raster backing changed without invalidation",
                ));
            }
            if !valid_tile_storage(pixels, tile.pixel_size)?
                || pixels.row_support.len() != tile.pixel_size.1 as usize
            {
                return Err(io::Error::other("invalid cached tile extent"));
            }
            tile_composition_geometry(tile, composition, width, height)?;
            if visible.is_some() && effect.direct_mask.is_none() {
                if direct_clip.is_some() {
                    if active
                        .last()
                        .and_then(|scope| prepared.coverage[scope.run].as_ref())
                        .is_some()
                        && tile.pixel_size.0 > TILE_SIZE
                    {
                        return Err(io::Error::other("invalid direct clip coverage extent"));
                    }
                } else if !active.is_empty() && composition.white_backing {
                    return Err(io::Error::other(
                        "masked/effected tile must have transparent backing",
                    ));
                }
            }
        }
    }
    while !active.is_empty() {
        close_scope(&mut active)?;
    }
    Ok(prepared)
}
