//! Conservative draw-quad occlusion within one continuous composition target.
//! Child of layer_raster: resources, sampling and effect execution stay there.
use super::*;
use layer_tile::FramePlan;
use layoutng_assembly::internal::paint_input::PaintBlendMode;

/// Device-space visible rectangles, in exactly the plan's layer/tile order.
/// Neither tile resources nor sampling origins/UVs are changed by this pass.
pub(super) fn visible_rects(
    plan: &FramePlan,
    effects: &[LayerEffectPlan],
    direct_clips: &[Option<layer_direct_clip::DirectClipRun>],
    replay: &LayerReplay<'_>,
    staged: &HashMap<TileId, CachedTile>,
    resident: &HashMap<TileId, CachedTile>,
    width: u32,
    height: u32,
) -> io::Result<Vec<Vec<Option<SurfaceBounds>>>> {
    if effects.len() != plan.layers.len() || direct_clips.len() != plan.layers.len() {
        return Err(io::Error::other("occlusion effect plan extent changed"));
    }
    let mut visible = Vec::with_capacity(plan.layers.len());
    for layer in &plan.layers {
        let composition = replay
            .composition(layer.id)
            .ok_or_else(|| io::Error::other("occlusion layer has no composition"))?;
        let rectangles = layer
            .tiles
            .iter()
            .map(|tile| {
                tile_composition_geometry(tile, composition, width, height)
                    .map(|geometry| geometry.map(|(bounds, _, _)| bounds))
            })
            .collect::<io::Result<Vec<_>>>()?;
        visible.push(rectangles);
    }

    let targets = draw_target_runs(
        effects,
        direct_clips,
        plan.layers
            .iter()
            .map(|layer| !layer.tiles.is_empty() || layer.properties.effect.is_mask),
    )?;
    // OcclusionTracker tracks actual render targets, not clip state. A direct
    // rounded clip is a SoftwareRenderer::SetClipRRect on the current target;
    // it does not discard opaque coverage accumulated in that target.
    let mut start = 0;
    while start < plan.layers.len() {
        if barrier(&effects[start], &plan.layers[start].properties.effect) {
            start += 1;
            continue;
        }
        let mut end = start + 1;
        while end < plan.layers.len()
            && !barrier(&effects[end], &plan.layers[end].properties.effect)
            && targets[end] == targets[start]
        {
            end += 1;
        }
        let mut opaque = EnclosedRegion::default();
        // cc traverses front to back to find what later-painted opaque quads
        // hide. Actual execution remains in the original back-to-front order.
        for index in (start..end).rev() {
            let layer = &plan.layers[index];
            let composition = replay
                .composition(layer.id)
                .ok_or_else(|| io::Error::other("occlusion layer has no composition"))?;
            let fractional = composition.translation.0.fract() != 0.0
                || composition.translation.1.fract() != 0.0;
            let mut neighbors =
                HashMap::with_capacity(if fractional { layer.tiles.len() } else { 0 });
            for tile in &layer.tiles {
                let pixels = checked_pixels(tile, composition, staged, resident)?;
                if fractional {
                    neighbors.insert(
                        (
                            device_position(tile.tile_rect.x * tile.raster_scale)?,
                            device_position(tile.tile_rect.y * tile.raster_scale)?,
                        ),
                        pixels,
                    );
                }
            }
            for (tile_index, tile) in layer.tiles.iter().enumerate().rev() {
                let Some(original) = visible[index][tile_index] else {
                    continue;
                };
                visible[index][tile_index] = subtract_visible(original, opaque.rect);
                // MarkOccludedBehindLayer refuses a nonempty mask_filter_info.
                // Rounded quads can be occluded, but never contribute their
                // enclosing rectangle as opaque coverage of the rounded corners.
                if direct_clips[index].is_some() {
                    continue;
                }
                // PendingLayer::MapRectKnownToBeOpaque carries source alpha
                // independently of transparent allocation padding. Restrict to
                // the actual sampled footprint and this quad's visible rect.
                if let Some(rect) = composition.opaque_bounds {
                    let left = original.left.max(i64::from(rect.x).max(0) as usize);
                    let top = original.top.max(i64::from(rect.y).max(0) as usize);
                    let right = original
                        .right
                        .min((i64::from(rect.x) + i64::from(rect.width)).max(0) as usize);
                    let bottom = original
                        .bottom
                        .min((i64::from(rect.y) + i64::from(rect.height)).max(0) as usize);
                    if left < right && top < bottom {
                        opaque.union(SurfaceBounds {
                            left,
                            top,
                            right,
                            bottom,
                        });
                    }
                }
                let pixels = checked_pixels(tile, composition, staged, resident)?;
                if !(pixels.white_backing || pixels.row_support.opaque) {
                    continue;
                }
                let origin = (
                    device_position(tile.tile_rect.x * tile.raster_scale)?,
                    device_position(tile.tile_rect.y * tile.raster_scale)?,
                );
                let horizontal = composition.translation.0.fract() != 0.0;
                let vertical = composition.translation.1.fract() != 0.0;
                let required = [
                    (
                        horizontal,
                        (origin.0 + i64::from(tile.pixel_size.0), origin.1),
                    ),
                    (
                        vertical,
                        (origin.0, origin.1 + i64::from(tile.pixel_size.1)),
                    ),
                    (
                        horizontal && vertical,
                        (
                            origin.0 + i64::from(tile.pixel_size.0),
                            origin.1 + i64::from(tile.pixel_size.1),
                        ),
                    ),
                ];
                // Same real neighbors as tile_edge_pixel. A missing neighbor
                // clamps this proven-opaque tile; an existing translucent one
                // forbids coverage. Ignore neighbors on an unsampled axis.
                if required.iter().all(|(used, key)| {
                    !used
                        || neighbors
                            .get(key)
                            .is_none_or(|next| next.white_backing || next.row_support.opaque)
                }) {
                    opaque.union(original);
                }
            }
        }
        start = end;
    }
    Ok(visible)
}

