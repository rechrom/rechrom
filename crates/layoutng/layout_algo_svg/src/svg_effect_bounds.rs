//! SVGLayoutSupport::ApplyFiltersToRect, svg_layout_support.cc:87-103.
//! Native SVG resource clients share the existing translated FilterEffect
//! forward mapper with Paint; cache identity follows the immutable resource.
use foundation::{gfx, paint_rect::PaintRect};
use layoutng_assembly::{
    internal::{layout_object::LayoutObject, paint_input::SvgFilterGraph},
    svg_filter_geometry::{
        map_svg_filter_forward_bounds, resolve_svg_filter_geometry, ResolvedSvgFilterGeometry,
    },
};
use std::{cell::RefCell, sync::Arc};
#[derive(Default)]
pub struct SvgEffectBoundsCache {
    entry: RefCell<Option<Entry>>,
}
struct Entry {
    owner: Arc<Vec<SvgFilterGraph>>,
    index: usize,
    reference: PaintRect,
    viewport: PaintRect,
    geometry: ResolvedSvgFilterGeometry,
}
fn paint(r: gfx::RectF) -> PaintRect {
    PaintRect {
        x: r.x() as f64,
        y: r.y() as f64,
        width: r.width() as f64,
        height: r.height() as f64,
    }
}
fn native(r: PaintRect) -> gfx::RectF {
    gfx::RectF::new(
        gfx::PointF::new(r.x as f32, r.y as f32),
        gfx::SizeF::new(r.width as f32, r.height as f32),
    )
}
impl SvgEffectBoundsCache {
    pub fn ApplyFiltersToRect(
        &self,
        tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
        object: &LayoutObject,
        mut rect: gfx::RectF,
    ) -> gfx::RectF {
        if !object.StyleRef().HasFilter() {
            return rect;
        }
        if object.NeedsPaintPropertyUpdate() {
            return gfx::RectF::from(foundation::InfiniteIntRect());
        }
        let mut reference = crate::svg_resources::ReferenceBoxForEffects(
            object,
            layoutng_style::style::computed_style_constants::GeometryBox::kFillBox,
        );
        if object.IsSVGForeignObject() {
            let zoom = object.StyleRef().EffectiveZoom();
            reference = gfx::RectF::new(
                gfx::PointF::default(),
                gfx::SizeF::new(reference.width() * zoom, reference.height() * zoom),
            );
        }
        // UnionEvenIfEmpty, as required by the source SVG filter gate.
        let left = rect.x().min(reference.x());
        let top = rect.y().min(reference.y());
        rect = gfx::RectF::new(
            gfx::PointF::new(left, top),
            gfx::SizeF::new(
                rect.right().max(reference.right()) - left,
                rect.bottom().max(reference.bottom()) - top,
            ),
        );
        let node = unsafe { &*object.GetNode() };
        let graphs = &node.InputStyleResources().svg_filter_graphs;
        let viewport = crate::svg_viewport::ResolveViewport(object);
        let viewport = PaintRect {
            x: 0.,
            y: 0.,
            width: viewport.width() as f64,
            height: viewport.height() as f64,
        };
        for operation in object.StyleRef().Filter().Operations() {
            let operation = unsafe { &*operation.Get() };
            if operation.GetType()
                != layoutng_style::style::filter_operation::OperationType::kReference
            {
                rect = operation.MapRect(&rect);
                continue;
            }
            let reference_op = unsafe {
                &*(operation as *const _
                    as *const layoutng_style::style::filter_operation::ReferenceFilterOperation)
            };
            let Some(target) =
                crate::svg_resources::FilterResourceTarget(tree, object, reference_op)
            else {
                continue;
            };
            let Some(index) = graphs.iter().position(|g| g.resource_node_id == target) else {
                continue;
            };
            let graph = &graphs[index];
            if graph.unsupported.is_some() {
                std::panic::panic_any(foundation::UnsupportedLayout::new(
                    "SVG filter primitive lowering is unsupported",
                ));
            }
            let reference = paint(reference);
            let mut entry = self.entry.borrow_mut();
            if !entry.as_ref().is_some_and(|e| {
                Arc::ptr_eq(&e.owner, graphs)
                    && e.index == index
                    && e.reference == reference
                    && e.viewport == viewport
            }) {
                let geometry = resolve_svg_filter_geometry(graph, reference, Some(viewport), 1.)
                    .expect("native SVG viewport is available");
                *entry = Some(Entry {
                    owner: graphs.clone(),
                    index,
                    reference,
                    viewport,
                    geometry,
                });
            }
            let geometry = &entry.as_ref().unwrap().geometry;
            if geometry.filter_region.is_empty() || geometry.primitives.is_empty() {
                continue;
            }
            rect = native(map_svg_filter_forward_bounds(graph, geometry, paint(rect)));
        }
        rect
    }
}
