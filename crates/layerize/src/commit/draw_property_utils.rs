//! cc/trees/draw_property_utils helpers shared by the main Layer and the
//! impl LayerImpl (both call ScreenSpaceTransformForLayer).
use super::property_tree::TransformTree;
use super::property_tree_services::PropertyTreeTransform;
use super::geometry::Vector2dF;

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
