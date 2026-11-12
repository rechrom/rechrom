//! Cull rectangle value and hysteresis rules.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/cull_rect.{h,cc}`.

use layoutng_assembly::internal::layout_input::{Offset, TransformMatrix};

use crate::geometry_mapper::{
    LocalToAncestorClipRect, MapRectWithTransforms, SourceToDestinationApproximateMinimumScale,
    SourceToDestinationRect,
};
use crate::paint_property_tree::{
    ClipPaintPropertyNode, PaintCompositedScrollingPreference, PropertyTreeState,
    ScrollPaintPropertyNode, TransformPaintPropertyNode,
};
use crate::PaintRect;

pub const SMALL_SCROLLER_AREA: f64 = 100_000.0;
pub const SMALL_SCROLLER_CHANGED_ENOUGH_DISTANCE: f64 = 512.0;
pub const CULL_RECT_CHANGED_ENOUGH_DISTANCE: f64 = 512.0;
pub const CULL_RECT_PIXEL_DISTANCE_TO_EXPAND: f64 = 4000.0;
const REASONABLE_PIXEL_LIMIT: f64 = foundation::LayoutUnit::kIntMax as f64;

#[inline]
fn SameTransform(a: &TransformPaintPropertyNode, b: &TransformPaintPropertyNode) -> bool {
    a.lifecycle.same_node(&b.lifecycle)
}

#[inline]
fn SameClip(a: &ClipPaintPropertyNode, b: &ClipPaintPropertyNode) -> bool {
    a.lifecycle.same_node(&b.lifecycle)
}

fn TransformIsAncestor(
    ancestor: &TransformPaintPropertyNode,
    descendant: &TransformPaintPropertyNode,
) -> bool {
    let mut current = Some(descendant);
    while let Some(node) = current {
        if SameTransform(node, ancestor) {
            return true;
        }
        current = node.parent.as_deref();
    }
    false
}

fn ClipIsAncestor(ancestor: &ClipPaintPropertyNode, descendant: &ClipPaintPropertyNode) -> bool {
    let mut current = Some(descendant);
    while let Some(node) = current {
        if SameClip(node, ancestor) {
            return true;
        }
        current = node.parent.as_deref();
    }
    false
}

#[inline]
fn RequiresCullRectExpansion(transform: &TransformPaintPropertyNode) -> bool {
    transform
        .direct_compositing_reasons
        .iter_reasons()
        .any(|reason| foundation::CompositingReasonCombos::kRequiresCullRectExpansion.Has(reason))
}

#[inline]
fn IsRootScroller(transform: &TransformPaintPropertyNode) -> bool {
    transform
        .direct_compositing_reasons
        .Has(foundation::CompositingReason::kRootScroller)
}

fn IsSmallScrollerTransform(transform: &TransformPaintPropertyNode, expansion_ratio: f64) -> bool {
    !IsRootScroller(transform)
        && transform.scroll.as_ref().is_some_and(|scroll| {
            scroll.ContainerRect().width * scroll.ContainerRect().height
                <= SMALL_SCROLLER_AREA * expansion_ratio * expansion_ratio
        })
}

fn LocalPixelDistanceToExpandForTransform(
    root: &TransformPaintPropertyNode,
    local: &TransformPaintPropertyNode,
    expansion_ratio: f64,
) -> (f64, f64) {
    let is_small = IsSmallScrollerTransform(local, expansion_ratio);
    let minimum = 2.0 * ChangedEnoughMinimumDistance(is_small, expansion_ratio);
    if is_small {
        return (minimum, minimum);
    }
    let base = CULL_RECT_PIXEL_DISTANCE_TO_EXPAND * expansion_ratio;
    if base == 0.0 {
        return (minimum, minimum);
    }
    let scale = SourceToDestinationApproximateMinimumScale(root, local);
    if scale > REASONABLE_PIXEL_LIMIT / base {
        return (base, minimum);
    }
    ((scale * base).max(minimum), minimum)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CullRect(Option<PaintRect>);

impl std::fmt::Display for CullRect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(rect) => write!(
                formatter,
                "{},{} {}x{}",
                rect.x, rect.y, rect.width, rect.height
            ),
            None => formatter.write_str("Infinite"),
        }
    }
}

