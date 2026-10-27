//! Cost-bearing layer merge decisions. Paint and cc object access stays in the
//! compositor facade; this module owns whether two pending layers may become
//! one and what the merged layer geometry must be.

use compositor_tree::geometry::{Rect, RectF, Size, SizeF, ToEnclosingRect, Vector2dF};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompositingType {
    kScrollHitTestLayer,
    kForeignLayer,
    kScrollbarLayer,
    kOverlap,
    #[default]
    kOther,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MergeResult {
    pub merged: bool,
    pub scroll_range_dependent: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LcdTextPreference {
    kStronglyPreferred,
    kWeaklyPreferred,
    kIgnored,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingUpcastResult<S> {
    pub upcasted_state: S,
    pub scroll_range_dependent: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PendingMappedVisualRect {
    pub rect: RectF,
    pub is_tight: bool,
}

pub const fn RequiresOwnLayer(compositing_type: CompositingType) -> bool {
    !matches!(
        compositing_type,
        CompositingType::kOverlap | CompositingType::kOther
    )
}

/// Read-only view of a facade-owned PendingLayer. It contains only values
/// needed by the merge decision; PaintArtifact and cc::Layer ownership never
/// cross this package boundary.
pub struct PendingLayerMergeInput<'a, S, T, D, O> {
    pub property_tree_state: &'a S,
    pub non_composited_scroll_translations: &'a [T],
    pub bounds: RectF,
    pub rect_known_to_be_opaque: RectF,
    pub solid_color_chunk_index: Option<usize>,
    pub solid_color_alpha: Option<f32>,
    pub chunk_count: usize,
    pub compositing_type: CompositingType,
    pub hit_test_opaqueness: O,
    pub canvas_child_id: D,
    pub has_text: bool,
    pub draws_content: bool,
    pub text_known_to_be_on_opaque_background: bool,
    pub has_decomposited_blend_mode: bool,
}

impl<S, T, D, O> PendingLayerMergeInput<'_, S, T, D, O> {
    fn is_solid_color(&self) -> bool {
        self.solid_color_chunk_index.is_some()
    }
}

pub struct PendingLayerMergeOutput<S, O> {
    pub bounds: RectF,
    pub property_tree_state: S,
    pub rect_known_to_be_opaque: RectF,
    pub text_known_to_be_on_opaque_background: bool,
    pub solid_color_chunk_index: Option<usize>,
    pub hit_test_opaqueness: O,
    pub scroll_range_dependent: bool,
}

pub struct PendingLayerUpcastOutput<S> {
    pub property_tree_state: S,
    pub bounds: RectF,
    pub rect_known_to_be_opaque: RectF,
    pub has_decomposited_blend_mode: bool,
}

/// Paint-specific geometry/property queries required by the package-owned
/// merge policy. The facade implements this against resident Paint nodes.
pub trait PendingLayerMergeGeometry<S, T, O> {
    fn SameEffectNode(&self, first: &S, second: &S) -> bool;
    fn CanUpcastWith(
        &self,
        first: &S,
        second: &S,
        merge_fixed_layers: bool,
        merge_sticky_layers: bool,
        is_composited_scroll: &mut dyn FnMut(&T) -> bool,
    ) -> Option<PendingUpcastResult<S>>;
    fn NearestScrollTranslationNode(&self, state: &S) -> T;
    fn SameNearestDirectlyCompositedAncestor(&self, first: &S, second: &S) -> bool;
    fn MightOverlapForCompositing(
        &self,
        first_bounds: RectF,
        first_state: &S,
        second_bounds: RectF,
        second_state: &S,
    ) -> bool;
    fn VisibilityLimit(&self, state: &S) -> Option<RectF>;
    fn MapLocalToAncestorVisualRect(
        &self,
        source: &S,
        destination: &S,
        rect: RectF,
    ) -> PendingMappedVisualRect;
    fn MaximumCoveredRect(&self, first: RectF, second: RectF) -> RectF;
    fn UnionRects(&self, first: RectF, second: RectF) -> RectF;
    fn ToRoundedRect(&self, rect: RectF) -> Rect;
    fn HitTestOpaquenessIsOpaque(&self, value: O) -> bool;
    fn MixedHitTestOpaqueness(&self) -> O;
    fn UnionHitTestOpaqueness(&self, first_rect: Rect, first: O, second_rect: Rect, second: O)
        -> O;
    fn MergeFixedLayersEnabled(&self) -> bool;
    fn MergeStickyLayersEnabled(&self) -> bool;
}

pub fn MightOverlapPendingLayers<S, T, D: Eq, O>(
    first: &PendingLayerMergeInput<'_, S, T, D, O>,
    second: &PendingLayerMergeInput<'_, S, T, D, O>,
    geometry: &dyn PendingLayerMergeGeometry<S, T, O>,
) -> bool {
    first.canvas_child_id == second.canvas_child_id
        && geometry.MightOverlapForCompositing(
            first.bounds,
            first.property_tree_state,
            second.bounds,
            second.property_tree_state,
        )
}

pub fn ComputePendingLayerUpcast<S: Clone, T: Clone + Eq, D, O>(
    layer: &PendingLayerMergeInput<'_, S, T, D, O>,
    new_state: &S,
    effect_has_non_source_over_blend: bool,
    geometry: &dyn PendingLayerMergeGeometry<S, T, O>,
) -> PendingLayerUpcastOutput<S> {
    let mapped_bounds =
        geometry.MapLocalToAncestorVisualRect(layer.property_tree_state, new_state, layer.bounds);
    PendingLayerUpcastOutput {
        property_tree_state: new_state.clone(),
        bounds: mapped_bounds.rect,
        rect_known_to_be_opaque: map_rect_known_to_be_opaque(
            layer,
            new_state,
            mapped_bounds,
            geometry,
        ),
        has_decomposited_blend_mode: layer.has_decomposited_blend_mode
            || effect_has_non_source_over_blend,
    }
}

pub fn CanUpcastPendingLayers<S: Clone, T: Clone + Eq, D, O>(
    home: &PendingLayerMergeInput<'_, S, T, D, O>,
    guest: &PendingLayerMergeInput<'_, S, T, D, O>,
    guest_state: &S,
    merge_fixed_layers: bool,
    merge_sticky_layers: bool,
    is_composited_scroll: &mut dyn FnMut(&T) -> bool,
    geometry: &dyn PendingLayerMergeGeometry<S, T, O>,
) -> Option<PendingUpcastResult<S>> {
    if RequiresOwnLayer(home.compositing_type) || RequiresOwnLayer(guest.compositing_type) {
        return None;
    }
    if !geometry.SameEffectNode(home.property_tree_state, guest_state) {
        return None;
    }
    let result = geometry.CanUpcastWith(
        home.property_tree_state,
        guest_state,
        merge_fixed_layers,
        merge_sticky_layers,
        is_composited_scroll,
    )?;
    let home_scroll = geometry.NearestScrollTranslationNode(home.property_tree_state);
    let guest_scroll = geometry.NearestScrollTranslationNode(guest_state);
    if home_scroll == guest_scroll {
        return Some(result);
    }
    let lca_scroll = geometry.NearestScrollTranslationNode(&result.upcasted_state);
    if (guest_scroll == lca_scroll
        || home
            .non_composited_scroll_translations
            .contains(&guest_scroll))
        && (home_scroll == lca_scroll
            || guest
                .non_composited_scroll_translations
                .contains(&home_scroll))
    {
        return Some(result);
    }
    None
}

pub fn ComputePendingLayerMerge<S: Clone + PartialEq, T: Clone + Eq, D: Copy + Eq, O: Copy>(
    home: &PendingLayerMergeInput<'_, S, T, D, O>,
    guest: &PendingLayerMergeInput<'_, S, T, D, O>,
    invalid_canvas_child_id: D,
    lcd_text_preference: LcdTextPreference,
    device_pixel_ratio: f32,
    is_composited_scroll: &mut dyn FnMut(&T) -> bool,
    geometry: &dyn PendingLayerMergeGeometry<S, T, O>,
) -> Option<PendingLayerMergeOutput<S, O>> {
    const K_MERGE_SPARSITY_AREA_TOLERANCE: f32 = 10_000.0;
    if home.canvas_child_id != guest.canvas_child_id {
        return None;
    }
    let mut force_merge = home.canvas_child_id != invalid_canvas_child_id;
    let upcast = CanUpcastPendingLayers(
        home,
        guest,
        guest.property_tree_state,
        geometry.MergeFixedLayersEnabled(),
        geometry.MergeStickyLayersEnabled(),
        is_composited_scroll,
        geometry,
    );
    let PendingUpcastResult {
        upcasted_state: merged_state,
        scroll_range_dependent,
    } = match upcast {
        Some(result) => result,
        None if force_merge => PendingUpcastResult {
            upcasted_state: home.property_tree_state.clone(),
            scroll_range_dependent: false,
        },
        None => return None,
    };
    let merged_visibility_limit = geometry.VisibilityLimit(&merged_state);
    if !guest.has_decomposited_blend_mode
        && merged_visibility_limit == Some(home.bounds)
        && merged_state == *home.property_tree_state
        && home.rect_known_to_be_opaque == home.bounds
    {
        return Some(PendingLayerMergeOutput {
            bounds: home.bounds,
            property_tree_state: merged_state,
            rect_known_to_be_opaque: home.bounds,
            text_known_to_be_on_opaque_background: true,
            solid_color_chunk_index: if guest.draws_content {
                None
            } else {
                home.solid_color_chunk_index
            },
            hit_test_opaqueness: geometry.UnionHitTestOpaqueness(
                geometry.ToRoundedRect(home.bounds),
                home.hit_test_opaqueness,
                geometry.ToRoundedRect(guest.bounds),
                guest.hit_test_opaqueness,
            ),
            scroll_range_dependent,
        });
    }

    let mut new_home_bounds =
        geometry.MapLocalToAncestorVisualRect(home.property_tree_state, &merged_state, home.bounds);
    let mut new_guest_bounds = geometry.MapLocalToAncestorVisualRect(
        guest.property_tree_state,
        &merged_state,
        guest.bounds,
    );
    if let Some(limit) = merged_visibility_limit {
        intersect_rectf(&mut new_home_bounds.rect, limit);
        intersect_rectf(&mut new_guest_bounds.rect, limit);
    }
    let mut merged_bounds = geometry.UnionRects(new_home_bounds.rect, new_guest_bounds.rect);
    if guest.has_decomposited_blend_mode {
        force_merge = true;
    }

    let mut merged_opaque = RectF::default();
    let mut merged_text_on_opaque = false;
    let mut merged_solid_color_chunk_index = None;
    if !force_merge {
        let sum_area = new_home_bounds.rect.width() * new_home_bounds.rect.height()
            + new_guest_bounds.rect.width() * new_guest_bounds.rect.height();
        let tolerance = K_MERGE_SPARSITY_AREA_TOLERANCE * device_pixel_ratio * device_pixel_ratio;
        if merged_bounds.width() * merged_bounds.height() - sum_area > tolerance {
            return None;
        }
        let home_opaque =
            map_rect_known_to_be_opaque(home, &merged_state, new_home_bounds, geometry);
        let guest_opaque =
            map_rect_known_to_be_opaque(guest, &merged_state, new_guest_bounds, geometry);
        merged_opaque = geometry.MaximumCoveredRect(home_opaque, guest_opaque);
        merged_text_on_opaque = home.text_known_to_be_on_opaque_background;
        if home.text_known_to_be_on_opaque_background != guest.text_known_to_be_on_opaque_background
        {
            if !home.text_known_to_be_on_opaque_background {
                if merged_opaque.Contains(new_home_bounds.rect) {
                    merged_text_on_opaque = true;
                }
            } else if !guest.text_known_to_be_on_opaque_background
                && !merged_opaque.Contains(new_guest_bounds.rect)
            {
                merged_text_on_opaque = false;
            }
        }
        if lcd_text_preference == LcdTextPreference::kStronglyPreferred && !merged_text_on_opaque {
            if home.has_text && home.text_known_to_be_on_opaque_background {
                return None;
            }
            if guest.has_text && guest.text_known_to_be_on_opaque_background {
                return None;
            }
        }
        if home.is_solid_color()
            && new_home_bounds.is_tight
            && !guest.draws_content
            && new_home_bounds.rect == merged_bounds
        {
            merged_solid_color_chunk_index = home.solid_color_chunk_index;
        } else if guest.is_solid_color()
            && new_guest_bounds.is_tight
            && new_guest_bounds.rect == merged_bounds
            && (!home.draws_content || guest.solid_color_alpha == Some(1.0))
        {
            merged_solid_color_chunk_index = guest
                .solid_color_chunk_index
                .map(|index| home.chunk_count + index);
        }
        if !geometry.SameNearestDirectlyCompositedAncestor(
            home.property_tree_state,
            guest.property_tree_state,
        ) {
            debug_assert!(
                geometry.MergeFixedLayersEnabled() || geometry.MergeStickyLayersEnabled()
            );
            if (home.is_solid_color() || guest.is_solid_color())
                && merged_solid_color_chunk_index.is_none()
            {
                return None;
            }
        }
    }

    let mut home_hit_test_opaqueness = home.hit_test_opaqueness;
    if geometry.HitTestOpaquenessIsOpaque(home_hit_test_opaqueness) && !new_home_bounds.is_tight {
        home_hit_test_opaqueness = geometry.MixedHitTestOpaqueness();
    }
    let mut guest_hit_test_opaqueness = guest.hit_test_opaqueness;
    if geometry.HitTestOpaquenessIsOpaque(guest_hit_test_opaqueness) && !new_guest_bounds.is_tight {
        guest_hit_test_opaqueness = geometry.MixedHitTestOpaqueness();
    }
    let merged_hit_test_opaqueness = geometry.UnionHitTestOpaqueness(
        geometry.ToRoundedRect(new_home_bounds.rect),
        home_hit_test_opaqueness,
        geometry.ToRoundedRect(new_guest_bounds.rect),
        guest_hit_test_opaqueness,
    );
    PreserveNearIntegralBounds(&mut merged_bounds);
    PreserveNearIntegralBounds(&mut merged_opaque);
    Some(PendingLayerMergeOutput {
        bounds: merged_bounds,
        property_tree_state: merged_state,
        rect_known_to_be_opaque: merged_opaque,
        text_known_to_be_on_opaque_background: merged_text_on_opaque,
        solid_color_chunk_index: merged_solid_color_chunk_index,
        hit_test_opaqueness: merged_hit_test_opaqueness,
        scroll_range_dependent,
    })
}

fn map_rect_known_to_be_opaque<S: Clone, T: Clone + Eq, D, O>(
    layer: &PendingLayerMergeInput<'_, S, T, D, O>,
    new_state: &S,
    mapped_layer_bounds: PendingMappedVisualRect,
    geometry: &dyn PendingLayerMergeGeometry<S, T, O>,
) -> RectF {
    if !mapped_layer_bounds.is_tight || layer.rect_known_to_be_opaque.IsEmpty() {
        return RectF::default();
    }
    if layer.rect_known_to_be_opaque == layer.bounds {
        return mapped_layer_bounds.rect;
    }
    let mut mapped_opaque = geometry.MapLocalToAncestorVisualRect(
        layer.property_tree_state,
        new_state,
        layer.rect_known_to_be_opaque,
    );
    intersect_rectf(&mut mapped_opaque.rect, mapped_layer_bounds.rect);
    debug_assert!(mapped_opaque.is_tight);
    mapped_opaque.rect
}

pub fn PreserveNearIntegralBounds(bounds: &mut RectF) {
    const K_TOLERANCE: f32 = 1e-3;
    let left = bounds.x();
    let top = bounds.y();
    let right = bounds.right();
    let bottom = bounds.bottom();
    if (left.round() - left).abs() <= K_TOLERANCE
        && (top.round() - top).abs() <= K_TOLERANCE
        && (right.round() - right).abs() <= K_TOLERANCE
        && (bottom.round() - bottom).abs() <= K_TOLERANCE
    {
        *bounds = RectF::new(
            left.round(),
            top.round(),
            right.round() - left.round(),
            bottom.round() - top.round(),
        );
    }
}

pub fn PendingLayerBounds(
    bounds: RectF,
    rect_known_to_be_opaque: RectF,
    is_solid_color: bool,
) -> (Vector2dF, Size) {
    let ceiled_size = Size::new(bounds.width().ceil() as i32, bounds.height().ceil() as i32);
    if is_solid_color {
        return (Vector2dF::new(bounds.x(), bounds.y()), ceiled_size);
    }

    let bounds_with_ceiled_size = RectF::FromOriginSize(
        bounds.origin(),
        SizeF::new(ceiled_size.width() as f32, ceiled_size.height() as f32),
    );
    let enclosing_bounds = ToEnclosingRect(bounds);
    if rect_known_to_be_opaque.Contains(bounds_with_ceiled_size)
        && !rect_known_to_be_opaque.Contains(RectF::from(enclosing_bounds))
    {
        return (Vector2dF::new(bounds.x(), bounds.y()), ceiled_size);
    }

    (
        Vector2dF::new(enclosing_bounds.x as f32, enclosing_bounds.y as f32),
        enclosing_bounds.size(),
    )
}

fn intersect_rectf(rect: &mut RectF, other: RectF) {
    let left = rect.x().max(other.x());
    let top = rect.y().max(other.y());
    let right = rect.right().min(other.right());
    let bottom = rect.bottom().min(other.bottom());
    *rect = if left >= right || top >= bottom {
        RectF::default()
    } else {
        RectF::new(left, top, right - left, bottom - top)
    };
}
