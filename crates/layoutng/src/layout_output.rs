//! Borrowed access to the engine's resident layout result. No exported tree,
//! DOM ownership or native LayoutObject handle crosses this interface.
use super::*;
use crate::internal::{
    layout_box_model_object::LayoutBoxModelObject, layout_inline::LayoutInline,
    layout_node_metadata::Element,
};
use foundation::{gfx, PhysicalOffset, PhysicalRect, PhysicalSize};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::style_transform::{
    ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
    ApplyTransformOrigin,
};

#[derive(Clone, Copy)]
pub struct LayoutOutput<'a> {
    pub(super) engine: &'a LayoutEngine,
}
pub struct LayoutNode<'a> {
    output: LayoutOutput<'a>,
    object: &'a LayoutObject,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutScrollRange {
    pub minimum: crate::internal::layout_input::Offset,
    pub maximum: crate::internal::layout_input::Offset,
}
#[derive(Clone, Copy, Debug)]
pub enum LayoutMetric {
    OffsetWidth,
    OffsetHeight,
    OffsetLeft,
    OffsetTop,
    ClientWidth,
    ClientHeight,
    ClientLeft,
    ClientTop,
    ScrollWidth,
    ScrollHeight,
    ContentWidth,
    ContentHeight,
    BorderBoxWidth,
    BorderBoxHeight,
    MinimumScrollLeft,
    MaximumScrollLeft,
    MinimumScrollTop,
    MaximumScrollTop,
}
impl TryFrom<&str> for LayoutMetric {
    type Error = ();
    fn try_from(name: &str) -> Result<Self, ()> {
        use LayoutMetric::*;
        Ok(match name {
            "offsetWidth" => OffsetWidth,
            "offsetHeight" => OffsetHeight,
            "offsetLeft" => OffsetLeft,
            "offsetTop" => OffsetTop,
            "clientWidth" => ClientWidth,
            "clientHeight" => ClientHeight,
            "clientLeft" => ClientLeft,
            "clientTop" => ClientTop,
            "scrollWidth" => ScrollWidth,
            "scrollHeight" => ScrollHeight,
            "contentWidth" => ContentWidth,
            "contentHeight" => ContentHeight,
            "borderBoxWidth" => BorderBoxWidth,
            "borderBoxHeight" => BorderBoxHeight,
            "minimumScrollLeft" => MinimumScrollLeft,
            "maximumScrollLeft" => MaximumScrollLeft,
            "minimumScrollTop" => MinimumScrollTop,
            "maximumScrollTop" => MaximumScrollTop,
            _ => return Err(()),
        })
    }
}
impl<'a> LayoutOutput<'a> {
    pub fn Node(self, id: u64) -> Option<LayoutNode<'a>> {
        Some(LayoutNode {
            object: self.engine.tree_.as_ref()?.OutputNode(id)?,
            output: self,
        })
    }
    pub fn Root(self) -> LayoutNode<'a> {
        LayoutNode {
            object: self.engine.tree_.as_ref().unwrap().Root(),
            output: self,
        }
    }
    pub fn ScrollOffsets(self) -> Vec<(u64, crate::internal::layout_input::Offset)> {
        self.engine.tree_.as_ref().unwrap().OutputScrollOffsets()
    }
    // Physical fragments retain their actual parent offsets, including columns
    // and out-of-flow placement. Only the path to the requested occurrence is
    // retained while mapping its rect; no parallel fragment tree is built.
    fn FindPath(self, target: *const PhysicalBoxFragment) -> Option<Vec<Step<'a>>> {
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
                    if item.Type() != layoutng_fragment_tree::fragment_item::ItemType::kBox {
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
        let root = DynamicTo::<LayoutBox>(self.engine.tree_.as_ref()?.Root() as *const _);
        let root = unsafe { root.as_ref() }?;
        let mut path = Vec::new();
        for fragment in root.PhysicalFragments() {
            if visit(fragment, PhysicalOffset::default(), target, &mut path) {
                return Some(path);
            }
        }
        None
    }
    fn TransformToRoot(
        self,
        target: &PhysicalBoxFragment,
        in_contents: bool,
    ) -> Option<gfx::Transform> {
        let path = self.FindPath(target)?;
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
                if let Some(query) = self.engine.algorithms_.svg_support.paint_geometry {
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
}
struct Step<'a> {
    fragment: &'a PhysicalBoxFragment,
    offset: PhysicalOffset,
}
impl LayoutNode<'_> {
    pub fn PaintClientId(&self) -> u64 {
        // LayoutObject's base-first representation retains DisplayItemClient.
        unsafe { &*(self.object as *const _ as *const foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient) }.Id() as u64
    }
    pub fn ClientRects(&self) -> Vec<LayoutRect> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let mut trace = browser_tracing::span("layout", "LayoutOutput.ClientRects");
        if self.object.IsSVGHiddenContainer() {
            return Vec::new();
        }
        let mut rects = Vec::new();
        if self.object.IsLayoutInline() && !self.object.IsSVG() {
            let inline = unsafe { &*DynamicTo::<LayoutInline>(self.object as *const _) };
            // InlineCursor emits several line rects in the same physical
            // container. Map that container once during this query, as the
            // native quad traversal does; no document geometry is cached.
            let mut previous = std::ptr::null();
            let mut transform = None;
            for (container, rect) in inline.ClientRectsInContainers().rects {
                if container != previous {
                    previous = container;
                    transform = unsafe { container.as_ref() }
                        .and_then(|container| self.output.TransformToRoot(container, true));
                }
                if let Some(transform) = &transform {
                    rects.push(MapRect(transform, rect));
                }
            }
        } else if let Some(box_) =
            unsafe { DynamicTo::<LayoutBox>(self.object as *const _).as_ref() }
        {
            for fragment in box_.PhysicalFragments() {
                if let Some(transform) = self.output.TransformToRoot(fragment, false) {
                    rects.push(MapRect(
                        &transform,
                        PhysicalRect::new(PhysicalOffset::default(), fragment.Size()),
                    ));
                }
            }
        }
        trace.set("rects", rects.len() as f64);
        rects
    }
    pub fn ScrollRange(&self) -> LayoutScrollRange {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = DynamicTo::<LayoutBox>(self.object as *const _);
        let Some(box_) = (unsafe { box_.as_ref() }) else {
            return LayoutScrollRange::default();
        };
        let area = box_.GetScrollableArea();
        if let Some(area) = unsafe { area.as_ref() } {
            let lo = area.MinimumScrollOffsetInt();
            let hi = area.MaximumScrollOffsetInt();
            return LayoutScrollRange {
                minimum: crate::internal::layout_input::Offset {
                    x: lo.x() as f64,
                    y: lo.y() as f64,
                },
                maximum: crate::internal::layout_input::Offset {
                    x: hi.x() as f64,
                    y: hi.y() as f64,
                },
            };
        }
        let clip = box_.OverflowClipRectForScrollNode();
        LayoutScrollRange {
            maximum: crate::internal::layout_input::Offset {
                x: (box_.ScrollWidth().ToDouble() - clip.size.width.ToDouble()).max(0.),
                y: (box_.ScrollHeight().ToDouble() - clip.size.height.ToDouble()).max(0.),
            },
            ..Default::default()
        }
    }
    pub fn Metric(&self, metric: LayoutMetric) -> Option<f64> {
        use LayoutMetric::*;
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let zoom = self.object.StyleRef().EffectiveZoom();
        let unzoom = |v: f64| {
            if zoom == 1. {
                v
            } else {
                (v as f32 / zoom) as f64
            }
        };
        let integer = |v: f64| unzoom(v).round();
        if self.object.IsLayoutInline() && !self.object.IsSVG() {
            let inline = unsafe { &*DynamicTo::<LayoutInline>(self.object as *const _) };
            let bounds = inline.ClientRectsInContainers().bounds;
            return match metric {
                OffsetWidth => Some(integer(bounds.size.width.ToDouble())),
                OffsetHeight => Some(integer(bounds.size.height.ToDouble())),
                OffsetLeft => Some(integer(bounds.offset.left.ToDouble())),
                OffsetTop => Some(integer(bounds.offset.top.ToDouble())),
                ContentWidth | ContentHeight | BorderBoxWidth | BorderBoxHeight => None,
                _ => Some(0.),
            };
        }
        let Some(box_) = (unsafe { DynamicTo::<LayoutBox>(self.object as *const _).as_ref() })
        else {
            return Some(0.);
        };
        let Some(fragment) = box_.PhysicalFragments().into_iter().next() else {
            return Some(0.);
        };
        let size = fragment.Size();
        // LayoutBoxModelObject::OffsetWidth/Height consume this native
        // bounding box, not just the first physical fragment's dimensions.
        // cpp: core/layout/layout_box.cc:4646-4672.
        let mut bounds = PhysicalRect::default();
        let mut first: Option<&PhysicalBoxFragment> = None;
        if matches!(metric, OffsetWidth | OffsetHeight) {
            for current in box_.PhysicalFragments() {
                let offset = first.map_or(PhysicalOffset::default(), |first| {
                    current.OffsetFromRootFragmentationContext()
                        - first.OffsetFromRootFragmentationContext()
                });
                if first.is_none() {
                    first = Some(current);
                }
                bounds.UniteEvenIfEmpty(&PhysicalRect::new(offset, current.Size()));
                if unsafe { current.GetBreakToken().as_ref() }
                    .is_some_and(|token| token.IsAtBlockEnd())
                {
                    break;
                }
            }
        }
        let padding = box_.PhysicalPaddingBoxRect();
        let css_box = !(self.object.IsSVG()
            && !self.object.IsSVGRoot()
            && !self.object.IsSVGText()
            && !self.object.IsSVGForeignObject());
        let client = if !css_box {
            PhysicalSize::default()
        } else if box_.IsLayoutView() {
            let view = unsafe {
                &*(self.object as *const _ as *const crate::internal::layout_view::LayoutView)
            };
            let s = view.GetLayoutSize(
                crate::internal::scroll_types::IncludeScrollbarsInRect::kExcludeScrollbars,
            );
            PhysicalSize::new(
                foundation::LayoutUnit::from_signed(s.width()),
                foundation::LayoutUnit::from_signed(s.height()),
            )
        } else if box_.IsTable() {
            box_.StitchedSize()
        } else {
            padding.size
        };
        let insets = fragment.Borders() + fragment.Padding();
        Some(match metric {
            OffsetWidth => integer(bounds.size.width.ToDouble()),
            OffsetHeight => integer(bounds.size.height.ToDouble()),
            OffsetLeft => integer(box_.PhysicalLocation().left.ToDouble()),
            OffsetTop => integer(box_.PhysicalLocation().top.ToDouble()),
            ClientWidth => integer(client.width.ToDouble()),
            ClientHeight => integer(client.height.ToDouble()),
            ClientLeft => {
                if css_box {
                    integer(padding.offset.left.ToDouble())
                } else {
                    0.
                }
            }
            ClientTop => {
                if css_box {
                    integer(padding.offset.top.ToDouble())
                } else {
                    0.
                }
            }
            ScrollWidth => {
                if css_box {
                    integer(box_.ScrollWidth().ToDouble())
                } else {
                    0.
                }
            }
            ScrollHeight => {
                if css_box {
                    integer(box_.ScrollHeight().ToDouble())
                } else {
                    0.
                }
            }
            ContentWidth => unzoom(
                (size.width - insets.HorizontalSum())
                    .ClampNegativeToZero()
                    .ToDouble(),
            ),
            ContentHeight => unzoom(
                (size.height - insets.VerticalSum())
                    .ClampNegativeToZero()
                    .ToDouble(),
            ),
            BorderBoxWidth => unzoom(size.width.ToDouble()),
            BorderBoxHeight => unzoom(size.height.ToDouble()),
            MinimumScrollLeft => unzoom(self.ScrollRange().minimum.x),
            MaximumScrollLeft => unzoom(self.ScrollRange().maximum.x),
            MinimumScrollTop => unzoom(self.ScrollRange().minimum.y),
            MaximumScrollTop => unzoom(self.ScrollRange().maximum.y),
        })
    }
}

fn MapRect(transform: &gfx::Transform, rect: PhysicalRect) -> LayoutRect {
    let mapped = transform.MapRect(gfx::RectF::from(rect));
    LayoutRect {
        x: mapped.x() as f64,
        y: mapped.y() as f64,
        width: mapped.width() as f64,
        height: mapped.height() as f64,
    }
}
