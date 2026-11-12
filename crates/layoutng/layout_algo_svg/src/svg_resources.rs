//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! core/layout/svg/svg_resources.cc:60-113. Queries borrow resident native
//! Layout owners; effect geometry is never reconstructed from a Paint graph.
#![allow(non_snake_case)]
use crate::layout_svg_shape::LayoutSVGShape;
use foundation::gfx;
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_style::style::computed_style_constants::GeometryBox;

pub fn ObjectBoundingBox(object: &LayoutObject) -> gfx::RectF {
    if object.IsSVGShape() {
        return unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }
            .ObjectBoundingBox();
    }
    if object.IsSVGText() || object.IsSVGInline() {
        return crate::svg_text_visual_rect::ObjectBoundingBox(object);
    }
    if object.IsSVGForeignObject() {
        return ForeignObjectViewport(object);
    }
    ContainerObjectBounds(object).bounds
}
// Source SVGContentContainer::UpdateBoundingBoxes (cc:200-223). Keep the
// decorated geometry on its native owner, beside the retained object bounds.
pub fn DecoratedBoundingBox(object: &LayoutObject) -> gfx::RectF {
    if object.IsSVGShape() {
        return unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }
            .DecoratedBoundingBox();
    }
    if object.IsSVGText() || object.IsSVGInline() {
        return crate::svg_text_visual_rect::VisualRectInLocalSVGCoordinates(object);
    }
    if object.IsSVGForeignObject() {
        return unsafe {
            &*(object as *const LayoutObject)
                .cast::<crate::layout_svg_foreign_object::LayoutSVGForeignObject>()
        }
        .DecoratedBoundingBox();
    }
    ContainerObjectBounds(object).decorated
}
pub fn StrokeBoundingBox(object: &LayoutObject) -> gfx::RectF {
    if object.IsSVGShape() {
        return unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }
            .StrokeBoundingBox();
    }
    if object.IsSVGText() || object.IsSVGInline() {
        return crate::svg_text_visual_rect::StrokeBoundingBox(object);
    }
    if object.IsSVGForeignObject() {
        return ForeignObjectViewport(object);
    }
    ContainerStrokeBoundingBox(object)
}
#[derive(Clone, Copy, Default)]
pub(crate) struct SvgObjectBounds {
    pub bounds: gfx::RectF,
    pub valid: bool,
    pub decorated: gfx::RectF,
}
fn ContainerObjectBounds(object: &LayoutObject) -> SvgObjectBounds {
    if object.IsSVGRoot() {
        return unsafe {
            &*(object as *const LayoutObject).cast::<crate::layout_svg_root::LayoutSVGRoot>()
        }
        .ObjectBounds();
    }
    assert!(
        object.IsSVGContainer(),
        "SVG bounds require a native geometry owner"
    );
    unsafe { &*(object as *const LayoutObject).cast::<crate::layout_svg_group::LayoutSVGGroup>() }
        .ObjectBounds()
}
fn HasValidBoundingBoxForContainer(object: &LayoutObject) -> bool {
    if object.IsSVGShape() {
        return !unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }
            .IsShapeEmpty();
    }
    if object.IsSVGText() {
        return unsafe {
            &*(object as *const LayoutObject).cast::<crate::layout_svg_text::LayoutSVGText>()
        }
        .IsObjectBoundingBoxValid();
    }
    if object.IsSVGForeignObject() {
        return !ObjectBoundingBox(object).IsEmpty();
    }
    object.IsSVGContainer() && !object.IsSVGHiddenContainer() && ContainerObjectBounds(object).valid
}
// core/layout/svg/svg_content_container.cc:200-225. This runs once after
// child layout and retains the native content owner's bounds and validity.
pub(crate) fn CollectObjectBounds(object: &LayoutObject) -> SvgObjectBounds {
    let mut result = SvgObjectBounds::default();
    let mut child = object.SlowFirstChild();
    while let Some(child_object) = unsafe { child.as_ref() } {
        if HasValidBoundingBoxForContainer(child_object) {
            result.decorated.Union(
                child_object
                    .LocalToSVGParentTransform()
                    .MapRect(DecoratedBoundingBox(child_object)),
            );
            let mut bounds = ObjectBoundingBox(child_object);
            if child_object.IsSVGForeignObject() {
                let zoom = child_object.StyleRef().EffectiveZoom();
                bounds = gfx::RectF::new(
                    gfx::PointF::new(bounds.x() * zoom, bounds.y() * zoom),
                    gfx::SizeF::new(bounds.width() * zoom, bounds.height() * zoom),
                );
            }
            let rect = child_object.LocalToSVGParentTransform().MapRect(bounds);
            if !result.valid {
                result.bounds = rect;
                result.valid = true;
            } else {
                // gfx::RectF::UnionEvenIfEmpty preserves degenerate line boxes.
                let left = result.bounds.x().min(rect.x());
                let top = result.bounds.y().min(rect.y());
                let right = result.bounds.right().max(rect.right());
                let bottom = result.bounds.bottom().max(rect.bottom());
                result.bounds = gfx::RectF::new(
                    gfx::PointF::new(left, top),
                    gfx::SizeF::new(right - left, bottom - top),
                );
            }
        }
        child = child_object.NextSibling();
    }
    result
}
// core/layout/svg/svg_content_container.cc:254-265 computes stroke bounds on
// demand from child native geometry, unlike the retained object bounding box.
fn ContainerStrokeBoundingBox(object: &LayoutObject) -> gfx::RectF {
    let mut bounds = gfx::RectF::default();
    let mut child = object.SlowFirstChild();
    while let Some(child_object) = unsafe { child.as_ref() } {
        if HasValidBoundingBoxForContainer(child_object) {
            bounds.Union(
                child_object
                    .LocalToSVGParentTransform()
                    .MapRect(StrokeBoundingBox(child_object)),
            );
        }
        child = child_object.NextSibling();
    }
    bounds
}
// layout_svg_foreign_object.cc:45-58,114-124. The viewport is in unzoomed
// SVG user space; CSS border-box dimensions are not its reference geometry.
fn ForeignObjectViewport(object: &LayoutObject) -> gfx::RectF {
    unsafe {
        &*(object as *const LayoutObject)
            .cast::<crate::layout_svg_foreign_object::LayoutSVGForeignObject>()
    }
    .ObjectBoundingBox()
}

