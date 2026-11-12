//! Native text quad enumeration. Source: core/layout/layout_text.cc:501-655,
//! 1161-1164. Cursor, offset mapping and transform owners remain resident.
use super::*;
use crate::offset_mapping::OffsetMapping;
use foundation::{gfx, PhysicalRect, Vector};
use layoutng::internal::{
    layout_box_model_object::LayoutBoxModelObject, layout_object::BoxQuadType,
    layout_text::ClippingOption, map_coordinates_flags::MapCoordinatesFlags,
};

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextQuadsInAncestorInternal(
    this: &LayoutText,
    quads: &mut Vector<gfx::QuadF>,
    ancestor: *const LayoutBoxModelObject,
    mode: MapCoordinatesFlags,
    _box_type: BoxQuadType,
) {
    this.CheckIsNotDestroyed();
    let quads = std::cell::RefCell::new(quads);
    this.CollectLineBoxRectsDefault(&|rect| {
        quads
            .borrow_mut()
            .push(this.LocalRectToAncestorQuad(rect, ancestor, mode));
    });
}

// Source: core/layout/layout_text.cc:482-499.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCollectLineBoxRects(
    this: &LayoutText,
    collector: &dyn Fn(&PhysicalRect),
    option: ClippingOption,
) {
    this.CheckIsNotDestroyed();
    if this.IsInLayoutNGInlineFormattingContext() {
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(this);
        while cursor.IsNotNull() {
            if option != ClippingOption::kNoClipping {
                debug_assert!(option == ClippingOption::kClipToEllipsis);
                if cursor.Current().IsHiddenForPaint() {
                    cursor.MoveToNextForSameLayoutObject();
                    continue;
                }
            }
            collector(&cursor.Current().RectInContainerFragment());
            cursor.MoveToNextForSameLayoutObject();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextGetOffsetMapping(this: &LayoutText) -> *const OffsetMapping {
    this.CheckIsNotDestroyed();
    OffsetMapping::GetForLayoutObject(this as *const LayoutText as *const LayoutObject)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextMapDOMOffsetToTextContentOffset(
    this: &LayoutText,
    mapping: &OffsetMapping,
    start: &mut u32,
    end: &mut u32,
) -> bool {
    this.CheckIsNotDestroyed();
    debug_assert!(*start <= *end);
    // These source OffsetMapping queries already accept a borrowed anchor owner.
    // Keep that owner rather than entering the unconnected global Position lookup.
    let node = if let Some(fragment) = unsafe {
        DynamicTo::<crate::layout_text_fragment::LayoutTextFragment>(this as *const _).as_ref()
    } {
        fragment.AssociatedTextNode().cast::<Node>()
    } else {
        this.GetNode()
    };
    let Some(node) = (unsafe { node.as_ref() }) else {
        return false;
    };
    let position = this.PositionForCaretOffset((*start).min(this.NonCollapsedCaretMaxOffset()));
    let mut first = mapping.NonCollapsedBoundaryForNode(node, &position, true);
    if first.IsNull() {
        first = mapping.NonCollapsedBoundaryForNode(node, &position, false);
        if first.IsNull() {
            return false;
        }
    }
    *start = mapping
        .GetTextContentOffsetForNode(node, &first)
        .expect("mapped start offset");
    let position = this.PositionForCaretOffset((*end).min(this.NonCollapsedCaretMaxOffset()));
    let last = mapping.NonCollapsedBoundaryForNode(node, &position, false);
    // Mapping's boundaries retain this anchor, including BR neighbor anchors.
    // Comparing the mapped offsets expresses the source Position ordering
    // without inventing a DOM order for the platform-neutral Position value.
    *end = if last.IsNull() {
        *start
    } else {
        mapping
            .GetTextContentOffsetForNode(node, &last)
            .expect("mapped end offset")
            .max(*start)
    };
    debug_assert!(*start <= *end);
    true
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextAbsoluteQuadsForRange(
    this: &LayoutText,
    quads: &mut Vector<gfx::QuadF>,
    mut start: u32,
    mut end: u32,
) {
    this.CheckIsNotDestroyed();
    debug_assert!(end == u32::MAX || end <= i32::MAX as u32);
    debug_assert!(start <= i32::MAX as u32);
    start = start.min(i32::MAX as u32);
    end = end.min(i32::MAX as u32);
    let Some(mapping) = (unsafe { this.GetOffsetMapping().as_ref() }) else {
        return;
    };
    if !this.MapDOMOffsetToTextContentOffset(mapping, &mut start, &mut end) {
        return;
    }
    let text_combine = unsafe { DynamicTo::<LayoutTextCombine>(this.Parent()).as_ref() };
    let mut found_non_collapsed_quad = false;
    let mut collapsed_quads_candidates = Vector::new();
    debug_assert!(start <= end);
    let mut cursor = InlineCursor::default();
    let mut is_last_end_included = false;
    cursor.MoveToLayoutObject(this);
    while cursor.IsNotNull() {
        let item = unsafe { &*cursor.CurrentItem() };
        debug_assert!(item.IsText());
        let mut is_collapsed = false;
        let mut rect;
        if !item.IsGeneratedText() {
            let offset = item.TextOffset();
            if start > offset.end || end < offset.start || (item.IsLineBreak() && start == end) {
                is_last_end_included = false;
                cursor.MoveToNextForSameLayoutObject();
                continue;
            }
            is_last_end_included = offset.end <= end;
            let clamped_start = start.max(offset.start);
            let clamped_end = end.min(offset.end);
            rect = cursor.CurrentLocalRect(clamped_start, clamped_end);
            is_collapsed = clamped_start >= clamped_end;
        } else if item.IsEllipsis() || !is_last_end_included {
            cursor.MoveToNextForSameLayoutObject();
            continue;
        } else {
            rect = item.LocalRect();
        }
        if let Some(text_combine) = text_combine {
            rect = text_combine.AdjustRectForBoundingBox(&rect);
        }
        let quad = if let Some(svg_data) = unsafe { item.GetSvgFragmentData().as_ref() } {
            let mut float_rect = gfx::RectF::from(rect);
            let offset = svg_data.rect.OffsetFromOrigin();
            float_rect.Offset(offset.x(), offset.y());
            let mut quad = item
                .BuildSvgTransformForBoundingBox()
                .MapQuad(gfx::QuadF::from(float_rect));
            let factor = 1.0 / item.SvgScalingFactor();
            quad.Scale(factor, factor);
            this.LocalToAbsoluteQuad(&quad, MapCoordinatesFlags::default())
        } else {
            rect.offset += cursor.CurrentOffsetInFirstContainerFragment();
            this.LocalRectToAbsoluteQuad(&rect, MapCoordinatesFlags::default())
        };
        if !is_collapsed {
            quads.push(quad);
            found_non_collapsed_quad = true;
        } else {
            collapsed_quads_candidates.push(quad);
        }
        cursor.MoveToNextForSameLayoutObject();
    }
    if !found_non_collapsed_quad {
        quads.extend(collapsed_quads_candidates);
    }
}
