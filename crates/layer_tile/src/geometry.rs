use crate::{CompositorScrollOffset, UnsupportedReason};
use layoutng_assembly::internal::layout_input::TransformMatrix;
use layoutng_assembly::internal::paint_input::{PaintBlendMode, PaintCornerRadii, PaintFilterType};
use paint::paint_engine::{PaintRect, RasterEffectOutset};
use paint::paint_property_tree::{
    ClipPaintPropertyNode, PropertyTreeState, TransformPaintPropertyNode,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn transform_diagnostic(first: &Arc<TransformPaintPropertyNode>, reason: &str, rejected: u64) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_none()
        || REPORTED.swap(true, Ordering::Relaxed)
    {
        return;
    }
    eprintln!(
        "[layer_tile transform unsupported] reason={reason} rejected={rejected} leaf={}",
        first.id
    );
    let mut current = Some(first);
    while let Some(node) = current {
        eprintln!("[layer_tile transform state] node={} parent={:?} matrix={:?} origin={:?} scroll={:?} reasons={:?}",
            node.id, node.parent.as_ref().map(|parent| parent.id), node.matrix.values,
            node.origin, node.scroll.as_ref().map(|scroll| scroll.id), node.direct_compositing_reasons);
        current = node.parent.as_ref();
    }
}

pub(crate) fn finite(rect: PaintRect) -> bool {
    [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        rect.x + rect.width,
        rect.y + rect.height,
    ]
    .iter()
    .all(|v| v.is_finite())
        && rect.width >= 0.0
        && rect.height >= 0.0
}

pub(crate) fn intersects(a: PaintRect, b: PaintRect) -> bool {
    !a.is_empty()
        && !b.is_empty()
        && a.x < b.x + b.width
        && b.x < a.x + a.width
        && a.y < b.y + b.height
        && b.y < a.y + a.height
}

pub(crate) fn intersection(a: PaintRect, b: PaintRect) -> PaintRect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    PaintRect {
        x,
        y,
        width: ((a.x + a.width).min(b.x + b.width) - x).max(0.0),
        height: ((a.y + a.height).min(b.y + b.height) - y).max(0.0),
    }
}

pub(crate) fn shifted(rect: PaintRect, delta: (f64, f64)) -> PaintRect {
    PaintRect {
        x: rect.x + delta.0,
        y: rect.y + delta.1,
        ..rect
    }
}

pub(crate) fn outset(rect: PaintRect, effect: RasterEffectOutset) -> PaintRect {
    if rect.is_empty() {
        return rect;
    }
    let extent = match effect {
        RasterEffectOutset::kNone => 0.0,
        RasterEffectOutset::kHalfPixel => 0.5,
        RasterEffectOutset::kWholePixel => 1.0,
    };
    PaintRect {
        x: rect.x - extent,
        y: rect.y - extent,
        width: rect.width + 2.0 * extent,
        height: rect.height + 2.0 * extent,
    }
}

fn translation(node: &Arc<TransformPaintPropertyNode>) -> Result<(f64, f64), UnsupportedReason> {
    let first = node;
    let mut delta = (0.0, 0.0);
    let mut current = Some(node);
    let identity = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    while let Some(node) = current {
        for (index, value) in node.matrix.values.iter().enumerate() {
            if !value.is_finite() || (index != 12 && index != 13 && *value != identity[index]) {
                transform_diagnostic(first, "non-translation-raster-anchor", node.id);
                return Err(UnsupportedReason::NonTranslationTransform);
            }
        }
        delta.0 += node.matrix.values[12];
        delta.1 += node.matrix.values[13];
        current = node.parent.as_ref();
    }
    if !delta.0.is_finite() || !delta.1.is_finite() {
        transform_diagnostic(first, "nonfinite-raster-anchor-placement", first.id);
        return Err(UnsupportedReason::NonTranslationTransform);
    }
    Ok(delta)
}

