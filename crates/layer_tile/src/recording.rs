//! PaintChunksToCcLayer-style record lowering before tile planning.
//! Skia is used here only for common matrix/point math, never raster execution.
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::RasterRecordContent;
use layoutng_assembly::internal::layout_input::{PaintPathVerb, TransformMatrix};
use layoutng_assembly::internal::paint_input::{
    PaintBlendMode, PaintCornerRadii, PaintMaskComposite, PaintMaskMode, PaintShaderKind,
};
use paint::paint_engine::{
    DisplayItem, DisplayItemType as Kind, PaintArtifact, PaintChunk, PaintRect, RecordedDisplayItem,
};
use paint::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PropertyTreeState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayUnsupported(pub &'static str);
impl ReplayUnsupported {
    pub fn reason(&self) -> &'static str {
        self.0
    }
}
impl std::fmt::Display for ReplayUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ReplayUnsupported {}

fn contains(a: PaintRect, b: PaintRect) -> bool {
    b.is_empty()
        || (a.x <= b.x
            && a.y <= b.y
            && a.x + a.width >= b.x + b.width
            && a.y + a.height >= b.y + b.height)
}
fn validate_mask(item: &DisplayItem) -> Result<(), ReplayUnsupported> {
    // compat/canvas.rs::end_mask_impl currently renders linear gradient mask
    // coverage. Images, luminance and non-Add composites need their actual
    // raster implementation; rendering these as white would silently lose CSS.
    if item.mask_layers.is_empty() {
        return Err(ReplayUnsupported("missing-mask-payload"));
    }
    for (index, layer) in item.mask_layers.iter().enumerate() {
        let valid = layer.resource_id == 0
            && layer.mode == PaintMaskMode::kAlpha
            && layer.composite == PaintMaskComposite::kAdd
            && layer
                .paint_shader
                .as_ref()
                .is_some_and(|shader| shader.kind == PaintShaderKind::kLinearGradient)
            && finite_rect(layer.clip_rect)
            && finite_rect(layer.tile_rect);
        if !valid {
            if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                eprintln!("layer-replay unsupported-mask node={} layer={index} resource={} mode={:?} composite={:?} shader={:?} clip={:?} tile={:?}",
                    item.node_id,layer.resource_id,layer.mode,layer.composite,
                    layer.paint_shader.as_ref().map(|shader|shader.kind),layer.clip_rect,layer.tile_rect);
            }
            return Err(ReplayUnsupported("unsupported-mask-payload"));
        }
    }
    Ok(())
}
// Only the real Mask DrawingDisplayItem may contain this outer DstIn
// wrapper. It is retained in full-frame commands but stripped from source-tile
// raster: blend belongs to the complete parent effect group at composition.
fn mask_record_body(
    raw: &[DisplayItem],
    is_mask: bool,
) -> Result<&[DisplayItem], ReplayUnsupported> {
    if !is_mask {
        if raw
            .iter()
            .any(|item| matches!(item.r#type, Kind::kDrawMask | Kind::kSaveLayerDstIn))
        {
            return Err(ReplayUnsupported("mask-drawing-without-mask-effect"));
        }
        return Ok(raw);
    }
    if raw.len() < 3
        || raw[0].r#type != Kind::kSaveLayerDstIn
        || raw.last().unwrap().r#type != Kind::kRestore
    {
        return Err(ReplayUnsupported("invalid-mask-drawing-boundary"));
    }
    let body = &raw[1..raw.len() - 1];
    if !body.iter().any(|item| item.r#type == Kind::kDrawMask)
        || body.iter().any(|item| item.r#type == Kind::kSaveLayerDstIn)
    {
        return Err(ReplayUnsupported("invalid-mask-source-record"));
    }
    Ok(body)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub translation: (f64, f64),
    pub axis_scale: (f64, f64),
    pub cross_axis: (f64, f64),
    pub clip: Option<PaintRect>,
    pub complex_clip: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct RoundedClip {
    rect: PaintRect,
    radii: PaintCornerRadii,
    antialias: bool,
    // Exact receiver CTM for a removed clip in the drawing's affine space.
    // Translation-only clips keep their established root-space representation.
    affine: Option<(f64, f64, f64, f64, f64, f64)>,
}
fn affine_key(state: State) -> (f64, f64, f64, f64, f64, f64) {
    (
        state.axis_scale.0,
        state.cross_axis.1,
        state.cross_axis.0,
        state.axis_scale.1,
        state.translation.0,
        state.translation.1,
    )
}
fn clip_shape(
    rect: PaintRect,
    radii: PaintCornerRadii,
    antialias: bool,
    state: State,
) -> RoundedClip {
    if state.axis_scale == (1.0, 1.0) && state.cross_axis == (0.0, 0.0) {
        RoundedClip {
            rect: shift(rect, state.translation),
            radii,
            antialias,
            affine: None,
        }
    } else {
        RoundedClip {
            rect,
            radii,
            antialias,
            affine: Some(affine_key(state)),
        }
    }
}
pub(crate) fn clip_transform_state(
    space: &Arc<paint::paint_property_tree::TransformPaintPropertyNode>,
) -> Result<State, ReplayUnsupported> {
    let mut chain = Vec::new();
    let mut current = Some(space);
    while let Some(node) = current {
        if node.origin != [0.0; 3] && translation(&node.matrix, 1.0).is_err() {
            return Err(ReplayUnsupported("unsupported-clip-transform-origin"));
        }
        chain.push(node);
        current = node.parent.as_ref();
    }
    let mut state = State::default();
    for node in chain.iter().rev() {
        concatenate_affine(&mut state, &node.matrix)?;
    }
    Ok(state)
}
fn rounded_clips_between(
    first: &Arc<ClipPaintPropertyNode>,
    stop: Option<&Arc<ClipPaintPropertyNode>>,
    scale: f64,
) -> Result<Arc<[RoundedClip]>, ReplayUnsupported> {
    rounded_clips_between_in_raster_space(first, stop, scale, (0.0, 0.0))
}
fn rounded_clips_between_in_raster_space(
    first: &Arc<ClipPaintPropertyNode>,
    stop: Option<&Arc<ClipPaintPropertyNode>>,
    scale: f64,
    raster_origin: (f64, f64),
) -> Result<Arc<[RoundedClip]>, ReplayUnsupported> {
    let mut result = Vec::new();
    let mut clip = Some(first);
    while let Some(node) = clip {
        if stop.is_some_and(|stop| stop.lifecycle.same_node(&node.lifecycle)) {
            break;
        }
        if node.radii.HasRadius()
            || node.rect.is_some_and(|rect| {
                let mut delta = (0.0, 0.0);
                let mut transform = Some(&node.local_transform_space);
                while let Some(current) = transform {
                    let Ok(offset) = translation(&current.matrix, scale) else {
                        return true;
                    };
                    delta.0 += offset.0;
                    delta.1 += offset.1;
                    transform = current.parent.as_ref();
                }
                // Flat wrapper validation uses root space. Removed clips are
                // rasterized in the retained anchor's local device, so their exact
                // plain-clip requirement must use that device's pixel phase too.
                !aligned_rect(
                    shift(rect, (delta.0 - raster_origin.0, delta.1 - raster_origin.1)),
                    scale,
                )
            })
        {
            let mut delta = (0.0, 0.0);
            let mut current = Some(&node.local_transform_space);
            let mut translations = true;
            while let Some(space) = current {
                match translation(&space.matrix, scale) {
                    Ok(offset) => {
                        delta.0 += offset.0;
                        delta.1 += offset.1;
                    }
                    Err(_) => {
                        translations = false;
                        break;
                    }
                }
                current = space.parent.as_ref();
            }
            let transform = if translations {
                State {
                    translation: delta,
                    ..Default::default()
                }
            } else {
                clip_transform_state(&node.local_transform_space)?
            };
            // Official SwitchToClip applies AA to plain rectangles too.
            result.push(clip_shape(
                node.rect
                    .ok_or(ReplayUnsupported("rounded-clip-without-rect"))?,
                node.radii,
                true,
                transform,
            ));
        }
        clip = node.parent.as_ref();
    }
    if stop.is_some() && clip.is_none() {
        return Err(ReplayUnsupported("raster-clip-upcast-is-not-ancestor"));
    }
    result.reverse();
    Ok(Arc::from(result))
}
// Rasterize only clips removed by PaintChunksToCcLayer lowering. The
// retained viewport/effect-output ancestors belong to final composition and
// must not move through cached layer-local PaintRecords during scrolling.
fn removed_clip_bounds(
    first: &Arc<ClipPaintPropertyNode>,
    stop: &Arc<ClipPaintPropertyNode>,
) -> Result<Option<PaintRect>, ReplayUnsupported> {
    let mut result = None;
    let mut clip = Some(first);
    while let Some(node) = clip {
        if node.lifecycle.same_node(&stop.lifecycle) {
            return Ok(result);
        }
        if node.pixel_moving_filter.is_some() || !node.clip_path.is_empty() {
            return Err(ReplayUnsupported("unsupported-removed-clip"));
        }
        if let Some(rect) = node.rect {
            let mut transforms = Vec::new();
            let mut transform = Some(&node.local_transform_space);
            while let Some(current) = transform {
                transforms.push(current.matrix);
                transform = current.parent.as_ref();
            }
            transforms.reverse();
            let mut root = paint::geometry_mapper::MapRectToRoot(rect, &transforms)
                .ok_or(ReplayUnsupported("invalid-removed-clip-transform"))?;
            // Include the receiver's narrowed SkMatrix projection too. This
            // enclosure schedules pixels; the exact ClipOp remains in record.
            if transforms
                .iter()
                .any(|matrix| translation(matrix, 1.0).is_err())
            {
                root.union(mapped_rect(
                    rect,
                    clip_transform_state(&node.local_transform_space)?,
                ));
            }
            result = intersect(result, root);
        }
        clip = node.parent.as_ref();
    }
    Err(ReplayUnsupported("raster-clip-upcast-is-not-ancestor"))
}
impl Default for State {
    fn default() -> Self {
        Self {
            translation: (0.0, 0.0),
            axis_scale: (1.0, 1.0),
            cross_axis: (0.0, 0.0),
            clip: None,
            complex_clip: false,
        }
    }
}
pub fn mapped_rect(rect: PaintRect, state: State) -> PaintRect {
    if state.cross_axis != (0.0, 0.0) {
        // A rotation/shear maps all four corners. This is support geometry
        // only; exact transformed paths and clips still reach Canvas unchanged.
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
            let x2 = x * state.axis_scale.0 + y * state.cross_axis.0 + state.translation.0;
            let y2 = x * state.cross_axis.1 + y * state.axis_scale.1 + state.translation.1;
            left = left.min(x2);
            top = top.min(y2);
            right = right.max(x2);
            bottom = bottom.max(y2);
        }
        // Canvas builds its rect path with f32 origin + f32 size, then maps
        // those points with SkMatrix. Include that actual projection as well
        // as the semantic f64 enclosure, rather than inventing an AA margin.
        let transform = skia::Transform::from_row(
            state.axis_scale.0 as f32,
            state.cross_axis.1 as f32,
            state.cross_axis.0 as f32,
            state.axis_scale.1 as f32,
            state.translation.0 as f32,
            state.translation.1 as f32,
        );
        let l = rect.x as f32;
        let t = rect.y as f32;
        let r = l + rect.width as f32;
        let b = t + rect.height as f32;
        for (x, y) in [(l, t), (r, t), (l, b), (r, b)] {
            let mut point = skia::Point::from_xy(x, y);
            transform.map_point(&mut point);
            left = left.min(point.x as f64);
            top = top.min(point.y as f64);
            right = right.max(point.x as f64);
            bottom = bottom.max(point.y as f64);
        }
        return PaintRect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        };
    }
    let x = rect.x * state.axis_scale.0 + state.translation.0;
    let y = rect.y * state.axis_scale.1 + state.translation.1;
    let x2 = (rect.x + rect.width) * state.axis_scale.0 + state.translation.0;
    let y2 = (rect.y + rect.height) * state.axis_scale.1 + state.translation.1;
    PaintRect {
        x: x.min(x2),
        y: y.min(y2),
        width: (x2 - x).abs(),
        height: (y2 - y).abs(),
    }
}
fn shift(mut r: PaintRect, delta: (f64, f64)) -> PaintRect {
    r.x += delta.0;
    r.y += delta.1;
    r
}
pub fn intersection(a: PaintRect, b: PaintRect) -> PaintRect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    PaintRect {
        x,
        y,
        width: ((a.x + a.width).min(b.x + b.width) - x).max(0.0),
        height: ((a.y + a.height).min(b.y + b.height) - y).max(0.0),
    }
}
pub fn intersect(clip: Option<PaintRect>, r: PaintRect) -> Option<PaintRect> {
    Some(clip.map_or(r, |c| intersection(c, r)))
}
pub fn finite_rect(r: PaintRect) -> bool {
    [r.x, r.y, r.width, r.height, r.x + r.width, r.y + r.height]
        .iter()
        .all(|v| v.is_finite())
        && r.width >= 0.0
        && r.height >= 0.0
}
fn pixel_integer(value: f64, scale: f64) -> bool {
    let v = value * scale;
    v.is_finite() && v.fract() == 0.0 && v.abs() <= (1 << 22) as f64
}
pub fn aligned_rect(r: PaintRect, scale: f64) -> bool {
    finite_rect(r)
        && [r.x, r.y, r.x + r.width, r.y + r.height]
            .iter()
            .all(|&v| pixel_integer(v, scale))
}
fn translation(matrix: &TransformMatrix, scale: f64) -> Result<(f64, f64), ReplayUnsupported> {
    let mut values = matrix.values;
    let delta = (values[12], values[13]);
    values[12] = 0.0;
    values[13] = 0.0;
    if values != TransformMatrix::default().values {
        return Err(ReplayUnsupported("non-translation-transform"));
    }
    if ![delta.0, delta.1]
        .iter()
        .all(|value| value.is_finite() && (value * scale).abs() <= (1 << 22) as f64)
    {
        return Err(ReplayUnsupported("invalid-translation-placement"));
    }
    Ok(delta)
}
fn property_error(error: crate::UnsupportedReason) -> ReplayUnsupported {
    use crate::UnsupportedReason as Reason;
    ReplayUnsupported(match error {
        Reason::NonTranslationTransform => "non-translation-transform",
        Reason::FractionalPixelPlacement => "fractional-pixel-placement",
        Reason::UnsupportedClip => "unsupported-property-clip",
        Reason::OpacityEffect => "opacity-effect",
        Reason::BlendEffect => "blend-effect",
        Reason::FilterEffect => "filter-effect",
        Reason::MaskEffect => "mask-effect",
        _ => "unsupported-raster-properties",
    })
}
fn diagnose_property_clip(
    chunk: RecordingChunk<'_>,
    stage: &str,
    error: ReplayUnsupported,
) -> ReplayUnsupported {
    use std::sync::atomic::{AtomicBool, Ordering};
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if error.reason() == "unsupported-property-clip"
        && std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some()
        && !REPORTED.swap(true, Ordering::Relaxed)
    {
        eprintln!("[layer_tile recording clip unsupported] stage={stage} chunk={:?} records={}..{} transform={} clip={} effect={} bounds={:?}",
            chunk.id,chunk.begin_index,chunk.end_index,chunk.properties.transform.id,
            chunk.properties.clip.id,chunk.properties.effect.id,chunk.bounds);
        let mut clip = Some(&chunk.properties.clip);
        while let Some(node) = clip {
            eprintln!("[layer_tile recording clip state] node={} parent={:?} rect={:?} radii={:?} pathlen={} pixelmovingfilter={} transform={} revision={}",
                node.id,node.parent.as_ref().map(|parent|parent.id),node.rect,node.radii,
                node.clip_path.len(),node.pixel_moving_filter.is_some(),node.local_transform_space.id,node.lifecycle.revision);
            clip = node.parent.as_ref();
        }
        let mut effect = Some(&chunk.properties.effect);
        while let Some(node) = effect {
            eprintln!(
                "[layer_tile recording clip effect] node={} parent={:?} output_clip={:?}",
                node.id,
                node.parent.as_ref().map(|parent| parent.id),
                node.output_clip.as_ref().map(|clip| clip.id)
            );
            effect = node.parent.as_ref();
        }
    }
    error
}
pub fn resolved(state: &PropertyTreeState, scale: f64) -> Result<State, ReplayUnsupported> {
    let (root_translation, clip) =
        crate::resolved_raster_properties(state, scale).map_err(property_error)?;
    if ![root_translation.0, root_translation.1]
        .iter()
        .all(|value| value.is_finite() && (value * scale).abs() <= (1 << 22) as f64)
    {
        return Err(ReplayUnsupported("large-property-placement"));
    }
    Ok(State {
        translation: root_translation,
        clip,
        ..Default::default()
    })
}
pub fn concatenate_affine(
    state: &mut State,
    matrix: &TransformMatrix,
) -> Result<(), ReplayUnsupported> {
    let mut identity = matrix.values;
    for index in [0, 1, 4, 5, 12, 13] {
        identity[index] = TransformMatrix::default().values[index];
    }
    if identity != TransformMatrix::default().values || !matrix.values.iter().all(|v| v.is_finite())
    {
        if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
            eprintln!(
                "layer-replay unsupported-record-transform matrix={:?}",
                matrix.values
            );
        }
        return Err(ReplayUnsupported("non-affine-record-transform"));
    }
    if state.cross_axis == (0.0, 0.0) && matrix.values[1] == 0.0 && matrix.values[4] == 0.0 {
        // Preserve the established axis-only receiver arithmetic exactly.
        let sx = state.axis_scale.0 as f32;
        let sy = state.axis_scale.1 as f32;
        state.translation.0 = (sx * matrix.values[12] as f32 + state.translation.0 as f32) as f64;
        state.translation.1 = (sy * matrix.values[13] as f32 + state.translation.1 as f32) as f64;
        state.axis_scale = (
            (sx * matrix.values[0] as f32) as f64,
            (sy * matrix.values[5] as f32) as f64,
        );
    } else {
        // Use the actual receiver's SkMatrix::concat, including its f64 cross
        // products followed by f32 narrowing for rotation/shear. No second matrix
        // convention or approximate geometry transform is introduced here.
        let a = skia::Transform::from_row(
            state.axis_scale.0 as f32,
            state.cross_axis.1 as f32,
            state.cross_axis.0 as f32,
            state.axis_scale.1 as f32,
            state.translation.0 as f32,
            state.translation.1 as f32,
        );
        let b = skia::Transform::from_row(
            matrix.values[0] as f32,
            matrix.values[1] as f32,
            matrix.values[4] as f32,
            matrix.values[5] as f32,
            matrix.values[12] as f32,
            matrix.values[13] as f32,
        );
        let composed = a.pre_concat(b);
        state.translation = (composed.tx as f64, composed.ty as f64);
        state.axis_scale = (composed.sx as f64, composed.sy as f64);
        state.cross_axis = (composed.kx as f64, composed.ky as f64);
    }
    if ![
        state.translation.0,
        state.translation.1,
        state.axis_scale.0,
        state.axis_scale.1,
        state.cross_axis.0,
        state.cross_axis.1,
    ]
    .iter()
    .all(|v| v.is_finite())
    {
        return Err(ReplayUnsupported("invalid-record-transform"));
    }
    Ok(())
}
pub fn is_rect_draw(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::kDrawRect
            | Kind::kDrawScrollbarTrack
            | Kind::kDrawScrollbarThumb
            | Kind::kDrawScrollbarButton
            | Kind::kDrawScrollbarCorner
    )
}
fn pixel_bounds(r: PaintRect, scale: f64) -> PaintRect {
    let x = (r.x * scale).floor() / scale;
    let y = (r.y * scale).floor() / scale;
    PaintRect {
        x,
        y,
        width: ((r.x + r.width) * scale).ceil() / scale - x,
        height: ((r.y + r.height) * scale).ceil() / scale - y,
    }
}
// Proven fill support, independent of opaque coverage. Rounded rectangles
// and ellipses have complete enclosing rects but do NOT cover their corners;
// is_rect_draw remains narrow for LCD/opaque-background proofs.
fn path_control_bounds(item: &DisplayItem) -> Option<PaintRect> {
    if item.inverse_winding || item.path.is_empty() {
        return None;
    }
    let mut left = f64::INFINITY;
    let mut top = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    for command in &item.path {
        if command.verb == PaintPathVerb::kClose {
            continue;
        }
        if command.verb == PaintPathVerb::kConicTo
            && (!command.conic_weight.is_finite() || command.conic_weight <= 0.0)
        {
            return None;
        }
        let count = match command.verb {
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => 2,
            PaintPathVerb::kCubicTo => 3,
            _ => 1,
        };
        for point in [command.point, command.control1, command.control2]
            .iter()
            .take(count)
        {
            if !point.x.is_finite() || !point.y.is_finite() {
                return None;
            }
            left = left.min(point.x);
            top = top.min(point.y);
            right = right.max(point.x);
            bottom = bottom.max(point.y);
        }
    }
    left.is_finite().then_some(PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}
fn rect_record_pixel_bounds(
    raw: &[DisplayItem],
    initial: State,
    clip: Option<PaintRect>,
    scale: f64,
) -> Option<PaintRect> {
    record_pixel_bounds(raw, initial, clip, scale, false)
}
pub fn record_pixel_bounds(
    raw: &[DisplayItem],
    initial: State,
    clip: Option<PaintRect>,
    scale: f64,
    ignore_clips: bool,
) -> Option<PaintRect> {
    let mut state = State { clip, ..initial };
    let mut saved = Vec::new();
    let mut bounds = PaintRect::default();
    for item in raw {
        match item.r#type {
            Kind::kSave => saved.push(state),
            Kind::kRestore => state = saved.pop()?,
            Kind::kConcat => concatenate_affine(&mut state, &item.transform).ok()?,
            Kind::kClipRect | Kind::kClipRoundedRect => {
                if !ignore_clips {
                    let rect = mapped_rect(item.rect, state);
                    if !finite_rect(rect) {
                        return None;
                    }
                    state.clip = intersect(state.clip, rect);
                }
            }
            Kind::kClipPath => {
                if !ignore_clips && !item.inverse_winding {
                    if let Some(rect) = path_control_bounds(item) {
                        state.clip = intersect(state.clip, mapped_rect(rect, state));
                    }
                }
            }
            // Difference clips only remove ink. Their unconstrained parent's
            // enclosing bound remains a safe (possibly larger) bound.
            Kind::kClipOutRoundedRect | Kind::kClipOutRect => {}
            kind if is_draw(kind) => {
                if !item.filters.is_empty()
                    || (kind != Kind::kDrawMask && !item.mask_layers.is_empty())
                {
                    return None;
                }
                let known = if kind == Kind::kDrawMask {
                    validate_mask(item).ok()?;
                    // Mask source has finite actual clip support. No viewport
                    // or effect output clip is baked into these local bounds.
                    let mut support = PaintRect::default();
                    for layer in &item.mask_layers {
                        support.union(mapped_rect(layer.clip_rect, state));
                    }
                    Some(support)
                } else if is_rect_draw(kind)
                    || matches!(
                        kind,
                        Kind::kDrawRoundedRect
                            | Kind::kDrawDoubleRoundedRect
                            | Kind::kDrawEllipse
                            | Kind::kDrawImageRect
                            | Kind::kDrawTiledImage
                            | Kind::kDrawGradientRect
                            | Kind::kDrawTiledGradient
                    )
                {
                    finite_rect(item.rect).then(|| mapped_rect(item.rect, state))
                } else if kind == Kind::kDrawBoxShadow
                    && [
                        item.blur_radius,
                        item.spread,
                        item.shadow_offset.x,
                        item.shadow_offset.y,
                    ]
                    .iter()
                    .all(|v| v.is_finite())
                    && item.blur_radius >= 0.0
                {
                    // compat/canvas.rs::draw_box_shadow clips its finite mask
                    // to the transformed spread box plus ceil(3*sigma)+2.
                    // Max axis scale encloses its geometric-mean sigma even
                    // before the receiver's 128px cap. Keep a native rounding
                    // guard; inset output cannot leave its receiving box.
                    let local = if item.inset {
                        item.rect
                    } else {
                        PaintRect {
                            x: item.rect.x - item.spread + item.shadow_offset.x,
                            y: item.rect.y - item.spread + item.shadow_offset.y,
                            width: item.rect.width + 2.0 * item.spread,
                            height: item.rect.height + 2.0 * item.spread,
                        }
                    };
                    finite_rect(local).then(|| {
                        let mut rect = mapped_rect(local, state);
                        let sigma = (item.blur_radius * 0.5) as f32;
                        let axis = if state.cross_axis == (0.0, 0.0) {
                            (state.axis_scale.0.abs().max(state.axis_scale.1.abs()) * scale) as f32
                        } else {
                            let d0 = skia::Point::from_xy(
                                state.axis_scale.0 as f32,
                                state.cross_axis.1 as f32,
                            )
                            .length();
                            let d1 = skia::Point::from_xy(
                                state.cross_axis.0 as f32,
                                state.axis_scale.1 as f32,
                            )
                            .length();
                            d0.max(d1) * scale as f32
                        };
                        let border = if item.inset {
                            1.0
                        } else {
                            (3.0 * (sigma * axis) as f64).ceil() + 3.0
                        };
                        let border = border / scale;
                        rect.x -= border;
                        rect.y -= border;
                        rect.width += 2.0 * border;
                        rect.height += 2.0 * border;
                        rect
                    })
                } else if kind == Kind::kDrawPath && item.blur_radius == 0.0 {
                    path_control_bounds(item).map(|rect| {
                        // Positive-weight Beziers remain inside their control
                        // hull. One native pixel encloses f32 scan/AA rounding.
                        let mut rect = mapped_rect(rect, state);
                        let fringe = 1.0 / scale;
                        rect.x -= fringe;
                        rect.y -= fringe;
                        rect.width += 2.0 * fringe;
                        rect.height += 2.0 * fringe;
                        rect
                    })
                } else {
                    None
                };
                // Every output pixel of a glyph/stroke/shadow replay is still
                // constrained by its actual finite record-local clip. This is
                // a complete support proof, not a claim that the draw fills it.
                // External retained viewport/output clips never enter state.
                let mut rect = match (known, state.clip) {
                    (Some(rect), Some(clip)) => intersection(rect, clip),
                    (Some(rect), None) => rect,
                    (None, Some(clip)) => clip,
                    (None, None) => return None,
                };
                if !finite_rect(rect) {
                    return None;
                }
                if !rect.is_empty() {
                    rect = pixel_bounds(rect, scale);
                    bounds.union(rect);
                }
            }
            _ => return None,
        }
    }
    saved.is_empty().then_some(bounds)
}
// The standalone scrollbar adapter applies its owner's overflow clip in the
// flat stream, while the semantic ScrollbarDisplayItem uses border-box state.
// Removing that extra clip is safe only when every rect's complete pixel
// enclosure lies inside BOTH external clips. Do not infer this from the
// recorder's conservative visual_rect or simply ignore unequal clip values.
fn rect_record_clips_are_redundant(
    raw: &[DisplayItem],
    actual: State,
    expected: State,
    scale: f64,
) -> bool {
    if actual.axis_scale != (1.0, 1.0) || actual.cross_axis != (0.0, 0.0) {
        return false;
    }
    let mut offset = actual.translation;
    let mut stack = Vec::new();
    for item in raw {
        match item.r#type {
            Kind::kSave => stack.push(offset),
            Kind::kRestore => match stack.pop() {
                Some(saved) => offset = saved,
                None => return false,
            },
            Kind::kConcat => match translation(&item.transform, scale) {
                Ok(delta) => {
                    offset.0 += delta.0;
                    offset.1 += delta.1;
                }
                Err(_) => return false,
            },
            Kind::kClipRect => {}
            kind if is_rect_draw(kind) => {
                let rect = shift(item.rect, offset);
                if !finite_rect(rect) {
                    return false;
                }
                let support = pixel_bounds(rect, scale);
                if actual.clip.is_some_and(|clip| !contains(clip, support))
                    || expected.clip.is_some_and(|clip| !contains(clip, support))
                {
                    return false;
                }
            }
            _ => return false,
        }
    }
    stack.is_empty()
}
pub fn is_draw(kind: Kind) -> bool {
    !matches!(
        kind,
        Kind::kSave
            | Kind::kRestore
            | Kind::kConcat
            | Kind::kClipRect
            | Kind::kClipRoundedRect
            | Kind::kClipOutRoundedRect
            | Kind::kClipPath
            | Kind::kClipOutRect
            | Kind::kSaveLayer
            | Kind::kSaveLayerAlpha
            | Kind::kSaveLayerBlend
            | Kind::kSaveLayerFilter
            | Kind::kBeginMask
            | Kind::kEndMask
            | Kind::kSaveLayerDstIn
    )
}
fn update_state(
    item: &DisplayItem,
    state: &mut State,
    stack: &mut Vec<State>,
    scale: f64,
    external: bool,
) -> Result<(), ReplayUnsupported> {
    match item.r#type {
        Kind::kSave => stack.push(*state),
        // This wrapper belongs to a real Mask DrawingDisplayItem. Its blend
        // is executed once by the effect tree, after source-tile rasterization.
        Kind::kSaveLayerDstIn
            if !external
                && finite_rect(item.rect)
                && item.opacity == 1.0
                && item.blend_mode == PaintBlendMode::kNormal
                && item.filters.is_empty()
                && item.mask_layers.is_empty() =>
        {
            stack.push(*state)
        }
        Kind::kRestore => {
            *state = stack
                .pop()
                .ok_or(ReplayUnsupported("unbalanced-replay-state"))?
        }
        Kind::kConcat => concatenate_affine(state, &item.transform)?,
        Kind::kClipRect => {
            let rect = mapped_rect(item.rect, *state);
            if !finite_rect(rect) {
                return Err(ReplayUnsupported("invalid-record-clip"));
            }
            state.clip = intersect(state.clip, rect);
            if state.cross_axis != (0.0, 0.0) {
                state.complex_clip = true;
            }
        }
        Kind::kClipRoundedRect => {
            state.clip = intersect(state.clip, mapped_rect(item.rect, *state));
            state.complex_clip = true;
        }
        Kind::kClipOutRoundedRect | Kind::kClipPath | Kind::kClipOutRect => {
            state.complex_clip = true
        }
        Kind::kSaveLayer | Kind::kSaveLayerAlpha
            if external
                && finite_rect(item.rect)
                && item.opacity.is_finite()
                && (0.0..=1.0).contains(&item.opacity)
                && item.blend_mode == PaintBlendMode::kNormal
                && item.filters.is_empty()
                && item.mask_layers.is_empty()
                && item.paint_shader.is_none() =>
        {
            // Alpha is applied once by its actual effect surface, not per
            // tile. The parallel scope walk validates effect identity and
            // records the native bounded-device clip for each PaintRecord.
            stack.push(*state);
        }
        Kind::kSaveLayer
        | Kind::kSaveLayerAlpha
        | Kind::kSaveLayerBlend
        | Kind::kSaveLayerFilter
        | Kind::kBeginMask
        | Kind::kEndMask
        | Kind::kSaveLayerDstIn => {
            if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                eprintln!("layer-replay unsupported-flat-state type={:?} external={} node={} rect={:?} opacity={} blend={:?} filters={} masks={} shader={}",
                    item.r#type,external,item.node_id,item.rect,item.opacity,item.blend_mode,
                    item.filters.len(),item.mask_layers.len(),item.paint_shader.is_some());
            }
            return Err(ReplayUnsupported("unsupported-flat-state"));
        }
        _ => {}
    }
    Ok(())
}
fn canonical_item(item: &DisplayItem, delta: (f64, f64)) -> Result<DisplayItem, ReplayUnsupported> {
    let gradient = matches!(
        item.r#type,
        Kind::kDrawGradientRect | Kind::kDrawTiledGradient
    );
    let box_shadow = item.r#type == Kind::kDrawBoxShadow;
    let mask = item.r#type == Kind::kDrawMask;
    if mask {
        validate_mask(item)?;
    }
    if !item.filters.is_empty()
        || (!mask && !item.mask_layers.is_empty())
        || item.blend_mode != PaintBlendMode::kNormal
        || !item.blur_radius.is_finite()
        || item.blur_radius < 0.0
        || (item.blur_radius != 0.0 && !box_shadow)
        || (box_shadow
            && ![item.spread, item.shadow_offset.x, item.shadow_offset.y]
                .iter()
                .all(|value| value.is_finite()))
        || (item.paint_shader.is_some() && !gradient)
    {
        if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
            eprintln!("layer-replay unsupported-record type={:?} node={} rect={:?} blur={} filters={} masks={} blend={:?} shader={:?}",
                item.r#type,item.node_id,item.rect,item.blur_radius,item.filters.len(),item.mask_layers.len(),item.blend_mode,
                item.paint_shader.as_ref().map(|shader|shader.kind));
        }
        return Err(ReplayUnsupported("unsupported-record-resource-or-effect"));
    }
    // compat/canvas.rs::draw_gradient_tile implements linear gradients only.
    // Its start/end are absolute points; shader transform and tile spacing
    // are local/resource quantities and must retain their original values.
    if gradient
        && !item
            .paint_shader
            .as_ref()
            .is_some_and(|shader| shader.kind == PaintShaderKind::kLinearGradient)
    {
        return Err(ReplayUnsupported("unsupported-gradient-kind"));
    }
    if item.r#type == Kind::kDrawGlyphRun && item.transform != TransformMatrix::default() {
        return Err(ReplayUnsupported("transformed-glyph-record"));
    }
    let mut output = item.clone();
    // Only destination positions move. Image source_rect, relative glyph
    // offsets, stroke widths and shadow_offset are resource/vector quantities.
    output.rect = shift(output.rect, delta);
    if !output.inner_rect.is_empty() {
        output.inner_rect = shift(output.inner_rect, delta);
    }
    if !output.tile_rect.is_empty() {
        output.tile_rect = shift(output.tile_rect, delta);
    }
    if gradient {
        let shader = output.paint_shader.as_mut().unwrap();
        shader.start.x += delta.0;
        shader.start.y += delta.1;
        shader.end.x += delta.0;
        shader.end.y += delta.1;
    }
    if mask {
        for layer in &mut output.mask_layers {
            layer.clip_rect = shift(layer.clip_rect, delta);
            layer.tile_rect = shift(layer.tile_rect, delta);
            if let Some(shader) = &mut layer.paint_shader {
                shader.start.x += delta.0;
                shader.start.y += delta.1;
                shader.end.x += delta.0;
                shader.end.y += delta.1;
            }
        }
    }
    if item.r#type == Kind::kDrawGlyphRun {
        output.text_blob_origin.x += delta.0;
        output.text_blob_origin.y += delta.1;
    }
    for point in &mut output.path {
        if point.verb == PaintPathVerb::kClose {
            continue;
        }
        point.point.x += delta.0;
        point.point.y += delta.1;
        if matches!(
            point.verb,
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo | PaintPathVerb::kCubicTo
        ) {
            point.control1.x += delta.0;
            point.control1.y += delta.1;
        }
        if point.verb == PaintPathVerb::kCubicTo {
            point.control2.x += delta.0;
            point.control2.y += delta.1;
        }
    }
    if !finite_rect(output.rect) {
        return Err(ReplayUnsupported("invalid-record-geometry"));
    }
    Ok(output)
}

