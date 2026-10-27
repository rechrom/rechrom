// CSSOM layout metrics are read-only projections of native fragment geometry.
// Keep scroll-range and metric computation in a fragment-owned sampling domain.

use super::*;

#[inline(never)]
pub(super) fn scroll_range(node: &LayoutNode<'_>) -> LayoutScrollRange {
    let _scope = LayoutPassScope::new(
        &node.output.engine.algorithms_,
        &node.output.engine.objects_,
    );
    let box_ = DynamicTo::<LayoutBox>(node.object as *const _);
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

#[inline(never)]
pub(super) fn metric(node: &LayoutNode<'_>, metric: LayoutMetric) -> Option<f64> {
    use LayoutMetric::*;
    let _scope = LayoutPassScope::new(
        &node.output.engine.algorithms_,
        &node.output.engine.objects_,
    );
    let zoom = node.object.StyleRef().EffectiveZoom();
    let unzoom = |v: f64| {
        if zoom == 1. {
            v
        } else {
            (v as f32 / zoom) as f64
        }
    };
    let integer = |v: f64| unzoom(v).round();
    if node.object.IsLayoutInline() && !node.object.IsSVG() {
        let inline = unsafe { &*DynamicTo::<LayoutInline>(node.object as *const _) };
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
    let Some(box_) = (unsafe { DynamicTo::<LayoutBox>(node.object as *const _).as_ref() }) else {
        return Some(0.);
    };
    let Some(fragment) = box_.PhysicalFragments().into_iter().next() else {
        return Some(0.);
    };
    let size = fragment.Size();
    // LayoutBoxModelObject::OffsetWidth/Height consume this native bounding
    // box, not just the first physical fragment's dimensions.
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
            if unsafe { current.GetBreakToken().as_ref() }.is_some_and(|token| token.IsAtBlockEnd())
            {
                break;
            }
        }
    }
    let padding = box_.PhysicalPaddingBoxRect();
    let css_box = !(node.object.IsSVG()
        && !node.object.IsSVGRoot()
        && !node.object.IsSVGText()
        && !node.object.IsSVGForeignObject());
    let client = if !css_box {
        PhysicalSize::default()
    } else if box_.IsLayoutView() {
        let view = unsafe {
            &*(node.object as *const _ as *const crate::internal::layout_view::LayoutView)
        };
        let size = view.GetLayoutSize(
            crate::internal::scroll_types::IncludeScrollbarsInRect::kExcludeScrollbars,
        );
        PhysicalSize::new(
            foundation::LayoutUnit::from_signed(size.width()),
            foundation::LayoutUnit::from_signed(size.height()),
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
        MinimumScrollLeft => unzoom(scroll_range(node).minimum.x),
        MaximumScrollLeft => unzoom(scroll_range(node).maximum.x),
        MinimumScrollTop => unzoom(scroll_range(node).minimum.y),
        MaximumScrollTop => unzoom(scroll_range(node).maximum.y),
    })
}
