//! Chromium core/page/touch_adjustment.h:37-70, .cc:61-608.
//! Source subtarget lists are call-scoped values; node and geometry ownership
//! remain in the live DOM and Layout owners supplied by the query context.
use foundation::{gfx, LayoutUnit, PhysicalSize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchAdjustmentCandidateType {
    kClickable,
    kContextMenu,
    kStylusWritable,
}

/// Only source leaf queries are injected. Candidate compilation, responder
/// preference, editable consolidation, scoring and snapping belong below.
pub trait TouchAdjustmentContext {
    type Node: Clone;
    fn Identity(&self, node: &Self::Node) -> u128;
    fn ParentOrShadowHost(&self, node: &Self::Node) -> Option<Self::Node>;
    fn ParentShadowHostOrOwner(&self, node: &Self::Node) -> Option<Self::Node>;
    fn WillRespondToMouseClickEvents(&self, node: &Self::Node) -> bool;
    fn WillRespondToMouseMoveEvents(&self, node: &Self::Node) -> bool;
    fn IsElement(&self, node: &Self::Node) -> bool;
    fn IsIFrameElement(&self, node: &Self::Node) -> bool;
    fn IsMouseFocusable(&self, node: &Self::Node) -> bool;
    fn ChildrenOrSiblingsAffectedByActiveOrHover(&self, node: &Self::Node) -> bool;
    fn ComputedStyleAffectedByActiveOrHover(&self, node: &Self::Node) -> bool;
    fn EffectiveTouchAction(&self, node: &Self::Node) -> foundation::TouchAction;
    fn ContextMenuFacts(&self, node: &Self::Node) -> ContextMenuNodeFacts;
    fn UpdateStyleAndLayoutTree(&self, node: &Self::Node);
    fn IsEditable(&self, node: &Self::Node) -> bool;
    fn HasLayoutObject(&self, node: &Self::Node) -> bool;
    fn AbsoluteQuads(&self, node: &Self::Node) -> Vec<gfx::QuadF>;
    fn AbsoluteQuadsForRange(
        &self,
        node: &Self::Node,
        range: std::ops::Range<u32>,
    ) -> Vec<gfx::QuadF>;
    fn ForEachWordRange(&self, node: &Self::Node, append: &mut dyn FnMut(std::ops::Range<u32>));
    fn LayoutSelectionRange(&self, node: &Self::Node) -> std::ops::Range<u32>;
    fn ConvertRectToRootFrame(&self, node: &Self::Node, rect: gfx::Rect) -> gfx::Rect;
    fn ConvertPointToRootFrame(&self, node: &Self::Node, point: gfx::Point) -> gfx::Point;
    fn IsDescendantOf(&self, node: &Self::Node, ancestor: &Self::Node) -> bool;
    fn IsPseudoElement(&self, node: &Self::Node) -> bool;
    fn HasActivationBehavior(&self, node: &Self::Node) -> bool;
}

pub struct SubtargetGeometry<N> {
    pub node: N,
    pub quad: gfx::QuadF,
}
pub struct ContextMenuNodeFacts {
    pub is_text: bool,
    pub is_shadow_root: bool,
    pub is_layout_box: bool,
    pub is_link: bool,
    pub is_image_media_or_svg_image: bool,
    pub can_be_selection_leaf: bool,
    pub should_select_on_contextual_menu_click: bool,
    pub is_selected: bool,
}
// Source .cc:126-168,214-259. These source-only helpers also retain the
// context-menu path; PointerEventManager itself requests only clickable/stylus.
pub fn NodeIsZoomTarget<C: TouchAdjustmentContext + ?Sized>(ctx: &C, node: &C::Node) -> bool {
    let facts = ctx.ContextMenuFacts(node);
    !facts.is_text && !facts.is_shadow_root && facts.is_layout_box
}
pub fn ProvidesContextMenuItems<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    node: &C::Node,
) -> bool {
    if !ctx.HasLayoutObject(node) {
        return false;
    }
    ctx.UpdateStyleAndLayoutTree(node);
    let facts = ctx.ContextMenuFacts(node);
    ctx.IsEditable(node)
        || facts.is_link
        || facts.is_image_media_or_svg_image
        || (facts.can_be_selection_leaf
            && (facts.should_select_on_contextual_menu_click || facts.is_selected))
}
fn ContextSubtargetQuads<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    node: &C::Node,
) -> Vec<gfx::QuadF> {
    let facts = ctx.ContextMenuFacts(node);
    if !facts.is_text {
        return ctx.AbsoluteQuads(node);
    }
    if facts.should_select_on_contextual_menu_click {
        let mut quads = Vec::new();
        ctx.ForEachWordRange(node, &mut |range| {
            quads.extend(ctx.AbsoluteQuadsForRange(node, range))
        });
        quads
    } else if facts.is_selected {
        ctx.AbsoluteQuadsForRange(node, ctx.LayoutSelectionRange(node))
    } else {
        ctx.AbsoluteQuads(node)
    }
}
// Source .cc:100-124,170-191.
pub fn NodeRespondsToTapGesture<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    node: &C::Node,
) -> bool {
    if ctx.WillRespondToMouseClickEvents(node) || ctx.WillRespondToMouseMoveEvents(node) {
        return true;
    }
    ctx.IsElement(node)
        && ((ctx.IsMouseFocusable(node) && !ctx.IsIFrameElement(node))
            || ctx.ChildrenOrSiblingsAffectedByActiveOrHover(node)
            || ctx.ComputedStyleAffectedByActiveOrHover(node))
}
pub fn NodeRespondsToTapOrMove<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    node: &C::Node,
) -> bool {
    ctx.UpdateStyleAndLayoutTree(node);
    if NodeRespondsToTapGesture(ctx, node) {
        return true;
    }
    let action = ctx.EffectiveTouchAction(node);
    action & foundation::TouchAction::kPan != foundation::TouchAction::kPan
        || action & foundation::TouchAction::kInternalNotWritable
            != foundation::TouchAction::kInternalNotWritable
}
fn BoundingBox(quad: gfx::QuadF) -> gfx::Rect {
    gfx::ToEnclosingRect(quad.BoundingBox())
}