impl CullRect {
    /// Source: `cull_rect.h:29-34`. `None` represents `InfiniteIntRect()`.
    pub fn Infinite() -> Self {
        Self(None)
    }

    pub fn FromRect(rect: PaintRect) -> Self {
        Self(Some(rect))
    }

    pub fn Rect(self) -> Option<PaintRect> {
        self.0
    }

    pub fn IsInfinite(self) -> bool {
        self.0.is_none()
    }

    /// Source: `cull_rect.cc:123-129`.
    pub fn Intersects(self, rect: PaintRect) -> bool {
        if rect.is_empty() {
            return false;
        }
        self.0
            .is_none_or(|cull| !cull.intersection(rect).is_empty())
    }

    /// Source: `cull_rect.cc:131-136`.
    pub fn IntersectsTransformed(self, transform: &TransformMatrix, rect: PaintRect) -> bool {
        if rect.is_empty() {
            return false;
        }
        self.0.is_none_or(|cull| {
            MapRectWithTransforms(rect, std::slice::from_ref(transform), false)
                .is_some_and(|mapped| !mapped.intersection(cull).is_empty())
        })
    }

    /// Source: `cull_rect.cc:138-144`.
    pub fn IntersectsHorizontalRange(self, lo: f64, hi: f64) -> bool {
        self.0
            .is_none_or(|rect| !(lo >= Right(rect) || hi <= rect.x))
    }

    pub fn IntersectsVerticalRange(self, lo: f64, hi: f64) -> bool {
        self.0
            .is_none_or(|rect| !(lo >= Bottom(rect) || hi <= rect.y))
    }

    /// Source: `cull_rect.cc:146-149`.
    pub fn Move(&mut self, offset: Offset) {
        if let Some(rect) = &mut self.0 {
            rect.x += offset.x;
            rect.y += offset.y;
        }
    }

    /// Source: `cull_rect.cc:151-158`. A transform node's matrix maps its
    /// local space to its parent, so mapping the parent-space cull into the
    /// node's local space uses the inverse direction.
    pub fn ApplyTransform(&mut self, transform: &TransformMatrix) {
        let Some(rect) = self.0 else {
            return;
        };
        self.0 = Some(
            MapRectWithTransforms(rect, std::slice::from_ref(transform), true).unwrap_or_default(),
        );
    }

    /// Source: `cull_rect.cc:108-123`.
    pub fn CanExpandForScroll(scroll: &ScrollPaintPropertyNode) -> bool {
        scroll.GetCompositedScrollingPreference()
            != PaintCompositedScrollingPreference::NotPreferred
            && scroll.UserScrollable()
            && (scroll.ContentsRect().width > scroll.ContainerRect().width
                || scroll.ContentsRect().height > scroll.ContainerRect().height)
    }

    /// Source: `cull_rect.cc:160-245`.
    fn ApplyScrollTranslation(
        &mut self,
        root_transform: &TransformPaintPropertyNode,
        scroll_translation: &TransformPaintPropertyNode,
        expansion_ratio: f64,
    ) -> (bool, bool) {
        let scroll = scroll_translation
            .scroll
            .as_ref()
            .expect("scroll translation owns a ScrollPaintPropertyNode");
        let container_rect = scroll.ContainerRect();
        let can_expand = expansion_ratio != 0.0 && Self::CanExpandForScroll(scroll);
        let Some(rect) = self.0 else {
            return (false, false);
        };
        if can_expand {
            if rect.intersection(container_rect).is_empty() {
                self.0 = Some(PaintRect::default());
                return (false, false);
            }
            // Deliberately drop ancestor clips so retained scrolling contents
            // always cover the complete scrollport.
            self.0 = Some(container_rect);
        } else {
            let clipped = rect.intersection(container_rect);
            self.0 = Some(clipped);
            if clipped.is_empty() {
                return (false, false);
            }
        }
        self.ApplyTransform(&scroll_translation.matrix);
        if !can_expand {
            return (false, false);
        }

        let contents_rect = scroll.ContentsRect();
        let (mut outset_x, minimum) = LocalPixelDistanceToExpandForTransform(
            root_transform,
            scroll_translation,
            expansion_ratio,
        );
        let mut outset_y = outset_x;
        let range_x = contents_rect.width - container_rect.width;
        let range_y = contents_rect.height - container_rect.height;
        if range_x <= 0.0 {
            outset_x = 0.0;
        }
        if range_y <= 0.0 {
            outset_y = 0.0;
        }
        if outset_x > 0.0 && outset_y > 0.0 {
            outset_x /= 2.0;
            outset_y /= 2.0;
            if outset_x > range_x {
                outset_y += outset_x - range_x;
            }
            if outset_y > range_y {
                outset_x += outset_y - range_y;
            }
        }
        outset_x = outset_x.max(minimum).min(range_x);
        outset_y = outset_y.max(minimum).min(range_y);
        self.0 = self
            .0
            .map(|rect| Outset(rect, outset_x, outset_y).intersection(contents_rect));
        (outset_x > 0.0, outset_y > 0.0)
    }