fn checked_pixels<'a>(
    tile: &TilePlacement,
    composition: LayerComposition,
    staged: &'a HashMap<TileId, CachedTile>,
    resident: &'a HashMap<TileId, CachedTile>,
) -> io::Result<&'a CachedTile> {
    let pixels = staged
        .get(&tile.tile_id)
        .or_else(|| resident.get(&tile.tile_id))
        .ok_or_else(|| io::Error::other("occlusion tile has no resident pixels"))?;
    if !pixels.matches(tile, composition) || !valid_tile_storage(pixels, tile.pixel_size)? {
        return Err(io::Error::other(
            "occlusion tile generation or extent changed",
        ));
    }
    Ok(pixels)
}

fn barrier(plan: &LayerEffectPlan, effect: &Arc<EffectPaintPropertyNode>) -> bool {
    // cc/trees/occlusion_tracker.cc::MarkOccludedBehindLayer permits only
    // occluding blend modes. Inspect the real chain too, including ancestors
    // which need no surface and therefore are absent from Scope.
    let mut node = Some(effect);
    while let Some(effect) = node {
        if effect.is_mask || effect.opacity == 0.0 || effect.blend_mode != PaintBlendMode::kNormal {
            return true;
        }
        node = effect.parent.as_ref();
    }
    plan.direct_mask.is_some()
        || plan.chain.iter().any(|scope| match scope {
            Scope::DstIn(_) => true,
            Scope::Effect(effect) => effect.opacity == 0.0,
            Scope::Clip(_) => false,
        })
}

