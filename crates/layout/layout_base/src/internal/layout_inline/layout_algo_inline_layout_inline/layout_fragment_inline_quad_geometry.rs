//! Source: core/layout/layout_inline.cc:420-508.
use super::*;

pub(super) fn LayoutInlineQuadsInAncestorInternal(
    this: &LayoutInline,
    quads: &mut Vector<gfx::QuadF>,
    ancestor: *const LayoutBoxModelObject,
    mode: MapCoordinatesFlags,
    box_type: BoxQuadType,
) {
    this.CheckIsNotDestroyed();
    LayoutInlineQuadsForSelfInternal(this, quads, ancestor, mode, true, box_type);
}

pub(super) fn LayoutInlineQuadsForSelfInternal(
    this: &LayoutInline,
    quads: &mut Vector<gfx::QuadF>,
    ancestor: *const LayoutBoxModelObject,
    mode: MapCoordinatesFlags,
    map_to_ancestor: bool,
    box_type: BoxQuadType,
) {
    this.CheckIsNotDestroyed();
    let mut mapping_to_ancestor: Option<gfx::Transform> = None;
    let mut push_quad = |quads: &mut Vector<gfx::QuadF>, rect: PhysicalRect| {
        if map_to_ancestor {
            let transform = mapping_to_ancestor.get_or_insert_with(|| {
                let mut transform = this.LocalToAncestorTransform(ancestor, mode);
                if !this.CanvasForDrawingLayoutObject().is_null() {
                    let origin = this.PhysicalLinesBoundingBox().offset;
                    transform.Translate(-origin.left.ToFloat(), -origin.top.ToFloat());
                }
                transform
            });
            quads.push(transform.MapQuad(gfx::QuadF::from(gfx::RectF::from(rect))));
        } else {
            quads.push(gfx::QuadF::from(gfx::RectF::from(rect)));
        }
    };
    let mut found_quad = false;
    if this.IsInLayoutNGInlineFormattingContext() {
        let mut cursor = InlineCursor::default();
        cursor.MoveToIncludingCulledInline(this);
        while cursor.IsNotNull() {
            let target = cursor.Current().GetLayoutObject();
            let source: &LayoutObject = this;
            let mut in_child_ruby_text = false;
            if !target.is_null() && this.IsInlineRuby() && !std::ptr::eq(target, source) {
                let mut child = target;
                while !child.is_null()
                    && unsafe { &*child }.Parent() != source as *const _ as *mut _
                {
                    child = unsafe { &*child }.Parent();
                }
                in_child_ruby_text = !child.is_null() && unsafe { &*child }.IsInlineRubyText();
            }
            if in_child_ruby_text {
                cursor.MoveToNextForSameLayoutObject();
                continue;
            }
            let mut rect = cursor.CurrentRectInFirstContainerFragment();
            if box_type == BoxQuadType::kMargin {
                let mut margins = this
                    .MarginOutsets()
                    .ConvertToLogical(this.StyleRef().GetWritingDirection());
                if !unsafe { &*cursor.CurrentItem() }.IsFirstForNode() {
                    margins.inline_start = LayoutUnit::default();
                }
                if !unsafe { &*cursor.CurrentItem() }.IsLastForNode() {
                    margins.inline_end = LayoutUnit::default();
                }
                rect.Expand(&margins.ConvertToPhysical(this.StyleRef().GetWritingDirection()));
            } else if let Some(fragment) = unsafe { cursor.Current().BoxFragment().as_ref() } {
                let offset = rect.offset;
                rect = LocalRectForBoxQuad(fragment, box_type);
                rect.offset += offset;
            } else if box_type == BoxQuadType::kPadding {
                rect.Contract(&this.BorderOutsets());
            } else if box_type == BoxQuadType::kContent {
                rect.Contract(&(this.BorderOutsets() + this.PaddingOutsets()));
            }
            rect.size.width = rect.size.width.ClampNegativeToZero();
            rect.size.height = rect.size.height.ClampNegativeToZero();
            push_quad(quads, rect);
            found_quad = true;
            cursor.MoveToNextForSameLayoutObject();
        }
    }
    if !found_quad {
        if map_to_ancestor {
            push_quad(quads, PhysicalRect::default());
        } else {
            quads.push(gfx::QuadF::default());
        }
    }
}

// Source: core/layout/layout_inline.cc:579-594.
pub(super) fn LayoutInlineMarginOutsets(this: &LayoutInline) -> PhysicalBoxStrut {
    let compute_margin = |margin: &foundation::Length| {
        if margin.IsFixed() {
            return LayoutUnit::from_f32(margin.Pixels());
        }
        if margin.IsPercent() || margin.IsCalculated() {
            return foundation::MinimumValueForLength(
                margin,
                unsafe { &*this.ContainingBlock() }.ContentLogicalWidth(),
            );
        }
        LayoutUnit::default()
    };
    PhysicalBoxStrut::new(
        compute_margin(this.StyleRef().MarginTop()),
        compute_margin(this.StyleRef().MarginRight()),
        compute_margin(this.StyleRef().MarginBottom()),
        compute_margin(this.StyleRef().MarginLeft()),
    )
}
