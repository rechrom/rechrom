//! Generic retained property-tree storage and node synchronization.

use crate::{
    clip_node::ClipNode,
    effect_node::EffectNode,
    geometry::{OutsetsF, RectF, Size, Vector2d, Vector2dF},
    math_util::MathUtil,
    property_ids::{kInvalidPropertyNodeId, kRootPropertyNodeId, kViewportPropertyNodeId},
    property_types::{
        FilterOperations, PropertyPath, PropertyTrace, PropertyTransform, TracedValueJSON,
    },
    sticky_position_constraint::StickyPositionConstraint,
    transform::PropertyTreeTransform,
    transform_node::{TransformCachedNodeData, TransformNode},
    ElementId,
};
use std::{
    cell::{RefCell, RefMut},
    collections::{BTreeMap, BTreeSet},
};

// Source: cc/trees/mutator_host.h:43.
const kInvalidScale: f32 = 0.0;

// Source: cc/trees/property_tree.h:414-427; cc/trees/property_tree.cc:44-51.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnchorPositionScrollData {
    pub adjustment_container_ids: Vec<ElementId>,
    pub accumulated_scroll_origin: Vector2d,
    pub needs_scroll_adjustment_in_x: bool,
    pub needs_scroll_adjustment_in_y: bool,
}

// Source: cc/trees/property_tree.h:429-454; cc/trees/property_tree.cc:52-54.
#[derive(Clone, Debug, PartialEq)]
pub struct StickyPositionNodeData {
    pub x_scroll_ancestor: i32,
    pub y_scroll_ancestor: i32,
    pub constraints: StickyPositionConstraint,
    pub nearest_node_shifting_sticky_box: i32,
    pub nearest_node_shifting_containing_block: i32,
    pub total_sticky_box_sticky_offset: Vector2dF,
    pub total_containing_block_sticky_offset: Vector2dF,
}

impl Default for StickyPositionNodeData {
    fn default() -> Self {
        Self {
            x_scroll_ancestor: kInvalidPropertyNodeId,
            y_scroll_ancestor: kInvalidPropertyNodeId,
            constraints: StickyPositionConstraint::default(),
            nearest_node_shifting_sticky_box: kInvalidPropertyNodeId,
            nearest_node_shifting_containing_block: kInvalidPropertyNodeId,
            total_sticky_box_sticky_offset: Vector2dF::default(),
            total_containing_block_sticky_offset: Vector2dF::default(),
        }
    }
}

// Source: cc/trees/property_tree.h:211-222; cc/trees/property_tree.cc:271-273.
#[derive(Default)]
pub struct UpdateTransformsData {
    pub stale_forward_dependencies: BTreeSet<i32>,
}

/// Authoritative work counts for one retained transform-tree update.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransformUpdateStats {
    pub passes: usize,
    pub node_visits: usize,
    pub forward_dependency_retries: usize,
    pub max_stale_forward_dependencies: usize,
}

/// Runs the retry policy for transform nodes whose anchor dependencies point
/// forward in tree order. The callback updates one resident node and records
/// unresolved dependencies in `UpdateTransformsData`.
pub fn RunTransformUpdatePasses(
    first_node_id: i32,
    node_count: i32,
    mut update_node: impl FnMut(i32, &mut UpdateTransformsData),
) -> TransformUpdateStats {
    let mut trace = browser_tracing::span("compositor.tree", "Compositor.Tree.UpdateTransforms");
    let mut stats = TransformUpdateStats::default();
    let mut update_data = UpdateTransformsData::default();
    loop {
        let stale_before_pass = update_data.stale_forward_dependencies.len();
        stats.passes += 1;
        for id in first_node_id..node_count {
            stats.node_visits += 1;
            update_node(id, &mut update_data);
        }
        let stale_after_pass = update_data.stale_forward_dependencies.len();
        stats.max_stale_forward_dependencies =
            stats.max_stale_forward_dependencies.max(stale_after_pass);
        assert!(stale_before_pass == 0 || stale_after_pass < stale_before_pass);
        if stale_after_pass == 0 {
            break;
        }
        stats.forward_dependency_retries += stale_after_pass;
    }
    trace.set("passes", stats.passes as f64);
    trace.set("node_visits", stats.node_visits as f64);
    trace.set(
        "forward_dependency_retries",
        stats.forward_dependency_retries as f64,
    );
    trace.set(
        "max_stale_forward_dependencies",
        stats.max_stale_forward_dependencies as f64,
    );
    stats
}

// Source: cc/trees/property_tree.h:1007-1031.
// This is the canonical retained state shared by transform/effect/scroll trees.
pub struct PropertyTreesState {
    pub needs_rebuild_: bool,
    pub changed_: bool,
    pub full_tree_damaged_: bool,
    pub is_main_thread_: bool,
    pub is_active_: bool,
    pub sequence_number_: i32,
    pub inner_viewport_container_bounds_delta_: Vector2dF,
    pub outer_viewport_container_bounds_delta_: Vector2dF,
    pub transform_delta_by_safe_area_inset_bottom_: f32,
}

