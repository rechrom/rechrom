//! cc/trees/draw_property_utils helpers shared by the main LayerTreeHost and
//! the impl LayerTreeImpl: ScreenSpaceTransformForLayer (Layer and LayerImpl),
//! the property-tree updates ComputeTransforms / ComputeEffects /
//! ComputeClips and the clip mapping they use, and the main-thread
//! UpdatePropertyTrees(LayerTreeHost*). They read only the property trees, so
//! they are generic over the render-surface type: the main trees carry none
//! (cc/trees/property_tree.cc EffectTree::operator= does not copy
//! render_surfaces_), the impl trees carry RenderSurfaceImpl. The impl-only
//! draw-property walk (render targets, surfaces, layer draw properties) stays
//! in the compositor (compositor/src/cc/trees/draw_property_utils.rs).
use super::clip_node::{ClipNode, ConditionalClip};
use super::effect_node::EffectNode;
use super::geometry::{Rect, RectF, Vector2dF};
use super::layer_value_backend::DrawFilters;
use super::math_util::MathUtil;
use super::property_ids::{kContentsRootPropertyNodeId, kRootPropertyNodeId, kViewportPropertyNodeId};
use super::property_tree::{
    EffectTree, EffectTreeContext, PropertyTrees, TransformTree, TransformTreeContext,
};
use super::property_tree_services::{
    PropertyTreeCopyOutputRequest, PropertyTreeFeatures, PropertyTreeRenderSurface,
    PropertyTreeTransform,
};
use super::property_types::PropertyPath;
use super::viewport_property_ids::ViewportPropertyIds;
use compositor_tree::clip_tree_update::{ClipTreeUpdateContext, UpdateClipTree};

// Rust borrow-boundary entry for the same source cc/trees/draw_property_utils.cc:620-628.
// Both native Layer and LayerImpl forward scalar live getter results to this
// unique body; it never stores or copies a layer or property tree.
pub fn ScreenSpaceTransformForLayer<T: PropertyTreeTransform>(
    offset: Vector2dF,
    transform_tree_index: i32,
    tree: &TransformTree<T>,
) -> T {
    let mut transform = T::default();
    transform.Translate(offset.x(), offset.y());
    let screen = tree.ToScreen(transform_tree_index);
    transform.PostConcat(screen);
    transform
}

// Source: cc/trees/draw_property_utils.cc:52-55.
pub fn ToEnclosingClipRect(rect: RectF) -> Rect {
    compositor_tree::geometry::ToEnclosingRectIgnoringError(rect, 0.00001)
}

// Source: cc/trees/draw_property_utils.cc:65-70.
pub fn PostConcatSurfaceContentsScale<T: PropertyTreeTransform, F: DrawFilters, P: PropertyPath>(
    node: &EffectNode<F, P>,
    transform: &mut T,
) {
    debug_assert!(node.HasRenderSurface());
    transform.PostScale(
        node.surface_contents_scale.x(),
        node.surface_contents_scale.y(),
    );
}

// Source: cc/trees/draw_property_utils.h:36-37; cc/trees/draw_property_utils.cc:1525-1533.
pub fn ConcatInverseSurfaceContentsScale<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
>(
    node: &EffectNode<F, P>,
    transform: &mut T,
) {
    debug_assert!(node.HasRenderSurface());
    if node.surface_contents_scale.x() != 0.0 && node.surface_contents_scale.y() != 0.0 {
        transform.Scale(
            (1.0_f64 / node.surface_contents_scale.x() as f64) as f32,
            (1.0_f64 / node.surface_contents_scale.y() as f64) as f32,
        );
    }
}

// Source: cc/trees/draw_property_utils.cc:72-104.
pub fn ConvertRectBetweenSurfaceSpaces<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    p: &PropertyTrees<T, F, P, R, C>,
    source_effect_id: i32,
    dest_effect_id: i32,
    clip_in_source_space: RectF,
    clip_in_dest_space: &mut RectF,
) -> bool {
    let source = p.effect_tree().Node(source_effect_id);
    let dest = p.effect_tree().Node(dest_effect_id);
    let mut transform = T::default();
    if source.transform_id > dest.transform_id {
        if p.GetToTarget(source.transform_id, dest_effect_id, &mut transform) {
            ConcatInverseSurfaceContentsScale(source, &mut transform);
            *clip_in_dest_space = MathUtil::MapClippedRect(&transform, clip_in_source_space);
        } else {
            return false;
        }
    } else if p.GetFromTarget(dest.transform_id, source_effect_id, &mut transform) {
        PostConcatSurfaceContentsScale(dest, &mut transform);
        *clip_in_dest_space = MathUtil::ProjectClippedRect(&transform, clip_in_source_space);
    } else {
        return false;
    }
    true
}

