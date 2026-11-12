//! IntersectionGeometry source, Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Borrow the resident LayoutObject/container/clip owners. The observation's
//! own derived rect cache crosses the boundary, never a parallel layout tree.
use super::*;
use crate::internal::{
    layout_box_model_object::LayoutBoxModelObject,
    layout_inline::LayoutInline,
    layout_object::{AncestorSkipInfo, LayoutObject},
    layout_replaced::LayoutReplaced,
    layout_text::LayoutText,
    map_coordinates_flags::MapCoordinatesFlags,
};
use foundation::graphics_types::graphics::visual_rect_flags::{VisualRectFlag, VisualRectFlags};
use foundation::{gfx, DynamicTo};
use intersection_observer::{
    geometry_flags as F, ApplyMargin, CachedRects, IntersectionGeometry,
    IntersectionObserverParams, RequiredIntersectionMechanism as Required, RootGeometry,
    MINIMUM_THRESHOLD,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
#[derive(Clone, Copy, Debug)]
pub enum NativeIntersectionRoot {
    Document,
    Element(u64),
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Relationship {
    Invalid,
    IntermediateClippers,
    NotScrollable,
    ScrollableByRootOnly,
}
struct RootAndTarget<'a> {
    root: &'a LayoutObject,
    target: &'a LayoutObject,
    relationship: Relationship,
    root_scrolls_target: bool,
    has_filter: bool,
    intermediate_scrollers: Vec<&'a LayoutBox>,
}
fn clips_self(object: &LayoutObject) -> bool {
    object.HasCSSClip() || object.HasClipPath() || object.HasMask() || object.IsSVGChild()
}
fn clips_contents(object: &LayoutObject) -> bool {
    clips_self(object) || object.ShouldClipOverflowAlongEitherAxis()
}
fn box_(object: &LayoutObject) -> Option<&LayoutBox> {
    unsafe { DynamicTo::<LayoutBox>(object as *const _).as_ref() }
}
impl<'a> RootAndTarget<'a> {
    fn new(root: &'a LayoutObject, target: &'a LayoutObject, scroll_margin: bool) -> Self {
        let mut result = Self {
            root,
            target,
            relationship: Relationship::Invalid,
            root_scrolls_target: false,
            has_filter: false,
            intermediate_scrollers: vec![],
        };
        if std::ptr::eq(root, target) {
            return result;
        }
        let mut clipping = clips_self(target);
        let mut container = target as *const LayoutObject;
        let mut previous = container;
        while container != root as *const _ {
            let object = unsafe { &*container };
            result.has_filter |= object.HasFilterInducingProperty();
            let mut skip = AncestorSkipInfo::new_with_filters(root, !result.has_filter);
            previous = container;
            container = object.ContainerWithSkipInfo(&mut skip);
            if !result.has_filter {
                result.has_filter = skip.FilterSkipped();
            }
            if skip.AncestorSkipped() || container.is_null() {
                return result;
            }
            let object = unsafe { &*container };
            if container != root as *const _ && clips_contents(object) {
                clipping = true;
            }
            if container != root as *const _ && scroll_margin && object.IsScrollContainer() {
                result
                    .intermediate_scrollers
                    .push(box_(object).expect("native scroll container box"));
            }
        }
        result.root_scrolls_target = root.IsScrollContainer()
            && box_(root).is_some_and(LayoutBox::HasScrollableOverflow)
            && !(root.IsLayoutView() && unsafe { &*previous }.IsFixedPositioned());
        result.relationship = if clipping {
            Relationship::IntermediateClippers
        } else if result.root_scrolls_target {
            Relationship::ScrollableByRootOnly
        } else {
            Relationship::NotScrollable
        };
        result
    }
}
fn object_to_view(object: &LayoutObject) -> gfx::Transform {
    // The native visual-rect boundary currently runs the source TransformState
    // slow path (layout_object_fragment_queries.rs). This is that same source
    // ObjectToViewTransform fallback, not a DOM bounding-rect projection.
    object.LocalToAbsoluteTransform(MapCoordinatesFlags::default())
}
fn root_rect(root: &LayoutObject) -> gfx::RectF {
    if let Some(box_) = box_(root) {
        if root.IsLayoutView() || box_.ShouldClipOverflowAlongBothAxis() {
            gfx::RectF::from(box_.OverflowClipRect())
        } else {
            gfx::RectF::from(box_.PhysicalBorderBoxRect())
        }
    } else {
        gfx::RectF::default()
    }
}
fn target_rect(target: &LayoutObject, flags: u32) -> Result<gfx::RectF, Required> {
    if flags & F::FRAME_VIEWPORT != 0 {
        // InitializeTargetRect, intersection_geometry.cc:114-116. The
        // FrameView caller supplies its LayoutEmbeddedContent owner, whose
        // translated replaced box retains this same content rect.
        let embedded = unsafe { DynamicTo::<LayoutReplaced>(target as *const _).as_ref() }
            .expect("frame-viewport intersection requires its native replaced owner");
        return Ok(gfx::RectF::from(embedded.ReplacedContentRect()));
    }
    if target.IsSVGChild() {
        return Ok(target.DecoratedBoundingBox());
    }
    if let Some(box_) = box_(target) {
        let mut bounds = box_.PhysicalBorderBoxRect();
        if flags & F::OVERFLOW_CLIP_EDGE != 0 && box_.ShouldApplyOverflowClipMargin() {
            bounds.Unite(&box_.OverflowClipRect());
        }
        return Ok(gfx::RectF::from(bounds));
    }
    if let Some(inline) = unsafe { DynamicTo::<LayoutInline>(target as *const _).as_ref() } {
        return Ok(inline.LocalBoundingBoxRectF());
    }
    let text = unsafe { DynamicTo::<LayoutText>(target as *const _).as_ref() }
        .expect("source allowed target kind");
    Ok(gfx::RectF::from(text.PhysicalLinesBoundingBox()))
}
fn offset_scrolled_contents(box_: &LayoutBox, rect: &mut gfx::RectF) {
    let area = unsafe { box_.GetScrollableArea().as_ref() }.expect("native scroll container area");
    let position = area.ScrollPosition().OffsetFromOrigin();
    rect.Offset(-position.x(), -position.y());
}
fn apply_clip(
    target: &LayoutObject,
    ancestor: &LayoutBox,
    root: &LayoutObject,
    root_rect: gfx::RectF,
    unclipped: &mut gfx::RectF,
    intersection: &mut gfx::RectF,
    scroll_margin: &[foundation::Length],
    ignore_local_clip_path: bool,
    root_scrolls_target: bool,
    use_cache: bool,
    cache: &mut CachedRects,
    flags: u32,
) -> Result<bool, Required> {
    let mut visual = VisualRectFlags::default();
    for flag in [
        VisualRectFlag::kEdgeInclusive,
        VisualRectFlag::kDontApplyMainFrameOverflowClip,
        VisualRectFlag::kUsePreciseClipPath,
    ] {
        visual.Add(flag);
    }
    if flags & F::RESPECT_FILTERS == 0 {
        visual.Add(VisualRectFlag::kIgnoreFilters);
    }
    if ignore_local_clip_path {
        visual.Add(VisualRectFlag::kIgnoreLocalClipPath);
    }
    let mut intersects = if use_cache {
        cache.does_intersect
    } else {
        target.MapToVisualRectInAncestorSpaceFloat(
            ancestor as *const LayoutBox as *const LayoutBoxModelObject,
            unclipped,
            visual,
        )
    };
    if !use_cache && ancestor.IsScrollContainer() && !root_scrolls_target {
        offset_scrolled_contents(ancestor, unclipped);
    }
    cache.unscrolled_unclipped_intersection_rect = *unclipped;
    cache.does_intersect = intersects;
    *intersection = gfx::RectF::default();
    if intersects {
        if root_scrolls_target {
            offset_scrolled_contents(ancestor, unclipped);
        } else {
            // SVG viewport replacement transforms need their canonical paint
            // property projection; never silently treat that transform as identity.
            if ancestor.IsSVGRoot() {
                return Err(Required::ReplacedContentTransform);
            }
        }
        let mut clip = root_rect;
        if !scroll_margin.is_empty() && root.IsScrollContainer() {
            let reference = clip.size();
            ApplyMargin(
                &mut clip,
                scroll_margin,
                root.StyleRef().EffectiveZoom(),
                reference,
            );
        }
        *intersection = *unclipped;
        intersects &= intersection.InclusiveIntersect(&clip);
    }
    Ok(intersects)
}
fn clip_to_root(
    relation: &RootAndTarget,
    root_rect: gfx::RectF,
    unclipped: &mut gfx::RectF,
    intersection: &mut gfx::RectF,
    scroll_margin: &[foundation::Length],
    use_cache: bool,
    cache: &mut CachedRects,
    flags: u32,
) -> Result<bool, Required> {
    let Some(root_box) = box_(relation.root) else {
        return Ok(false);
    };
    let mut target = relation.target;
    let mut ignore_local = false;
    for scroller in &relation.intermediate_scrollers {
        // The precise local clip-path bound is required by this source branch.
        if scroller.HasClipPath() {
            return Err(Required::PreciseClipPath);
        }
        let rect = gfx::RectF::from(scroller.OverflowClipRect());
        if !apply_clip(
            target,
            scroller,
            scroller,
            rect,
            unclipped,
            intersection,
            scroll_margin,
            ignore_local,
            true,
            use_cache,
            cache,
            flags,
        )? {
            return Ok(false);
        }
        *unclipped = *intersection;
        target = scroller;
        ignore_local = true;
    }
    apply_clip(
        target,
        root_box,
        relation.root,
        root_rect,
        unclipped,
        intersection,
        scroll_margin,
        ignore_local,
        relation.root_scrolls_target,
        use_cache,
        cache,
        flags,
    )
}
fn min_scroll_delta(
    geometry: &IntersectionGeometry,
    relation: &RootAndTarget,
    target_transform: &gfx::Transform,
    root_transform: &gfx::Transform,
    thresholds: &[f32],
    scroll_margin: &[foundation::Length],
    flags: u32,
) -> [f32; 2] {
    if !scroll_margin.is_empty() || flags & F::COMPUTE_VISIBILITY != 0 {
        return [0.0; 2];
    }
    if relation.relationship == Relationship::NotScrollable {
        return [f32::MAX; 2];
    }
    if relation.has_filter && flags & F::RESPECT_FILTERS != 0 {
        return [0.0; 2];
    }
    let translation = |t: &gfx::Transform| t.IsIdentityOrTranslation() && t.rc(2, 3) == 0.0;
    if !translation(target_transform) || !translation(root_transform) {
        return [0.0; 2];
    }
    let root = geometry.root_rect;
    let target = geometry.target_rect;
    if thresholds[0] == 1.0 {
        if flags & F::FRACTION_OF_ROOT != 0 {
            if root.width() > target.width() || root.height() > target.height() {
                return [f32::MAX; 2];
            }
            if target.Contains(&root) && relation.relationship == Relationship::IntermediateClippers
            {
                return [0.0; 2];
            }
        } else {
            if target.width() > root.width() || target.height() > root.height() {
                return [f32::MAX; 2];
            }
            if root.Contains(&target) && relation.relationship == Relationship::IntermediateClippers
            {
                return [0.0; 2];
            }
        }
        return [
            (root.x() - target.x())
                .abs()
                .min((root.right() - target.right()).abs()),
            (root.y() - target.y())
                .abs()
                .min((root.bottom() - target.bottom()).abs()),
        ];
    }
    let mut intersect = root;
    let intersects = intersect.InclusiveIntersect(&target);
    if intersects
        && (thresholds.len() != 1
            || thresholds[0] > MINIMUM_THRESHOLD
            || relation.relationship == Relationship::IntermediateClippers
            || flags & F::FRAME_VIEWPORT != 0)
    {
        return [0.0; 2];
    }
    [
        (root.right() - target.x())
            .abs()
            .min((target.right() - root.x()).abs()),
        (root.bottom() - target.y())
            .abs()
            .min((target.bottom() - root.y()).abs()),
    ]
}
impl LayoutEngine {
    /// DOM validates connection/display-lock and explicit-root frame identity
    /// before this synchronous resident query. No lifecycle is triggered here.
    pub fn ComputeIntersectionGeometryForNode(
        &self,
        target_id: u64,
        root_node: Option<NativeIntersectionRoot>,
        standardized_browser_zoom: bool,
        frame_occlusion: intersection_observer::OcclusionState,
        params: &IntersectionObserverParams,
        flags: u32,
        root_geometry: &mut Option<RootGeometry>,
        cache: &mut CachedRects,
    ) -> Result<IntersectionGeometry, Required> {
        let _scope = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        let mut result = IntersectionGeometry {
            should_report_root_bounds: flags & F::REPORT_ROOT_BOUNDS != 0,
            ..Default::default()
        };
        let Some(tree) = self.GetLayoutTree() else {
            cache.valid = false;
            return Ok(result);
        };
        let Some(target) = tree.GetLayoutObjectForNode(target_id) else {
            cache.valid = false;
            return Ok(result);
        };
        if !(target.IsBoxModelObject() || target.IsText() || target.IsSVG())
            || tree.Root().NeedsLayout()
        {
            cache.valid = false;
            return Ok(result);
        }
        let root = match root_node {
            None | Some(NativeIntersectionRoot::Document) => tree.Root(),
            Some(NativeIntersectionRoot::Element(id)) => {
                let Some(root) = tree.GetLayoutObjectForNode(id) else {
                    cache.valid = false;
                    return Ok(result);
                };
                root
            }
        };
        let honor = flags & F::REPORT_ROOT_BOUNDS != 0;
        let root_margin = if honor { params.RootMargin() } else { &[] };
        let target_margin = if honor { params.TargetMargin() } else { &[] };
        let scroll_margin = if honor {
            params.scroll_margin.as_slice()
        } else {
            &[]
        };
        let relation = RootAndTarget::new(root, target, !scroll_margin.is_empty());
        let was_valid = cache.valid;
        cache.valid = false;
        if relation.relationship == Relationship::Invalid {
            return Ok(result);
        }
        // Keep lower mechanisms explicit until their precise native clip and
        // hit-test providers are translated; no viewport/bounding-box fallback.
        let mut object = Some(target);
        while let Some(current) = object {
            if current.HasClipPath() {
                return Err(Required::PreciseClipPath);
            }
            if current.HasMask() {
                return Err(Required::MaskVisualClip);
            }
            if current.HasFilterInducingProperty() && flags & F::RESPECT_FILTERS != 0 {
                return Err(Required::FilterVisualRect);
            }
            if std::ptr::eq(current, root) {
                break;
            }
            object = unsafe { current.Parent().as_ref() };
        }
        let use_cache = was_valid
            && relation.intermediate_scrollers.is_empty()
            && flags & F::SCROLL_AND_VISIBILITY_ONLY != 0
            && matches!(
                relation.relationship,
                Relationship::NotScrollable | Relationship::ScrollableByRootOnly
            );
        if root_geometry.is_none() {
            *root_geometry = Some(if root.IsBox() {
                let pre = root_rect(root);
                let mut g = RootGeometry {
                    zoom: root.StyleRef().EffectiveZoom(),
                    pre_margin_local_root_rect: pre,
                    local_root_rect: pre,
                    root_to_view_transform: object_to_view(root),
                };
                g.UpdateMargin(root_margin);
                g
            } else {
                RootGeometry {
                    zoom: 1.0,
                    pre_margin_local_root_rect: gfx::RectF::default(),
                    local_root_rect: gfx::RectF::default(),
                    root_to_view_transform: gfx::Transform::default(),
                }
            });
        }
        let root_geometry = root_geometry.as_ref().expect("initialized root geometry");
        result.did_compute_geometry = true;
        assert!(
            !params.thresholds.is_empty(),
            "source requires at least one threshold"
        );
        let (local_target, pre_margin_empty, mut unclipped) = if use_cache {
            (
                cache.local_target_rect,
                cache.pre_margin_target_rect_is_empty,
                cache.unscrolled_unclipped_intersection_rect,
            )
        } else {
            let mut target_rect = target_rect(target, flags)?;
            let empty = target_rect.IsEmpty();
            ApplyMargin(
                &mut target_rect,
                target_margin,
                root_geometry.zoom,
                root_geometry.pre_margin_local_root_rect.size(),
            );
            (target_rect, empty, target_rect)
        };
        cache.local_target_rect = local_target;
        cache.pre_margin_target_rect_is_empty = pre_margin_empty;
        result.root_rect = root_geometry.local_root_rect;
        let intersects = clip_to_root(
            &relation,
            result.root_rect,
            &mut unclipped,
            &mut result.intersection_rect,
            scroll_margin,
            use_cache,
            cache,
            flags,
        )?;
        let target_transform = object_to_view(target);
        result.target_rect = target_transform.MapRect(local_target);
        if intersects {
            if root_node.is_some() {
                result.intersection_rect = root_geometry
                    .root_to_view_transform
                    .MapRect(result.intersection_rect);
                // Source cc:632-636 maps its existing output rect here.
                unclipped = root_geometry
                    .root_to_view_transform
                    .MapRect(gfx::RectF::default());
            }
            // In the Page's independent local main frame the implicit-root to
            // target-document transform is the native root-view transform.
            // Its inverse/project step is retained even when it is identity.
            else {
                let matrix = object_to_view(tree.Root());
                result.intersection_rect = matrix
                    .ProjectQuad(gfx::QuadF::from(result.intersection_rect))
                    .BoundingBox();
                unclipped = matrix
                    .ProjectQuad(gfx::QuadF::from(unclipped))
                    .BoundingBox();
            }
            result.unclipped_intersection_rect = unclipped;
        } else {
            result.intersection_rect = gfx::RectF::default();
            result.unclipped_intersection_rect = unclipped;
        }
        result.root_rect = root_geometry
            .root_to_view_transform
            .MapRect(result.root_rect);
        if intersects {
            let comparison = if flags & F::FRACTION_OF_ROOT != 0 {
                result.root_rect
            } else {
                result.target_rect
            };
            let empty_override = flags & F::FRACTION_OF_ROOT == 0 && pre_margin_empty;
            let ratio = if comparison.IsEmpty() || empty_override {
                1.0
            } else {
                let ratio = (result.intersection_rect.width() * result.intersection_rect.height())
                    / (comparison.width() * comparison.height());
                // C++ std::min retains a NaN first operand.
                if 1.0 < ratio {
                    1.0
                } else {
                    ratio
                }
            };
            result.intersection_ratio = f64::from(ratio);
            result.threshold_index = params
                .thresholds
                .iter()
                .take_while(|&&threshold| threshold <= ratio)
                .count();
        }
        if result.IsIntersecting()
            && flags & F::COMPUTE_VISIBILITY != 0
            && frame_occlusion == intersection_observer::OcclusionState::GuaranteedNotOccluded
        {
            // ComputeVisibilityInfo cc:145-156. A truly occluded frame is a
            // computed false result; an unoccluded frame still requires the
            // canonical LocalBorderBoxProperties visual-effect proof before
            // any penetrating hit test. HasDistortingVisualEffects currently
            // has no native provider, so never call its panic placeholder.
            if target.CanvasDrawElementEnabledForLayout() && target.IsInCanvasSubtree() {
                result.is_visible = false;
            } else {
                return Err(Required::DistortingVisualEffects);
            }
        }
        cache.min_scroll_delta_to_update = min_scroll_delta(
            &result,
            &relation,
            &target_transform,
            &root_geometry.root_to_view_transform,
            &params.thresholds,
            scroll_margin,
            flags,
        );
        cache.valid = true;
        if flags & F::CONVERT_TO_CSS_PIXELS != 0 {
            // AdjustForAbsoluteZoom::GetAbsoluteZoom, h:39-46. The canonical
            // Document runtime/OriginTrial fact controls CSS zoom exclusion.
            let target_zoom = if standardized_browser_zoom {
                self.constraints_.layout_zoom_factor as f32
            } else {
                target.StyleRef().EffectiveZoom()
            };
            let root_zoom = if standardized_browser_zoom {
                self.constraints_.layout_zoom_factor as f32
            } else {
                root.StyleRef().EffectiveZoom()
            };
            assert!(
                target_zoom.is_finite()
                    && target_zoom != 0.0
                    && root_zoom.is_finite()
                    && root_zoom != 0.0,
                "source absolute zoom"
            );
            result
                .target_rect
                .Scale(1.0 / target_zoom, 1.0 / target_zoom);
            result
                .intersection_rect
                .Scale(1.0 / target_zoom, 1.0 / target_zoom);
            result.root_rect.Scale(1.0 / root_zoom, 1.0 / root_zoom);
        }
        Ok(result)
    }
}
