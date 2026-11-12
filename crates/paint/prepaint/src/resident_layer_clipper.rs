//! Source PaintLayerClipper over canonical borrowed FragmentData.
//! Source: core/paint/paint_layer_clipper.cc:97-276, clip_rect.cc:43-91.
use crate::{pre_paint_tree_walk::PrePaintTreeWalk, resident_hit_test::ResidentPaintHitTest};
use foundation::{gfx, OverlayScrollbarClipBehavior, PhysicalOffset, PhysicalRect};
use layoutng_assembly::layout_engine::PaintFragmentCursor;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use paint_model::{
    geometry_mapper_source::{
        float_clip_rect::FloatClipRect,
        geometry_mapper::GeometryMapper,
        native::{
            NativeEffectNode, NativeGeometryMapperContext, NativeGeometryState, NativeTransformNode,
        },
        traits::{GeometryMapperState, VisualRectFlags},
    },
    source_geometry::{RectF, Vector2dF},
};

#[derive(Clone, Copy)]
pub struct ResidentLayerClipRect {
    pub rect: PhysicalRect,
    pub has_radius: bool,
    pub is_infinite: bool,
}
impl ResidentLayerClipRect {
    fn from_float(value: FloatClipRect) -> Self {
        let rect = value.Rect();
        Self {
            rect: PhysicalRect::FastAndLossyFromRectF(&gfx::RectF::new(
                gfx::PointF::new(rect.x(), rect.y()),
                gfx::SizeF::new(rect.width(), rect.height()),
            )),
            has_radius: value.HasRadius(),
            is_infinite: value.IsInfinite(),
        }
    }
    fn intersect(&mut self, other: Self) {
        if other.is_infinite {
            return;
        }
        if self.is_infinite {
            self.rect = other.rect;
            self.is_infinite = false;
        } else {
            self.rect.Intersect(&other.rect);
        }
        self.has_radius |= other.has_radius;
    }
}
pub struct ResidentLayerClipRects {
    pub layer_offset: PhysicalOffset,
    pub background: ResidentLayerClipRect,
    pub foreground: ResidentLayerClipRect,
}
#[derive(Clone, Copy)]
pub struct ResidentLayerClipOptions {
    pub same_root_layer: bool,
    pub respect_root_clip: bool,
    pub unbounded_element: bool,
    pub singular_transform_fix: bool,
    pub overlay_scrollbar_behavior: OverlayScrollbarClipBehavior,
}
impl PrePaintTreeWalk {
    pub fn CalculateResidentLayerClipRects(
        &self,
        source: PaintFragmentCursor<'_>,
        layer: ResidentPaintHitTest<'_>,
        root: ResidentPaintHitTest<'_>,
        fragment: ResidentPaintHitTest<'_>,
        options: ResidentLayerClipOptions,
    ) -> ResidentLayerClipRects {
        let mapper = NativeGeometryMapperContext {
            singular_transform_fix: options.singular_transform_fix,
        };
        let layer_offset = if options.same_root_layer {
            PhysicalOffset::default()
        } else {
            let matrix =
                GeometryMapper::<NativeGeometryMapperContext>::SourceToDestinationProjection(
                    &NativeTransformNode::from_arc(fragment.PreTransform().clone()),
                    &NativeTransformNode::from_arc(
                        root.LocalBorderBoxProperties().transform.clone(),
                    ),
                );
            let offset = fragment.PaintOffset();
            let point = matrix.as_source().MapPoint(gfx::PointF::new(
                offset.left.ToFloat(),
                offset.top.ToFloat(),
            ));
            PhysicalOffset::FromPointFRound(&point) - root.PaintOffset()
        };
        let geometry = layer
            .LayoutObjectIdentity()
            .expect("native layer owns LayoutObject")
            .LayerClipGeometry(
                options.overlay_scrollbar_behavior,
                !options.same_root_layer || options.respect_root_clip,
            );
        let mut background = ResidentLayerClipRect::from_float(FloatClipRect::default());
        if !options.same_root_layer || options.respect_root_clip {
            let local = NativeGeometryState::from_state(fragment.LocalBorderBoxProperties());
            let mut destination = NativeGeometryState::from_state(root.LocalBorderBoxProperties());
            if options.respect_root_clip {
                destination.SetClip(root.PreClip().clone());
                destination.SetEffect(NativeEffectNode::from_arc(root.PreEffect().clone()));
            } else {
                destination.SetClip(root.ContentsProperties().clip.clone());
            }
            if options.unbounded_element {
                if let Some(ancestor) = layer
                    .LayoutObjectIdentity()
                    .expect("native layer")
                    .ActiveUnboundedAncestor()
                {
                    let ancestor = source
                        .ForPaintObject(ancestor)
                        .expect("native unbounded ancestor fragment");
                    let ancestor = self
                        .ResidentHitTest(ancestor)
                        .expect("unbounded ancestor has committed Paint");
                    destination.SetClip(ancestor.LocalBorderBoxProperties().clip.clone());
                }
            }
            if geometry.has_non_visible_overflow {
                let rect = geometry
                    .local_visual_rect
                    .expect("overflow layer is LayoutBox");
                let mut visual = FloatClipRect::FromRect(RectF::new(
                    rect.X().ToFloat(),
                    rect.Y().ToFloat(),
                    rect.Width().ToFloat(),
                    rect.Height().ToFloat(),
                ));
                let offset = fragment.PaintOffset();
                visual.Move(Vector2dF::new(offset.left.ToFloat(), offset.top.ToFloat()));
                let behavior = if options.same_root_layer {
                    OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize
                } else {
                    options.overlay_scrollbar_behavior
                };
                GeometryMapper::<NativeGeometryMapperContext>::LocalToAncestorVisualRect(
                    &mapper,
                    &local,
                    &destination,
                    &mut visual,
                    behavior,
                    VisualRectFlags::default(),
                );
                background = ResidentLayerClipRect::from_float(visual);
            } else if !local
                .Clip()
                .lifecycle
                .same_node(&destination.Clip().lifecycle)
            {
                background = ResidentLayerClipRect::from_float(GeometryMapper::<
                    NativeGeometryMapperContext,
                >::LocalToAncestorClipRect(
                    &mapper,
                    &local,
                    &destination,
                    options.overlay_scrollbar_behavior,
                ));
            }
            if !background.is_infinite {
                background.rect.Move(&(-root.PaintOffset()));
            }
        }
        let foreground =
            if (!options.same_root_layer || options.respect_root_clip) && geometry.clips_overflow {
                let mut foreground = ResidentLayerClipRect {
                    rect: geometry
                        .overflow_clip_rect
                        .expect("clipping layer is LayoutBox"),
                    has_radius: geometry.has_border_radius,
                    is_infinite: false,
                };
                foreground.rect.Move(&layer_offset);
                foreground.intersect(background);
                foreground
            } else {
                background
            };
        ResidentLayerClipRects {
            layer_offset,
            background,
            foreground,
        }
    }
}