fn axis_aligned(matrix: &TransformMatrix) -> bool {
    let identity = TransformMatrix::default();
    matrix.values.iter().enumerate().all(|(index, value)| {
        value.is_finite() && (matches!(index, 0 | 5 | 12 | 13) || *value == identity.values[index])
    })
}

fn affine_2d(matrix: &TransformMatrix) -> bool {
    let identity = TransformMatrix::default();
    matrix.values.iter().enumerate().all(|(index, value)| {
        value.is_finite()
            && (matches!(index, 0 | 1 | 4 | 5 | 12 | 13) || *value == identity.values[index])
    })
}

/// Compositor screen/device-space mapping, separate from logical paint/raster
/// mapping. cc::TransformTree::UpdateSnapping rounds only scroll nodes, before
/// descendants inherit their transform; an ordinary CSS translation retains
/// its own fractional phase. Active transform animation reasons match Blink's
/// TransformPaintPropertyNode::HasActiveTransformAnimation.
pub fn compositor_transform(
    space: &Arc<TransformPaintPropertyNode>,
    scale: f64,
) -> Result<TransformMatrix, UnsupportedReason> {
    compositor_transform_with_scroll(space, scale, None)
}

pub fn compositor_transform_with_scroll(
    space: &Arc<TransformPaintPropertyNode>,
    scale: f64,
    scroll_override: Option<CompositorScrollOffset>,
) -> Result<TransformMatrix, UnsupportedReason> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(UnsupportedReason::NonTranslationTransform);
    }
    let mut ancestors = Vec::new();
    let mut current = Some(space);
    while let Some(node) = current {
        let m = &node.matrix.values;
        let pure_translation = m[0] == 1.0 && m[5] == 1.0 && m[1] == 0.0 && m[4] == 0.0;
        // A pivot has no effect on translation. General affine origins are
        // still required to be folded by the existing source builder.
        if !affine_2d(&node.matrix) || (!pure_translation && node.origin != [0.0; 3]) {
            return Err(UnsupportedReason::NonTranslationTransform);
        }
        ancestors.push(node);
        current = node.parent.as_ref();
    }
    let mut result = TransformMatrix::default();
    result.values[0] = scale;
    result.values[5] = scale;
    let mut potentially_animated = false;
    let root_scroll = PropertyTreeState::default().transform.scroll.clone();
    for node in ancestors.into_iter().rev() {
        potentially_animated |= node.direct_compositing_reasons.iter().any(|reason| {
            matches!(
                *reason,
                "ActiveTransformAnimation"
                    | "ActiveScaleAnimation"
                    | "ActiveRotateAnimation"
                    | "ActiveTranslateAnimation"
            )
        });
        let p = result.values;
        let mut m = node.matrix.values;
        if scroll_override.is_some_and(|override_| {
            node.scroll
                .as_ref()
                .is_some_and(|scroll| scroll.id == override_.scroll_node_id)
        }) {
            m[13] += scroll_override.unwrap().translation_y;
        }
        result.values[0] = p[0] * m[0] + p[4] * m[1];
        result.values[1] = p[1] * m[0] + p[5] * m[1];
        result.values[4] = p[0] * m[4] + p[4] * m[5];
        result.values[5] = p[1] * m[4] + p[5] * m[5];
        result.values[12] = p[0] * m[12] + p[4] * m[13] + p[12];
        result.values[13] = p[1] * m[12] + p[5] * m[13] + p[13];
        let to_screen = &mut result.values;
        // The shared root scroll is a sentinel, not a scroll translation.
        // Scale/translation, invertibility and animation gates mirror cc.
        let real_scroll = node.scroll.as_ref().is_some_and(|scroll| {
            root_scroll
                .as_ref()
                .is_none_or(|root| !scroll.lifecycle.same_node(&root.lifecycle))
        });
        if real_scroll
            && !potentially_animated
            && to_screen[1] == 0.0
            && to_screen[4] == 0.0
            && to_screen[0] != 0.0
            && to_screen[5] != 0.0
        {
            to_screen[12] = to_screen[12].round();
            to_screen[13] = to_screen[13].round();
        }
    }
    if !result.values.iter().all(|value| value.is_finite()) {
        return Err(UnsupportedReason::NonTranslationTransform);
    }
    Ok(result)
}