// Source: cc/trees/draw_property_utils.cc:127-146.
pub fn ComputeLocalRectInTargetSpace<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    rect: RectF,
    p: &PropertyTrees<T, F, P, R, C>,
    current_transform_id: i32,
    target_transform_id: i32,
    target_effect_id: i32,
) -> ConditionalClip {
    let mut transform = T::default();
    if !p.GetToTarget(current_transform_id, target_effect_id, &mut transform) {
        return ConditionalClip::default();
    }
    ConditionalClip {
        is_clipped: true,
        clip_rect: if current_transform_id > target_transform_id {
            MathUtil::MapClippedRect(&transform, rect)
        } else {
            MathUtil::ProjectClippedRect(&transform, rect)
        },
    }
}

// Source: cc/trees/draw_property_utils.cc:148-168.
pub fn ComputeCurrentClip<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    node: &ClipNode,
    p: &PropertyTrees<T, F, P, R, C>,
    target_transform_id: i32,
    target_effect_id: i32,
) -> ConditionalClip {
    if node.transform_id != target_transform_id {
        return ComputeLocalRectInTargetSpace(
            node.clip,
            p,
            node.transform_id,
            target_transform_id,
            target_effect_id,
        );
    }
    let mut clip = node.clip;
    let scale = p
        .effect_tree()
        .Node(target_effect_id)
        .surface_contents_scale;
    if scale.x() > 0.0 && scale.y() > 0.0 && node.transform_id != kRootPropertyNodeId {
        clip.Scale(scale.x(), scale.y());
    }
    ConditionalClip {
        is_clipped: true,
        clip_rect: clip,
    }
}

// Source: cc/trees/draw_property_utils.cc:170-205.
pub fn ExpandClipForPixelMovingFilter<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    p: &PropertyTrees<T, F, P, R, C>,
    target_id: i32,
    node: &EffectNode<F, P>,
    clip: &mut RectF,
) -> bool {
    let mut mapping = RectF::default();
    if !ConvertRectBetweenSurfaceSpaces(p, target_id, node.id, *clip, &mut mapping) {
        return false;
    }
    let expanded = RectF::from(
        node.filters
            .ExpandRect(ToEnclosingClipRect(mapping), node.surface_contents_scale),
    );
    let original = *clip;
    if !ConvertRectBetweenSurfaceSpaces(p, node.id, target_id, expanded, clip) {
        return false;
    }
    clip.Union(original);
    true
}

// Source: cc/trees/draw_property_utils.cc:207-231.
pub fn ApplyClipNodeToAccumulatedClip<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    p: &PropertyTrees<T, F, P, R, C>,
    include_expanding_clips: bool,
    target_id: i32,
    target_transform_id: i32,
    node: &ClipNode,
    clip: &mut RectF,
) -> bool {
    if !node.AppliesLocalClip() {
        if !include_expanding_clips {
            return true;
        }
        return ExpandClipForPixelMovingFilter(
            p,
            target_id,
            p.effect_tree().Node(node.pixel_moving_filter_id),
            clip,
        );
    }
    let current = ComputeCurrentClip(node, p, target_transform_id, target_id);
    if !current.is_clipped {
        return false;
    }
    clip.Intersect(current.clip_rect);
    true
}

// Source: cc/trees/draw_property_utils.cc:858-885.
struct ClipTreeUpdateAdapter<
    'a,
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
> {
    p: &'a mut PropertyTrees<T, F, P, R, C>,
}