// A pointer tuple denotes immutable snapshots only while their owning Arcs
// remain alive. It is not a retained identity or a cross-frame content key.
fn property_snapshot_key(state: &PropertyTreeState) -> (usize, usize, usize) {
    (
        Arc::as_ptr(&state.transform) as usize,
        Arc::as_ptr(&state.clip) as usize,
        Arc::as_ptr(&state.effect) as usize,
    )
}
// Compare the exact normalized payload without allocating a new glyph/path
// vector. Exhaustive destructuring makes new DisplayItem fields a compile-time
// obligation rather than silently omitting them from this content proof.
fn translated_item_matches(item: &DisplayItem, delta: (f64, f64), old: &DisplayItem) -> bool {
    if item.r#type == Kind::kDrawMask {
        return canonical_item(item, delta).is_ok_and(|item| item == *old);
    }
    if matches!(item.r#type, Kind::kSave | Kind::kRestore) {
        return item == old;
    }
    let DisplayItem {
        r#type: _,
        phase: _,
        node_id: _,
        fragment_instance_id: _,
        rect: _,
        source_rect: _,
        inner_rect: _,
        tile_rect: _,
        color: _,
        line_style: _,
        decoration_style: _,
        stroke_width: _,
        corner_radius: _,
        inner_corner_radius: _,
        corner_radii: _,
        inner_corner_radii: _,
        dash_intervals: _,
        path: _,
        even_odd: _,
        inverse_winding: _,
        svg_line_cap: _,
        svg_line_join: _,
        dash_offset: _,
        dash_fit_thickness: _,
        miter_limit: _,
        antialias: _,
        non_scaling_stroke: _,
        paint_shader: _,
        blend_mode: _,
        svg_marker_resource_id: _,
        filters: _,
        mask_layers: _,
        round_cap: _,
        resource_id: _,
        repeat_x: _,
        repeat_y: _,
        background_repeat_x: _,
        background_repeat_y: _,
        tile_rule_x: _,
        tile_rule_y: _,
        tile_scale: _,
        tile_phase: _,
        tile_spacing: _,
        shadow_offset: _,
        blur_radius: _,
        spread: _,
        inset: _,
        is_shadow: _,
        shadow_has_opaque_background: _,
        is_text_decoration: _,
        stroke_glyphs: _,
        skip_ink: _,
        opacity: _,
        transform: _,
        font_face_index: _,
        font_variations: _,
        font_size: _,
        baseline: _,
        text_blob_origin: _,
        horizontal: _,
        at_start: _,
        rtl: _,
        synthetic_bold: _,
        synthetic_italic: _,
        font_smoothing: _,
        writing_mode: _,
        glyphs: _,
    } = item;
    if item.r#type != old.r#type {
        return false;
    }
    if item.phase != old.phase {
        return false;
    }
    if item.node_id != old.node_id {
        return false;
    }
    if item.fragment_instance_id != old.fragment_instance_id {
        return false;
    }
    if item.source_rect != old.source_rect {
        return false;
    }
    if item.color != old.color {
        return false;
    }
    if item.line_style != old.line_style {
        return false;
    }
    if item.decoration_style != old.decoration_style {
        return false;
    }
    if item.stroke_width != old.stroke_width {
        return false;
    }
    if item.corner_radius != old.corner_radius {
        return false;
    }
    if item.inner_corner_radius != old.inner_corner_radius {
        return false;
    }
    if item.corner_radii != old.corner_radii {
        return false;
    }
    if item.inner_corner_radii != old.inner_corner_radii {
        return false;
    }
    if item.dash_intervals != old.dash_intervals {
        return false;
    }
    if item.even_odd != old.even_odd {
        return false;
    }
    if item.inverse_winding != old.inverse_winding {
        return false;
    }
    if item.svg_line_cap != old.svg_line_cap {
        return false;
    }
    if item.svg_line_join != old.svg_line_join {
        return false;
    }
    if item.dash_offset != old.dash_offset {
        return false;
    }
    if item.dash_fit_thickness != old.dash_fit_thickness {
        return false;
    }
    if item.miter_limit != old.miter_limit {
        return false;
    }
    if item.antialias != old.antialias {
        return false;
    }
    if item.non_scaling_stroke != old.non_scaling_stroke {
        return false;
    }
    if item.blend_mode != old.blend_mode {
        return false;
    }
    if item.svg_marker_resource_id != old.svg_marker_resource_id {
        return false;
    }
    if item.filters != old.filters {
        return false;
    }
    if item.mask_layers != old.mask_layers {
        return false;
    }
    if item.round_cap != old.round_cap {
        return false;
    }
    if item.resource_id != old.resource_id {
        return false;
    }
    if item.repeat_x != old.repeat_x {
        return false;
    }
    if item.repeat_y != old.repeat_y {
        return false;
    }
    if item.background_repeat_x != old.background_repeat_x {
        return false;
    }
    if item.background_repeat_y != old.background_repeat_y {
        return false;
    }
    if item.tile_rule_x != old.tile_rule_x {
        return false;
    }
    if item.tile_rule_y != old.tile_rule_y {
        return false;
    }
    if item.tile_scale != old.tile_scale {
        return false;
    }
    if item.tile_phase != old.tile_phase {
        return false;
    }
    if item.tile_spacing != old.tile_spacing {
        return false;
    }
    if item.shadow_offset != old.shadow_offset {
        return false;
    }
    if item.blur_radius != old.blur_radius {
        return false;
    }
    if item.spread != old.spread {
        return false;
    }
    if item.inset != old.inset {
        return false;
    }
    if item.is_shadow != old.is_shadow {
        return false;
    }
    if item.shadow_has_opaque_background != old.shadow_has_opaque_background {
        return false;
    }
    if item.is_text_decoration != old.is_text_decoration {
        return false;
    }
    if item.stroke_glyphs != old.stroke_glyphs {
        return false;
    }
    if item.skip_ink != old.skip_ink {
        return false;
    }
    if item.opacity != old.opacity {
        return false;
    }
    if item.transform != old.transform {
        return false;
    }
    if item.font_face_index != old.font_face_index {
        return false;
    }
    if item.font_variations != old.font_variations {
        return false;
    }
    if item.font_size != old.font_size {
        return false;
    }
    if item.baseline != old.baseline {
        return false;
    }
    if item.horizontal != old.horizontal {
        return false;
    }
    if item.at_start != old.at_start {
        return false;
    }
    if item.rtl != old.rtl {
        return false;
    }
    if item.synthetic_bold != old.synthetic_bold {
        return false;
    }
    if item.synthetic_italic != old.synthetic_italic {
        return false;
    }
    if item.font_smoothing != old.font_smoothing {
        return false;
    }
    if item.writing_mode != old.writing_mode {
        return false;
    }
    if item.glyphs != old.glyphs {
        return false;
    }
    if shift(item.rect, delta) != old.rect {
        return false;
    }
    if item.r#type == Kind::kClipRect {
        // This state op changes only rect in the canonical builder.
        return item.inner_rect == old.inner_rect
            && item.tile_rect == old.tile_rect
            && item.text_blob_origin == old.text_blob_origin
            && item.path == old.path
            && item.paint_shader == old.paint_shader;
    }
    if item.inner_rect.is_empty() {
        if item.inner_rect != old.inner_rect {
            return false;
        }
    } else if shift(item.inner_rect, delta) != old.inner_rect {
        return false;
    }
    if item.tile_rect.is_empty() {
        if item.tile_rect != old.tile_rect {
            return false;
        }
    } else if shift(item.tile_rect, delta) != old.tile_rect {
        return false;
    }
    let mut origin = item.text_blob_origin;
    if item.r#type == Kind::kDrawGlyphRun {
        origin.x += delta.0;
        origin.y += delta.1;
    }
    if origin != old.text_blob_origin {
        return false;
    }
    // Nonzero gradient offsets alter shader endpoints. Keep those records on
    // the existing builder rather than duplicating its shader field contract.
    if delta != (0.0, 0.0) && item.paint_shader.is_some() {
        return false;
    }
    if item.paint_shader != old.paint_shader || item.path.len() != old.path.len() {
        return false;
    }
    item.path.iter().zip(&old.path).all(|(source, old)| {
        let mut point = source.clone();
        if point.verb != PaintPathVerb::kClose {
            point.point.x += delta.0;
            point.point.y += delta.1;
            if matches!(
                point.verb,
                PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo | PaintPathVerb::kCubicTo
            ) {
                point.control1.x += delta.0;
                point.control1.y += delta.1;
            }
            if point.verb == PaintPathVerb::kCubicTo {
                point.control2.x += delta.0;
                point.control2.y += delta.1;
            }
        }
        point == *old
    })
}

