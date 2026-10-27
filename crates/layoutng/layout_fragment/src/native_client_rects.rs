// Client-rect queries walk and transform the canonical fragment graph. Keep
// path search, coordinate projection and rect collection in one fragment-owned
// cost domain so profiles do not charge their self time to the output facade.

use super::*;

struct Step<'a> {
    fragment: &'a PhysicalBoxFragment,
    offset: PhysicalOffset,
}

#[inline(never)]
fn find_path<'a>(
    output: LayoutOutput<'a>,
    target: *const PhysicalBoxFragment,
) -> Option<Vec<Step<'a>>> {
    #[inline(never)]
    fn visit<'a>(
        fragment: &'a PhysicalBoxFragment,
        offset: PhysicalOffset,
        target: *const PhysicalBoxFragment,
        path: &mut Vec<Step<'a>>,
    ) -> bool {
        path.push(Step { fragment, offset });
        if std::ptr::eq(fragment, target) {
            return true;
        }
        for child in fragment.Children() {
            if !child.is_present() || !child.IsBox() {
                continue;
            }
            let box_ = unsafe { &*(child.get().cast::<PhysicalBoxFragment>()) };
            if visit(box_, child.Offset(), target, path) {
                return true;
            }
        }
        if let Some(items) = unsafe { fragment.Items().as_ref() } {
            for item in items.Items() {
                if item.Type() != layout_fragment::fragment_item::ItemType::kBox {
                    continue;
                }
                if let Some(box_) = unsafe { item.BoxFragment().as_ref() } {
                    if visit(box_, *item.OffsetInContainerFragment(), target, path) {
                        return true;
                    }
                }
            }
        }
        let object = fragment.GetLayoutObject();
        if let Some(object) = unsafe { object.as_ref() } {
            if object.IsSVGRoot() || (object.IsSVGContainer() && !object.IsSVGForeignObject()) {
                let mut child = object.SlowFirstChild();
                while let Some(object) = unsafe { child.as_ref() } {
                    let box_ = DynamicTo::<LayoutBox>(child);
                    if let Some(box_) = unsafe { box_.as_ref() } {
                        for fragment in box_.PhysicalFragments() {
                            if visit(fragment, box_.PhysicalLocation(), target, path) {
                                return true;
                            }
                        }
                    }
                    child = object.NextSibling();
                }
            }
        }
        path.pop();
        false
    }

    let root = DynamicTo::<LayoutBox>(output.engine.tree_.as_ref()?.Root() as *const _);
    let root = unsafe { root.as_ref() }?;
    let mut path = Vec::new();
    for fragment in root.PhysicalFragments() {
        if visit(fragment, PhysicalOffset::default(), target, &mut path) {
            return Some(path);
        }
    }
    None
}

