// Traversals over LayoutPaintOutput are fragment work even though the move-only
// output handle remains part of LayoutEngine's public compatibility surface.

use super::*;

#[inline(never)]
pub(super) fn paint_cull_rect_has_scrolled_enough(
    output: &LayoutPaintOutput,
    id: u64,
    offset: Offset,
) -> bool {
    #[inline(never)]
    fn changed(cursor: NativePaintFragmentCursor<'_>, id: u64, offset: Offset) -> bool {
        const CHANGED_ENOUGH_DISTANCE: f64 = 512.0;
        let expansion_ratio = cursor
            .paint
            .resources
            .as_ref()
            .map_or(1.0, |resources| resources.device_pixel_ratio)
            .max(f64::MIN_POSITIVE);
        let threshold = CHANGED_ENOUGH_DISTANCE * expansion_ratio;
        (cursor.node_id == id
            && cursor.paint_cull_scroll_offset.is_some_and(|origin| {
                (offset.x - origin.x).abs() >= threshold || (offset.y - origin.y).abs() >= threshold
            }))
            || cursor
                .Children()
                .into_iter()
                .any(|child| changed(child, id, offset))
    }

    changed(output.PaintCursor(), id, offset)
}

#[inline(never)]
pub(super) fn visit_paint_inputs_mut(
    output: &mut LayoutPaintOutput,
    mut visitor: impl FnMut(&mut FragmentPaintData),
) {
    #[inline(never)]
    unsafe fn visit(
        cursor: NativePaintFragmentCursor<'_>,
        visitor: &mut impl FnMut(&mut FragmentPaintData),
    ) {
        let children = cursor.Children();
        visitor(unsafe { cursor.PaintDataMut() });
        for child in children {
            unsafe { visit(child, visitor) };
        }
    }

    unsafe { visit(output.PaintCursor(), &mut visitor) };
}