fn draw_target_runs<'a>(
    effects: &'a [LayerEffectPlan],
    direct_clips: &[Option<layer_direct_clip::DirectClipRun>],
    drawn: impl Iterator<Item = bool>,
) -> io::Result<Vec<Option<usize>>> {
    let mut active: Vec<(&'a Scope, usize, bool)> = Vec::new();
    let mut targets = Vec::with_capacity(effects.len());
    for ((effect, direct), drawn) in effects.iter().zip(direct_clips).zip(drawn) {
        if drawn
            && !effect
                .chain
                .iter()
                .any(|scope| matches!(scope, Scope::Effect(effect) if effect.opacity == 0.0))
        {
            let common = active
                .iter()
                .zip(&effect.chain)
                .take_while(|((old, _, _), new)| old.same_node(new))
                .count();
            if active
                .iter()
                .zip(&effect.chain)
                .take(common)
                .any(|((old, _, _), new)| !old.same_values(new))
                || effect.chain.len() - common != effect.opened_runs.len()
            {
                return Err(io::Error::other("occlusion render target scope changed"));
            }
            active.truncate(common);
            for ((index, scope), &run) in effect
                .chain
                .iter()
                .enumerate()
                .skip(common)
                .zip(&effect.opened_runs)
            {
                let is_target = !direct
                    .is_some_and(|clip| clip.scope_index == index && clip.opening_run == run);
                active.push((scope, run, is_target));
            }
        }
        targets.push(
            active
                .iter()
                .rev()
                .find(|(_, _, target)| *target)
                .map(|(_, run, _)| *run),
        );
    }
    Ok(targets)
}

fn contains(a: SurfaceBounds, b: SurfaceBounds) -> bool {
    a.left <= b.left && a.top <= b.top && a.right >= b.right && a.bottom >= b.bottom
}

fn area(rect: SurfaceBounds) -> u128 {
    (rect.right - rect.left) as u128 * (rect.bottom - rect.top) as u128
}

/// cc/base/simple_enclosed_region.cc::Union: constant-sized, no false opaque
/// coverage. Expand only completely covered edges, then apply the official
/// area/overlap weighting. A bounding union around a hole is never allowed.
#[derive(Default)]
struct EnclosedRegion {
    rect: Option<SurfaceBounds>,
}
impl EnclosedRegion {
    fn union(&mut self, new: SurfaceBounds) {
        if new.left >= new.right || new.top >= new.bottom {
            return;
        }
        let Some(mut old) = self.rect else {
            self.rect = Some(new);
            return;
        };
        if contains(old, new) {
            return;
        }
        if contains(new, old) {
            self.rect = Some(new);
            return;
        }
        let mut adjusted = new;
        if new.top <= old.top && new.bottom >= old.bottom {
            if new.left < old.left && new.right >= old.left {
                old.left = new.left;
            }
            if new.right > old.right && new.left <= old.right {
                old.right = new.right;
            }
        } else if new.left <= old.left && new.right >= old.right {
            if new.top < old.top && new.bottom >= old.top {
                old.top = new.top;
            }
            if new.bottom > old.bottom && new.top <= old.bottom {
                old.bottom = new.bottom;
            }
        } else if old.top <= new.top && old.bottom >= new.bottom {
            if old.left < new.left && old.right >= new.left {
                adjusted.left = old.left;
            }
            if old.right > new.right && old.left <= new.right {
                adjusted.right = old.right;
            }
        } else if old.left <= new.left && old.right >= new.right {
            if old.top < new.top && old.bottom >= new.top {
                adjusted.top = old.top;
            }
            if old.bottom > new.bottom && old.top <= new.bottom {
                adjusted.bottom = old.bottom;
            }
        }
        let overlap_width = old
            .right
            .min(adjusted.right)
            .saturating_sub(old.left.max(adjusted.left));
        let overlap_height = old
            .bottom
            .min(adjusted.bottom)
            .saturating_sub(old.top.max(adjusted.top));
        let overlap = overlap_width as u128 * overlap_height as u128;
        self.rect = Some(if area(adjusted) * 2 > area(old) + overlap {
            adjusted
        } else {
            old
        });
    }
}

/// ui/gfx/geometry/rect.cc::Rect::Subtract, used by cc/trees/occlusion.cc.
/// Preserve the enclosing rectangle of ALL remaining visible pixels. Unlike
/// SimpleEnclosedRegion::Subtract, this must not pick just the largest piece.
fn subtract_visible(
    mut visible: SurfaceBounds,
    occluded: Option<SurfaceBounds>,
) -> Option<SurfaceBounds> {
    let Some(occluded) = occluded else {
        return Some(visible);
    };
    if visible.left >= occluded.right
        || visible.right <= occluded.left
        || visible.top >= occluded.bottom
        || visible.bottom <= occluded.top
    {
        return Some(visible);
    }
    if contains(occluded, visible) {
        return None;
    }
    if occluded.top <= visible.top && occluded.bottom >= visible.bottom {
        if occluded.left <= visible.left {
            visible.left = occluded.right;
        } else if occluded.right >= visible.right {
            visible.right = occluded.left;
        }
    } else if occluded.left <= visible.left && occluded.right >= visible.right {
        if occluded.top <= visible.top {
            visible.top = occluded.bottom;
        } else if occluded.bottom >= visible.bottom {
            visible.bottom = occluded.top;
        }
    }
    Some(visible)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_clip_keeps_target_but_real_surface_openings_remain_distinct() {
        let state = paint::paint_property_tree::PropertyTreeState::default();
        let mut clip = (*state.clip).clone();
        clip.lifecycle = Default::default();
        clip.id = 1;
        let clip = Scope::Clip(Arc::new(clip));
        let mut effect = (*state.effect).clone();
        effect.lifecycle = Default::default();
        effect.id = 1;
        effect.opacity = 0.5;
        let effect = Scope::Effect(Arc::new(effect));
        let layer = |chain, opened_runs| LayerEffectPlan {
            chain,
            opened_runs,
            direct_mask: None,
        };
        let effects = [
            layer(vec![], vec![]),
            layer(vec![clip.clone()], vec![0]),
            layer(vec![], vec![]),
            layer(vec![effect.clone()], vec![1]),
            layer(vec![effect.clone(), clip.clone()], vec![2]),
            layer(vec![], vec![]), // undrawn metadata leaves the live target intact
            layer(vec![effect.clone()], vec![]),
            layer(vec![], vec![]),
            layer(vec![effect], vec![3]),
            layer(vec![clip], vec![4]), // isolated clip really opens a new target
        ];
        let mut clips = vec![None; effects.len()];
        clips[1] = Some(layer_direct_clip::DirectClipRun {
            scope_index: 0,
            opening_run: 0,
        });
        clips[4] = Some(layer_direct_clip::DirectClipRun {
            scope_index: 1,
            opening_run: 2,
        });
        let targets =
            draw_target_runs(&effects, &clips, (0..effects.len()).map(|index| index != 5)).unwrap();
        assert_eq!(
            targets,
            [
                None,
                None,
                None,
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                None,
                Some(3),
                Some(4)
            ]
        );
    }

    #[test]
    fn enclosed_union_and_visible_subtraction_do_not_invent_or_drop_pixels() {
        let rect = |left, top, right, bottom| SurfaceBounds {
            left,
            top,
            right,
            bottom,
        };
        let inside =
            |r: SurfaceBounds, x, y| x >= r.left && x < r.right && y >= r.top && y < r.bottom;
        // Adjacent edges may join; diagonal and crossing rectangles leave
        // holes. Central/full-height/full-width occluders split visible work.
        let pairs = [
            (rect(0, 0, 4, 4), rect(4, 0, 8, 4)),
            (rect(0, 0, 4, 4), rect(3, 3, 8, 8)),
            (rect(0, 2, 8, 4), rect(3, 0, 5, 8)),
            (rect(0, 0, 8, 8), rect(2, 2, 6, 6)),
            (rect(0, 0, 8, 8), rect(2, 0, 6, 8)),
            (rect(0, 0, 8, 8), rect(0, 2, 8, 6)),
            (rect(0, 0, 8, 8), rect(0, 0, 4, 8)),
            (rect(2, 2, 6, 6), rect(0, 0, 8, 8)),
        ];
        for (a, b) in pairs {
            let mut region = EnclosedRegion::default();
            region.union(a);
            region.union(b);
            let remainder = subtract_visible(a, Some(b));
            for y in 0..8 {
                for x in 0..8 {
                    if region.rect.is_some_and(|r| inside(r, x, y)) {
                        assert!(inside(a, x, y) || inside(b, x, y), "union invented opacity");
                    }
                    if inside(a, x, y) && !inside(b, x, y) {
                        assert!(
                            remainder.is_some_and(|r| inside(r, x, y)),
                            "subtraction dropped visible pixels"
                        );
                    }
                }
            }
        }
        assert!(subtract_visible(rect(2, 2, 6, 6), Some(rect(0, 0, 8, 8))).is_none());
        let trimmed = subtract_visible(rect(0, 0, 8, 8), Some(rect(0, 0, 4, 8))).unwrap();
        assert_eq!(
            (trimmed.left, trimmed.top, trimmed.right, trimmed.bottom),
            (4, 0, 8, 8)
        );
    }
}