impl PropertyTreesState {
    pub fn is_main_thread(&self) -> bool {
        self.is_main_thread_
    }

    pub fn is_active(&self) -> bool {
        self.is_active_
    }

    pub fn set_changed(&mut self, value: bool) {
        self.changed_ = value;
    }

    pub fn inner_viewport_container_bounds_delta(&self) -> Vector2dF {
        self.inner_viewport_container_bounds_delta_
    }
}

impl Default for PropertyTreesState {
    fn default() -> Self {
        Self {
            needs_rebuild_: true,
            changed_: false,
            full_tree_damaged_: false,
            is_main_thread_: true,
            is_active_: false,
            sequence_number_: 0,
            inner_viewport_container_bounds_delta_: Vector2dF::default(),
            outer_viewport_container_bounds_delta_: Vector2dF::default(),
            transform_delta_by_safe_area_inset_bottom_: 0.0,
        }
    }
}

// Source: cc/trees/property_tree.h:798-818.
pub const kInvalidUpdateNumber: i32 = -1;

#[derive(Clone, Debug)]
pub struct AnimationScaleData {
    pub update_number: i32,
    pub maximum_to_screen_scale: f32,
    pub affected_by_animation_scale: bool,
    pub affected_by_invalid_scale: bool,
}

impl Default for AnimationScaleData {
    fn default() -> Self {
        Self {
            update_number: kInvalidUpdateNumber,
            maximum_to_screen_scale: 0.0,
            affected_by_animation_scale: false,
            affected_by_invalid_scale: false,
        }
    }
}

// Source: cc/trees/property_tree.h:820-843.
#[derive(Clone, Debug)]
pub struct DrawTransforms<T: PropertyTransform> {
    pub might_be_invertible: bool,
    pub from_valid: bool,
    pub to_valid: bool,
    pub from_target: T,
    pub to_target: T,
}

impl<T: PropertyTransform> DrawTransforms<T> {
    pub fn new(from: T, to: T) -> Self {
        Self {
            might_be_invertible: true,
            from_valid: false,
            to_valid: false,
            from_target: from,
            to_target: to,
        }
    }
}

impl<T: PropertyTransform> PartialEq for DrawTransforms<T> {
    fn eq(&self, other: &Self) -> bool {
        self.from_valid == other.from_valid
            && self.to_valid == other.to_valid
            && self.from_target == other.from_target
            && self.to_target == other.to_target
    }
}

// Source: cc/trees/property_tree.h:845-852.
#[derive(Clone, Debug)]
pub struct DrawTransformData<T: PropertyTransform> {
    pub update_number: i32,
    pub effect_id: i32,
    pub transforms: DrawTransforms<T>,
}

impl<T: PropertyTransform> Default for DrawTransformData<T> {
    fn default() -> Self {
        Self {
            update_number: kInvalidUpdateNumber,
            effect_id: kInvalidPropertyNodeId,
            transforms: DrawTransforms::new(T::default(), T::default()),
        }
    }
}

// Source: cc/trees/property_tree.h:854-861; cc/trees/property_tree.cc:2429-2434.
pub struct PropertyTreesCachedData<T: PropertyTransform> {
    transform_tree_update_number: i32,
    animation_scales: Vec<AnimationScaleData>,
    draw_transforms: RefCell<Vec<Vec<DrawTransformData<T>>>>,
}

impl<T: PropertyTransform> Default for PropertyTreesCachedData<T> {
    fn default() -> Self {
        Self {
            transform_tree_update_number: 0,
            animation_scales: Vec::new(),
            draw_transforms: RefCell::new(Vec::new()),
        }
    }
}

impl<T: PropertyTransform> PropertyTreesCachedData<T> {
    pub fn UpdateTransformTreeUpdateNumber(&mut self) {
        self.transform_tree_update_number += 1;
    }

    pub fn transform_tree_update_number(&self) -> i32 {
        self.transform_tree_update_number
    }

    pub fn animation_scales(&self) -> &[AnimationScaleData] {
        &self.animation_scales
    }

    pub fn animation_scales_mut(&mut self) -> &mut [AnimationScaleData] {
        &mut self.animation_scales
    }

    pub fn AnimationScaleCacheIsInvalid(&self, id: i32) -> bool {
        assert!(id >= 0 && (id as usize) < self.animation_scales.len());
        self.animation_scales[id as usize].update_number == kInvalidUpdateNumber
    }