#[cfg(test)]
mod scroll_snapping_test {
    use super::*;
    use paint::paint_property_tree::{EffectPaintPropertyNode, ScrollPaintPropertyNode};

    #[test]
    fn scroll_snapping_preserves_css_phase_and_logical_geometry_and_maps_clips() {
        let root = PropertyTreeState::default();
        let scroll = Arc::new(ScrollPaintPropertyNode {
            lifecycle: Default::default(),
            id: 1,
            parent: root.transform.scroll.clone(),
            overflow_clip: None,
            container_rect: PaintRect::default(),
            contents_rect: PaintRect::default(),
            user_scrollable_horizontal: false,
            user_scrollable_vertical: true,
        });
        let node = |id, parent, y, scroll, reasons| {
            let mut matrix = TransformMatrix::default();
            matrix.values[13] = y;
            Arc::new(TransformPaintPropertyNode {
                lifecycle: Default::default(),
                id,
                parent: Some(parent),
                matrix,
                origin: [0.0; 3],
                scroll,
                direct_compositing_reasons: reasons,
            })
        };
        let scrolling = node(
            2,
            root.transform.clone(),
            -0.625,
            Some(scroll.clone()),
            vec![],
        );
        let child = node(
            3,
            scrolling.clone(),
            0.125,
            None,
            vec!["WillChangeTransform"],
        );
        let ordinary = node(
            4,
            root.transform.clone(),
            0.125,
            None,
            vec!["WillChangeTransform"],
        );
        assert_eq!(
            compositor_transform(&scrolling, 2.0).unwrap().values[13],
            -1.0
        );
        assert_eq!(compositor_transform(&child, 2.0).unwrap().values[13], -0.75);
        assert_eq!(
            compositor_transform(&ordinary, 2.0).unwrap().values[13],
            0.25
        );
        assert_eq!(
            compositor_transform(&scrolling, 1.3).unwrap().values[13],
            -1.0
        );
        let with_origin = Arc::new(TransformPaintPropertyNode {
            origin: [25.0, 40.0, 0.0],
            ..(*scrolling).clone()
        });
        assert_eq!(
            compositor_transform(&with_origin, 2.0).unwrap().values[13],
            -1.0
        );
        let animated = node(
            5,
            root.transform.clone(),
            0.0,
            None,
            vec!["ActiveTransformAnimation"],
        );
        let animated_scroll = node(6, animated, -0.625, Some(scroll.clone()), vec![]);
        assert_eq!(
            compositor_transform(&animated_scroll, 2.0).unwrap().values[13],
            -1.25
        );
        let nested = node(7, scrolling.clone(), -0.625, Some(scroll), vec![]);
        assert_eq!(compositor_transform(&nested, 2.0).unwrap().values[13], -2.0);
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let clip = Arc::new(ClipPaintPropertyNode {
            lifecycle: Default::default(),
            id: 8,
            parent: Some(root.clip.clone()),
            local_transform_space: child.clone(),
            rect: Some(rect),
            radii: PaintCornerRadii::default(),
            clip_path: vec![],
            clip_path_even_odd: false,
            pixel_moving_filter: None,
        });
        let effect = Arc::new(EffectPaintPropertyNode {
            lifecycle: Default::default(),
            id: 9,
            parent: Some(root.effect.clone()),
            local_transform_space: child.clone(),
            output_clip: Some(clip),
            opacity: 0.5,
            ..(*root.effect).clone()
        });
        // The viewport clip stays in its ancestor space, while the effect's
        // output clip inherits the same snapped scroll as its content.
        let state = PropertyTreeState {
            transform: child,
            clip: root.clip,
            effect,
        };
        let (logical, logical_clip) = resolved_raster_properties(&state, 2.0).unwrap();
        let (composed, composed_clip) = resolved_compositor_properties(&state, 2.0).unwrap();
        assert_eq!(logical.1, -0.5);
        assert_eq!(composed.1, -0.75);
        assert_eq!(logical_clip.unwrap().y, -0.5);
        assert_eq!(composed_clip.unwrap().y, -0.75);
        let (scrolled, scrolled_clip) = resolved_compositor_properties_with_scroll(
            &state,
            2.0,
            Some(CompositorScrollOffset {
                scroll_node_id: 1,
                translation_y: -1.0,
            }),
        )
        .unwrap();
        assert_eq!(scrolled.1, -2.75);
        assert_eq!(scrolled_clip.unwrap().y, -2.75);
        assert_eq!(scrolling.matrix.values[13], -0.625);
    }
}

