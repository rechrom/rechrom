//! Source: core/layout/physical_box_fragment.cc:1521-1750. Point positioning
//! consumes canonical PhysicalFragmentLink and FragmentItems owners.
use super::*;
use crate::inline_cursor::InlineCursor;
use crate::internal::editing::forward::{EditingPositionOptions, PositionWithAffinity};

impl PhysicalBoxFragment {
    pub fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity {
        self.EditingPositionForPoint(point, EditingPositionOptions::default())
    }
    pub fn EditingPositionForPoint(
        &self,
        point: &PhysicalOffset,
        options: EditingPositionOptions,
    ) -> PositionWithAffinity {
        let object = unsafe { &*self.GetLayoutObject() };
        if object.IsLayoutReplaced() {
            return crate::layout_algo_inline_editing_caret::PositionForPoint(
                object, point, options,
            );
        }
        let point = if self.IsScrollContainer() {
            *point + PhysicalOffset::FromVector2d(&self.PixelSnappedScrolledContentOffset())
        } else {
            *point
        };
        if !object.ChildPaintBlockedByDisplayLock() {
            if let Some(items) = unsafe { self.Items().as_ref() } {
                let mut cursor = InlineCursor::new_with_items(self, items);
                let position =
                    cursor.PositionForPointInInlineFormattingContext(&point, self, options);
                if !position.position.IsNull() {
                    return position;
                }
                return object.CreatePositionWithAffinityDefault(0);
            }
        }
        if !options.prevent_text_selection_jump
            && !DynamicTo::<LayoutBlockFlow>(object).is_null()
            && object.ChildrenInline()
        {
            return object.CreatePositionWithAffinityDefault(0);
        }
        if object.IsTable() {
            if !object.NonPseudoNode().is_null() {
                let left = if self.Style().IsFlippedBlocksWritingMode() {
                    self.Size().width - point.left
                } else {
                    point.left
                };
                if left < LayoutUnit::default()
                    || left > self.Size().width
                    || point.top < LayoutUnit::default()
                    || point.top > self.Size().height
                {
                    return if left <= self.Size().width / 2 {
                        object.FirstPositionInOrBeforeThis()
                    } else {
                        object.LastPositionInOrAfterThis()
                    };
                }
            }
        } else if ShouldUsePositionForPointInBlockFlowDirection(object) {
            return self.EditingPositionForPointInBlockFlowDirection(&point, options);
        }
        self.EditingPositionForPointByClosestChild(&point, options)
    }

    fn EditingPositionForPointByClosestChild(
        &self,
        point: &PhysicalOffset,
        options: EditingPositionOptions,
    ) -> PositionWithAffinity {
        let object = unsafe { &*self.GetLayoutObject() };
        if object.ChildPaintBlockedByDisplayLock() {
            return object.FirstPositionInOrBeforeThis();
        }
        let mut closest = None;
        let mut shortest = LayoutUnit::Max();
        let mut found_candidate = false;
        let one = LayoutUnit::from_signed(1);
        for child in self.Children() {
            let fragment = unsafe { &*(child.fragment.Get() as *const PhysicalBoxFragment) };
            let candidate = IsHitTestCandidate(fragment);
            if !candidate {
                if found_candidate {
                    continue;
                }
                if fragment.Style().Visibility() != EVisibility::kVisible
                    || (fragment.Children().is_empty() && !fragment.IsBlockFlow())
                {
                    continue;
                }
            }
            let left = child.offset.left;
            let right = left + fragment.Size().width;
            let top = child.offset.top;
            let bottom = top + fragment.Size().height;
            let dx = if left > point.left {
                left - point.left
            } else if point.left + one > right {
                point.left + one - right
            } else {
                LayoutUnit::default()
            };
            let dy = if top > point.top {
                top - point.top
            } else if point.top + one > bottom {
                point.top + one - bottom
            } else {
                LayoutUnit::default()
            };
            if dx == LayoutUnit::default() && dy == LayoutUnit::default() {
                closest = Some((fragment, child.offset));
                break;
            }
            let distance = dx * dx + dy * dy;
            if distance < shortest || (candidate && !found_candidate) {
                shortest = distance;
                closest = Some((fragment, child.offset));
                found_candidate = candidate;
            }
        }
        if let Some((fragment, offset)) = closest {
            fragment.EditingPositionForPoint(&(*point - offset), options)
        } else {
            object.FirstPositionInOrBeforeThis()
        }
    }

    fn EditingPositionForPointInBlockFlowDirection(
        &self,
        point: &PhysicalOffset,
        options: EditingPositionOptions,
    ) -> PositionWithAffinity {
        let object = unsafe { &*self.GetLayoutObject() };
        if object.ChildPaintBlockedByDisplayLock() {
            return object.FirstPositionInOrBeforeThis();
        }
        let converter = WritingModeConverter::new(self.Style().GetWritingDirection(), self.Size());
        let logical = converter.ToLogicalOffset(*point, foundation::PhysicalSize::default());
        let flipped = self.Style().IsFlippedBlocksWritingMode();
        let mut last = None;
        for child in self.Children() {
            let fragment = unsafe { &*(child.fragment.Get() as *const PhysicalBoxFragment) };
            if !IsHitTestCandidate(fragment) {
                continue;
            }
            let end = converter
                .ToLogicalRect(PhysicalRect::new(child.offset, fragment.Size()))
                .BlockEndOffset();
            if logical.block_offset < end || (flipped && logical.block_offset == end) {
                return self.EditingPositionRespectingBoundaries(
                    fragment,
                    &(*point - child.offset),
                    options,
                );
            }
            last = Some((fragment, child.offset));
        }
        if let Some((fragment, offset)) = last {
            return self.EditingPositionRespectingBoundaries(fragment, &(*point - offset), options);
        }
        self.EditingPositionForPointByClosestChild(point, options)
    }

    fn EditingPositionRespectingBoundaries(
        &self,
        child: &PhysicalBoxFragment,
        point: &PhysicalOffset,
        options: EditingPositionOptions,
    ) -> PositionWithAffinity {
        let child_object = unsafe { &*child.GetLayoutObject() };
        let child_node = unsafe { child_object.NonPseudoNode().as_ref() };
        if !child.IsCSSBox() || child_node.is_none() {
            return child.EditingPositionForPoint(point, options);
        }
        let mut ancestor = self.GetLayoutObject();
        while !ancestor.is_null() && unsafe { &*ancestor }.NonPseudoNode().is_null() {
            ancestor = unsafe { &*ancestor }.Parent();
        }
        let Some(ancestor) = (unsafe { ancestor.as_ref() }) else {
            return child.EditingPositionForPoint(point, options);
        };
        let parent = unsafe { ancestor.Parent().as_ref() };
        let editable_agrees = unsafe { &*ancestor.NonPseudoNode() }
            .InputEditingFacts()
            .is_editable
            == child_node.unwrap().InputEditingFacts().is_editable;
        if parent.is_none()
            || (ancestor.HasLayer() && parent.unwrap().IsLayoutView())
            || editable_agrees
        {
            return child.EditingPositionForPoint(point, options);
        }
        let converter =
            WritingModeConverter::new(child.Style().GetWritingDirection(), child.Size());
        if converter
            .ToLogicalOffset(*point, foundation::PhysicalSize::default())
            .inline_offset
            < converter.ToLogicalSize(child.Size()).inline_size / 2
        {
            child_object.PositionBeforeThis()
        } else {
            child_object.PositionAfterThis()
        }
    }
}