    pub fn ResetForTransformCount(&mut self, count: usize) {
        self.transform_tree_update_number = 0;
        self.animation_scales
            .resize_with(count, AnimationScaleData::default);
        for scale in &mut self.animation_scales {
            scale.update_number = kInvalidUpdateNumber;
        }
        let entries = self.draw_transforms.get_mut();
        entries.resize_with(count, || vec![DrawTransformData::default()]);
        for values in entries {
            values.resize_with(1, DrawTransformData::default);
            values[0].update_number = kInvalidUpdateNumber;
            values[0].effect_id = kInvalidPropertyNodeId;
        }
    }

    pub fn FetchDrawTransformsData(
        &self,
        transform_id: i32,
        effect_id: i32,
    ) -> RefMut<'_, DrawTransformData<T>> {
        let mut all = self.draw_transforms.borrow_mut();
        assert!(transform_id >= 0 && (transform_id as usize) < all.len());
        let entries = &mut all[transform_id as usize];
        let index = if let Some(index) = entries.iter().position(|data| {
            data.effect_id == effect_id || data.effect_id == kInvalidPropertyNodeId
        }) {
            index
        } else {
            entries.push(DrawTransformData::default());
            let index = entries.len() - 1;
            entries[index].update_number = kInvalidUpdateNumber;
            entries[index].effect_id = effect_id;
            index
        };
        RefMut::map(all, |all| &mut all[transform_id as usize][index])
    }
}

/// Updates one animation-scale cache entry. The screen-space scale callback is
/// intentionally lazy because computing it walks the transform tree.
pub fn UpdateAnimationScaleData<T: PropertyTreeTransform>(
    current: &mut AnimationScaleData,
    node: &TransformNode<T>,
    parent: Option<&AnimationScaleData>,
    update_number: i32,
    to_screen_scale: impl FnOnce() -> Vector2dF,
) {
    current.update_number = update_number;
    let ancestor_affected = parent.is_some_and(|scale| scale.affected_by_animation_scale);
    let node_affected = node.has_potential_animation && node.maximum_animation_scale != 1.0;
    current.affected_by_animation_scale = node_affected || ancestor_affected;
    current.affected_by_invalid_scale = parent.is_some_and(|scale| scale.affected_by_invalid_scale)
        || node.to_parent.HasPerspective()
        || (node.has_potential_animation && node.maximum_animation_scale == kInvalidScale);
    let failed_for_multiple = ancestor_affected && node_affected;
    let mut local_maximum = 1.0;
    if current.affected_by_invalid_scale || failed_for_multiple {
    } else if !node.to_screen_is_potentially_animated {
        let scales = to_screen_scale();
        current.maximum_to_screen_scale = scales.x().max(scales.y());
        return;
    } else if !node.has_potential_animation {
        let scales = node.local.ComputeTransform2dScaleComponents(1.0);
        local_maximum = scales.x().max(scales.y());
    } else {
        debug_assert_ne!(node.maximum_animation_scale, kInvalidScale);
        local_maximum = node.maximum_animation_scale;
    }
    current.maximum_to_screen_scale = local_maximum;
    if let Some(parent) = parent {
        current.maximum_to_screen_scale *= parent.maximum_to_screen_scale;
    }
}

/// Read-only projection used by transform-composition algorithms. The concrete
/// retained owner stays in the embedding compositor while tree math lives here.
pub trait TransformTreeRead<T: PropertyTreeTransform> {
    fn TransformNode(&self, id: i32) -> &TransformNode<T>;
    fn ToScreen(&self, id: i32) -> &T;
    fn FromScreen(&self, id: i32) -> &T;
}

fn IsTransformDescendant<T: PropertyTreeTransform>(
    tree: &impl TransformTreeRead<T>,
    mut descendant_id: i32,
    source_id: i32,
) -> bool {
    while descendant_id != source_id {
        if descendant_id == kInvalidPropertyNodeId {
            return false;
        }
        descendant_id = tree.TransformNode(descendant_id).parent_id;
    }
    true
}

// Source: cc/trees/property_tree.cc:319-396.
pub fn CombineTransformsBetween<T: PropertyTreeTransform>(
    tree: &impl TransformTreeRead<T>,
    source_id: i32,
    destination_id: i32,
    transform: &mut T,
) {
    debug_assert!(source_id > destination_id);
    let current = tree.TransformNode(source_id);
    let destination = tree.TransformNode(destination_id);
    if destination.ancestors_are_invertible && destination.node_and_ancestors_are_flat {
        transform.PostConcat(tree.ToScreen(current.id));
        transform.PostConcat(tree.FromScreen(destination.id));
        return;
    }
    let mut source_to_destination = vec![current.id];
    let mut current_node = (current.parent_id != kInvalidPropertyNodeId)
        .then(|| tree.TransformNode(current.parent_id));
    while let Some(node) = current_node.filter(|node| node.id > destination_id) {
        source_to_destination.push(node.id);
        current_node =
            (node.parent_id != kInvalidPropertyNodeId).then(|| tree.TransformNode(node.parent_id));
    }
    let mut combined_transform = T::default();
    let ancestor = current_node.expect("source/destination common ancestor");
    if ancestor.id < destination_id {
        debug_assert!(IsTransformDescendant(tree, destination_id, ancestor.id));
        CombineInversesBetween(tree, ancestor.id, destination_id, &mut combined_transform);
    }
    for node_id in source_to_destination.into_iter().rev() {
        let node = tree.TransformNode(node_id);
        if node.flattens_inherited_transform {
            combined_transform.Flatten();
        }
        combined_transform.PreConcat(&node.to_parent);
    }
    transform.PostConcat(&combined_transform);
}