/// PendingLayer-style raster-space upcast for ordinary 2D affine local
/// transforms, including SVG viewBox scaling and paint-offset translations.
/// Stop at a real scroll/direct-compositing anchor. Return existing ancestor
/// nodes, never replacement identities or modified native chunk properties.
/// Renderer must bake omitted transforms and clips into its local PaintRecord.
pub fn raster_properties(
    state: &PropertyTreeState,
) -> Result<PropertyTreeState, UnsupportedReason> {
    let root = PropertyTreeState::default();
    let root_scroll = root.transform.scroll.as_ref();
    let mut omitted = Vec::new();
    let mut ancestor = state.transform.clone();
    loop {
        let real_scroll = ancestor.scroll.as_ref().is_some_and(|scroll| {
            root_scroll.is_none_or(|root| !scroll.lifecycle.same_node(&root.lifecycle))
        });
        if real_scroll
            || !ancestor.direct_compositing_reasons.is_empty()
            || ancestor.parent.is_none()
        {
            break;
        }
        // CanUpcastWith permits transforms within the same compositing boundary;
        // PaintChunksToCcLayer::ApplyTransform emits ConcatOp for non-translation
        // projections. Renderer retains the exact affine matrix in PaintRecord.
        // Current source builder folds pivots into matrix and exports origin
        // zero. Explicit nonzero affine pivots need their own correct mapping.
        let scaled = ancestor.matrix.values[0] != 1.0 || ancestor.matrix.values[5] != 1.0;
        let cross_axis = ancestor.matrix.values[1] != 0.0 || ancestor.matrix.values[4] != 0.0;
        if !affine_2d(&ancestor.matrix) || ((scaled || cross_axis) && ancestor.origin != [0.0; 3]) {
            transform_diagnostic(
                &state.transform,
                "non-affine-omitted-transform-or-origin",
                ancestor.id,
            );
            return Err(UnsupportedReason::NonTranslationTransform);
        }
        let parent = ancestor.parent.as_ref().unwrap().clone();
        omitted.push(ancestor);
        ancestor = parent;
    }
    translation(&ancestor)?;

    // PaintChunksToCcLayer can replay simple clips inside a root picture
    // layer, including clips whose local transform is already the root.
    // Clips in the retained anchor's own space can likewise be baked into
    // its picture layer (PropertyTreeState::CanUpcastWith, same-transform
    // fast path). A scroller viewport in an ancestor space stays external.
    // Never move a clip across an effect's output-clip boundary.
    let mut clip = state.clip.clone();
    loop {
        // PaintChunksToCcLayer can SwitchToClip in an ancestor transform and
        // later SwitchToTransform for the drawing. Our retained property tree
        // can express that directly, while one canonical PaintRecord cannot
        // remove an intermediate-transform clip without emitting the same
        // transform-state switch. Only bake a clip when it already lives in
        // the drawing's own transform space (or the retained raster anchor).
        // Keeping all other clips external preserves Chromium's property-tree
        // semantics instead of flattening them into the wrong local space.
        let local_space_is_record = state
            .transform
            .lifecycle
            .same_node(&clip.local_transform_space.lifecycle);
        let local_space_is_anchor = ancestor
            .lifecycle
            .same_node(&clip.local_transform_space.lifecycle);
        let mut effect = Some(&state.effect);
        let mut output_boundary = false;
        while let Some(current) = effect {
            if current
                .output_clip
                .as_ref()
                .is_some_and(|output| output.lifecycle.same_node(&clip.lifecycle))
            {
                output_boundary = true;
                break;
            }
            effect = current.parent.as_ref();
        }
        if (!local_space_is_record && !local_space_is_anchor) || output_boundary {
            break;
        }
        let Some(parent) = clip.parent.clone() else {
            break;
        };
        // Path clips remain owned by the compositor's real A8 clip surface.
        // Stop upcasting at this boundary instead of rejecting web content or
        // replacing the path with its scheduling rectangle.
        if !clip.clip_path.is_empty() || clip.pixel_moving_filter.is_some() {
            break;
        }
        if !valid_record_clip_shape(&clip) {
            clip_diagnostic(&clip, "raster-upcast-shape-or-filter", None, 1.0);
            return Err(UnsupportedReason::UnsupportedClip);
        }
        if let Some(rect) = clip.rect {
            if !finite(rect) {
                clip_diagnostic(&clip, "raster-upcast-nonfinite-rect", None, 1.0);
                return Err(UnsupportedReason::UnsupportedClip);
            }
            project_record_rect(rect, &clip.local_transform_space).map_err(|reason| {
                clip_diagnostic(&clip, "raster-upcast-transform-projection", None, 1.0);
                reason
            })?;
        }
        clip = parent;
    }
    Ok(PropertyTreeState {
        transform: ancestor,
        clip,
        effect: state.effect.clone(),
    })
}