pub fn ReferenceBoxForEffects(object: &LayoutObject, geometry: GeometryBox) -> gfx::RectF {
    let mut owner = object;
    if owner.IsSVGInline() {
        while !owner.IsSVGText() {
            owner = unsafe { owner.Parent().as_ref() }
                .expect("SVG inline requires its native SVG text ancestor");
        }
    }
    match geometry {
        GeometryBox::kPaddingBox | GeometryBox::kContentBox | GeometryBox::kFillBox => {
            ObjectBoundingBox(owner)
        }
        GeometryBox::kMarginBox | GeometryBox::kBorderBox | GeometryBox::kStrokeBox => {
            StrokeBoundingBox(owner)
        }
        GeometryBox::kViewBox => gfx::RectF::new(
            gfx::PointF::default(),
            crate::svg_viewport::ResolveViewport(owner),
        ),
        _ => panic!("unreachable SVG effects geometry box"),
    }
}

// SVGResource::ClientEntry / ResourceContainer / FindCycle,
// core/svg/svg_resource.cc:138-194. The existing resource snapshot supplies
// target identity and the invalidation version; its Paint graphs are unused.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CycleCheck {
    NeedCheck,
    PerformingCheck,
    HasCycle,
    NoCycle,
}
// SVGResource's normal element client and ImageResourceObserver client have
// distinct identities. Multiple mask layers targeting the same resource share
// its observer client (svg_resource.cc:87-109, style_mask_source_image.cc:90-94).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceClientKind {
    Element,
    ImageObserver,
}
#[derive(Default)]
pub(crate) struct ResourceCycleCache {
    version: Option<layoutng_assembly::internal::paint_input::SvgClipResourceVersion>,
    clients: std::collections::HashMap<(u64, ResourceClientKind), CycleCheck>,
}
impl ResourceCycleCache {
    fn InvalidateFor(
        &mut self,
        version: layoutng_assembly::internal::paint_input::SvgClipResourceVersion,
    ) {
        if self.version != Some(version) {
            // SVGResource::InvalidateCycleCache cc:112-115. These entries
            // cache query results, not ownership/registration of DOM clients.
            self.clients.clear();
            self.version = Some(version);
        }
    }
}
fn LookupResource<'a>(
    tree: &'a layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    client: &LayoutObject,
    url: &str,
    scope: Option<layoutng_style::style::reference_clip_path_operation::SvgClipResourceScope>,
) -> Option<&'a crate::layout_svg_resource_clipper::LayoutSVGResourceContainer> {
    let node = unsafe { client.GetNode().as_ref() }?;
    let binding = &node.InputStyleResources().svg_clip_resources;
    let scope = scope.unwrap_or(binding.scope);
    if scope.document_id != binding.snapshot.version.document_id {
        return None;
    }
    let Some(fragment) = url.strip_prefix('#') else {
        std::panic::panic_any(foundation::UnsupportedLayout::new(
            "external SVG resource loading is not translated",
        ));
    };
    if fragment.contains('%') {
        std::panic::panic_any(foundation::UnsupportedLayout::new(
            "encoded SVG fragment resource identity is not translated",
        ));
    }
    let entry = binding
        .snapshot
        .entries
        .iter()
        .find(|entry| entry.scope_id == scope.tree_scope_id && entry.id == fragment)?;
    tree.GetLayoutObjectForNode(entry.target_node_id)
        .and_then(AsResource)
}
fn LookupClipper<'a>(
    tree: &'a layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    client: &LayoutObject,
    reference: &layoutng_style::style::reference_clip_path_operation::ReferenceClipPathOperation,
) -> Option<&'a crate::layout_svg_resource_clipper::LayoutSVGResourceClipper> {
    let resource = LookupResource(
        tree,
        client,
        &reference.Url().Utf8(),
        reference.ResourceScope(),
    )?;
    (resource.RuntimeClass()
        == layoutng_assembly::internal::layout_object::LayoutObjectClass::SvgResourceClipper)
        .then(|| unsafe { &*(resource as *const _ as *const LayoutObject).cast() })
}

