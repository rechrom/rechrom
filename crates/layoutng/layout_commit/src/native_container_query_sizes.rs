// Publishing post-layout container sizes is commit work. Keep the complete
// native tree traversal out of the LayoutEngine facade for profiler ownership.

use super::*;

#[inline(never)]
pub(super) fn collect_container_query_sizes(
    engine: &LayoutEngine,
) -> Vec<ContainerQuerySizeSnapshot> {
    let Some(tree) = engine.tree_.as_ref() else {
        return Vec::new();
    };
    // This traversal uses LayoutObject's translated virtual child dispatch
    // through NextInPreOrder(). Keep the same installed object factories
    // available here as during Layout(), including SVG root children.
    let _object_factory_scope = LayoutObjectFactoryScope::new(&engine.objects_);
    let root = tree.Root() as *const LayoutObject;
    let mut current = root as *mut LayoutObject;
    let mut result = Vec::new();
    while let Some(object) = unsafe { current.as_ref() } {
        let real_element =
            unsafe { object.GetNode().as_ref() }.is_some_and(|node| !node.InputIsStyleGenerated());
        if real_element && object.StyleRef().ContainerType() & 3 != 0 && !object.NeedsLayout() {
            let box_ = DynamicTo::<LayoutBox>(current);
            if let Some(box_) = unsafe { box_.as_ref() }.filter(|b| b.PhysicalFragmentCount() != 0)
            {
                let size = box_.PhysicalContentBoxRect().size;
                // style_engine.cc:3888-3890 adjusts layout's physical size
                // back to CSS pixels before evaluating container features.
                let zoom = object.StyleRef().EffectiveZoom() as f64;
                result.push(ContainerQuerySizeSnapshot {
                    node_id: object.OwnerNodeIdDefault() as u64,
                    width: size.width.ToDouble().max(0.0) / zoom,
                    height: size.height.ToDouble().max(0.0) / zoom,
                });
            }
        }
        current = object.NextInPreOrder(root);
    }
    result
}