// Borrow current chunk metadata and override only its recording coordinate
// snapshot. Cache hits need neither a projected chunk Vec nor cloned Arc handles.
#[derive(Clone, Copy)]
struct RecordingChunks<'a> {
    chunks: &'a [PaintChunk],
    properties: Option<&'a [PropertyTreeState]>,
}
#[derive(Clone, Copy)]
struct RecordingChunk<'a> {
    chunk: &'a PaintChunk,
    properties: &'a PropertyTreeState,
}
impl std::ops::Deref for RecordingChunk<'_> {
    type Target = PaintChunk;
    fn deref(&self) -> &PaintChunk {
        self.chunk
    }
}
impl<'a> RecordingChunks<'a> {
    fn current(chunks: &'a [PaintChunk]) -> Self {
        Self {
            chunks,
            properties: None,
        }
    }
    fn len(self) -> usize {
        self.chunks.len()
    }
    fn is_empty(self) -> bool {
        self.chunks.is_empty()
    }
    fn iter(self) -> impl ExactSizeIterator<Item = RecordingChunk<'a>> {
        self.chunks
            .iter()
            .enumerate()
            .map(move |(index, chunk)| RecordingChunk {
                chunk,
                properties: self
                    .properties
                    .map_or(&chunk.properties, |states| &states[index]),
            })
    }
    fn snapshot(self) -> Vec<PaintChunk> {
        self.iter()
            .map(|view| PaintChunk {
                properties: view.properties.clone(),
                ..view.chunk.clone()
            })
            .collect()
    }
}
fn recording_chunks(list: &PaintArtifact) -> Result<RecordingChunks<'_>, ReplayUnsupported> {
    let current = RecordingChunks::current(&list.chunks);
    if list.recorded_properties.is_empty() {
        return Ok(current);
    }
    if list.recorded_properties.len() != list.chunks.len() {
        return Err(ReplayUnsupported("invalid-recorded-property-count"));
    }
    if list
        .chunks
        .iter()
        .zip(&list.recorded_properties)
        .all(|(chunk, recorded)| {
            property_snapshot_key(&chunk.properties) == property_snapshot_key(recorded)
        })
    {
        return Ok(current);
    }
    // Scrollbar records may have just been re-recorded and ranges may have
    // moved. Keep current metadata; project only recording coordinates.
    Ok(RecordingChunks {
        chunks: &list.chunks,
        properties: Some(&list.recorded_properties),
    })
}
fn same_optional_node<T>(
    a: Option<&Arc<T>>,
    b: Option<&Arc<T>>,
    identity: impl Fn(&T, &T) -> bool,
) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => identity(a, b),
        (None, None) => true,
        _ => false,
    }
}
fn validate_property_only_update(
    recording: RecordingChunks<'_>,
    current: &[PaintChunk],
    scale: f64,
) -> Result<(), ReplayUnsupported> {
    let reject = ReplayUnsupported("property-update-changes-recording-space");
    // Chunk metadata/ranges and real property identities must remain native.
    if recording.len() != current.len()
        || !recording
            .iter()
            .zip(current)
            .all(|(a, b)| same_chunk_metadata(a.chunk, b, false) && a.properties == &b.properties)
    {
        return Err(reject);
    }
    if recording
        .iter()
        .zip(current)
        .all(|(a, b)| property_snapshot_key(&a.properties) == property_snapshot_key(&b.properties))
    {
        return Ok(());
    }
    let old = paint::paint_property_tree::PaintPropertyTrees::from_states(
        recording.iter().map(|c| c.properties),
    );
    let new = paint::paint_property_tree::PaintPropertyTrees::from_states(
        current.iter().map(|c| &c.properties),
    );
    if old.transforms.keys().ne(new.transforms.keys())
        || old.clips.keys().ne(new.clips.keys())
        || old.effects.keys().ne(new.effects.keys())
        || old.scrolls.keys().ne(new.scrolls.keys())
    {
        return Err(reject);
    }
    let root = PropertyTreeState::default();
    for (&id, a) in &old.transforms {
        let b = &new.transforms[&id];
        if !a.lifecycle.same_node(&b.lifecycle)
            || a.origin != b.origin
            || a.direct_compositing_reasons != b.direct_compositing_reasons
            || !same_optional_node(a.parent.as_ref(), b.parent.as_ref(), |a, b| {
                a.lifecycle.same_node(&b.lifecycle)
            })
            || !same_optional_node(a.scroll.as_ref(), b.scroll.as_ref(), |a, b| {
                a.lifecycle.same_node(&b.lifecycle)
            })
        {
            return Err(reject);
        }
        if a.matrix != b.matrix {
            let real_scroll = a.scroll.as_ref().is_some_and(|scroll| {
                root.transform
                    .scroll
                    .as_ref()
                    .is_none_or(|sentinel| !scroll.lifecycle.same_node(&sentinel.lifecycle))
            });
            if !real_scroll
                || translation(&a.matrix, scale).is_err()
                || translation(&b.matrix, scale).is_err()
            {
                return Err(reject);
            }
        }
    }
    for (&id, a) in &old.clips {
        let b = &new.clips[&id];
        if !a.lifecycle.same_node(&b.lifecycle)
            || a.rect != b.rect
            || a.radii != b.radii
            || a.clip_path != b.clip_path
            || a.clip_path_even_odd != b.clip_path_even_odd
            || !a
                .local_transform_space
                .lifecycle
                .same_node(&b.local_transform_space.lifecycle)
            || !same_optional_node(a.parent.as_ref(), b.parent.as_ref(), |a, b| {
                a.lifecycle.same_node(&b.lifecycle)
            })
            || !same_optional_node(
                a.pixel_moving_filter.as_ref(),
                b.pixel_moving_filter.as_ref(),
                |a, b| a.lifecycle.same_node(&b.lifecycle),
            )
        {
            return Err(reject);
        }
    }
    for (&id, a) in &old.effects {
        let b = &new.effects[&id];
        if !a.lifecycle.same_node(&b.lifecycle)
            || a.opacity != b.opacity
            || a.blend_mode != b.blend_mode
            || a.filters != b.filters
            || a.isolates_blending != b.isolates_blending
            || a.has_mask != b.has_mask
            || a.direct_compositing_reasons != b.direct_compositing_reasons
            || !a
                .local_transform_space
                .lifecycle
                .same_node(&b.local_transform_space.lifecycle)
            || !same_optional_node(a.parent.as_ref(), b.parent.as_ref(), |a, b| {
                a.lifecycle.same_node(&b.lifecycle)
            })
            || !same_optional_node(a.output_clip.as_ref(), b.output_clip.as_ref(), |a, b| {
                a.lifecycle.same_node(&b.lifecycle)
            })
        {
            return Err(reject);
        }
    }
    for (&id, a) in &old.scrolls {
        let b = &new.scrolls[&id];
        if !a.lifecycle.same_node(&b.lifecycle)
            || a.container_rect != b.container_rect
            || a.contents_rect != b.contents_rect
            || a.user_scrollable_horizontal != b.user_scrollable_horizontal
            || a.user_scrollable_vertical != b.user_scrollable_vertical
            || !same_optional_node(a.parent.as_ref(), b.parent.as_ref(), |a, b| {
                a.lifecycle.same_node(&b.lifecycle)
            })
            || !same_optional_node(
                a.overflow_clip.as_ref(),
                b.overflow_clip.as_ref(),
                |a, b| a.lifecycle.same_node(&b.lifecycle),
            )
        {
            return Err(reject);
        }
    }
    let mut checked = BTreeSet::new();
    for (a, b) in recording.iter().zip(current) {
        if !checked.insert((
            property_snapshot_key(&a.properties),
            property_snapshot_key(&b.properties),
        )) {
            continue;
        }
        let old = crate::raster_properties(&a.properties).map_err(property_error)?;
        let new = crate::raster_properties(&b.properties).map_err(property_error)?;
        if old != new {
            return Err(reject);
        }
        let old_origin = resolved(&old, scale)?.translation;
        let new_origin = resolved(&new, scale)?.translation;
        let old_clip = removed_clip_bounds(&a.properties.clip, &old.clip)?
            .map(|r| pixel_bounds(shift(r, (-old_origin.0, -old_origin.1)), scale));
        let new_clip = removed_clip_bounds(&b.properties.clip, &new.clip)?
            .map(|r| pixel_bounds(shift(r, (-new_origin.0, -new_origin.1)), scale));
        if old_clip != new_clip {
            return Err(reject);
        }
        // The retained raster clip's phase is local to its anchor. Root-space
        // integer/fractional classification can change on fractional scroll,
        // while these actual baked clip shapes remain exactly unchanged.
        let mut old_node = &a.properties.clip;
        let mut new_node = &b.properties.clip;
        while !old_node.lifecycle.same_node(&old.clip.lifecycle) {
            let old_parent = old_node.parent.as_ref().ok_or(reject)?;
            let new_parent = new_node.parent.as_ref().ok_or(reject)?;
            let old_shape = removed_clip_bounds(old_node, old_parent)?
                .map(|r| shift(r, (-old_origin.0, -old_origin.1)));
            let new_shape = removed_clip_bounds(new_node, new_parent)?
                .map(|r| shift(r, (-new_origin.0, -new_origin.1)));
            if old_shape != new_shape {
                return Err(reject);
            }
            old_node = old_parent;
            new_node = new_parent;
        }
    }
    Ok(())
}
struct CanonicalSourceSnapshot {
    items: Arc<Vec<DisplayItem>>,
    records: Arc<[RecordedDisplayItem]>,
    chunks: Arc<[PaintChunk]>,
    recording_revision: u64,
}
fn same_recording_metadata(a: &RecordedDisplayItem, b: &RecordedDisplayItem) -> bool {
    // Scroll hit-test/compositing metadata is refreshed without changing this
    // flat recording. The recording coordinate basis comes from chunks above.
    // Keep every real semantic field and the actual scroll node identity.
    let RecordedDisplayItem {
        kind,
        id,
        visual_rect,
        visual_rect_is_accurate,
        draws_content,
        raster_effect_outset,
        record_begin,
        record_end,
        scroll_translation,
    } = a;
    *kind == b.kind
        && *id == b.id
        && *visual_rect == b.visual_rect
        && *visual_rect_is_accurate == b.visual_rect_is_accurate
        && *draws_content == b.draws_content
        && *raster_effect_outset == b.raster_effect_outset
        && *record_begin == b.record_begin
        && *record_end == b.record_end
        && same_optional_node(
            scroll_translation.as_ref(),
            b.scroll_translation.as_ref(),
            |a, b| a.lifecycle.same_node(&b.lifecycle),
        )
}
fn same_chunk_metadata(a: &PaintChunk, b: &PaintChunk, ignore_cached_provenance: bool) -> bool {
    // Exhaustive destructuring requires every future semantic field to join
    // this proof. Property snapshots are compared separately by each caller.
    let PaintChunk {
        id,
        is_cacheable,
        client_is_just_created,
        begin_index,
        end_index,
        bounds,
        drawable_bounds,
        rect_known_to_be_opaque,
        bounds_are_complete,
        raster_effect_outset,
        properties: _,
        is_moved_from_cached_subsequence,
        effectively_invisible,
    } = a;
    *id == b.id
        && *is_cacheable == b.is_cacheable
        && *client_is_just_created == b.client_is_just_created
        && *begin_index == b.begin_index
        && *end_index == b.end_index
        && *bounds == b.bounds
        && *drawable_bounds == b.drawable_bounds
        && *rect_known_to_be_opaque == b.rect_known_to_be_opaque
        && *bounds_are_complete == b.bounds_are_complete
        && *raster_effect_outset == b.raster_effect_outset
        && *effectively_invisible == b.effectively_invisible
        && (ignore_cached_provenance
            || *is_moved_from_cached_subsequence == b.is_moved_from_cached_subsequence)
}
fn same_recording_chunk(a: &PaintChunk, b: RecordingChunk<'_>) -> bool {
    same_chunk_metadata(a, b.chunk, true) && &a.properties == b.properties
}
impl CanonicalSourceSnapshot {
    fn same_trusted_recording(&self, list: &PaintArtifact) -> bool {
        self.recording_revision != 0
            && self.recording_revision == list.recording_revision
            && Arc::ptr_eq(&self.items, &list.items)
    }
    fn same_recording_context(&self, list: &PaintArtifact, chunks: RecordingChunks<'_>) -> bool {
        // Reject changed topology/metadata before walking large glyph/path
        // payloads. Distinct command backings require complete field equality;
        // sharing the immutable backing proves the same operations directly.
        if self.items.len() != list.items.len()
            || self.records.len() != list.display_items.len()
            || self.chunks.len() != chunks.len()
            || !self
                .records
                .iter()
                .zip(&list.display_items)
                .all(|(a, b)| same_recording_metadata(a, b))
            || !self
                .chunks
                .iter()
                .zip(chunks.iter())
                .all(|(a, b)| same_recording_chunk(a, b))
        {
            return false;
        }
        // Immutable source snapshots stay alive for this comparison. Memoize
        // only successful COMPLETE value proofs for each old/new snapshot pair;
        // stable node identity by itself cannot hide changed ancestor values.
        let mut verified = BTreeSet::new();
        for (old, new) in self.chunks.iter().zip(chunks.iter()) {
            let old_key = property_snapshot_key(&old.properties);
            let new_key = property_snapshot_key(&new.properties);
            if old_key != new_key
                && verified.insert((old_key, new_key))
                && !old.properties.same_values(&new.properties)
            {
                return false;
            }
        }
        true
    }
    fn matches(&self, list: &PaintArtifact, chunks: RecordingChunks<'_>) -> bool {
        self.same_recording_context(list, chunks)
            && (Arc::ptr_eq(&self.items, &list.items)
                || self
                    .items
                    .iter()
                    .zip(list.items.iter())
                    .all(|(old, new)| old == new))
    }
    fn new(list: &PaintArtifact, chunks: RecordingChunks<'_>, _previous: Option<&Self>) -> Self {
        // PaintRecord source storage is already immutable. Retaining it avoids
        // cloning every glyph/path payload when publishing cache provenance.
        Self {
            items: list.items.clone(),
            records: Arc::from(list.display_items.clone()),
            chunks: Arc::from(chunks.snapshot()),
            recording_revision: list.recording_revision,
        }
    }
}
struct CachedCanonicalContent {
    source: CanonicalSourceSnapshot,
    scale: f64,
    records: Arc<[RasterRecordContent]>,
}
/// Canonical geometry is independent of decoded resource pixels. Those are
/// invalidated by layer_tile's per-record actual-resource dependencies. This
/// PaintEngine artifacts use their committed recording revision plus the same
/// immutable command backing. Externally assembled artifacts (revision zero)
/// retain the complete command, metadata and property-value proof, including
/// mask payloads and node revisions, with no ID/hash-only shortcut.
/// The cache stores only proven geometry/records, independent of raster success.
#[derive(Default)]
pub struct CanonicalRasterContentCache {
    published: Option<CachedCanonicalContent>,
    pending: Option<CachedCanonicalContent>,
    last_fast_hit: bool,
}
impl CanonicalRasterContentCache {
    pub fn published_revision(&self) -> u64 {
        self.published
            .as_ref()
            .map_or(0, |entry| entry.source.recording_revision)
    }
    pub fn published_shares_items(&self, list: &PaintArtifact) -> bool {
        self.published
            .as_ref()
            .is_some_and(|entry| Arc::ptr_eq(&entry.source.items, &list.items))
    }
    pub fn prepare(
        &mut self,
        list: &PaintArtifact,
        scale: f64,
    ) -> Result<Arc<[RasterRecordContent]>, ReplayUnsupported> {
        self.pending = None;
        self.last_fast_hit = false;
        if let Some(previous) = &self.published {
            if previous.scale == scale && previous.source.same_trusted_recording(list) {
                // PaintEngine has already proved that this retained update only
                // changes compositor properties. This is the Rust equivalent
                // of consuming the same committed display-item recording: do
                // not re-walk every item/chunk to rediscover its identity.
                self.last_fast_hit = true;
                return Ok(Arc::clone(&previous.records));
            }
        }
        let chunks = recording_chunks(list)?;
        self.prepare_with_chunk_view(list, scale, chunks, chunks.properties.is_some())
    }
    /// Flat commands retain the coordinate system of the properties present
    /// when they were recorded. A compositor-only update supplies that actual
    /// snapshot separately; current chunk properties still drive layer_tile.
    pub fn prepare_with_recording_chunks(
        &mut self,
        list: &PaintArtifact,
        scale: f64,
        recording_chunks: Option<&[PaintChunk]>,
    ) -> Result<Arc<[RasterRecordContent]>, ReplayUnsupported> {
        self.pending = None;
        self.last_fast_hit = false;
        let chunks = RecordingChunks::current(recording_chunks.unwrap_or(&list.chunks));
        self.prepare_with_chunk_view(list, scale, chunks, recording_chunks.is_some())
    }
    fn prepare_with_chunk_view(
        &mut self,
        list: &PaintArtifact,
        scale: f64,
        chunks: RecordingChunks<'_>,
        property_update: bool,
    ) -> Result<Arc<[RasterRecordContent]>, ReplayUnsupported> {
        self.pending = None;
        if property_update {
            validate_property_only_update(chunks, &list.chunks, scale)?;
        }
        if let Some(previous) = &self.published {
            if previous.scale == scale && previous.source.matches(list, chunks) {
                // Share the immutable directory as well as the record payloads.
                // Previous source already passed the complete walk.
                return Ok(Arc::clone(&previous.records));
            }
        }
        let content: Arc<[RasterRecordContent]> = prepare_raster_content(
            list,
            chunks,
            scale,
            self.published.as_ref().map(|old| old.records.as_ref()),
            self.published
                .as_ref()
                .filter(|old| old.scale == scale)
                .map(|old| &old.source),
        )?
        .into();
        self.pending = Some(CachedCanonicalContent {
            source: CanonicalSourceSnapshot::new(
                list,
                chunks,
                self.published.as_ref().map(|old| &old.source),
            ),
            scale,
            records: Arc::clone(&content),
        });
        Ok(content)
    }
    pub fn commit(&mut self) {
        if let Some(current) = self.pending.take() {
            self.published = Some(current);
        }
    }
    pub fn clear(&mut self) {
        self.pending = None;
        self.published = None;
    }
    pub fn last_fast_hit(&self) -> bool {
        self.last_fast_hit
    }
}