    /// Source: `cull_rect.cc:248-273`.
    fn ApplyPaintPropertiesWithoutExpansion(
        &mut self,
        source: &PropertyTreeState,
        destination: &PropertyTreeState,
    ) -> bool {
        let clip_rect = LocalToAncestorClipRect(destination, source);
        if clip_rect.is_some_and(|clip| clip.is_empty()) {
            self.0 = Some(PaintRect::default());
            return false;
        }
        if let Some(clip) = clip_rect {
            let clipped = self.0.map_or(clip, |rect| rect.intersection(clip));
            self.0 = Some(clipped);
            if clipped.is_empty() {
                return false;
            }
        }
        if let Some(rect) = self.0 {
            self.0 = Some(
                SourceToDestinationRect(&source.transform, &destination.transform, rect)
                    .unwrap_or_default(),
            );
        }
        // An empty result from an edge-on 3D transform may become visible
        // again under a later transform.
        true
    }

    /// Source: `cull_rect.cc:275-430`.
    pub fn ApplyPaintProperties(
        &mut self,
        root: &PropertyTreeState,
        source: &PropertyTreeState,
        destination: &PropertyTreeState,
        old_cull_rect: Option<Self>,
        expansion_ratio: f64,
    ) -> bool {
        debug_assert!(source != destination);
        if self.IsInfinite() && SameClip(&destination.clip, &source.clip) {
            return false;
        }

        let abnormal_hierarchy = !ClipIsAncestor(&source.clip, &destination.clip)
            || !TransformIsAncestor(&source.transform, &destination.transform);
        if abnormal_hierarchy {
            *self = Self::Infinite();
            return root != destination
                && self.ApplyPaintProperties(
                    root,
                    root,
                    destination,
                    old_cull_rect,
                    expansion_ratio,
                );
        }

        let mut scroll_translations = Vec::new();
        let mut has_transform_requiring_expansion = false;
        let mut current = Some(destination.transform.clone());
        while let Some(transform) = current {
            if SameTransform(&transform, &source.transform) {
                break;
            }
            if transform.scroll.is_some() {
                scroll_translations.push(transform.clone());
            } else if RequiresCullRectExpansion(&transform) {
                has_transform_requiring_expansion = true;
            }
            current = transform.parent.clone();
        }

        let mut last_transform = source.transform.clone();
        let mut last_clip = source.clip.clone();
        let mut expanded = (false, false);
        let effect_root = PropertyTreeState::default().effect;
        for scroll_translation in scroll_translations.into_iter().rev() {
            let Some(overflow_clip) = scroll_translation
                .scroll
                .as_ref()
                .and_then(|scroll| scroll.OverflowClipNode().cloned())
            else {
                break;
            };
            let Some(parent_transform) = scroll_translation.parent.clone() else {
                return false;
            };
            let source_state = PropertyTreeState {
                transform: last_transform,
                clip: last_clip,
                effect: effect_root.clone(),
            };
            let scroll_container_state = PropertyTreeState {
                transform: parent_transform,
                clip: overflow_clip.clone(),
                effect: effect_root.clone(),
            };
            if !self.ApplyPaintPropertiesWithoutExpansion(&source_state, &scroll_container_state) {
                return false;
            }
            last_clip = overflow_clip;
            expanded =
                self.ApplyScrollTranslation(&root.transform, &scroll_translation, expansion_ratio);
            last_transform = scroll_translation;
        }

        let final_source = PropertyTreeState {
            transform: last_transform.clone(),
            clip: last_clip.clone(),
            effect: effect_root.clone(),
        };
        if !self.ApplyPaintPropertiesWithoutExpansion(&final_source, destination) {
            return false;
        }
        if self.IsInfinite() {
            return false;
        }

        if let Some(rect) = &mut self.0 {
            if rect.x < -REASONABLE_PIXEL_LIMIT {
                rect.x = -REASONABLE_PIXEL_LIMIT;
            }
            if rect.y < -REASONABLE_PIXEL_LIMIT {
                rect.y = -REASONABLE_PIXEL_LIMIT;
            }
            if Right(*rect) > REASONABLE_PIXEL_LIMIT {
                rect.width = (REASONABLE_PIXEL_LIMIT - rect.x).max(0.0);
            }
            if Bottom(*rect) > REASONABLE_PIXEL_LIMIT {
                rect.height = (REASONABLE_PIXEL_LIMIT - rect.y).max(0.0);
            }
        }

        let mut expansion_bounds = if expanded.0 || expanded.1 {
            last_transform
                .scroll
                .as_ref()
                .map(|scroll| scroll.ContentsRect())
        } else {
            None
        };
        if let Some(bounds) = &mut expansion_bounds {
            if !SameTransform(&last_transform, &destination.transform)
                || !SameClip(&last_clip, &destination.clip)
            {
                let intermediate = PropertyTreeState {
                    transform: last_transform.clone(),
                    clip: last_clip.clone(),
                    effect: effect_root.clone(),
                };
                if let Some(clip) = LocalToAncestorClipRect(destination, &intermediate) {
                    *bounds = bounds.intersection(clip);
                }
                *bounds = SourceToDestinationRect(&last_transform, &destination.transform, *bounds)
                    .unwrap_or_default();
            }
        }

        if expansion_ratio > 0.0 && has_transform_requiring_expansion {
            let pixel_distance = LocalPixelDistanceToExpandForTransform(
                &root.transform,
                &destination.transform,
                expansion_ratio,
            )
            .0;
            if let Some(rect) = &mut self.0 {
                if rect.width < pixel_distance {
                    *rect = Outset(*rect, pixel_distance, 0.0);
                    expansion_bounds =
                        expansion_bounds.map(|bounds| Outset(bounds, pixel_distance, 0.0));
                    expanded.0 = true;
                }
                if rect.height < pixel_distance {
                    *rect = Outset(*rect, 0.0, pixel_distance);
                    expansion_bounds =
                        expansion_bounds.map(|bounds| Outset(bounds, 0.0, pixel_distance));
                    expanded.1 = true;
                }
            }
        }

        if let Some(old) = old_cull_rect {
            let is_small = IsSmallScrollerTransform(&destination.transform, expansion_ratio);
            if !self.ChangedEnough(expanded, old, expansion_bounds, is_small, expansion_ratio) {
                *self = old;
            }
        }
        expanded.0 || expanded.1
    }