/// Shared renderer/planner resolution of an already-upcast raster property
/// state. Scale/translation clips are projected using their real ancestry.
/// Returned clips are bounding rectangles, not exact rounded-clip coverage.
/// Renderer converts retained axis-aligned rect clips to enclosing integer
/// scissors (cc::ComputeLayerClipAndVisibleRect). Rounded/path clips retain
/// their real coverage (PropertyTreeManager::SyntheticEffectType). Fractional
/// geometry stays exact here; scissor enclosure does not snap layer placement
/// or descendant AA clips recorded in the layer's local raster space.
pub fn resolved_raster_properties(
    state: &PropertyTreeState,
    scale: f64,
) -> Result<((f64, f64), Option<PaintRect>), UnsupportedReason> {
    resolve(state, scale)
}

/// External layer placement and clip bounds in device coordinates after
/// compositor scroll snapping. Keep this space through final composition so
/// a divide/multiply by a noninteger device scale cannot reintroduce a phase.
/// Raster recording and logical DOM geometry continue using the unsnapped tree.
pub fn resolved_compositor_properties(
    state: &PropertyTreeState,
    scale: f64,
) -> Result<((f64, f64), Option<PaintRect>), UnsupportedReason> {
    resolve_with_record_clip_bounds(state, scale, true, false, true, None)
}

/// Resolve an impl-side scroll offset through the whole property state. Clips
/// whose local transform space descends from the scroll move with the content;
/// ancestor viewport clips remain fixed.
pub fn resolved_compositor_properties_with_scroll(
    state: &PropertyTreeState,
    scale: f64,
    scroll_override: Option<CompositorScrollOffset>,
) -> Result<((f64, f64), Option<PaintRect>), UnsupportedReason> {
    resolve_with_record_clip_bounds(state, scale, true, false, true, scroll_override)
}

/// Bounding geometry for validating original record clips before raster
/// upcasting. Rounded rectangles retain only their bounding rectangle here:
/// renderer must prove and replay the actual rounded shape in the record.
/// Effect output clips likewise retain conservative bounds here; compositor
/// scissoring and nontrivial shape coverage keep their external ownership.
pub fn resolved_record_clip_bounds(
    state: &PropertyTreeState,
    scale: f64,
) -> Result<((f64, f64), Option<PaintRect>), UnsupportedReason> {
    resolve_with_record_clip_bounds(state, scale, true, true, false, None)
}