#[cfg(test)]
mod canonical_directory_tests {
    use super::*;
    use paint::display_item_id::{DisplayItemId, DisplayItemIdType};
    use paint::paint_engine::{RasterEffectOutset, RecordedDisplayItemKind};

    #[test]
    fn borrowed_recording_view_reuses_cache_and_preserves_rejections() {
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 64.0,
            height: 64.0,
        };
        let id = DisplayItemId {
            client_id: 1,
            fragment: 0,
            r#type: DisplayItemIdType::kBoxDecorationBackground,
        };
        let mut artifact = PaintArtifact {
            items: Arc::new(vec![DisplayItem {
                r#type: Kind::kDrawRect,
                rect,
                ..Default::default()
            }]),
            display_items: vec![RecordedDisplayItem {
                kind: RecordedDisplayItemKind::Drawing,
                id,
                visual_rect: rect,
                visual_rect_is_accurate: true,
                draws_content: true,
                raster_effect_outset: RasterEffectOutset::kNone,
                record_begin: 0,
                record_end: 1,
                scroll_translation: None,
            }],
            chunks: vec![PaintChunk {
                id,
                is_cacheable: true,
                end_index: 1,
                bounds: rect,
                drawable_bounds: rect,
                ..Default::default()
            }],
            ..Default::default()
        };
        artifact.recorded_properties = vec![artifact.chunks[0].properties.clone()];
        let mut cache = CanonicalRasterContentCache::default();
        let original = cache.prepare(&artifact, 1.0).unwrap();
        cache.commit();
        // A new immutable current snapshot still names the same property entity
        // and has equal values; recording coordinates retain the old snapshot.
        artifact.chunks[0].properties.transform =
            Arc::new((*artifact.chunks[0].properties.transform).clone());
        artifact.chunks[0].is_moved_from_cached_subsequence = true;
        let old_transform = &artifact.recorded_properties[0].transform;
        let count = Arc::strong_count(old_transform);
        {
            let view = recording_chunks(&artifact).unwrap();
            assert!(std::ptr::eq(
                view.iter().next().unwrap().properties,
                &artifact.recorded_properties[0]
            ));
            assert_eq!(
                Arc::strong_count(old_transform),
                count,
                "projection borrows the snapshot"
            );
        }
        assert!(Arc::ptr_eq(
            &original,
            &cache.prepare(&artifact, 1.0).unwrap()
        ));
        // The public override API must still reject changed chunk metadata.
        let mut wrong = artifact.chunks.clone();
        wrong[0].bounds.width = 32.0;
        assert_eq!(
            cache
                .prepare_with_recording_chunks(&artifact, 1.0, Some(&wrong))
                .err(),
            Some(ReplayUnsupported("property-update-changes-recording-space"))
        );
        // Equal node identity cannot hide an effect value change.
        let mut changed = (*artifact.chunks[0].properties.effect).clone();
        changed.opacity = 0.5;
        artifact.chunks[0].properties.effect = Arc::new(changed);
        assert_eq!(
            cache.prepare(&artifact, 1.0).err(),
            Some(ReplayUnsupported("property-update-changes-recording-space"))
        );
        assert_eq!(
            original[0]
                .items
                .iter()
                .find(|op| op.r#type == Kind::kDrawRect)
                .unwrap()
                .rect
                .width,
            64.0
        );
    }