    /// Source: `cull_rect.cc:433-505`.
    pub fn ChangedEnough(
        self,
        expanded: (bool, bool),
        old: Self,
        expansion_bounds: Option<PaintRect>,
        is_small_scroller: bool,
        expansion_ratio: f64,
    ) -> bool {
        let (Some(new_rect), Some(old_rect)) = (self.0, old.0) else {
            return self != old;
        };
        if old_rect.is_empty() && new_rect.is_empty() {
            return false;
        }
        if !expanded.0 && (new_rect.x != old_rect.x || new_rect.width != old_rect.width) {
            return true;
        }
        if !expanded.1 && (new_rect.y != old_rect.y || new_rect.height != old_rect.height) {
            return true;
        }
        if old_rect.contains(new_rect) {
            return false;
        }
        if old_rect.is_empty() {
            return true;
        }
        let threshold = ChangedEnoughMinimumDistance(is_small_scroller, expansion_ratio);
        if !Outset(old_rect, threshold, threshold).contains(new_rect) {
            return true;
        }
        let Some(bounds) = expansion_bounds else {
            return false;
        };
        (new_rect.x == bounds.x && old_rect.x != bounds.x)
            || (new_rect.y == bounds.y && old_rect.y != bounds.y)
            || (Right(new_rect) == Right(bounds) && Right(old_rect) != Right(bounds))
            || (Bottom(new_rect) == Bottom(bounds) && Bottom(old_rect) != Bottom(bounds))
    }