#[inline(never)]
fn transform_to_root(
    output: LayoutOutput<'_>,
    target: &PhysicalBoxFragment,
    in_contents: bool,
) -> Option<gfx::Transform> {
    let path = find_path(output, target)?;
    let mut world = gfx::Transform::default();
    for (index, step) in path.iter().enumerate() {
        let object = step.fragment.GetLayoutObject();
        let object = unsafe { object.as_ref() };
        let box_model = DynamicTo::<LayoutBoxModelObject>(step.fragment.GetLayoutObject());
        let box_model = unsafe { box_model.as_ref() };
        // LayoutView's fixed-position context is independent of its scroll.
        if object.is_some_and(|object| {
            unsafe { DynamicTo::<LayoutBox>(object as *const _).as_ref() }
                .is_some_and(|box_| box_.IsFixedToView(std::ptr::null()))
        }) {
            world = gfx::Transform::default();
        }
        let mut offset = step.offset;
        if let Some(model) = box_model {
            offset += model.StickyPositionOffset();
        }
        world.Translate3d(offset.left.ToFloat(), offset.top.ToFloat(), 0.);
        let mut svg_viewport = None;
        let mut svg_local = None;
        if let Some(object) = object.filter(|o| o.IsSVG()) {
            if let Some(query) = output.engine.algorithms_.svg_support.paint_geometry {
                (svg_local, svg_viewport) = query(object);
            }
        }
        if let Some(matrix) = svg_local {
            world.PreConcat(&gfx::Transform::ColMajor(&matrix.values));
        } else if let Some(object) = object {
            if object.IsBox() && object.StyleRef().HasTransformRelatedProperty() {
                let size = step.fragment.Size();
                let mut local = gfx::Transform::default();
                object.StyleRef().ApplyTransformRectF::<LayoutBox>(
                    &mut local,
                    unsafe { DynamicTo::<LayoutBox>(object as *const _).as_ref() },
                    &gfx::RectF::new(
                        gfx::PointF::default(),
                        gfx::SizeF::new(size.width.ToFloat(), size.height.ToFloat()),
                    ),
                    ApplyTransformOperations::kIncludeTransformOperations,
                    ApplyTransformOrigin::kIncludeTransformOrigin,
                    ApplyMotionPath::kIncludeMotionPath,
                    ApplyIndependentTransformProperties::kIncludeIndependentTransformProperties,
                );
                world.PreConcat(&local);
            }
        }
        if index + 1 == path.len() && !in_contents {
            break;
        }
        if let Some(viewport) = svg_viewport {
            world.PreConcat(&gfx::Transform::ColMajor(
                &viewport.content_transform.values,
            ));
        }
        if let Some(object) = object {
            if let Some(element) = unsafe { DynamicTo::<Element>(object.GetNode()).as_ref() } {
                if let Some(data) = element.InputElementData() {
                    world.Translate3d(
                        -data.scroll_offset.x as f32,
                        -data.scroll_offset.y as f32,
                        0.,
                    );
                }
            }
        }
    }
    Some(world)
}

#[inline(never)]
pub(super) fn client_rects(node: &LayoutNode<'_>) -> Vec<LayoutRect> {
    let _scope = LayoutPassScope::new(
        &node.output.engine.algorithms_,
        &node.output.engine.objects_,
    );
    let mut trace = layout_fragment_boundary::trace::client_rects();
    if node.object.IsSVGHiddenContainer() {
        return Vec::new();
    }
    let mut rects = Vec::new();
    if node.object.IsLayoutInline() && !node.object.IsSVG() {
        let inline = unsafe { &*DynamicTo::<LayoutInline>(node.object as *const _) };
        // InlineCursor emits several line rects in the same physical
        // container. Map that container once during this query, as the native
        // quad traversal does; no document geometry is cached.
        let mut previous = std::ptr::null();
        let mut transform = None;
        for (container, rect) in inline.ClientRectsInContainers().rects {
            if container != previous {
                previous = container;
                transform = unsafe { container.as_ref() }
                    .and_then(|container| transform_to_root(node.output, container, true));
            }
            if let Some(transform) = &transform {
                rects.push(map_rect(transform, rect));
            }
        }
    } else if let Some(box_) = unsafe { DynamicTo::<LayoutBox>(node.object as *const _).as_ref() } {
        for fragment in box_.PhysicalFragments() {
            if let Some(transform) = transform_to_root(node.output, fragment, false) {
                rects.push(map_rect(
                    &transform,
                    PhysicalRect::new(PhysicalOffset::default(), fragment.Size()),
                ));
            }
        }
    }
    trace.set("rects", rects.len() as f64);
    rects
}

#[inline(never)]
fn map_rect(transform: &gfx::Transform, rect: PhysicalRect) -> LayoutRect {
    let mapped = transform.MapRect(gfx::RectF::from(rect));
    LayoutRect {
        x: mapped.x() as f64,
        y: mapped.y() as f64,
        width: mapped.width() as f64,
        height: mapped.height() as f64,
    }
}