// Source: cc/trees/property_tree.cc:397-424.
pub fn CombineInversesBetween<T: PropertyTreeTransform>(
    tree: &impl TransformTreeRead<T>,
    source_id: i32,
    destination_id: i32,
    transform: &mut T,
) -> bool {
    debug_assert!(source_id < destination_id);
    let current = tree.TransformNode(destination_id);
    let destination = tree.TransformNode(source_id);
    if current.ancestors_are_invertible && current.node_and_ancestors_are_flat {
        transform.PreConcat(tree.FromScreen(current.id));
        transform.PreConcat(tree.ToScreen(destination.id));
        return true;
    }
    let mut destination_to_source = T::default();
    CombineTransformsBetween(tree, destination_id, source_id, &mut destination_to_source);
    let mut source_to_destination = T::default();
    let all_are_invertible = destination_to_source.GetInverse(&mut source_to_destination);
    transform.PreConcat(&source_to_destination);
    all_are_invertible
}

pub fn UpdateAnimationProperties<T: PropertyTreeTransform>(
    node: &mut TransformNode<T>,
    parent: &TransformNode<T>,
) {
    node.to_screen_is_potentially_animated =
        node.has_potential_animation || parent.to_screen_is_potentially_animated;
}

pub fn UpdateScreenSpaceTransform<T: PropertyTreeTransform>(
    node: &mut TransformNode<T>,
    parent: &TransformNode<T>,
    parent_to_screen: &T,
    cached: &mut TransformCachedNodeData<T>,
) {
    let mut to_screen = parent_to_screen.clone();
    if node.flattens_inherited_transform {
        to_screen.Flatten();
    }
    to_screen.PreConcat(&node.to_parent);
    node.ancestors_are_invertible = parent.ancestors_are_invertible;
    node.node_and_ancestors_are_flat =
        parent.node_and_ancestors_are_flat && node.to_parent.IsFlat();
    cached.to_screen = to_screen;
    let mut from_screen = T::default();
    if !cached.to_screen.GetInverse(&mut from_screen) {
        node.ancestors_are_invertible = false;
    }
    cached.from_screen = from_screen;
}

pub fn UndoTransformSnapping<T: PropertyTreeTransform>(node: &mut TransformNode<T>) {
    node.to_parent
        .Translate(-node.snap_amount.x(), -node.snap_amount.y());
    node.snap_amount = Vector2dF::default();
}

pub fn UpdateTransformSnapping<T: PropertyTreeTransform>(
    node: &mut TransformNode<T>,
    cached: &mut TransformCachedNodeData<T>,
) {
    if !node.should_be_snapped
        || node.to_screen_is_potentially_animated
        || !cached.to_screen.IsScaleOrTranslation()
        || !node.ancestors_are_invertible
    {
        return;
    }
    cached.to_screen.Round2dTranslationComponents();
    let mut delta = cached.from_screen.clone();
    delta.MultiplyAssign(&cached.to_screen);
    const kTolerance: f32 = 1e-4;
    debug_assert!(delta.IsApproximatelyIdentityOrTranslation(kTolerance));
    let translation = delta.To2dTranslation();
    node.snap_amount = translation;
    if translation.IsZero() {
        return;
    }
    cached
        .from_screen
        .PostTranslate(Vector2dF::new(-translation.x(), -translation.y()));
    node.to_parent.Translate(translation.x(), translation.y());
    if node
        .to_parent
        .IsApproximatelyIdentityOrIntegerTranslation(kTolerance)
    {
        node.to_parent.RoundToIdentityOrIntegerTranslation();
    }
}

pub fn PropagateTransformChanged<T: PropertyTreeTransform>(
    node: &mut TransformNode<T>,
    parent: &TransformNode<T>,
) {
    if parent.transform_changed() {
        node.CopyTransformChangedFrom(parent);
    }
}