    #[test]
    fn cache_shares_unchanged_directory_and_preserves_changed_snapshots() {
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 64.0,
            height: 64.0,
        };
        let id = DisplayItemId {
            client_id: 1,
            fragment: 0,
            r#type: DisplayItemIdType::kBoxDecorationBackground,
        };
        let mut artifact = PaintArtifact {
            items: Arc::new(vec![DisplayItem {
                r#type: Kind::kDrawRect,
                rect,
                ..Default::default()
            }]),
            display_items: vec![RecordedDisplayItem {
                kind: RecordedDisplayItemKind::Drawing,
                id,
                visual_rect: rect,
                visual_rect_is_accurate: true,
                draws_content: true,
                raster_effect_outset: RasterEffectOutset::kNone,
                record_begin: 0,
                record_end: 1,
                scroll_translation: None,
            }],
            chunks: vec![PaintChunk {
                id,
                is_cacheable: true,
                end_index: 1,
                bounds: rect,
                drawable_bounds: rect,
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut cache = CanonicalRasterContentCache::default();
        let original = cache.prepare(&artifact, 1.0).unwrap();
        cache.commit();
        let unchanged = cache.prepare(&artifact, 1.0).unwrap();
        assert!(Arc::ptr_eq(&original, &unchanged));

        // Equal operations on distinct backing still pass the value proof.
        artifact.items = Arc::new((*artifact.items).clone());
        let equivalent = cache.prepare(&artifact, 1.0).unwrap();
        assert!(Arc::ptr_eq(&original, &equivalent));

        Arc::make_mut(&mut artifact.items)[0].rect.width = 32.0;
        let changed = cache.prepare(&artifact, 1.0).unwrap();
        assert!(!Arc::ptr_eq(&original, &changed));
        let draw_width = |records: &[RasterRecordContent]| {
            records[0]
                .items
                .iter()
                .find(|item| item.r#type == Kind::kDrawRect)
                .unwrap()
                .rect
                .width
        };
        assert_eq!(
            draw_width(&original),
            64.0,
            "retained frame remains immutable"
        );
        assert_eq!(draw_width(&changed), 32.0, "changed input is lowered again");
        cache.commit();
        let changed_again = cache.prepare(&artifact, 1.0).unwrap();
        assert!(Arc::ptr_eq(&changed, &changed_again));
    }
}
/// Normalize supported records once, before layer/tile bounds and cache keys
/// are computed. Actual flat state minus property state removes the adapter's
/// already-folded scroll displacement, including nested transforms/resets.
#[allow(non_snake_case)]
pub fn PrepareRasterContent(
    list: &PaintArtifact,
    scale: f64,
) -> Result<Vec<RasterRecordContent>, ReplayUnsupported> {
    PrepareRasterContentWithPrevious(list, scale, None)
}

/// Reuse only an exactly equal, already normalized record. No hash or address
/// comparison can hide changed glyph/color/resource payload. Callers retain
/// previous records only after successful raster and final composition.
#[allow(non_snake_case)]
pub fn PrepareRasterContentWithPrevious(
    list: &PaintArtifact,
    scale: f64,
    previous: Option<&[RasterRecordContent]>,
) -> Result<Vec<RasterRecordContent>, ReplayUnsupported> {
    let chunks = recording_chunks(list)?;
    if chunks.properties.is_some() {
        validate_property_only_update(chunks, &list.chunks, scale)?;
    }
    prepare_raster_content(list, chunks, scale, previous, None)
}
fn prepare_raster_content(
    list: &PaintArtifact,
    chunks: RecordingChunks<'_>,
    scale: f64,
    previous: Option<&[RasterRecordContent]>,
    previous_source: Option<&CanonicalSourceSnapshot>,
) -> Result<Vec<RasterRecordContent>, ReplayUnsupported> {
    if !scale.is_finite() || scale <= 0.0 || scale as f32 as f64 != scale {
        return Err(ReplayUnsupported("unsupported-raster-scale"));
    }
    // Streaming layout can emit only balanced Save/Clip/Restore wrappers
    // before any drawing exists. An empty PaintArtifact is legitimate: let
    // the state-stack walk below validate it instead of inventing an item.
    if (list.display_items.is_empty() || chunks.is_empty())
        && list.items.iter().any(|item| is_draw(item.r#type))
    {
        if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
            eprintln!(
                "layer-replay missing-semantic-records flat={} records={} chunks={}",
                list.items.len(),
                list.display_items.len(),
                list.chunks.len()
            );
            for (index, item) in list.items.iter().take(16).enumerate() {
                eprintln!(
                    "layer-replay flat[{index}] type={:?} node={} phase={:?}",
                    item.r#type, item.node_id, item.phase
                );
            }
        }
        return Err(ReplayUnsupported("missing-semantic-records"));
    }
    // SkCanvas.cpp get_layer_mapping_and_bounds currently restricts the
    // bounded device to a nonempty saveLayer hint. Keep that real restriction,
    // but never reuse its baked record clip when a record scrolls relative to
    // the layer's transform space. New PaintContext groups omit this hint;
    // their only hard limits are the actual clip/effect property chains.
    let bounded_property_update = chunks.iter().zip(&list.chunks).any(|(old, new)| {
        property_snapshot_key(&old.properties) != property_snapshot_key(&new.properties)
    }) && list.items.iter().any(|item| {
        matches!(item.r#type, Kind::kSaveLayer | Kind::kSaveLayerAlpha) && !item.rect.is_empty()
    });
    let current_trees = bounded_property_update.then(|| {
        paint::paint_property_tree::PaintPropertyTrees::from_states(
            list.chunks.iter().map(|c| &c.properties),
        )
    });
    let mut current_origins = bounded_property_update.then(|| vec![None; list.display_items.len()]);
    if let Some(origins) = &mut current_origins {
        let mut resolved = BTreeMap::new();
        for chunk in &list.chunks {
            let key = property_snapshot_key(&chunk.properties);
            let origin = *match resolved.entry(key) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let raster =
                        crate::raster_properties(&chunk.properties).map_err(property_error)?;
                    entry.insert(
                        crate::resolved_raster_properties(&raster, scale)
                            .map_err(property_error)?
                            .0,
                    )
                }
            };
            let start = chunk.begin_index as usize;
            let end = chunk.end_index as usize;
            if start > end || end > origins.len() {
                return Err(ReplayUnsupported("invalid-chunk-range"));
            }
            for slot in &mut origins[start..end] {
                *slot = Some(origin);
            }
        }
    }
    let mut bounded_scopes = BTreeSet::new();
    let mut mask_records = vec![false; list.display_items.len()];
    let mut properties = vec![None; list.display_items.len()];
    let mut record_clips = vec![None; list.display_items.len()];
    let mut record_rounded = vec![Arc::<[RoundedClip]>::from([]); list.display_items.len()];
    let mut baked_rounded = vec![Arc::<[RoundedClip]>::from([]); list.display_items.len()];
    let mut expected_effects =
        vec![Arc::<[Arc<EffectPaintPropertyNode>]>::from([]); list.display_items.len()];
    // Frame-local immutable snapshot identity only. All source Arcs stay alive
    // in list.chunks throughout this call; addresses cannot be recycled, and
    // scale is fixed/validated above. No cross-frame ID/hash cache is involved.
    type ResolvedChunk = (
        State,
        Option<PaintRect>,
        Arc<[RoundedClip]>,
        Arc<[RoundedClip]>,
        Arc<[Arc<EffectPaintPropertyNode>]>,
    );
    let mut resolved_chunks: BTreeMap<(usize, usize, usize), ResolvedChunk> = BTreeMap::new();
    for chunk in chunks.iter() {
        let start = chunk.begin_index as usize;
        let end = chunk.end_index as usize;
        if start > end || end > properties.len() {
            return Err(ReplayUnsupported("invalid-chunk-range"));
        }
        for index in start..end {
            mask_records[index] = chunk.properties.effect.is_mask;
        }
        let key = property_snapshot_key(&chunk.properties);
        // Borrow the resolved tuple in place. Only the per-record arrays need
        // Arc handles; neither cache hits nor insertions clone the whole tuple.
        let (state, record_clip, rounded, baked, effects) = match resolved_chunks.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let lowered = crate::raster_properties(&chunk.properties).map_err(|error| {
                    diagnose_property_clip(chunk, "raster-upcast", property_error(error))
                })?;
                let mut effects = Vec::new();
                let mut effect = Some(chunk.properties.effect.clone());
                while let Some(current) = effect {
                    effect = current.parent.clone();
                    if current.opacity != 1.0 {
                        effects.push(current);
                    }
                }
                effects.reverse();
                let effects: Arc<[Arc<EffectPaintPropertyNode>]> = Arc::from(effects);
                let layer_state = resolved(&lowered, scale)
                    .map_err(|error| diagnose_property_clip(chunk, "layer-resolution", error))?;
                // Validate the flat ABI against its ORIGINAL clip even when an
                // ordinary SVG clip is upcast to an ancestor for layer merging. The
                // removed clip becomes a real record op; it is never silently omitted.
                let original = PropertyTreeState {
                    transform: lowered.transform.clone(),
                    clip: chunk.properties.clip.clone(),
                    effect: chunk.properties.effect.clone(),
                };
                let (translation, clip) = crate::resolved_record_clip_bounds(&original, scale)
                    .map_err(|error| {
                        diagnose_property_clip(
                            chunk,
                            "original-record-resolution",
                            property_error(error),
                        )
                    })?;
                let state = State {
                    translation,
                    clip,
                    ..Default::default()
                };
                let rounded = rounded_clips_between(&original.clip, None, scale)?;
                let baked = rounded_clips_between_in_raster_space(
                    &original.clip,
                    Some(&lowered.clip),
                    scale,
                    layer_state.translation,
                )?;
                let record_clip = removed_clip_bounds(&original.clip, &lowered.clip)?.map(|root| {
                    // Enclose in layer coordinates, not viewport/root pixels: a
                    // fractional scroll must not change cached clip raster phase.
                    let local = shift(
                        root,
                        (-layer_state.translation.0, -layer_state.translation.1),
                    );
                    pixel_bounds(local, scale)
                });
                entry.insert((state, record_clip, rounded, baked, effects))
            }
        };
        for index in start..end {
            if properties[index].replace(*state).is_some() {
                return Err(ReplayUnsupported("overlapping-chunk-range"));
            }
            record_clips[index] = *record_clip;
            record_rounded[index] = rounded.clone();
            baked_rounded[index] = baked.clone();
            expected_effects[index] = effects.clone();
        }
    }
    let mut owners = vec![None; list.items.len()];
    let mut begins: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let mut ends: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (index, record) in list.display_items.iter().enumerate() {
        if record.record_begin > record.record_end
            || record.record_end > list.items.len()
            || properties[index].is_none()
        {
            return Err(ReplayUnsupported("invalid-semantic-record-range"));
        }
        begins.entry(record.record_begin).or_default().push(index);
        ends.entry(record.record_end).or_default().push(index);
        for owner in &mut owners[record.record_begin..record.record_end] {
            if owner.replace(index).is_some() {
                return Err(ReplayUnsupported("overlapping-semantic-records"));
            }
        }
    }
    // A native scrollbar can change while all other PaintRecords and their
    // recording coordinate provenance remain untouched. Prove the complete
    // external wrapper stream too; equal raw ops alone cannot prove its CTM.
    let unchanged_recording_context = previous_source.is_some_and(|old| {
        old.same_recording_context(list, chunks)
            && (Arc::ptr_eq(&old.items, &list.items)
                || list.items.iter().enumerate().all(|(cursor, item)| {
                    owners[cursor].is_some() || old.items.get(cursor).is_some_and(|old| old == item)
                }))
    });
    let mut state = State::default();
    let mut stack = Vec::new();
    let mut starts = vec![None; list.display_items.len()];
    // Flat cursor identifies only this frame's source wrapper scope. Its
    // bidirectional binding is to a real retained effect entity, never a new
    // compositing ID or a product of per-layer alpha values.
    let mut alpha_scopes: Vec<(usize, u64, f32)> = Vec::new();
    let mut device_clip = None;
    let mut saved_effects = Vec::new();
    let mut rounded_scopes = Vec::new();
    let mut unsupported_complex = false;
    let mut shape_starts = vec![None; list.display_items.len()];
    let mut effect_starts: Vec<Option<Vec<(usize, u64, f32)>>> =
        vec![None; list.display_items.len()];
    let mut scope_effects: BTreeMap<usize, Arc<EffectPaintPropertyNode>> = BTreeMap::new();
    let mut scope_output_clips: BTreeMap<usize, Arc<[RoundedClip]>> = BTreeMap::new();
    let mut effect_scopes = BTreeMap::new();
    let mut effect_output_shapes: BTreeMap<usize, Arc<[RoundedClip]>> = BTreeMap::new();
    for cursor in 0..=list.items.len() {
        if let Some(records) = ends.get(&cursor) {
            for &index in records {
                if list.display_items[index].record_begin == cursor {
                    continue;
                }
                if starts[index] != Some((state, stack.len())) {
                    return Err(ReplayUnsupported("record-state-escapes-boundary"));
                }
                if effect_starts[index].as_deref() != Some(alpha_scopes.as_slice()) {
                    return Err(ReplayUnsupported("record-effect-escapes-boundary"));
                }
                if shape_starts[index] != Some((rounded_scopes.len(), unsupported_complex)) {
                    return Err(ReplayUnsupported("record-clip-shape-escapes-boundary"));
                }
            }
        }
        if let Some(records) = begins.get(&cursor) {
            for &index in records {
                let expected = properties[index].unwrap();
                let effects = &expected_effects[index];
                if effects.len() != alpha_scopes.len()
                    || effects
                        .iter()
                        .zip(&alpha_scopes)
                        .any(|(effect, scope)| effect.opacity != scope.2)
                {
                    if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                        eprintln!("layer-replay alpha-mismatch record={index} wrappers={alpha_scopes:?} effects={:?}",
                            effects.iter().map(|effect|(effect.id,effect.opacity)).collect::<Vec<_>>());
                    }
                    return Err(ReplayUnsupported("external-opacity-property-mismatch"));
                }
                for (effect, scope) in effects.iter().zip(&alpha_scopes) {
                    if bounded_scopes.contains(&scope.0) {
                        if let (Some(trees), Some(origins)) = (&current_trees, &current_origins) {
                            let current = trees
                                .effects
                                .get(&effect.id)
                                .ok_or(ReplayUnsupported("bounded-effect-property-missing"))?;
                            let old_origin =
                                clip_transform_state(&effect.local_transform_space)?.translation;
                            let new_origin =
                                clip_transform_state(&current.local_transform_space)?.translation;
                            let current_origin =
                                origins[index].ok_or(ReplayUnsupported("invalid-chunk-range"))?;
                            if (new_origin.0 - old_origin.0, new_origin.1 - old_origin.1)
                                != (
                                    current_origin.0 - expected.translation.0,
                                    current_origin.1 - expected.translation.1,
                                )
                            {
                                return Err(ReplayUnsupported(
                                    "property-update-moves-bounded-effect-clip",
                                ));
                            }
                        }
                    }
                    let effect_key = Arc::as_ptr(effect) as usize;
                    let output = match effect_output_shapes.entry(effect_key) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(
                            effect
                                .output_clip
                                .as_ref()
                                .map(|clip| rounded_clips_between(clip, None, scale))
                                .transpose()?
                                .unwrap_or_else(|| Arc::from([])),
                        ),
                    };
                    if scope_output_clips
                        .get(&scope.0)
                        .is_none_or(|actual| **actual != **output)
                    {
                        if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                            eprintln!("layer-replay effect-output-clip-mismatch record={index} wrapper={scope:?} effect={} actual={:?} expected={output:?}",
                                effect.id,scope_output_clips.get(&scope.0));
                        }
                        return Err(ReplayUnsupported("external-effect-output-clip-mismatch"));
                    }
                    let identity = Arc::as_ptr(&effect.lifecycle.identity);
                    if scope_effects
                        .get(&scope.0)
                        .is_some_and(|old| !old.lifecycle.same_node(&effect.lifecycle))
                        || effect_scopes
                            .get(&identity)
                            .is_some_and(|&old| old != scope.0)
                    {
                        if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                            eprintln!("layer-replay alpha-identity-mismatch record={index} wrapper={scope:?} effect={} previous_wrapper={:?} previous_effect={:?}",
                                effect.id,effect_scopes.get(&identity),scope_effects.get(&scope.0).map(|node|node.id));
                        }
                        return Err(ReplayUnsupported(
                            "external-opacity-scope-identity-mismatch",
                        ));
                    }
                    scope_effects.insert(scope.0, effect.clone());
                    effect_scopes.insert(identity, scope.0);
                }
                if unsupported_complex || &*record_rounded[index] != rounded_scopes.as_slice() {
                    if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                        eprintln!("layer-replay rounded-mismatch record={index} unsupported={unsupported_complex} actual={rounded_scopes:?} expected={:?}",record_rounded[index]);
                    }
                    return Err(ReplayUnsupported("external-rounded-clip-property-mismatch"));
                }
                // If any flat external clip is not represented by properties,
                // removing the wrapper would omit actual pixels/state.
                if expected.clip != state.clip
                    && !rect_record_clips_are_redundant(
                        &list.items[list.display_items[index].record_begin
                            ..list.display_items[index].record_end],
                        state,
                        expected,
                        scale,
                    )
                {
                    if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                        let record = &list.display_items[index];
                        eprintln!("layer-replay record={index} id={:?} bounds={:?} expected={expected:?} actual={state:?}",
                            record.id,record.visual_rect);
                    }
                    return Err(ReplayUnsupported("external-clip-property-mismatch"));
                }
                starts[index] = Some((state, stack.len()));
                effect_starts[index] = Some(alpha_scopes.clone());
                shape_starts[index] = Some((rounded_scopes.len(), unsupported_complex));
                if let Some(bounds) = device_clip {
                    let local = shift(bounds, (-expected.translation.0, -expected.translation.1));
                    record_clips[index] = intersect(record_clips[index], local);
                }
            }
        }
        if cursor == list.items.len() {
            break;
        }
        let item = &list.items[cursor];
        if owners[cursor].is_none() && is_draw(item.r#type) {
            if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                eprintln!("layer-replay unrecorded cursor={cursor} type={:?} node={} phase={:?} rect={:?}",
                    item.r#type,item.node_id,item.phase,item.rect);
            }
            return Err(ReplayUnsupported("unrecorded-drawing-or-external-state"));
        }
        update_state(
            item,
            &mut state,
            &mut stack,
            scale,
            owners[cursor].is_none(),
        )?;
        match item.r#type {
            Kind::kSave | Kind::kSaveLayerDstIn => saved_effects.push((
                alpha_scopes.len(),
                device_clip,
                rounded_scopes.len(),
                unsupported_complex,
            )),
            Kind::kSaveLayer | Kind::kSaveLayerAlpha => {
                saved_effects.push((
                    alpha_scopes.len(),
                    device_clip,
                    rounded_scopes.len(),
                    unsupported_complex,
                ));
                if item.opacity != 1.0 {
                    scope_output_clips.insert(cursor, Arc::from(rounded_scopes.clone()));
                    alpha_scopes.push((cursor, item.node_id, item.opacity));
                }
                if !item.rect.is_empty() {
                    // An opacity-one bounded wrapper has no corresponding
                    // effect entry, so its relative space cannot be certified.
                    if bounded_property_update && item.opacity == 1.0 {
                        return Err(ReplayUnsupported(
                            "property-update-moves-bounded-effect-clip",
                        ));
                    }
                    bounded_scopes.insert(cursor);
                    // SkCanvas::save_layer_device maps then rounds out the
                    // rectangle to bound its actual child device. Preserve
                    // that integer clip even when properties have no equivalent
                    // output_clip. BW clipping commutes with pure group alpha.
                    device_clip = intersect(
                        device_clip,
                        pixel_bounds(mapped_rect(item.rect, state), scale),
                    );
                }
                if std::env::var_os("LAYOUTNG_LAYER_REPLAY_VERBOSE").is_some()
                    && item.opacity != 1.0
                {
                    eprintln!("layer-replay alpha-wrapper cursor={cursor} type={:?} node={} alpha={} rect={:?} device_clip={device_clip:?}",
                        item.r#type,item.node_id,item.opacity,item.rect);
                }
            }
            Kind::kRestore => {
                let (depth, bounds, rounded_depth, unsupported) = saved_effects
                    .pop()
                    .ok_or(ReplayUnsupported("unbalanced-effect-scope"))?;
                alpha_scopes.truncate(depth);
                device_clip = bounds;
                rounded_scopes.truncate(rounded_depth);
                unsupported_complex = unsupported;
            }
            Kind::kClipRect
                if state.axis_scale != (1.0, 1.0)
                    || state.cross_axis != (0.0, 0.0)
                    || !aligned_rect(mapped_rect(item.rect, state), scale) =>
            {
                rounded_scopes.push(clip_shape(
                    item.rect,
                    PaintCornerRadii::default(),
                    item.antialias,
                    state,
                ));
            }
            Kind::kClipRoundedRect => {
                if item.antialias {
                    rounded_scopes.push(clip_shape(
                        item.rect,
                        item.corner_radii,
                        item.antialias,
                        state,
                    ));
                } else {
                    unsupported_complex = true;
                }
            }
            Kind::kClipPath | Kind::kClipOutRect | Kind::kClipOutRoundedRect => {
                unsupported_complex = true
            }
            _ => {}
        }
    }
    if !stack.is_empty()
        || !saved_effects.is_empty()
        || !alpha_scopes.is_empty()
        || state != State::default()
    {
        return Err(ReplayUnsupported("unbalanced-external-state"));
    }
    let mut content = Vec::with_capacity(list.display_items.len());
    let mut already_canonical = vec![false; list.display_items.len()];
    for (record_index, record) in list.display_items.iter().enumerate() {
        if unchanged_recording_context
            && previous_source.is_some_and(|old| {
                Arc::ptr_eq(&old.items, &list.items)
                    || old.items[record.record_begin..record.record_end]
                        .iter()
                        .zip(&list.items[record.record_begin..record.record_end])
                        .all(|(a, b)| a == b)
            })
        {
            if let Some(old) = previous
                .and_then(|records| records.get(record_index))
                .filter(|old| old.record_index == record_index)
            {
                // Bounds, baked clips and finite shadow support were already
                // derived from this exact stream at this exact raster scale.
                content.push(old.clone());
                already_canonical[record_index] = true;
                continue;
            }
        }
        let actual = starts[record_index]
            .ok_or(ReplayUnsupported("missing-record-boundary"))?
            .0;
        let expected = properties[record_index].unwrap();
        let delta = (
            actual.translation.0 - expected.translation.0,
            actual.translation.1 - expected.translation.1,
        );
        let raw = mask_record_body(
            &list.items[record.record_begin..record.record_end],
            mask_records[record_index],
        )?;
        // The source ink contract is proven for text records. Keep the
        // existing backend bounds for mixed/SVG/decoration records whose
        // adapter metadata can still be conservative or in a local space.
        let accurate_source_ink = record.visual_rect_is_accurate
            && raw.iter().any(|item| item.r#type == Kind::kDrawGlyphRun)
            && raw
                .iter()
                .all(|item| !is_draw(item.r#type) || item.r#type == Kind::kDrawGlyphRun);
        let scaled = actual.axis_scale != (1.0, 1.0)
            || actual.cross_axis != (0.0, 0.0)
            || raw.iter().any(|item| {
                item.r#type == Kind::kConcat
                    && (item.transform.values[0] != 1.0
                        || item.transform.values[5] != 1.0
                        || item.transform.values[1] != 0.0
                        || item.transform.values[4] != 0.0)
            });
        let initial = State {
            translation: delta,
            axis_scale: actual.axis_scale,
            cross_axis: actual.cross_axis,
            ..Default::default()
        };
        let rect_bounds = rect_record_pixel_bounds(raw, initial, record_clips[record_index], scale);
        if actual.axis_scale != (1.0, 1.0)
            || actual.cross_axis != (0.0, 0.0)
            || raw.iter().any(|item| {
                item.r#type == Kind::kConcat && translation(&item.transform, scale).is_err()
            })
        {
            // Non-composited local SVG/viewBox transforms raster into their
            // translation-only ancestor layer. Preserve their real matrix and
            // record ops instead of inventing a composited layer or baking the
            // scale into glyph/image/path geometry with different semantics.
            let mut prefix = DisplayItem {
                r#type: Kind::kConcat,
                ..Default::default()
            };
            prefix.transform.values[0] = actual.axis_scale.0;
            prefix.transform.values[5] = actual.axis_scale.1;
            prefix.transform.values[1] = actual.cross_axis.1;
            prefix.transform.values[4] = actual.cross_axis.0;
            prefix.transform.values[12] = delta.0;
            prefix.transform.values[13] = delta.1;
            let mut items = Vec::with_capacity(raw.len() + 2);
            if let Some(rect) = record_clips[record_index] {
                // Clip in the ancestor layer's coordinates before restoring
                // the omitted local transform (PaintChunksToCcLayer ordering).
                items.push(DisplayItem {
                    r#type: Kind::kClipRect,
                    rect,
                    antialias: false,
                    ..Default::default()
                });
            }
            let mut affine_started = false;
            for rounded in &*baked_rounded[record_index] {
                if let Some(transform) = rounded.affine {
                    affine_started = true;
                    if transform != affine_key(actual) {
                        return Err(ReplayUnsupported("removed-clip-not-in-record-affine-space"));
                    }
                    continue;
                }
                if affine_started {
                    return Err(ReplayUnsupported("removed-clip-affine-order-mismatch"));
                }
                items.push(DisplayItem {
                    r#type: if rounded.radii.HasRadius() {
                        Kind::kClipRoundedRect
                    } else {
                        Kind::kClipRect
                    },
                    rect: shift(
                        rounded.rect,
                        (-expected.translation.0, -expected.translation.1),
                    ),
                    corner_radii: rounded.radii,
                    antialias: rounded.antialias,
                    ..Default::default()
                });
            }
            items.push(prefix);
            // PaintChunksToCcLayer::StartClip emits ApplyTransform followed
            // by the ClipOp. Shared-CTM clips need no inverse projection or
            // replacement path: preserve the original rect/radii under the
            // very same affine prefix used for the drawing record.
            for shape in baked_rounded[record_index]
                .iter()
                .filter(|shape| shape.affine.is_some())
            {
                items.push(DisplayItem {
                    r#type: if shape.radii.HasRadius() {
                        Kind::kClipRoundedRect
                    } else {
                        Kind::kClipRect
                    },
                    rect: shape.rect,
                    corner_radii: shape.radii,
                    antialias: shape.antialias,
                    ..Default::default()
                });
            }
            for item in raw {
                if item.r#type == Kind::kConcat {
                    let mut validate = State::default();
                    concatenate_affine(&mut validate, &item.transform)?;
                    items.push(item.clone());
                } else {
                    items.push(canonical_item(item, (0.0, 0.0))?);
                }
            }
            let mut visual_rect = if scaled {
                shift(
                    mapped_rect(record.visual_rect, actual),
                    (-expected.translation.0, -expected.translation.1),
                )
            } else {
                shift(record.visual_rect, delta)
            };
            if let Some(clip) = record_clips[record_index] {
                visual_rect = intersection(visual_rect, clip);
            }
            if let Some(proven) = rect_bounds {
                visual_rect = if !scaled && accurate_source_ink {
                    intersection(proven, pixel_bounds(visual_rect, scale))
                } else {
                    proven
                };
            }
            if !finite_rect(visual_rect) {
                return Err(ReplayUnsupported("invalid-scaled-record-bounds"));
            }
            content.push(RasterRecordContent {
                record_index,
                items: Arc::from(items),
                visual_rect,
                rect_known_to_be_opaque: PaintRect::default(),
                // A clip-only/translation prefix cannot add ink outside the
                // original complete source bounds. Scale still lacks a proven
                // glyph/stroke ink mapping and keeps all records in tasks.
                bounds_are_complete: rect_bounds.is_some()
                    || (!scaled && record.visual_rect_is_accurate),
            });
            continue;
        }
        let reusable = previous
            .and_then(|records| records.get(record_index))
            .filter(|old| {
                old.record_index == record_index
                    && previous_source.is_none_or(|source| {
                        source.records.get(record_index).is_some_and(|source| {
                            source.id == record.id && source.kind == record.kind
                        })
                    })
                    && record_clips[record_index].is_none()
                    && baked_rounded[record_index].is_empty()
                    && raw.iter().all(|item| item.r#type != Kind::kConcat)
                    && ((delta == (0.0, 0.0) && &*old.items == raw)
                        || (previous_source
                            .and_then(|source| source.records.get(record_index))
                            .is_some_and(|source| {
                                source.id == record.id && source.kind == record.kind
                            })
                            && raw.len() == old.items.len()
                            && raw
                                .iter()
                                .zip(old.items.iter())
                                .all(|(item, old)| translated_item_matches(item, delta, old))))
            });
        let mut local = State {
            translation: delta,
            ..Default::default()
        };
        let mut stack = Vec::new();
        let mut items = Vec::new();
        if let Some(rect) = record_clips[record_index] {
            items.push(DisplayItem {
                r#type: Kind::kClipRect,
                rect,
                antialias: false,
                ..Default::default()
            });
        }
        for shape in &*baked_rounded[record_index] {
            if shape.affine.is_some() {
                return Err(ReplayUnsupported("removed-clip-not-in-record-affine-space"));
            }
            items.push(DisplayItem {
                r#type: if shape.radii.HasRadius() {
                    Kind::kClipRoundedRect
                } else {
                    Kind::kClipRect
                },
                rect: shift(
                    shape.rect,
                    (-expected.translation.0, -expected.translation.1),
                ),
                corner_radii: shape.radii,
                antialias: shape.antialias,
                ..Default::default()
            });
        }
        // A clip-only scope still normalizes the actual draw destinations.
        // Keeping prefix Concat(delta) + folded raw body would change payload
        // on every scroll even though their effective local pixels are equal.
        for item in raw {
            if reusable.is_some() {
                // Bounds and state validation still consume current metadata;
                // the equal Arc alone is only a payload allocation shortcut.
                match item.r#type {
                    Kind::kSave => stack.push(local),
                    Kind::kRestore => {
                        local = stack.pop().ok_or(ReplayUnsupported("unbalanced-record"))?
                    }
                    Kind::kClipRect => {
                        if !finite_rect(item.rect) {
                            return Err(ReplayUnsupported("invalid-canonical-clip"));
                        }
                        local.clip = intersect(local.clip, shift(item.rect, local.translation));
                    }
                    _ => {}
                }
                continue;
            }
            match item.r#type {
                Kind::kSave => {
                    stack.push(local);
                    items.push(item.clone());
                }
                Kind::kRestore => {
                    local = stack.pop().ok_or(ReplayUnsupported("unbalanced-record"))?;
                    items.push(item.clone());
                }
                Kind::kConcat => {
                    let t = translation(&item.transform, scale)?;
                    local.translation.0 += t.0;
                    local.translation.1 += t.1;
                }
                Kind::kClipRect => {
                    let mut normalized = item.clone();
                    normalized.rect = shift(item.rect, local.translation);
                    if !finite_rect(normalized.rect) {
                        return Err(ReplayUnsupported("invalid-canonical-clip"));
                    }
                    local.clip = intersect(local.clip, normalized.rect);
                    items.push(normalized);
                }
                _ => {
                    let normalized = canonical_item(item, local.translation)?;
                    items.push(normalized);
                }
            }
        }
        if !stack.is_empty() {
            return Err(ReplayUnsupported("unbalanced-record"));
        }
        let source_rect = shift(record.visual_rect, delta);
        let source_rect =
            record_clips[record_index].map_or(source_rect, |clip| intersection(source_rect, clip));
        let visual_rect = match rect_bounds {
            // Both the translated source ink and the actual record clip bound
            // enclose all output. Keep their intersection: a clip must never
            // replace accurate glyph ink with an entire card/viewport.
            Some(proven) if accurate_source_ink => {
                intersection(proven, pixel_bounds(source_rect, scale))
            }
            Some(proven) => proven,
            None => source_rect,
        };
        content.push(RasterRecordContent {
            record_index,
            items: reusable.map_or_else(|| Arc::from(items), |old| old.items.clone()),
            // ScrollbarDisplayItem currently advertises conservative adapter
            // bounds. The supported backend really draws only these rects
            // (compat/canvas.rs:757), so their clipped device enclosure is a
            // complete bound, including fractional thumb AA support.
            visual_rect,
            rect_known_to_be_opaque: PaintRect::default(),
            bounds_are_complete: rect_bounds.is_some() || record.visual_rect_is_accurate,
        });
    }
    for (index, record) in content.iter_mut().enumerate() {
        if already_canonical[index] {
            continue;
        }
        let draws = record.items.iter().any(|item| is_draw(item.r#type));
        if !draws {
            // The source semantic client/range is retained. Balanced state
            // without any draw contributes no pixels and has exact empty ink.
            record.items = Arc::from([]);
            record.visual_rect = PaintRect::default();
            record.bounds_are_complete = true;
            continue;
        }
        // Only remove a leading BW clip after proving EVERY un-clipped draw's
        // complete pixel support is inside it. Fractional/AA clips and unknown
        // glyph/stroke support remain intact. This stabilizes harmless optional
        // clip prefixes without discarding a real viewport or inner clip.
        if let Some(natural) =
            record_pixel_bounds(&record.items, State::default(), None, scale, true)
        {
            let count = record
                .items
                .iter()
                .take_while(|item| {
                    item.r#type == Kind::kClipRect
                        && !item.antialias
                        && aligned_rect(item.rect, scale)
                        && contains(item.rect, natural)
                })
                .count();
            if count != 0 {
                record.items = Arc::from(record.items[count..].to_vec());
            }
        }
    }
    // DecorationVisualRect includes ceil(1.5 * CSS blur) plus spread/offset.
    // compat/canvas.rs::draw_box_shadow's finite mask support additionally
    // encloses two native pixels. Preserve that fringe in canonical metadata;
    // conservative source bounds remain conservative (never upgrade complete).
    for (index, record) in content.iter_mut().enumerate() {
        if already_canonical[index] {
            continue;
        }
        if record
            .items
            .iter()
            .any(|item| item.r#type == Kind::kDrawBoxShadow && item.blur_radius > 0.0)
        {
            let extent = 2.0 / scale;
            record.visual_rect = PaintRect {
                x: record.visual_rect.x - extent,
                y: record.visual_rect.y - extent,
                width: record.visual_rect.width + extent * 2.0,
                height: record.visual_rect.height + extent * 2.0,
            };
        }
    }
    for (index, record) in content.iter_mut().enumerate() {
        if already_canonical[index] {
            continue;
        }
        // Analyze the lowered record, never the old folded-scroll coordinates.
        // Cache hits above retain this proof with the same canonical content.
        record.rect_known_to_be_opaque = paint::paint_engine::CalculateRectKnownToBeOpaqueForRecord(
            record.visual_rect,
            &record.items,
        );
    }
    Ok(content)
}