// Source .cc:270-361. The responder map is a per-query source cache, never a
// retained node model or a second DOM walk.
pub fn CompileSubtargetList<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    nodes: &[C::Node],
    kind: TouchAdjustmentCandidateType,
) -> Vec<SubtargetGeometry<C::Node>> {
    let mut responders: HashMap<u128, C::Node> = HashMap::new();
    let mut responder_ancestors = HashSet::new();
    let mut candidates = Vec::new();
    let mut editable_ancestors = HashSet::new();
    for node in nodes {
        let mut visited = Vec::new();
        let mut responding = None;
        let mut current = Some(node.clone());
        while let Some(value) = current {
            if let Some(found) = responders.get(&ctx.Identity(&value)) {
                responding = Some(found.clone());
                break;
            }
            visited.push(value.clone());
            let responds = match kind {
                TouchAdjustmentCandidateType::kClickable => NodeRespondsToTapGesture(ctx, &value),
                TouchAdjustmentCandidateType::kContextMenu => ProvidesContextMenuItems(ctx, &value),
                TouchAdjustmentCandidateType::kStylusWritable => {
                    NodeRespondsToTapOrMove(ctx, &value)
                }
            };
            if responds {
                responding = Some(value.clone());
                let mut ancestor = ctx.ParentShadowHostOrOwner(&value);
                while let Some(value) = ancestor {
                    if !responder_ancestors.insert(ctx.Identity(&value)) {
                        break;
                    }
                    ancestor = ctx.ParentShadowHostOrOwner(&value);
                }
                break;
            }
            current = ctx.ParentOrShadowHost(&value);
        }
        if let Some(responding) = responding {
            for value in visited {
                responders
                    .entry(ctx.Identity(&value))
                    .or_insert_with(|| responding.clone());
            }
            candidates.push(node.clone());
        }
    }
    let mut subtargets = Vec::new();
    for candidate in candidates {
        if responder_ancestors.contains(&ctx.Identity(&responders[&ctx.Identity(&candidate)])) {
            continue;
        }
        if editable_ancestors.contains(&ctx.Identity(&candidate)) {
            continue;
        }
        ctx.UpdateStyleAndLayoutTree(&candidate);
        let mut candidate = Some(candidate);
        if ctx.IsEditable(candidate.as_ref().unwrap()) {
            let mut parent = ctx.ParentOrShadowHost(candidate.as_ref().unwrap());
            while let Some(value) = parent {
                if !ctx.IsEditable(&value) || !ctx.HasLayoutObject(&value) {
                    break;
                }
                if editable_ancestors.contains(&ctx.Identity(&value)) {
                    candidate = None;
                    break;
                }
                editable_ancestors.insert(ctx.Identity(&value));
                parent = ctx.ParentOrShadowHost(&value);
                candidate = Some(value);
            }
        }
        if let Some(node) = candidate {
            let quads = if kind == TouchAdjustmentCandidateType::kContextMenu {
                ContextSubtargetQuads(ctx, &node)
            } else {
                ctx.AbsoluteQuads(&node)
            };
            for quad in quads {
                subtargets.push(SubtargetGeometry {
                    node: node.clone(),
                    quad,
                });
            }
        }
    }
    subtargets
}