    /// Source: `cull_rect.cc:507-530`.
    pub fn HasScrolledEnough(
        self,
        delta: (f64, f64),
        can_expand_for_scroll: bool,
        is_small_scroller: bool,
        expansion_ratio: f64,
        contents_rect: PaintRect,
    ) -> bool {
        if !can_expand_for_scroll {
            return delta != (0.0, 0.0);
        }
        let threshold = ChangedEnoughMinimumDistance(is_small_scroller, expansion_ratio);
        if delta.0.abs() < threshold && delta.1.abs() < threshold {
            return false;
        }
        let Some(rect) = self.0 else {
            return false;
        };
        if rect.contains(contents_rect) {
            return false;
        }
        (delta.0 < 0.0 && rect.x != contents_rect.x)
            || (delta.0 > 0.0 && Right(rect) != Right(contents_rect))
            || (delta.1 < 0.0 && rect.y != contents_rect.y)
            || (delta.1 > 0.0 && Bottom(rect) != Bottom(contents_rect))
    }
}

#[inline]
pub fn IsSmallScroller(container: PaintRect, root_scroller: bool, expansion_ratio: f64) -> bool {
    !root_scroller
        && container.width * container.height
            <= SMALL_SCROLLER_AREA * expansion_ratio * expansion_ratio
}

/// Source: `cull_rect.cc:48-56`.
#[inline]
pub fn ChangedEnoughMinimumDistance(is_small_scroller: bool, expansion_ratio: f64) -> f64 {
    if is_small_scroller {
        SMALL_SCROLLER_CHANGED_ENOUGH_DISTANCE * expansion_ratio
    } else {
        CULL_RECT_CHANGED_ENOUGH_DISTANCE * expansion_ratio
    }
}

/// Source: `cull_rect.cc:58-103` for the supported unit-scale property tree.
#[inline]
pub fn LocalPixelDistanceToExpand(is_small_scroller: bool, expansion_ratio: f64) -> (f64, f64) {
    let minimum = 2.0 * ChangedEnoughMinimumDistance(is_small_scroller, expansion_ratio);
    if is_small_scroller {
        (minimum, minimum)
    } else {
        (
            (CULL_RECT_PIXEL_DISTANCE_TO_EXPAND * expansion_ratio).max(minimum),
            minimum,
        )
    }
}

/// Source: `cull_rect.cc:158-244`.
///
/// Coordinates are in scrolling-content space and remain there when retained
/// on FragmentData; callers map only while crossing a property-state boundary.
pub fn ApplyScrollExpansion(
    container_rect: PaintRect,
    contents_rect: PaintRect,
    scroll_offset: (f64, f64),
    is_small_scroller: bool,
    expansion_ratio: f64,
) -> (CullRect, (bool, bool)) {
    let mut rect = PaintRect {
        x: scroll_offset.0,
        y: scroll_offset.1,
        ..container_rect
    };
    if rect.intersection(contents_rect).is_empty() {
        return (CullRect::FromRect(PaintRect::default()), (false, false));
    }
    let (mut outset_x, minimum) = LocalPixelDistanceToExpand(is_small_scroller, expansion_ratio);
    let mut outset_y = outset_x;
    let range_x = (contents_rect.width - container_rect.width).max(0.0);
    let range_y = (contents_rect.height - container_rect.height).max(0.0);
    if range_x == 0.0 {
        outset_x = 0.0;
    }
    if range_y == 0.0 {
        outset_y = 0.0;
    }
    if outset_x > 0.0 && outset_y > 0.0 {
        outset_x /= 2.0;
        outset_y /= 2.0;
        if outset_x > range_x {
            outset_y += outset_x - range_x;
        }
        if outset_y > range_y {
            outset_x += outset_y - range_y;
        }
    }
    outset_x = outset_x.max(minimum).min(range_x);
    outset_y = outset_y.max(minimum).min(range_y);
    rect = Outset(rect, outset_x, outset_y).intersection(contents_rect);
    (CullRect::FromRect(rect), (outset_x > 0.0, outset_y > 0.0))
}