fn valid_record_clip_shape(node: &ClipPaintPropertyNode) -> bool {
    let radii = node.radii;
    let finite_radii = [
        radii.top_left,
        radii.top_right,
        radii.bottom_right,
        radii.bottom_left,
    ]
    .iter()
    .all(|radius| {
        radius.x.is_finite() && radius.y.is_finite() && radius.x >= 0.0 && radius.y >= 0.0
    });
    finite_radii && (radii == PaintCornerRadii::default() || node.rect.is_some_and(finite))
}

/// Shared raster backing policy for opacity, masks and synthetic rounded-clip
/// surfaces. A page's opaque background must stay outside these groups.
pub fn needs_transparent_backing(state: &PropertyTreeState) -> bool {
    let mut effect = Some(&state.effect);
    while let Some(node) = effect {
        if node.opacity != 1.0 || node.is_mask || clip_chain_needs_mask(node.output_clip.as_ref()) {
            return true;
        }
        effect = node.parent.as_ref();
    }
    raster_properties(state)
        .map(|lowered| clip_chain_needs_mask(Some(&lowered.clip)))
        .unwrap_or(true)
}

fn clip_chain_needs_mask(mut clip: Option<&Arc<ClipPaintPropertyNode>>) -> bool {
    while let Some(node) = clip {
        let affine = crate::recording::clip_transform_state(&node.local_transform_space)
            .map(|state| state.cross_axis != (0.0, 0.0))
            .unwrap_or(true);
        if affine || node.radii != PaintCornerRadii::default() || !node.clip_path.is_empty() {
            return true;
        }
        clip = node.parent.as_ref();
    }
    false
}

fn project_axis_rect(
    rect: PaintRect,
    space: &Arc<TransformPaintPropertyNode>,
) -> Result<PaintRect, UnsupportedReason> {
    let mut sx = 1.0;
    let mut sy = 1.0;
    let mut tx = 0.0;
    let mut ty = 0.0;
    let mut node = Some(space);
    while let Some(current) = node {
        let scaled = current.matrix.values[0] != 1.0 || current.matrix.values[5] != 1.0;
        if !axis_aligned(&current.matrix) || (scaled && current.origin != [0.0; 3]) {
            return Err(UnsupportedReason::UnsupportedClip);
        }
        let m = &current.matrix.values;
        tx = m[0] * tx + m[12];
        ty = m[5] * ty + m[13];
        sx *= m[0];
        sy *= m[5];
        node = current.parent.as_ref();
    }
    let left = rect.x * sx + tx;
    let right = (rect.x + rect.width) * sx + tx;
    let top = rect.y * sy + ty;
    let bottom = (rect.y + rect.height) * sy + ty;
    let projected = PaintRect {
        x: left.min(right),
        y: top.min(bottom),
        width: (right - left).abs(),
        height: (bottom - top).abs(),
    };
    if !finite(projected) {
        return Err(UnsupportedReason::UnsupportedClip);
    }
    Ok(projected)
}

// GeometryMapper bounds are only scheduling/validation support. The exact
// transformed ClipOp is retained by recording.rs, never replaced by this box.
fn project_record_rect(
    rect: PaintRect,
    space: &Arc<TransformPaintPropertyNode>,
) -> Result<PaintRect, UnsupportedReason> {
    if let Ok(rect) = project_axis_rect(rect, space) {
        return Ok(rect);
    }
    let state = crate::recording::clip_transform_state(space)
        .map_err(|_| UnsupportedReason::UnsupportedClip)?;
    let projected = crate::recording::mapped_rect(rect, state);
    if !finite(projected) {
        return Err(UnsupportedReason::UnsupportedClip);
    }
    Ok(projected)
}