impl<
        T: PropertyTreeTransform,
        F: DrawFilters,
        P: PropertyPath,
        R: PropertyTreeRenderSurface,
        C: PropertyTreeCopyOutputRequest,
    > ClipTreeUpdateContext for ClipTreeUpdateAdapter<'_, T, F, P, R, C>
{
    fn needs_update(&self) -> bool {
        self.p.clip_tree().needs_update()
    }

    fn first_node_id(&self) -> i32 {
        kViewportPropertyNodeId
    }

    fn node_count(&self) -> i32 {
        self.p.clip_tree().size() as i32
    }

    fn clear_node_cache(&mut self, id: i32) {
        self.p
            .clip_tree()
            .Node(id)
            .cached_clip_rects
            .borrow_mut()
            .clear();
    }

    fn initialize_viewport_clip(&mut self, id: i32) {
        let clip = self.p.clip_tree().Node(id).clip;
        self.p
            .clip_tree_mutable()
            .MutableNode(id)
            .cached_accumulated_rect_in_screen_space = clip;
    }

    fn update_accumulated_clip(&mut self, id: i32) -> bool {
        let node = self.p.clip_tree().Node(id);
        let mut clip = self
            .p
            .clip_tree()
            .parent(node)
            .cached_accumulated_rect_in_screen_space;
        let success = ApplyClipNodeToAccumulatedClip(
            self.p,
            true,
            kContentsRootPropertyNodeId,
            kRootPropertyNodeId,
            node,
            &mut clip,
        );
        if success {
            self.p
                .clip_tree_mutable()
                .MutableNode(id)
                .cached_accumulated_rect_in_screen_space = clip;
        }
        success
    }

    fn clear_needs_update(&mut self) {
        self.p.clip_tree_mutable().set_needs_update(false);
    }
}

// Source: cc/trees/draw_property_utils.cc:858-885.
pub fn ComputeClips<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    p: &mut PropertyTrees<T, F, P, R, C>,
) {
    debug_assert!(!p.transform_tree().needs_update());
    let mut adapter = ClipTreeUpdateAdapter { p };
    UpdateClipTree(&mut adapter);
}

// Source: cc/trees/draw_property_utils.cc:1576-1579.
pub fn ComputeTransforms<T: PropertyTreeTransform>(
    tree: &mut TransformTree<T>,
    ids: &ViewportPropertyIds,
    context: &mut TransformTreeContext<'_, T>,
) {
    tree.UpdateAllTransforms(ids, context);
}

// Source: cc/trees/draw_property_utils.h:46-46; cc/trees/draw_property_utils.cc:1581-1588.
pub fn ComputeEffects<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    tree: &mut EffectTree<F, P, R, C>,
    context: &mut EffectTreeContext<'_, T>,
) {
    let needs_update = tree.needs_update();
    let end = tree.size() as i32;
    let stats = compositor_tree::effect_tree_update::UpdateEffectTree(
        needs_update,
        kContentsRootPropertyNodeId,
        end,
        |id| tree.UpdateEffects(id, context),
    );
    if stats.updated {
        tree.set_needs_update(false);
    }
}

// Source: cc/trees/draw_property_utils.h:48; cc/trees/draw_property_utils.cc:1590-1605
// (UpdatePropertyTrees(LayerTreeHost*), run by LayerTreeHost::DoUpdateLayers,
// cc/trees/layer_tree_host.cc:1093-1125, over the main PropertyTrees).
pub fn UpdatePropertyTrees<
    T: PropertyTreeTransform,
    F: DrawFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    property_trees: &mut PropertyTrees<T, F, P, R, C>,
    viewport_property_ids: &ViewportPropertyIds,
    features: &dyn PropertyTreeFeatures,
) {
    if property_trees.transform_tree().needs_update() {
        property_trees.clip_tree_mutable().set_needs_update(true);
        property_trees.effect_tree_mutable().set_needs_update(true);
    }
    {
        let (tree, mut context) = property_trees.transform_tree_with_context();
        ComputeTransforms(tree, viewport_property_ids, &mut context);
    }
    {
        let (tree, mut context) = property_trees.effect_tree_with_context(features);
        ComputeEffects(tree, &mut context);
    }
    // Computation of clips uses ToScreen which is updated while computing
    // transforms. So, ComputeTransforms should be before ComputeClips.
    ComputeClips(property_trees);
}