fn ClientCycleState(
    resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer,
    client: &LayoutObject,
) -> CycleCheck {
    ClientCycleStateWithIdentity(resource, client, ResourceClientKind::Element)
}
fn SetClientCycleState(
    resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer,
    client: &LayoutObject,
    state: CycleCheck,
) {
    SetClientCycleStateWithIdentity(resource, client, state, ResourceClientKind::Element)
}
fn ClientCycleStateWithIdentity(
    resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer,
    client: &LayoutObject,
    identity: ResourceClientKind,
) -> CycleCheck {
    let node = unsafe { &*client.GetNode() };
    // All clients observe the target's resource generation. Borrowing a
    // client's older style binding here could reset a live DFS mid-check.
    let version = unsafe { &*resource.GetNode() }
        .InputStyleResources()
        .svg_clip_resources
        .snapshot
        .version;
    let mut cache = resource.client_cycle_cache.borrow_mut();
    cache.InvalidateFor(version);
    *cache
        .clients
        .entry((node.InputId(), identity))
        .or_insert(CycleCheck::NeedCheck)
}
fn SetClientCycleStateWithIdentity(
    resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer,
    client: &LayoutObject,
    state: CycleCheck,
    identity: ResourceClientKind,
) {
    let node = unsafe { &*client.GetNode() };
    // All clients observe the target's resource generation. Borrowing a
    // client's older style binding here could reset a live DFS mid-check.
    let version = unsafe { &*resource.GetNode() }
        .InputStyleResources()
        .svg_clip_resources
        .snapshot
        .version;
    let mut cache = resource.client_cycle_cache.borrow_mut();
    cache.InvalidateFor(version);
    cache.clients.insert((node.InputId(), identity), state);
}
fn ReferenceOperation(
    object: &LayoutObject,
) -> Option<&layoutng_style::style::reference_clip_path_operation::ReferenceClipPathOperation> {
    let operation = unsafe { object.StyleRef().ClipPath()?.as_ref() }?;
    if operation.GetType() != layoutng_style::style::clip_path_operation::OperationType::kReference
    {
        return None;
    }
    Some(unsafe {
        &*(operation as *const dyn layoutng_style::style::clip_path_operation::ClipPathOperation as *const layoutng_style::style::reference_clip_path_operation::ReferenceClipPathOperation)
    })
}
fn AsResource(
    object: &LayoutObject,
) -> Option<&crate::layout_svg_resource_clipper::LayoutSVGResourceContainer> {
    object
        .IsSVGResourceContainer()
        .then(|| unsafe { &*(object as *const LayoutObject).cast() })
}
fn FindCycleInResources(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    object: &LayoutObject,
) -> bool {
    if !object.IsSVG() || object.IsText() {
        return false;
    }
    if let Some(reference) = ReferenceOperation(object) {
        if let Some(resource) = LookupResource(
            tree,
            object,
            &reference.Url().Utf8(),
            reference.ResourceScope(),
        ) {
            if FindCycleForClient(tree, resource, object) {
                return true;
            }
        }
    }
    // CollectResources, resource_container.cc:214-238: preserve source order.
    for operation in object.StyleRef().Filter().Operations() {
        let operation = unsafe { &*operation.Get() };
        if operation.GetType() != layoutng_style::style::filter_operation::OperationType::kReference
        {
            continue;
        }
        let reference = unsafe {
            &*(operation as *const _
                as *const layoutng_style::style::filter_operation::ReferenceFilterOperation)
        };
        if let Some(resource) = LookupResource(tree, object, &reference.Url().Utf8(), None) {
            if FindCycleForClient(tree, resource, object) {
                return true;
            }
        }
    }
    for marker in if HasMarkerClient(object) {
        [
            object.StyleRef().MarkerStartResource(),
            object.StyleRef().MarkerMidResource(),
            object.StyleRef().MarkerEndResource(),
        ]
    } else {
        [std::ptr::null_mut(); 3]
    } {
        let Some(marker) = (unsafe { marker.as_ref() }) else {
            continue;
        };
        if let Some(resource) = LookupResource(tree, object, &marker.Url().Utf8(), None) {
            if FindCycleForClient(tree, resource, object) {
                return true;
            }
        }
    }
    if object.IsSVGShape() || object.IsSVGText() || object.IsSVGInline() {
        for paint in [
            object.StyleRef().FillPaint(),
            object.StyleRef().StrokePaint(),
        ] {
            if !paint.HasUrl() {
                continue;
            }
            if let Some(resource) = LookupResource(tree, object, &paint.GetUrl().Utf8(), None) {
                if FindCycleForClient(tree, resource, object) {
                    return true;
                }
            }
        }
    }
    // Mask images share this resource's ImageResourceObserver adapter for
    // the same native LayoutObject, independently of its SVG element client.
    let mut layer =
        object.StyleRef().MaskLayers() as *const layoutng_style::style::fill_layer::FillLayer;
    while let Some(current) = unsafe { layer.as_ref() } {
        layer = current.Next();
        let Some(image) = (unsafe { current.GetImage().as_ref() }) else {
            continue;
        };
        let Some(mask) = layoutng_style::style::style_mask_source_image::MaskSourceImage(image)
        else {
            continue;
        };
        if !mask.IsLocal() {
            continue;
        }
        if mask.HasEncodedFragment() {
            std::panic::panic_any(foundation::UnsupportedLayout::new(
                "encoded SVG mask resource identity is not translated",
            ));
        }
        let url = format!("#{}", mask.NormalizedFragment());
        if let Some(resource) = LookupResource(tree, object, &url, mask.ResourceScope()) {
            if FindCycleForClientWithIdentity(
                tree,
                resource,
                object,
                ResourceClientKind::ImageObserver,
            ) {
                return true;
            }
        }
    }
    false
}
fn FindCycleInSubtree(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    root: &LayoutObject,
) -> bool {
    FindCycleInResources(tree, root) || FindCycleInDescendants(tree, root)
}
fn FindCycleInDescendants(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    root: &LayoutObject,
) -> bool {
    let mut next = root.SlowFirstChild();
    while let Some(object) = unsafe { next.as_ref() } {
        if object.IsSVGResourceContainer() {
            next = object.NextInPreOrderAfterChildren(root);
            continue;
        }
        if FindCycleInResources(tree, object) {
            return true;
        }
        next = object.NextInPreOrder(root);
    }
    false
}
fn FindCycleFromSelf(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer,
) -> bool {
    use layoutng_assembly::internal::layout_object::LayoutObjectClass;
    let root: &LayoutObject = resource;
    if root.RuntimeClass() == LayoutObjectClass::SvgResourceClipper
        && FindCycleInResources(tree, root)
    {
        return true;
    }
    if root.RuntimeClass() == LayoutObjectClass::SvgResourceAuxiliary {
        let resource = unsafe {
            &*(root as *const LayoutObject)
                .cast::<crate::layout_svg_resource_auxiliary::LayoutSVGResourceAuxiliary>()
        };
        if !resource.IsFilter() {
            return FindCycleInSubtree(tree, root);
        }
        let element = foundation::DynamicTo::<
            layoutng_assembly::internal::layout_node_metadata::Element,
        >(root.GetNode());
        return unsafe { element.as_ref() }
            .and_then(|element| element.InputElementData().as_ref())
            .is_some_and(|data| {
                data.svg_filter_image_targets
                    .iter()
                    .filter_map(|id| tree.GetLayoutObjectForNode(*id))
                    .any(|target| FindCycleInSubtree(tree, target))
            });
    }
    if root.RuntimeClass() == LayoutObjectClass::SvgResourcePaintServer {
        let server = unsafe {
            &*(root as *const LayoutObject)
                .cast::<crate::layout_svg_resource_paint_server::LayoutSVGResourcePaintServer>()
        };
        if server.IsPattern() {
            // pattern.cc:115-124 uses the DOM-collected content element,
            // including href inheritance, then traverses its native descendants.
            let target = unsafe { &*root.GetNode() }
                .InputStyleResources()
                .svg_pattern_content_target;
            return target
                .and_then(|id| tree.GetLayoutObjectForNode(id))
                .is_some_and(|content| FindCycleInDescendants(tree, content));
        }
    }
    FindCycleInDescendants(tree, root)
}
fn FindCycleForClient(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer,
    client: &LayoutObject,
) -> bool {
    FindCycleForClientWithIdentity(tree, resource, client, ResourceClientKind::Element)
}
fn FindCycleForClientWithIdentity(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer,
    client: &LayoutObject,
    identity: ResourceClientKind,
) -> bool {
    match ClientCycleStateWithIdentity(resource, client, identity) {
        CycleCheck::NeedCheck => {
            SetClientCycleStateWithIdentity(
                resource,
                client,
                CycleCheck::PerformingCheck,
                identity,
            );
            let cycle = FindCycleFromSelf(tree, resource);
            SetClientCycleStateWithIdentity(
                resource,
                client,
                if cycle {
                    CycleCheck::HasCycle
                } else {
                    CycleCheck::NoCycle
                },
                identity,
            );
            false
        }
        CycleCheck::NoCycle => {
            SetClientCycleStateWithIdentity(
                resource,
                client,
                CycleCheck::PerformingCheck,
                identity,
            );
            let cycle = FindCycleFromSelf(tree, resource);
            SetClientCycleStateWithIdentity(resource, client, CycleCheck::NoCycle, identity);
            cycle
        }
        CycleCheck::PerformingCheck => true,
        CycleCheck::HasCycle => false,
    }
}
fn ResourceHasNoCycle(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer,
    client: &LayoutObject,
) -> bool {
    if ClientCycleState(resource, client) == CycleCheck::NeedCheck {
        SetClientCycleState(resource, client, CycleCheck::PerformingCheck);
        let cycle = FindCycleFromSelf(tree, resource);
        SetClientCycleState(
            resource,
            client,
            if cycle {
                CycleCheck::HasCycle
            } else {
                CycleCheck::NoCycle
            },
        );
    }
    ClientCycleState(resource, client) == CycleCheck::NoCycle
}
pub(crate) fn FilterResourceTarget(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    client: &LayoutObject,
    reference: &layoutng_style::style::filter_operation::ReferenceFilterOperation,
) -> Option<u64> {
    let resource = LookupResource(tree, client, &reference.Url().Utf8(), None)?;
    if !ResourceHasNoCycle(tree, resource, client)
        || resource.RuntimeClass()
            != layoutng_assembly::internal::layout_object::LayoutObjectClass::SvgResourceAuxiliary
    {
        return None;
    }
    let filter = unsafe {
        &*(resource as *const _ as *const LayoutObject)
            .cast::<crate::layout_svg_resource_auxiliary::LayoutSVGResourceAuxiliary>()
    };
    filter
        .IsFilter()
        .then(|| unsafe { &*resource.GetNode() }.InputId())
}
fn HasMarkerClient(object: &LayoutObject) -> bool {
    if !object.IsSVGShape() {
        return false;
    }
    let element = foundation::DynamicTo::<layoutng_assembly::internal::layout_node_metadata::Element>(
        object.GetNode(),
    );
    unsafe { element.as_ref() }
        .and_then(|element| element.InputElementData().as_ref())
        .is_some_and(|data| {
            data.svg_path_element
                || data.svg_shape.as_ref().is_some_and(|shape| {
                    shape.geometry
                        == layoutng_assembly::internal::layout_input::SvgShapeGeometry::kPath
                })
        })
}
pub(crate) fn ResolveMarker<'a>(
    tree: &'a layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    client: &LayoutObject,
    reference: *mut layoutng_style::style::style_svg_resource::StyleSVGResource,
) -> Option<&'a crate::layout_svg_resource_auxiliary::LayoutSVGResourceAuxiliary> {
    let reference = unsafe { reference.as_ref() }?;
    let resource = LookupResource(tree, client, &reference.Url().Utf8(), None)?;
    if !ResourceHasNoCycle(tree, resource, client)
        || resource.RuntimeClass()
            != layoutng_assembly::internal::layout_object::LayoutObjectClass::SvgResourceAuxiliary
    {
        return None;
    }
    let marker = unsafe {
        &*(resource as *const _ as *const LayoutObject)
            .cast::<crate::layout_svg_resource_auxiliary::LayoutSVGResourceAuxiliary>()
    };
    (!marker.IsFilter()).then_some(marker)
}
// SVGContentContainer::LayoutMarkerResourcesIfNeeded, cc:24-34,56-72.
pub(crate) fn LayoutMarkerResourcesIfNeeded(
    object: &LayoutObject,
    info: &layoutng_assembly::internal::svg_layout_info::SVGLayoutInfo,
) -> bool {
    if !HasMarkerClient(object) || !object.StyleRef().HasMarkers() {
        return false;
    }
    layoutng_assembly::internal::layout_pass_scope::LayoutPassScope::WithSVGResourceTree(|tree| {
        let style = object.StyleRef();
        let mut dependent = false;
        for reference in [
            style.MarkerStartResource(),
            style.MarkerMidResource(),
            style.MarkerEndResource(),
        ] {
            let Some(marker) = ResolveMarker(tree, object, reference) else {
                continue;
            };
            let pointer = unsafe { &*marker.GetNode() }.GetLayoutObject();
            let Some(marker) = (unsafe { pointer.as_mut() }) else {
                continue;
            };
            if info.viewport_changed && marker.HasViewportDependence() {
                marker.SetNeedsLayoutWithMarking(
                    unsafe {
                        &layoutng_assembly::internal::layout_invalidation_reason::kSvgChanged
                    },
                    layoutng_assembly::internal::layout_object::MarkingBehavior::kMarkOnlyThis,
                );
            }
            if marker.NeedsLayout() {
                let result = marker.UpdateSVGLayout(info);
                marker.SetHasViewportDependence(result.has_viewport_dependence);
            }
            dependent |= marker.HasViewportDependence();
        }
        dependent
    })
    .expect("SVG marker layout borrows the native resource tree scope")
}
pub fn ResolveClipPathTarget<'a>(
    tree: &'a layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    client: &LayoutObject,
    reference: &layoutng_style::style::reference_clip_path_operation::ReferenceClipPathOperation,
) -> Option<&'a crate::layout_svg_resource_clipper::LayoutSVGResourceClipper> {
    let clipper = LookupClipper(tree, client, reference)?;
    let resource: &crate::layout_svg_resource_clipper::LayoutSVGResourceContainer = clipper;
    ResourceHasNoCycle(tree, resource, client).then_some(clipper)
}

