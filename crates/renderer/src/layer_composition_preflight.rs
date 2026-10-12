//! Validate a root render pass before beginning writes to the borrowed output.
//! The subsequent draw consumes these immutable plans and resident tile pixels.
use super::*;

struct ActiveScope {
    scope: Scope,
    direct_dst_in: bool,
    direct_clip: bool,
    run: usize,
}

fn close_scope(
    active: &mut Vec<ActiveScope>,
    root_owner: Option<&Arc<EffectPaintPropertyNode>>,
) -> io::Result<ActiveScope> {
    let group = active.pop().expect("active preflight scope");
    if !group.direct_dst_in && !group.direct_clip {
        if let Scope::DstIn(effect) = &group.scope {
            let expected = effect
                .parent
                .as_ref()
                .ok_or_else(|| io::Error::other("mask effect has no content parent"))?;
            let root_matches =
                root_owner.is_some_and(|owner| owner.lifecycle.same_node(&expected.lifecycle));
            if !root_matches
                && !active.last().is_some_and(|parent| {
                    matches!(&parent.scope, Scope::Effect(actual)
                    if actual.lifecycle.same_node(&expected.lifecycle))
                })
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
    /// Last layer index contributing to each allocation run. Renderer uses
    /// this to suspend a non-contiguous effect surface without restoring it.
    pub run_last_use: Vec<usize>,
    pub clip_time: std::time::Duration,
}

pub(super) fn prepare(
    plan: &CompositionPlan,
    effects: &[LayerEffectPlan],
    bounds: &[Option<SurfaceBounds>],
    direct_clips: &[Option<layer_direct_clip::DirectClipRun>],
    visible: &[Vec<Option<SurfaceBounds>>],
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
        run_last_use: vec![0; bounds.len()],
        clip_time: std::time::Duration::ZERO,
    };
    let mut active: Vec<ActiveScope> = Vec::new();
    let mut suspended: HashMap<usize, ActiveScope> = HashMap::new();
    for (layer_index, (((layer, effect), direct_clip), visible)) in plan
        .layers
        .iter()
        .zip(effects)
        .zip(direct_clips)
        .zip(visible)
        .enumerate()
    {
        if !layer.has_drawable_content()
            && !layer.properties.effect.is_mask
            && !effect.has_backdrop()
        {
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
        if effect.scope_output_clips.len() != effect.chain.len() {
            return Err(io::Error::other("effect output clip plan extent changed"));
        }
        while active.len() > common {
            let group = close_scope(&mut active, plan.owner_effect.as_ref())?;
            if suspended.insert(group.run, group).is_some() {
                return Err(io::Error::other(
                    "composition run suspended more than once without resume",
                ));
            }
        }
        for ((scope, output_clips), &run) in effect
            .chain
            .iter()
            .zip(&effect.scope_output_clips)
            .skip(common)
            .zip(&effect.opened_runs)
        {
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
                Scope::Effect(_) | Scope::DstIn(_) => {}
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
            if !output_clips.is_empty() && bounds.left < bounds.right && bounds.top < bounds.bottom
            {
                if !matches!(scope, Scope::Effect(_)) {
                    return Err(io::Error::other(
                        "output clip coverage has no owning effect surface",
                    ));
                }
                let started = timing.then(std::time::Instant::now);
                let coverage =
                    clips.coverage(output_clips, width, height, scale, layer.compositor_scroll)?;
                if coverage
                    .stride
                    .checked_mul(coverage.rows.len())
                    .is_none_or(|len| len != coverage.pixels.len())
                {
                    return Err(io::Error::other("invalid effect output clip extent"));
                }
                if let Some(started) = started {
                    prepared.clip_time += started.elapsed();
                }
                prepared.coverage[run] = Some(coverage);
            }
            let direct_dst_in = effect.direct_mask.is_some() && matches!(scope, Scope::DstIn(_));
            let direct_clip = direct_clip.is_some_and(|candidate| candidate.opening_run == run);
            if let Some(resumed) = suspended.remove(&run) {
                if !resumed.scope.same_node(scope)
                    || !resumed.scope.same_values(scope)
                    || resumed.direct_dst_in != direct_dst_in
                    || resumed.direct_clip != direct_clip
                {
                    return Err(io::Error::other("resumed composition scope changed values"));
                }
                active.push(resumed);
            } else {
                active.push(ActiveScope {
                    scope: scope.clone(),
                    run,
                    direct_dst_in,
                    direct_clip,
                });
            }
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
        for scope in &active {
            prepared.run_last_use[scope.run] = layer_index;
        }
        let composition = layer.composition;
        for (tile, visible) in layer.tiles.iter().zip(visible) {
            let pixels = pixels
                .get(&tile.tile_id)
                .ok_or_else(|| io::Error::other("ready tile has no resident pixels"))?;
            if !pixels.matches(tile, composition.white_backing) {
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
        let group = close_scope(&mut active, plan.owner_effect.as_ref())?;
        suspended.insert(group.run, group);
    }
    Ok(prepared)
}