fn project_affine_rect(
    rect: PaintRect,
    matrix: &TransformMatrix,
) -> Result<PaintRect, UnsupportedReason> {
    if !affine_2d(matrix) {
        return Err(UnsupportedReason::UnsupportedClip);
    }
    let m = &matrix.values;
    let mut left = f64::INFINITY;
    let mut top = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    for (x, y) in [
        (rect.x, rect.y),
        (rect.x + rect.width, rect.y),
        (rect.x, rect.y + rect.height),
        (rect.x + rect.width, rect.y + rect.height),
    ] {
        let projected_x = x * m[0] + y * m[4] + m[12];
        let projected_y = x * m[1] + y * m[5] + m[13];
        left = left.min(projected_x);
        top = top.min(projected_y);
        right = right.max(projected_x);
        bottom = bottom.max(projected_y);
    }
    let projected = PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    };
    finite(projected)
        .then_some(projected)
        .ok_or(UnsupportedReason::UnsupportedClip)
}

pub(crate) fn pixel_aligned(value: f64, scale: f64) -> bool {
    let pixels = value * scale;
    pixels.is_finite() && pixels == pixels.round()
}

/// Resolve geometry for the supported CPU effect subset. The real effect chain
/// remains in LayerPlan and must be composed as groups; layer/tile opacity
/// multipliers cannot substitute for an effect surface shared by many layers.
pub(crate) fn resolve(
    state: &PropertyTreeState,
    scale: f64,
) -> Result<((f64, f64), Option<PaintRect>), UnsupportedReason> {
    // PaintChunksToCcLayer retains affine clips as synthesized clip effects.
    // Their projected rectangles are culling bounds only; renderer applies
    // the original transformed rect/rrect/path as an A8 coverage mask.
    resolve_with_record_clip_bounds(state, scale, true, true, false, None)
}

fn resolve_with_record_clip_bounds(
    state: &PropertyTreeState,
    scale: f64,
    allow_rounded_record_clip: bool,
    allow_affine_record_clip: bool,
    compositor: bool,
    scroll_override: Option<CompositorScrollOffset>,
) -> Result<((f64, f64), Option<PaintRect>), UnsupportedReason> {
    let mut delta = translation(&state.transform)?;
    if compositor {
        let matrix = compositor_transform_with_scroll(&state.transform, scale, scroll_override)?;
        delta = (matrix.values[12], matrix.values[13]);
    }
    let mut effect = Some(&state.effect);
    let mut root_clip = None;
    while let Some(node) = effect {
        if !node.opacity.is_finite() || !(0.0..=1.0).contains(&node.opacity) {
            return Err(UnsupportedReason::OpacityEffect);
        }
        if node.blend_mode != PaintBlendMode::kNormal || node.isolates_blending {
            return Err(UnsupportedReason::BlendEffect);
        }
        // Chromium promotes CSS filters to an isolated effect/render pass.
        // This CPU path currently implements Skia's blur image filter; keep
        // every other operation explicit instead of silently flattening it.
        if node.filters.iter().any(|filter| {
            filter.r#type != PaintFilterType::kBlur
                || !filter.amount.is_finite()
                || filter.amount < 0.0
        }) {
            return Err(UnsupportedReason::FilterEffect);
        }
        // A Mask is a real DstIn effect child, not a compositor coverage
        // callback. Its source pixels travel through ordinary persistent tiles.
        if node.is_mask && (node.opacity != 1.0 || node.parent.is_none()) {
            return Err(UnsupportedReason::MaskEffect);
        }
        if node.has_mask {
            return Err(UnsupportedReason::MaskEffect);
        }
        if let Some(output_clip) = &node.output_clip {
            intersect_clip_chain(
                output_clip,
                scale,
                &mut root_clip,
                true,
                false,
                compositor,
                scroll_override,
            )?;
        }
        effect = node.parent.as_ref();
    }
    intersect_clip_chain(
        &state.clip,
        scale,
        &mut root_clip,
        allow_rounded_record_clip,
        allow_affine_record_clip,
        compositor,
        scroll_override,
    )?;
    Ok((delta, root_clip))
}

