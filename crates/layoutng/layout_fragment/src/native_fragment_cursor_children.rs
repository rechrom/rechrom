// Native fragment child expansion is fragment topology work. The public cursor
// stays in the assembly facade, while all physical/inline/SVG traversal lives
// behind this out-of-line fragment entry.

use super::*;

#[inline(never)]
pub(super) fn native_paint_fragment_children<'a>(
    cursor: NativePaintFragmentCursor<'a>,
) -> Vec<NativePaintFragmentCursor<'a>> {
    native_fragment_children(cursor.0)
        .into_iter()
        .filter(|child| child.PaintInput().is_some())
        .map(NativePaintFragmentCursor)
        .collect()
}

#[inline(never)]
pub(super) fn native_fragment_children<'a>(
    cursor: NativeFragmentCursor<'a>,
) -> Vec<NativeFragmentCursor<'a>> {
    #[inline(never)]
    fn append_svg_children<'a>(
        object: *const LayoutObject,
        output: &mut Vec<NativeFragmentCursor<'a>>,
    ) {
        let Some(owner) = (unsafe { object.as_ref() }) else {
            return;
        };
        if !(owner.IsSVGRoot() || owner.IsSVGContainer() && !owner.IsSVGForeignObject()) {
            return;
        }
        let mut child = owner.SlowFirstChild();
        while let Some(object) = unsafe { child.as_ref() } {
            let child_box = DynamicTo::<LayoutBox>(child);
            if let Some(child_box) = unsafe { child_box.as_ref() } {
                let offset = child_box.PhysicalLocation();
                output.extend(
                    child_box
                        .PhysicalFragments()
                        .into_iter()
                        .map(|fragment| NativeFragmentCursor::Physical { fragment, offset }),
                );
            }
            child = object.NextSibling();
        }
    }

    #[inline(never)]
    fn append_box_children<'a>(
        box_fragment: &'a PhysicalBoxFragment,
        output: &mut Vec<NativeFragmentCursor<'a>>,
    ) {
        let item_list = unsafe { box_fragment.Items().as_ref() };
        for child in box_fragment.Children() {
            if !child.is_present() || item_list.is_some() && child.IsLineBox() {
                continue;
            }
            output.push(NativeFragmentCursor::Physical {
                fragment: child,
                offset: child.Offset(),
            });
        }
        if let Some(items) = item_list {
            let span = items.Items();
            let mut index = 0;
            while index < span.len() {
                let item = &span[index];
                output.push(NativeFragmentCursor::Inline {
                    item,
                    items,
                    index,
                    offset: *item.OffsetInContainerFragment(),
                });
                index += item.DescendantsCount().max(1) as usize;
            }
        }
        append_svg_children(box_fragment.GetLayoutObject(), output);
    }

    match cursor {
        NativeFragmentCursor::Physical { fragment, .. } => {
            if !fragment.IsBox() {
                return Vec::new();
            }
            let box_fragment =
                unsafe { &*(fragment as *const PhysicalFragment as *const PhysicalBoxFragment) };
            let mut output = Vec::new();
            append_box_children(box_fragment, &mut output);
            output
        }
        NativeFragmentCursor::Inline {
            item, items, index, ..
        } => {
            let descendants = item.DescendantsCount() as usize;
            let span = items.Items();
            let mut output = Vec::new();
            if descendants > 1 {
                let end = index.saturating_add(descendants).min(span.len());
                let mut child_index = index + 1;
                while child_index < end {
                    let child = &span[child_index];
                    output.push(NativeFragmentCursor::Inline {
                        item: child,
                        items,
                        index: child_index,
                        offset: *child.OffsetInContainerFragment()
                            - *item.OffsetInContainerFragment(),
                    });
                    child_index += child.DescendantsCount().max(1) as usize;
                }
            } else if let Some(box_fragment) = unsafe { item.BoxFragment().as_ref() } {
                append_box_children(box_fragment, &mut output);
            }
            output
        }
    }
}