// Source .cc:392-417.
// The source zoom scoring helper is kept alongside the selected hybrid metric.
pub fn ZoomableIntersectionQuotient<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    hotspot: gfx::Point,
    touch: gfx::Rect,
    subtarget: &SubtargetGeometry<C::Node>,
) -> f32 {
    let rect = ctx.ConvertRectToRootFrame(&subtarget.node, BoundingBox(subtarget.quad));
    if !rect.ContainsPoint(hotspot) {
        return f32::INFINITY;
    }
    let mut intersection = rect;
    intersection.Intersect(touch);
    (i64::from(rect.width()) * i64::from(rect.height())) as f32
        / (i64::from(intersection.width()) * i64::from(intersection.height())) as f32
}
pub fn HybridDistanceFunction<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    hotspot: gfx::Point,
    touch: gfx::Rect,
    subtarget: &SubtargetGeometry<C::Node>,
) -> f32 {
    let mut rect =
        gfx::RectF::from(ctx.ConvertRectToRootFrame(&subtarget.node, BoundingBox(subtarget.quad)));
    let radius_squared = 0.25 * ((touch.width() as f32).powi(2) + (touch.height() as f32).powi(2));
    let closest = rect.ClosestPoint(gfx::PointF::new(hotspot.x() as f32, hotspot.y() as f32));
    let x = closest.x();
    let y = closest.y();
    let distance =
        ((x - hotspot.x() as f32).powi(2) + (y - hotspot.y() as f32).powi(2)) / radius_squared;
    let overlap = ((touch.width() as f32).min(rect.width())
        * (touch.height() as f32).min(rect.height()))
    .max(1.0);
    rect.Intersect(gfx::RectF::from(touch));
    1.0 - rect.width() * rect.height() / overlap + distance
}