#[inline]
fn Outset(rect: PaintRect, x: f64, y: f64) -> PaintRect {
    PaintRect {
        x: rect.x - x,
        y: rect.y - y,
        width: rect.width + 2.0 * x,
        height: rect.height + 2.0 * y,
    }
}

#[inline]
fn Right(rect: PaintRect) -> f64 {
    rect.x + rect.width
}

#[inline]
fn Bottom(rect: PaintRect) -> f64 {
    rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn ChildTransform(
        parent: &Arc<TransformPaintPropertyNode>,
        id: u64,
        matrix: TransformMatrix,
        scroll: Option<Arc<ScrollPaintPropertyNode>>,
    ) -> Arc<TransformPaintPropertyNode> {
        let mut node = (**parent).clone();
        node.lifecycle = Default::default();
        node.id = id;
        node.parent = Some(parent.clone());
        node.matrix = matrix;
        node.scroll = scroll;
        node.direct_compositing_reasons = Default::default();
        Arc::new(node)
    }

    fn ChildClip(
        parent: &Arc<ClipPaintPropertyNode>,
        transform: &Arc<TransformPaintPropertyNode>,
        id: u64,
        rect: PaintRect,
    ) -> Arc<ClipPaintPropertyNode> {
        let mut node = (**parent).clone();
        node.lifecycle = Default::default();
        node.id = id;
        node.parent = Some(parent.clone());
        node.local_transform_space = transform.clone();
        node.rect = Some(rect);
        Arc::new(node.with_layout_clips())
    }

    #[test]
    fn two_axis_scroll_expansion_is_clipped_to_contents() {
        let container = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let contents = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 3000.0,
            height: 3000.0,
        };
        let (cull, expanded) =
            ApplyScrollExpansion(container, contents, (1400.0, 1400.0), false, 1.0);
        assert_eq!(expanded, (true, true));
        assert_eq!(
            cull.Rect(),
            Some(PaintRect {
                x: 0.0,
                y: 0.0,
                width: 3000.0,
                height: 3000.0
            })
        );
    }

    #[test]
    fn edge_transition_updates_before_the_distance_threshold() {
        let bounds = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 1000.0,
        };
        let old = CullRect::FromRect(PaintRect {
            x: 0.0,
            y: 100.0,
            width: 100.0,
            height: 800.0,
        });
        let new = CullRect::FromRect(PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 800.0,
        });
        assert!(new.ChangedEnough((false, true), old, Some(bounds), false, 1.0));
    }

    #[test]
    fn transform_and_range_queries_use_the_cull_rect_coordinate_space() {
        let mut transform = TransformMatrix::default();
        transform.values[12] = 10.0;
        transform.values[13] = 20.0;
        let mut cull = CullRect::FromRect(PaintRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        });
        cull.ApplyTransform(&transform);
        assert_eq!(
            cull.Rect(),
            Some(PaintRect {
                x: 0.0,
                y: 0.0,
                width: 30.0,
                height: 40.0,
            })
        );
        assert!(cull.IntersectsHorizontalRange(29.0, 31.0));
        assert!(!cull.IntersectsHorizontalRange(30.0, 31.0));
        assert!(cull.IntersectsVerticalRange(-1.0, 1.0));
        cull.Move(Offset { x: 2.0, y: 3.0 });
        assert!(cull.IntersectsTransformed(
            &transform,
            PaintRect {
                x: -8.0,
                y: -17.0,
                width: 1.0,
                height: 1.0,
            }
        ));
    }

    #[test]
    fn paint_properties_apply_clip_before_mapping_into_destination_transform() {
        let root = PropertyTreeState::default();
        let transform = ChildTransform(
            &root.transform,
            1,
            crate::geometry_mapper::TranslationTransform(10.0, 20.0),
            None,
        );
        let clip = ChildClip(
            &root.clip,
            &root.transform,
            1,
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 80.0,
            },
        );
        let destination = PropertyTreeState {
            transform,
            clip,
            effect: root.effect.clone(),
        };
        let mut cull = CullRect::FromRect(PaintRect {
            x: -50.0,
            y: -50.0,
            width: 200.0,
            height: 200.0,
        });
        assert!(!cull.ApplyPaintProperties(&root, &root, &destination, None, 1.0));
        assert_eq!(
            cull.Rect(),
            Some(PaintRect {
                x: -10.0,
                y: -20.0,
                width: 100.0,
                height: 80.0,
            })
        );
    }

    #[test]
    fn paint_properties_scroll_translation_expands_from_complete_container() {
        let root = PropertyTreeState::default();
        let container = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let overflow_clip = ChildClip(&root.clip, &root.transform, 1, container);
        let scroll = Arc::new(ScrollPaintPropertyNode {
            compositor: Default::default(),
            lifecycle: Default::default(),
            id: 1,
            owner_client_id: 1,
            owner_node_id: 0,
            parent: root.transform.scroll.clone(),
            overflow_clip: Some(overflow_clip.clone()),
            container_rect: container,
            contents_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 1000.0,
            },
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset { x: 0.0, y: 900.0 },
            user_scrollable_horizontal: false,
            user_scrollable_vertical: true,
        });
        assert!(CullRect::CanExpandForScroll(&scroll));
        let scroll_translation = ChildTransform(
            &root.transform,
            1,
            crate::geometry_mapper::TranslationTransform(0.0, -200.0),
            Some(scroll),
        );
        let destination = PropertyTreeState {
            transform: scroll_translation,
            clip: overflow_clip,
            effect: root.effect.clone(),
        };
        let mut cull = CullRect::FromRect(container);
        assert!(cull.ApplyPaintProperties(&root, &root, &destination, None, 1.0));
        assert_eq!(
            cull.Rect(),
            Some(PaintRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 1000.0,
            })
        );
    }

    #[test]
    fn relative_projection_does_not_depend_on_a_singular_shared_ancestor() {
        let root = PropertyTreeState::default();
        let mut singular = TransformMatrix::default();
        singular.values[0] = 0.0;
        let plane = ChildTransform(&root.transform, 1, singular, None);
        let child = ChildTransform(
            &plane,
            2,
            crate::geometry_mapper::TranslationTransform(10.0, 20.0),
            None,
        );
        assert_eq!(
            crate::geometry_mapper::SourceToDestinationRect(
                &plane,
                &child,
                PaintRect {
                    x: 10.0,
                    y: 20.0,
                    width: 30.0,
                    height: 40.0,
                },
            ),
            Some(PaintRect {
                x: 0.0,
                y: 0.0,
                width: 30.0,
                height: 40.0,
            })
        );
    }

    #[test]
    fn abnormal_clip_hierarchy_declines_to_clip() {
        let root = PropertyTreeState::default();
        let first = ChildClip(
            &root.clip,
            &root.transform,
            1,
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
        );
        let second = ChildClip(
            &root.clip,
            &root.transform,
            2,
            PaintRect {
                x: 20.0,
                y: 20.0,
                width: 10.0,
                height: 10.0,
            },
        );
        let local = PropertyTreeState {
            transform: root.transform.clone(),
            clip: first,
            effect: root.effect.clone(),
        };
        let ancestor = PropertyTreeState {
            transform: root.transform.clone(),
            clip: second,
            effect: root.effect.clone(),
        };
        assert_eq!(
            crate::geometry_mapper::LocalToAncestorClipRect(&local, &ancestor),
            None
        );
    }
}