pub fn UpdateNodeAndAncestorsAreAnimatedOrInvertible<T: PropertyTreeTransform>(
    node: &mut TransformNode<T>,
    parent: &TransformNode<T>,
) {
    if !parent.node_and_ancestors_are_animated_or_invertible {
        node.node_and_ancestors_are_animated_or_invertible = false;
        return;
    }
    let mut is_invertible = node.is_invertible;
    if !node.ancestors_are_invertible && parent.ancestors_are_invertible {
        is_invertible = false;
    }
    node.node_and_ancestors_are_animated_or_invertible =
        node.has_potential_animation || is_invertible;
}

pub fn UpdateNodeOrAncestorsWillChangeTransform<T: PropertyTreeTransform>(
    node: &mut TransformNode<T>,
    parent: &TransformNode<T>,
) {
    node.node_or_ancestors_will_change_transform =
        node.will_change_transform || parent.node_or_ancestors_will_change_transform;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RootScaleUpdate {
    pub device_transform_scale_factor: f32,
    pub root_transform_changed: bool,
    pub contents_transform_changed: bool,
}

pub fn UpdateRootScaleAndTransform<T: PropertyTreeTransform>(
    device_scale_factor: f32,
    device_transform: &T,
    root_cache: &mut TransformCachedNodeData<T>,
    contents_node: &mut TransformNode<T>,
) -> RootScaleUpdate {
    let components = device_transform.ComputeTransform2dScaleComponents(1.0);
    let device_transform_scale_factor = components.x().max(components.y());
    let mut contents_transform = device_transform.clone();
    contents_transform.Scale(device_scale_factor, device_scale_factor);
    let screen_space_scale =
        contents_transform.ComputeTransform2dScaleComponents(device_scale_factor);
    debug_assert!(screen_space_scale.x() != 0.0);
    debug_assert!(screen_space_scale.y() != 0.0);
    let mut root_to_screen = T::default();
    root_to_screen.Scale(screen_space_scale.x(), screen_space_scale.y());
    let root_from_screen = root_to_screen.GetCheckedInverse();
    let root_transform_changed = root_to_screen != root_cache.to_screen;
    if root_transform_changed {
        root_cache.to_screen = root_to_screen;
        root_cache.from_screen = root_from_screen.clone();
        root_cache.is_showing_backface = root_cache.to_screen.IsBackFaceVisible();
    }
    contents_transform.PostConcat(&root_from_screen);
    let contents_transform_changed = contents_node.local != contents_transform;
    if contents_transform_changed {
        contents_node.local = contents_transform;
        contents_node.needs_local_transform_update = true;
    }
    RootScaleUpdate {
        device_transform_scale_factor,
        root_transform_changed,
        contents_transform_changed,
    }
}

pub fn MarkTransformsNeedingLocalUpdate<T: PropertyTreeTransform>(
    nodes: &mut [TransformNode<T>],
    affected_node_ids: &[i32],
) -> bool {
    if affected_node_ids.is_empty() {
        return false;
    }
    for &id in affected_node_ids {
        nodes[id as usize].needs_local_transform_update = true;
    }
    true
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StickyPositionOffsetResult {
    pub translated_offset: Vector2dF,
    pub total_sticky_box_sticky_offset: Vector2dF,
    pub total_containing_block_sticky_offset: Vector2dF,
}

pub fn ComputeStickyPositionOffset(
    constraint: &StickyPositionConstraint,
    scroll_position: crate::geometry::PointF,
    constraint_box_expansion: Vector2dF,
    ancestor_sticky_box_offset: Vector2dF,
    ancestor_containing_block_offset: Vector2dF,
) -> StickyPositionOffsetResult {
    let sticky_offset = constraint.StickyPositionOffset(
        scroll_position,
        constraint_box_expansion,
        ancestor_sticky_box_offset,
        ancestor_containing_block_offset,
    );
    StickyPositionOffsetResult {
        translated_offset: crate::geometry::RoundedVector2dF(
            sticky_offset + constraint.pixel_snap_offset,
        ),
        total_sticky_box_sticky_offset: ancestor_sticky_box_offset + sticky_offset,
        total_containing_block_sticky_offset: ancestor_sticky_box_offset
            + ancestor_containing_block_offset
            + sticky_offset,
    }
}

pub fn TransformAnchorOffset<T: PropertyTreeTransform>(
    offset: Vector2dF,
    container_to_screen: &T,
    parent_from_screen: &T,
) -> Vector2dF {
    if offset.IsZero() {
        return offset;
    }
    let mut mapper = container_to_screen.clone();
    mapper.PostConcat(parent_from_screen);
    let transformed_offset = mapper.MapPoint(crate::geometry::PointF::new(offset.x(), offset.y()));
    transformed_offset - mapper.MapPoint(crate::geometry::PointF::default())
}

pub fn ComputeAnchorPositionOffset(
    data: &AnchorPositionScrollData,
    accumulated_offset: Vector2dF,
) -> Vector2dF {
    let mut result = Vector2dF::new(
        data.accumulated_scroll_origin.x as f32,
        data.accumulated_scroll_origin.y as f32,
    ) - accumulated_offset;
    if !data.needs_scroll_adjustment_in_x {
        result.set_x(0.0);
    }
    if !data.needs_scroll_adjustment_in_y {
        result.set_y(0.0);
    }
    result
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverscrollUndoResult {
    pub position_adjustment: Vector2dF,
    pub clip_outsets: OutsetsF,
}

pub fn ComputeOverscrollUndo(
    position_adjustment: Vector2dF,
    overscroll_offset: Vector2dF,
    page_scale_factor: f32,
) -> OverscrollUndoResult {
    let position_adjustment =
        position_adjustment + MathUtil::ScaleVectorByInverse(overscroll_offset, page_scale_factor);
    let mut clip_outsets = OutsetsF::default();
    if position_adjustment.x() < 0.0 {
        clip_outsets.left = -position_adjustment.x();
    } else {
        clip_outsets.right = position_adjustment.x();
    }
    if position_adjustment.y() < 0.0 {
        clip_outsets.top = -position_adjustment.y();
    } else {
        clip_outsets.bottom = position_adjustment.y();
    }
    OverscrollUndoResult {
        position_adjustment,
        clip_outsets,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ElasticOverscrollTransform {
    #[default]
    None,
    Translate {
        offset: Vector2dF,
    },
    Stretch {
        offset: Vector2dF,
        scroller_size: Size,
        scrolls_inner_viewport: bool,
        page_scale_factor: f32,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LocalTransformInputs {
    pub position_adjustment: Vector2dF,
    pub elastic_overscroll: ElasticOverscrollTransform,
    pub sticky_offset: Vector2dF,
    pub anchor_offset: Vector2dF,
}

/// Composes the complete local transform in source order. Tree adapters resolve
/// scroll/viewport identities, while this package owns the matrix work and its
/// performance attribution.
pub fn ComposeLocalTransform<T: PropertyTreeTransform>(
    node: &TransformNode<T>,
    inputs: LocalTransformInputs,
) -> T {
    let mut transform = T::default();
    transform.Translate3d(
        node.post_translation.x() + node.origin.x(),
        node.post_translation.y() + node.origin.y(),
        node.origin.z(),
    );
    transform.Translate(
        inputs.position_adjustment.x(),
        inputs.position_adjustment.y(),
    );
    match inputs.elastic_overscroll {
        ElasticOverscrollTransform::None => {}
        ElasticOverscrollTransform::Translate { offset } => {
            transform.Translate(-offset.x(), -offset.y());
        }
        ElasticOverscrollTransform::Stretch {
            offset,
            scroller_size,
            scrolls_inner_viewport,
            page_scale_factor,
        } => {
            if !scroller_size.IsEmpty() && !offset.IsZero() {
                let scale_factor = Vector2dF::new(
                    1.0 + offset.x().abs() / scroller_size.width() as f32,
                    1.0 + offset.y().abs() / scroller_size.height() as f32,
                );
                let mut pivot = crate::geometry::PointF::default();
                if offset.x() > 0.0 {
                    pivot.set_x(scroller_size.width() as f32);
                }
                if offset.y() > 0.0 {
                    pivot.set_y(scroller_size.height() as f32);
                }
                if scrolls_inner_viewport {
                    pivot = MathUtil::ScalePointByInverse(pivot, page_scale_factor);
                }
                transform.Translate(pivot.x(), pivot.y());
                transform.Scale(scale_factor.x(), scale_factor.y());
                transform.Translate(-pivot.x(), -pivot.y());
            }
        }
    }
    transform.Translate(-node.scroll_offset().x(), -node.scroll_offset().y());
    transform.Translate(inputs.sticky_offset.x(), inputs.sticky_offset.y());
    transform.Translate(inputs.anchor_offset.x(), inputs.anchor_offset.y());
    transform.PreConcat(&node.local);
    transform.Translate3d(-node.origin.x(), -node.origin.y(), -node.origin.z());
    transform
}

/// Non-owning identity replacing the C++ PropertyTrees backpointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropertyTreesOwnerId(u64);

impl PropertyTreesOwnerId {
    pub const fn FromOwnerValue(value: u64) -> Self {
        Self(value)
    }
}

/// Field projection shared by every retained property-tree node family.
pub trait PropertyNode: Default + Clone + PartialEq {
    fn id(&self) -> i32;
    fn set_id(&mut self, id: i32);
    fn parent_id(&self) -> i32;
    fn set_parent_id(&mut self, id: i32);
    fn AsValueInto(&self, value: &mut impl PropertyTrace);
}

macro_rules! property_node_fields {
    () => {
        fn id(&self) -> i32 {
            self.id
        }
        fn set_id(&mut self, id: i32) {
            self.id = id;
        }
        fn parent_id(&self) -> i32 {
            self.parent_id
        }
        fn set_parent_id(&mut self, id: i32) {
            self.parent_id = id;
        }
    };
}

impl<T: PropertyTransform> PropertyNode for TransformNode<T> {
    property_node_fields!();
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        TransformNode::AsValueInto(self, value);
    }
}

impl PropertyNode for ClipNode {
    property_node_fields!();
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        ClipNode::AsValueInto(self, value);
    }
}

impl<F: FilterOperations, P: PropertyPath> PropertyNode for EffectNode<F, P> {
    property_node_fields!();
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        EffectNode::AsValueInto(self, value);
    }
}

// Source: cc/trees/property_tree.h:66-75,150-159,162-169.
// C++ deletes copy construction while permitting assignment; Rust similarly
// exposes explicit AssignFrom without deriving Clone for this owner.
pub struct PropertyTree<T: PropertyNode> {
    pub(crate) nodes_: Vec<T>,
    element_id_to_node_index_: BTreeMap<ElementId, i32>,
    needs_update_: bool,
    property_trees_: Option<PropertyTreesOwnerId>,
}

impl<T: PropertyNode> PropertyTree<T> {
    // Source: cc/trees/property_tree.h:150; cc/trees/property_tree.cc:55-62.
    pub fn new(property_trees: Option<PropertyTreesOwnerId>) -> Self {
        let mut result = Self {
            nodes_: vec![T::default()],
            element_id_to_node_index_: BTreeMap::new(),
            needs_update_: false,
            property_trees_: property_trees,
        };
        result.MutableBack().unwrap().set_id(kRootPropertyNodeId);
        result
            .MutableBack()
            .unwrap()
            .set_parent_id(kInvalidPropertyNodeId);
        result
    }

    // Source: cc/trees/property_tree.h:75; cc/trees/property_tree.cc:70-72.
    pub fn AssignFrom(&mut self, other: &Self) {
        self.nodes_.clone_from(&other.nodes_);
        self.element_id_to_node_index_
            .clone_from(&other.element_id_to_node_index_);
        self.needs_update_ = other.needs_update_;
        self.property_trees_ = other.property_trees_;
    }

    // Source: cc/trees/property_tree.h:82; cc/trees/property_tree.cc:91-100.
    pub fn Insert(&mut self, tree_node: &T, parent_id: i32) -> i32 {
        debug_assert!(!self.nodes_.is_empty());
        self.nodes_.push(tree_node.clone());
        let node_id = self.nodes_.len() as i32 - 1;
        let node = self.nodes_.last_mut().unwrap();
        node.set_parent_id(parent_id);
        node.set_id(node_id);
        node_id
    }

    // Source: cc/trees/property_tree.h:84-85; cc/trees/property_tree.cc:101-111.
    pub fn RemoveNodes(&mut self, n: usize) {
        assert!(n <= self.nodes_.len());
        self.nodes_.truncate(self.nodes_.len() - n);
        let upper_bound = i32::try_from(self.nodes_.len()).expect("checked node count");
        self.element_id_to_node_index_
            .retain(|_, index| *index < upper_bound);
    }

    pub fn Node(&self, i: i32) -> &T {
        assert!(i < self.nodes_.len() as i32);
        assert!(i > kInvalidPropertyNodeId);
        &self.nodes_[i as usize]
    }

    pub fn MutableNode(&mut self, i: i32) -> &mut T {
        assert!(i < self.nodes_.len() as i32);
        assert!(i > kInvalidPropertyNodeId);
        &mut self.nodes_[i as usize]
    }

    pub fn HasParent(&self, node: &T) -> bool {
        node.parent_id() != kInvalidPropertyNodeId
    }

    pub fn parent(&self, node: &T) -> &T {
        self.Node(node.parent_id())
    }

    pub fn MutableParent(&mut self, node: &T) -> &mut T {
        self.MutableNode(node.parent_id())
    }

    pub fn MutableBack(&mut self) -> Option<&mut T> {
        self.nodes_.last_mut()
    }

    pub fn back(&self) -> Option<&T> {
        self.nodes_.last()
    }

    pub fn SetElementIdForNodeId(&mut self, node_id: i32, element_id: ElementId) {
        self.element_id_to_node_index_.insert(element_id, node_id);
    }

    pub fn MutableFindNodeFromElementId(&mut self, id: ElementId) -> Option<&mut T> {
        let index = *self.element_id_to_node_index_.get(&id)?;
        if index == kInvalidPropertyNodeId {
            return None;
        }
        Some(self.MutableNode(index))
    }

    pub fn FindNodeFromElementId(&self, id: ElementId) -> Option<&T> {
        let index = *self.element_id_to_node_index_.get(&id)?;
        if index == kInvalidPropertyNodeId {
            return None;
        }
        Some(self.Node(index))
    }

    pub fn clear(&mut self) {
        self.needs_update_ = false;
        self.nodes_.clear();
        self.nodes_.push(T::default());
        self.MutableBack().unwrap().set_id(kRootPropertyNodeId);
        self.MutableBack()
            .unwrap()
            .set_parent_id(kInvalidPropertyNodeId);
        self.element_id_to_node_index_.clear();
        #[cfg(debug_assertions)]
        {
            let tree = Self::new(None);
            debug_assert!(tree == *self);
        }
    }

    pub fn size(&self) -> usize {
        self.nodes_.len()
    }

    pub fn set_needs_update(&mut self, value: bool) {
        self.needs_update_ = value;
    }

    pub fn needs_update(&self) -> bool {
        self.needs_update_
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<T> {
        &mut self.nodes_
    }

    pub fn nodes(&self) -> &Vec<T> {
        &self.nodes_
    }

    pub fn next_available_id(&self) -> i32 {
        self.size() as i32
    }

    pub fn property_trees(&self) -> Option<PropertyTreesOwnerId> {
        self.property_trees_
    }

    pub fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        value.BeginArray("nodes");
        for node in &self.nodes_ {
            value.BeginDictionaryUnnamed();
            node.AsValueInto(value);
            value.EndDictionary();
        }
        value.EndArray();
        value.SetBoolean("needs_update", self.needs_update_);
    }

    pub fn element_id_to_node_index(&self) -> &BTreeMap<ElementId, i32> {
        &self.element_id_to_node_index_
    }

    pub fn CopyElementIdMapFrom(&mut self, other: &Self) {
        self.element_id_to_node_index_
            .clone_from(&other.element_id_to_node_index_);
    }

    pub fn SetPropertyTrees(&mut self, property_trees: Option<PropertyTreesOwnerId>) {
        self.property_trees_ = property_trees;
    }

    #[cfg(debug_assertions)]
    pub fn ToString(&self) -> String {
        let mut value = TracedValueJSON::default();
        self.AsValueInto(&mut value);
        value.ToFormattedJSON()
    }
}

#[cfg(debug_assertions)]
impl<T: PropertyNode> PartialEq for PropertyTree<T> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes() == other.nodes()
            && self.needs_update() == other.needs_update()
            && self.element_id_to_node_index() == other.element_id_to_node_index()
    }
}

// Source: cc/trees/property_tree.h:456-470; cc/trees/property_tree.cc:1690-1714.
pub struct ClipTree {
    base_: PropertyTree<ClipNode>,
}

impl std::ops::Deref for ClipTree {
    type Target = PropertyTree<ClipNode>;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl std::ops::DerefMut for ClipTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

impl ClipTree {
    pub fn new(owner: Option<PropertyTreesOwnerId>) -> Self {
        Self {
            base_: PropertyTree::new(owner),
        }
    }

    pub fn AssignFrom(&mut self, from: &Self) -> &mut Self {
        self.base_.AssignFrom(&from.base_);
        self
    }

    pub fn SetViewportClip(&mut self, viewport_rect: RectF) {
        if self.size() < 2 {
            return;
        }
        let node = self.MutableNode(1);
        if viewport_rect == node.clip {
            return;
        }
        node.clip = viewport_rect;
        self.set_needs_update(true);
    }

    pub fn ViewportClip(&self) -> RectF {
        debug_assert!(self.size() > 1);
        self.Node(kViewportPropertyNodeId).clip
    }
}

#[cfg(debug_assertions)]
impl PartialEq for ClipTree {
    fn eq(&self, other: &Self) -> bool {
        self.base_ == other.base_
    }
}

#[cfg(test)]
mod tests {
    use super::RunTransformUpdatePasses;

    #[test]
    fn transform_update_retries_forward_dependencies_until_resolved() {
        let mut first_node_visits = 0;
        let stats = RunTransformUpdatePasses(1, 3, |id, data| {
            if id != 1 {
                return;
            }
            first_node_visits += 1;
            if first_node_visits == 1 {
                data.stale_forward_dependencies.insert(2);
            } else {
                data.stale_forward_dependencies.remove(&2);
            }
        });

        assert_eq!(stats.passes, 2);
        assert_eq!(stats.node_visits, 4);
        assert_eq!(stats.forward_dependency_retries, 1);
        assert_eq!(stats.max_stale_forward_dependencies, 1);
    }

    #[test]
    fn transform_update_empty_tree_still_finishes_one_pass() {
        let stats = RunTransformUpdatePasses(1, 1, |_, _| unreachable!());
        assert_eq!(stats.passes, 1);
        assert_eq!(stats.node_visits, 0);
    }
}