// Source .cc:419-489. C++'s float-to-int conversion intentionally truncates
// toward zero after adding 0.5 here; it differs from round for negative values.
pub fn SnapTo<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    geom: &SubtargetGeometry<C::Node>,
    hotspot: gfx::Point,
    touch: gfx::Rect,
) -> Option<gfx::Point> {
    if geom.quad.IsRectilinear() {
        let rect = ctx.ConvertRectToRootFrame(&geom.node, BoundingBox(geom.quad));
        if rect.ContainsPoint(hotspot) {
            return Some(hotspot);
        }
        let left = rect.x().max(touch.x());
        let top = rect.y().max(touch.y());
        let right = rect.right().min(touch.right());
        let bottom = rect.bottom().min(touch.bottom());
        return (left < right && top < bottom)
            .then(|| gfx::Point::new(left + (right - left) / 2, top + (bottom - top) / 2));
    }
    let convert = |point: &gfx::PointF| {
        let point = ctx.ConvertPointToRootFrame(
            &geom.node,
            gfx::Point::new((point.x() + 0.5) as i32, (point.y() + 0.5) as i32),
        );
        gfx::PointF::new(point.x() as f32, point.y() as f32)
    };
    let quad = gfx::QuadF::new(
        convert(geom.quad.p1()),
        convert(geom.quad.p2()),
        convert(geom.quad.p3()),
        convert(geom.quad.p4()),
    );
    if quad.Contains(gfx::PointF::new(hotspot.x() as f32, hotspot.y() as f32)) {
        return Some(hotspot);
    }
    let center = quad.CenterPoint();
    let mut x = center.x();
    let mut y = center.y();
    if x < touch.x() as f32 {
        x = touch.x() as f32;
    } else if x >= touch.right() as f32 {
        x = (touch.right() - 1) as f32;
    }
    if y < touch.y() as f32 {
        y = touch.y() as f32;
    } else if y >= touch.bottom() as f32 {
        y = (touch.bottom() - 1) as f32;
    }
    let snapped = gfx::Point::new(x.round() as i32, y.round() as i32);
    quad.Contains(gfx::PointF::new(snapped.x() as f32, snapped.y() as f32))
        .then_some(snapped)
}

// Source .cc:494-583.
pub fn FindBestTouchAdjustmentCandidate<C: TouchAdjustmentContext + ?Sized>(
    ctx: &C,
    kind: TouchAdjustmentCandidateType,
    hotspot: gfx::Point,
    touch: gfx::Rect,
    nodes: &[C::Node],
) -> Option<(C::Node, gfx::Point)> {
    let subtargets = CompileSubtargetList(ctx, nodes, kind);
    let mut best = f32::INFINITY;
    let mut adjusted: Option<(C::Node, gfx::Point)> = None;
    for subtarget in subtargets {
        let metric = HybridDistanceFunction(ctx, hotspot, touch, &subtarget);
        if metric < best {
            if let Some(point) = SnapTo(ctx, &subtarget, hotspot, touch) {
                best = metric;
                adjusted = Some((subtarget.node, point));
            }
        } else if metric - best < 1e-6 {
            if let Some(point) = SnapTo(ctx, &subtarget, hotspot, touch) {
                if adjusted
                    .as_ref()
                    .is_some_and(|(node, _)| ctx.IsDescendantOf(&subtarget.node, node))
                {
                    adjusted = Some((subtarget.node, point));
                }
            }
        }
    }
    while adjusted
        .as_ref()
        .is_some_and(|(node, _)| ctx.IsPseudoElement(node) && !ctx.HasActivationBehavior(node))
    {
        let (node, point) = adjusted.take().unwrap();
        adjusted = ctx.ParentOrShadowHost(&node).map(|node| (node, point));
    }
    adjusted
}

// Source .cc:586-608. LayoutUnit multiplication retains source subpixel
// rounding; the caller supplies the ChromeClient/Inspector/Page values.
pub fn GetHitTestRectForAdjustment(
    touch: PhysicalSize,
    mut device_scale: f32,
    inspector_override: f32,
    page_scale: f32,
) -> PhysicalSize {
    if inspector_override != 1.0 {
        device_scale = 1.0;
    }
    let scale = device_scale / page_scale;
    let max = PhysicalSize::new(LayoutUnit::from_signed(32), LayoutUnit::from_signed(32)) * scale;
    let min = PhysicalSize::new(LayoutUnit::from_signed(20), LayoutUnit::from_signed(20)) * scale;
    touch.ShrunkTo(&max).ExpandedTo(&min)
}
