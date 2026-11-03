// Fragment child expansion is fragment topology work. Iterate Chromium's
// PhysicalFragment/FragmentItem links directly; a cursor walk must not build
// a second Vec of the same children at every node.

use super::*;
use crate::fragment_item::FragmentItem;
use crate::internal::layout_box::PhysicalFragmentIterator;

enum FragmentChildrenState<'a> {
    Empty,
    Inline {
        item: &'a FragmentItem,
        items: &'a layout_fragment::fragment_items::FragmentItems,
        container: &'a PhysicalBoxFragment,
        next: usize,
        end: usize,
    },
    Box {
        fragment: &'a PhysicalBoxFragment,
        physical: usize,
        inline: usize,
        svg_object: *const LayoutObject,
        svg_fragments: Option<PhysicalFragmentIterator<'a>>,
        svg_offset: PhysicalOffset,
    },
}

pub(crate) struct FragmentChildren<'a> {
    state: FragmentChildrenState<'a>,
}

impl<'a> FragmentChildren<'a> {
    fn box_children(fragment: &'a PhysicalBoxFragment) -> Self {
        let object = fragment.GetLayoutObject();
        let svg_object = unsafe { object.as_ref() }
            .filter(|owner| {
                owner.IsSVGRoot() || owner.IsSVGContainer() && !owner.IsSVGForeignObject()
            })
            .map_or(std::ptr::null(), |owner| owner.SlowFirstChild());
        Self {
            state: FragmentChildrenState::Box {
                fragment,
                physical: 0,
                inline: 0,
                svg_object,
                svg_fragments: None,
                svg_offset: PhysicalOffset::default(),
            },
        }
    }

    fn new(cursor: FragmentCursor<'a>) -> Self {
        match cursor {
            FragmentCursor::Physical { fragment, .. } if fragment.IsBox() => {
                let fragment = unsafe {
                    &*(fragment as *const PhysicalFragment as *const PhysicalBoxFragment)
                };
                Self::box_children(fragment)
            }
            FragmentCursor::Physical { .. } => Self {
                state: FragmentChildrenState::Empty,
            },
            FragmentCursor::Inline {
                item,
                items,
                container,
                index,
                ..
            } => {
                let descendants = item.DescendantsCount() as usize;
                if descendants > 1 {
                    Self {
                        state: FragmentChildrenState::Inline {
                            item,
                            items,
                            container,
                            next: index + 1,
                            end: index.saturating_add(descendants).min(items.Items().len()),
                        },
                    }
                } else if let Some(fragment) = unsafe { item.BoxFragment().as_ref() } {
                    Self::box_children(fragment)
                } else {
                    Self {
                        state: FragmentChildrenState::Empty,
                    }
                }
            }
        }
    }
}

impl<'a> Iterator for FragmentChildren<'a> {
    type Item = FragmentCursor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.state {
                FragmentChildrenState::Empty => return None,
                FragmentChildrenState::Inline {
                    item,
                    items,
                    container,
                    next,
                    end,
                } => {
                    if *next >= *end {
                        self.state = FragmentChildrenState::Empty;
                        continue;
                    }
                    let index = *next;
                    let child = &items.Items()[index];
                    *next += child.DescendantsCount().max(1) as usize;
                    return Some(FragmentCursor::Inline {
                        item: child,
                        items,
                        container,
                        index,
                        offset: *child.OffsetInContainerFragment()
                            - *item.OffsetInContainerFragment(),
                    });
                }
                FragmentChildrenState::Box {
                    fragment,
                    physical,
                    inline,
                    svg_object,
                    svg_fragments,
                    svg_offset,
                } => {
                    let item_list = unsafe { fragment.Items().as_ref() };
                    let children = fragment.Children();
                    while *physical < children.len() {
                        let child = &children[*physical];
                        *physical += 1;
                        if !child.is_present() || item_list.is_some() && child.IsLineBox() {
                            continue;
                        }
                        return Some(FragmentCursor::Physical {
                            fragment: child,
                            offset: child.Offset(),
                        });
                    }
                    if let Some(items) = item_list {
                        let span = items.Items();
                        if *inline < span.len() {
                            let index = *inline;
                            let item = &span[index];
                            *inline += item.DescendantsCount().max(1) as usize;
                            return Some(FragmentCursor::Inline {
                                item,
                                items,
                                container: fragment,
                                index,
                                offset: *item.OffsetInContainerFragment(),
                            });
                        }
                    }
                    if let Some(iterator) = svg_fragments.as_mut() {
                        if let Some(child) = iterator.next() {
                            return Some(FragmentCursor::Physical {
                                fragment: child,
                                offset: *svg_offset,
                            });
                        }
                        *svg_fragments = None;
                    }
                    let Some(object) = (unsafe { svg_object.as_ref() }) else {
                        self.state = FragmentChildrenState::Empty;
                        continue;
                    };
                    *svg_object = object.NextSibling();
                    let child_box = DynamicTo::<LayoutBox>(object as *const LayoutObject);
                    let Some(child_box): Option<&'a LayoutBox> = (unsafe { child_box.as_ref() })
                    else {
                        continue;
                    };
                    *svg_offset = child_box.PhysicalLocation();
                    *svg_fragments = Some(child_box.PhysicalFragments().into_iter());
                }
            }
        }
    }
}

pub(crate) struct PaintFragmentChildren<'a> {
    sources: FragmentChildren<'a>,
    state: Option<&'a LayoutPaintState>,
}

impl<'a> PaintFragmentChildren<'a> {
    pub(crate) fn new(cursor: PaintFragmentCursor<'a>) -> Self {
        Self {
            sources: FragmentChildren::new(cursor.source),
            state: cursor.state,
        }
    }
}

impl<'a> Iterator for PaintFragmentChildren<'a> {
    type Item = PaintFragmentCursor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let child = PaintFragmentCursor {
                source: self.sources.next()?,
                state: self.state,
            };
            if child.PaintDataOption().is_some() {
                return Some(child);
            }
        }
    }
}

#[inline]
pub(super) fn paint_fragment_children<'a>(
    cursor: PaintFragmentCursor<'a>,
) -> PaintFragmentChildren<'a> {
    PaintFragmentChildren::new(cursor)
}

#[inline]
pub(super) fn fragment_children<'a>(cursor: FragmentCursor<'a>) -> FragmentChildren<'a> {
    FragmentChildren::new(cursor)
}