fn intersect_clip_chain(
    first: &Arc<ClipPaintPropertyNode>,
    scale: f64,
    root_clip: &mut Option<PaintRect>,
    allow_rounded_record_clip: bool,
    allow_affine_record_clip: bool,
    compositor: bool,
    scroll_override: Option<CompositorScrollOffset>,
) -> Result<(), UnsupportedReason> {
    let mut clip = Some(first);
    while let Some(node) = clip {
        if (node.radii != PaintCornerRadii::default() && !allow_rounded_record_clip)
            || !valid_record_clip_shape(node)
            || node.pixel_moving_filter.as_ref().is_some_and(|effect| {
                // This is an input-cull expander for the actual filter effect,
                // not an additional clip operation. Its output clip stays on
                // the effect surface and the renderer supplies blur padding.
                node.rect.is_some()
                    || !node.clip_path.is_empty()
                    || effect.filters.is_empty()
                    || effect.filters.iter().any(|filter| {
                        filter.r#type != PaintFilterType::kBlur
                            || !filter.amount.is_finite()
                            || filter.amount < 0.0
                    })
            })
        {
            clip_diagnostic(node, "shape-or-filter", None, scale);
            return Err(UnsupportedReason::UnsupportedClip);
        }
        if let Some(rect) = node.rect {
            if !finite(rect) {
                clip_diagnostic(node, "nonfinite-rect", None, scale);
                return Err(UnsupportedReason::UnsupportedClip);
            }
            let rect = if compositor {
                let matrix = compositor_transform_with_scroll(
                    &node.local_transform_space,
                    scale,
                    scroll_override,
                )?;
                project_affine_rect(rect, &matrix)
            } else if allow_affine_record_clip {
                project_record_rect(rect, &node.local_transform_space)
            } else {
                project_axis_rect(rect, &node.local_transform_space)
            }
            .map_err(|reason| {
                clip_diagnostic(node, "transform-projection", None, scale);
                reason
            })?;
            // GeometryMapper maps/intersects FloatClipRect without snapping
            // (geometry_mapper.cc:487). PaintClipRect may also be non-snapped
            // (clip_paint_property_node.h:185). Preserve actual coordinates;
            // compositor scissor enclosure and nontrivial shape coverage are
            // separate work. Descendant record clips keep their local AA phase.
            *root_clip = Some(root_clip.map_or(rect, |old| intersection(old, rect)));
        }
        clip = node.parent.as_ref();
    }
    Ok(())
}

fn clip_diagnostic(
    node: &ClipPaintPropertyNode,
    reason: &str,
    projected: Option<PaintRect>,
    scale: f64,
) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_none()
        || REPORTED.swap(true, Ordering::Relaxed)
    {
        return;
    }
    let transform = &node.local_transform_space;
    let m = &transform.matrix.values;
    let projected_edges = projected.map(|rect| {
        [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height]
            .map(|value| (value * scale, pixel_aligned(value, scale)))
    });
    eprintln!(
        "[layer_tile clip unsupported] reason={} node={} parent={:?} rect={:?} radii={:?} pathlen={} pixelmovingfilter={} projected={:?} device_edges={:?} scale={} transform={} parent_transform={:?} axis={} sx={} sy={} tx={} ty={} origin={:?}",
        reason, node.id, node.parent.as_ref().map(|parent| parent.id), node.rect,
        node.radii, node.clip_path.len(), node.pixel_moving_filter.is_some(), projected,
        projected_edges, scale, transform.id, transform.parent.as_ref().map(|parent| parent.id),
        axis_aligned(&transform.matrix), m[0], m[5], m[12], m[13], transform.origin,
    );
    let mut current = Some(transform);
    while let Some(space) = current {
        eprintln!(
            "[layer_tile clip transform] node={} parent={:?} matrix={:?} origin={:?} scroll={:?}",
            space.id,
            space.parent.as_ref().map(|parent| parent.id),
            space.matrix.values,
            space.origin,
            space.scroll.as_ref().map(|scroll| scroll.id)
        );
        current = space.parent.as_ref();
    }
    if !node.clip_path.is_empty() {
        eprintln!(
            "[layer_tile clip path] node={} even_odd={} commands={:?}",
            node.id, node.clip_path_even_odd, node.clip_path
        );
    }
}