/// Native SVG client owns the cached FilterEffect bounds data, independently
/// of the thread or Paint projection consuming its exact result.
pub fn ApplyFiltersToRect(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    object: &LayoutObject,
    rect: gfx::RectF,
) -> gfx::RectF {
    let cache = if object.IsSVGShape() {
        &unsafe {
            &*(object as *const LayoutObject).cast::<crate::layout_svg_shape::LayoutSVGShape>()
        }
        .effect_bounds
    } else if object.IsSVGText() {
        &unsafe {
            &*(object as *const LayoutObject).cast::<crate::layout_svg_text::LayoutSVGText>()
        }
        .effect_bounds
    } else if object.IsSVGRoot() {
        &unsafe {
            &*(object as *const LayoutObject).cast::<crate::layout_svg_root::LayoutSVGRoot>()
        }
        .effect_bounds
    } else {
        assert!(object.IsSVGContainer() || object.IsSVGForeignObject());
        &unsafe {
            &*(object as *const LayoutObject).cast::<crate::layout_svg_group::LayoutSVGGroup>()
        }
        .effect_bounds
    };
    cache.ApplyFiltersToRect(tree, object, rect)
}
pub fn HasPaintServer(
    tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
    object: &LayoutObject,
    stroke: bool,
) -> bool {
    let paint = if stroke {
        object.StyleRef().StrokePaint()
    } else {
        object.StyleRef().FillPaint()
    };
    if paint.HasColor() {
        return true;
    }
    if !paint.HasUrl() {
        return false;
    }
    let url = paint.GetUrl().Utf8();
    if !url.starts_with('#') || url.contains('%') {
        std::panic::panic_any(foundation::UnsupportedLayout::new(
            "external or encoded SVG paint-server IRI is not translated",
        ));
    }
    let Some(resource) = LookupResource(tree, object, &url, None) else {
        return false;
    };
    let target: &LayoutObject = resource;
    if target.RuntimeClass()
        != layoutng_assembly::internal::layout_object::LayoutObjectClass::SvgResourcePaintServer
    {
        return false;
    }
    ResourceHasNoCycle(tree, resource, object)
}
pub fn IsObjectBoundingBoxValid(object: &LayoutObject) -> bool {
    HasValidBoundingBoxForContainer(object)
}

// SVGGraphicsElement.cc:49-70,89-104,233-261. Native validity and ancestor
// ownership are queried here; Browser routes only the source counter outputs.
pub fn ResizeObserverBBoxDiagnostics(object: &LayoutObject) -> (bool, bool) {
    let bbox = ObjectBoundingBox(object);
    if bbox == gfx::RectF::default() {
        return (false, false);
    }
    let valid = if object.IsSVGImage() {
        !bbox.IsEmpty()
    } else {
        HasValidBoundingBoxForContainer(object)
    };
    let mut current = object as *const LayoutObject;
    let mut non_rendered = false;
    while let Some(ancestor) = unsafe { current.as_ref() } {
        if ancestor.IsSVGHiddenContainer() {
            non_rendered = true;
            break;
        }
        if ancestor.IsSVGRoot() {
            break;
        }
        current = ancestor.Parent();
    }
    let nested_empty = object.IsSVGViewportContainer()
        && unsafe {
            &*(object as *const LayoutObject)
                .cast::<crate::layout_svg_viewport_container::LayoutSVGViewportContainer>()
        }
        .Viewport()
        .IsEmpty();
    (!valid || non_rendered, nested_empty)
}
