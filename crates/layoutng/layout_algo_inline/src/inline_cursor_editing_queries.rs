//! Source: core/layout/inline/inline_cursor.cc:829-915 and
//! core/layout/inline/fragment_item.cc:1258-1283.
use super::*;
use crate::internal::editing::forward::PositionWithAffinity;
use crate::layout_algo_inline_editing_caret::{
    AdjustForHitTest, InlineCaretPosition, InlineCaretPositionType,
};

impl InlineCursor {
    pub fn PositionForPointInText(
        &self,
        text_offset: u32,
        bidi_caret_affinity: bool,
    ) -> PositionWithAffinity {
        debug_assert!(self.Current().IsText());
        if !self.HasRoot() || self.Current().IsGeneratedText() {
            return PositionWithAffinity::default();
        }
        let start = self.Current().TextStartOffset();
        let end = self.Current().TextEndOffset();
        debug_assert!(text_offset <= end);
        let caret = InlineCaretPosition {
            cursor: self.clone(),
            position_type: InlineCaretPositionType::AtTextOffset,
            text_offset: Some(text_offset),
        };
        if bidi_caret_affinity || (start < text_offset && text_offset < end) {
            return caret.ToPositionInDOMTreeWithAffinity();
        }
        AdjustForHitTest(caret).ToPositionInDOMTreeWithAffinity()
    }

    pub fn PositionForPointInChild(
        &self,
        point: &PhysicalOffset,
        options: crate::EditingPositionOptions,
    ) -> PositionWithAffinity {
        let item = unsafe { &*self.CurrentItem() };
        match item.Type() {
            FragmentItemType::kText => self.PositionForPointInText(
                item.TextOffsetForPoint(*point - *item.OffsetInContainerFragment(), self.Items()),
                options.bidi_caret_affinity,
            ),
            FragmentItemType::kBox => {
                let fragment = item.BoxFragment();
                if !fragment.is_null() && !unsafe { &*fragment }.IsInlineBox() {
                    let object = unsafe { &*item.GetLayoutObject() };
                    // The box path is dispatched through its resident owner.
                    return crate::layout_algo_inline_editing_caret::PositionForPoint(
                        object,
                        &(*point - *item.OffsetInContainerFragment()),
                        options,
                    );
                }
                PositionWithAffinity::default()
            }
            _ => PositionWithAffinity::default(),
        }
    }

    pub fn PositionForStartOfLine(&self, bidi_caret_affinity: bool) -> PositionWithAffinity {
        debug_assert!(self.Current().IsLineBox());
        let mut leaf = self.CursorForDescendants();
        if self.Current().BaseDirection() == TextDirection::kLtr {
            leaf.MoveToFirstNonPseudoLeaf();
        } else {
            leaf.MoveToLastNonPseudoLeaf();
        }
        if leaf.IsNull() {
            return PositionWithAffinity::default();
        }
        let item = unsafe { &*leaf.CurrentItem() };
        let object = if item.IsBlockInInline() {
            item.BlockInInline()
        } else {
            unsafe { &*item.GetLayoutObject() }
        };
        let Some(node) = (unsafe { object.NonPseudoNode().as_ref() }) else {
            return PositionWithAffinity::default();
        };
        if !node.IsTextNode() {
            return PositionWithAffinity {
                position: crate::EditingPosition::WithAnchor(
                    node.InputId(),
                    0,
                    crate::EditingPositionAnchorType::BeforeAnchor,
                ),
                affinity: crate::EditingTextAffinity::Downstream,
            };
        }
        let offset = if self.Current().BaseDirection() == leaf.Current().ResolvedDirection() {
            leaf.Current().TextStartOffset()
        } else {
            leaf.Current().TextEndOffset()
        };
        leaf.PositionForPointInText(offset, bidi_caret_affinity)
    }

    pub fn PositionForEndOfLine(&self, bidi_caret_affinity: bool) -> PositionWithAffinity {
        debug_assert!(self.Current().IsLineBox());
        let mut leaf = self.CursorForDescendants();
        if self.Current().BaseDirection() == TextDirection::kLtr {
            leaf.MoveToLastNonPseudoLeaf();
        } else {
            leaf.MoveToFirstNonPseudoLeaf();
        }
        if leaf.IsNull() {
            return PositionWithAffinity::default();
        }
        let item = unsafe { &*leaf.CurrentItem() };
        let object = if item.IsBlockInInline() {
            item.BlockInInline()
        } else {
            unsafe { &*item.GetLayoutObject() }
        };
        let Some(node) = (unsafe { object.NonPseudoNode().as_ref() }) else {
            return PositionWithAffinity::default();
        };
        if object.IsBR() || !node.IsTextNode() {
            return PositionWithAffinity {
                position: crate::EditingPosition::WithAnchor(
                    node.InputId(),
                    0,
                    if object.IsBR() {
                        crate::EditingPositionAnchorType::BeforeAnchor
                    } else {
                        crate::EditingPositionAnchorType::AfterAnchor
                    },
                ),
                affinity: crate::EditingTextAffinity::Downstream,
            };
        }
        let offset = self.GetTextOffsetForEndOfLine(&leaf);
        leaf.PositionForPointInText(offset, bidi_caret_affinity)
    }
}

fn ShouldIgnoreForPoint(item: &FragmentItem) -> bool {
    match item.Type() {
        FragmentItemType::kBox => {
            let fragment = item.BoxFragment();
            if fragment.is_null() {
                return true;
            }
            let fragment = unsafe { &*fragment };
            if fragment.IsInlineBox() {
                return true;
            }
            if fragment.IsBlockInInline() {
                return false;
            }
            unsafe { &*item.GetLayoutObject() }
                .NonPseudoNode()
                .is_null()
        }
        FragmentItemType::kText => item.IsGeneratedText(),
        _ => true,
    }
}
fn ShouldIgnoreLineForPoint(line: &InlineCursor) -> bool {
    let mut descendants = line.CursorForDescendants();
    while descendants.IsNotNull() {
        let item = unsafe { &*descendants.CurrentItem() };
        if item.IsBlockInInline() || descendants.Current().IsInlineLeaf() {
            return false;
        }
        descendants.MoveToNext();
    }
    true
}
impl InlineCursor {
    /// Source: core/layout/inline/inline_cursor.cc:713-826.
    pub fn PositionForPointInInlineBox(
        &self,
        point: &PhysicalOffset,
        options: crate::EditingPositionOptions,
    ) -> PositionWithAffinity {
        let item = unsafe { &*self.CurrentItem() };
        let combine = DynamicTo::<crate::internal::layout_text_combine::LayoutTextCombine>(
            item.GetLayoutObject(),
        );
        let point = if combine.is_null() {
            *point
        } else {
            unsafe { &*combine }.AdjustOffsetForHitTest(point)
        };
        let converter = WritingModeConverter::new(item.Style().GetWritingDirection(), item.Size());
        let point_inline = converter
            .ToLogicalOffset(
                point,
                PhysicalSize::new(LayoutUnit::from_signed(1), LayoutUnit::from_signed(1)),
            )
            .inline_offset;
        let mut before: Option<(LayoutUnit, InlineCursorPosition)> = None;
        let mut after: Option<(LayoutUnit, InlineCursorPosition)> = None;
        let mut descendants = self.CursorForDescendants();
        while descendants.IsNotNull() {
            let child = unsafe { &*descendants.CurrentItem() };
            if ShouldIgnoreForPoint(child) {
                descendants.MoveToNext();
                continue;
            }
            let start = converter
                .ToLogicalOffset(*child.OffsetInContainerFragment(), child.Size())
                .inline_offset;
            if point_inline < start {
                if !child.IsFloating() && after.as_ref().is_none_or(|(old, _)| start < *old) {
                    after = Some((start, descendants.Current().clone()));
                }
                descendants.MoveToNext();
                continue;
            }
            let end = start + converter.ToLogicalSize(child.Size()).inline_size;
            if point_inline >= end {
                if !child.IsFloating() && before.as_ref().is_none_or(|(old, _)| end > *old) {
                    before = Some((end, descendants.Current().clone()));
                }
                descendants.MoveToNext();
                continue;
            }
            let position = descendants.PositionForPointInChild(&point, options);
            if !position.position.IsNull() {
                return position;
            }
            descendants.MoveToNext();
        }
        for (_, candidate) in after.into_iter().chain(before) {
            descendants.MoveToPosition(candidate);
            let position = descendants.PositionForPointInChild(&point, options);
            if !position.position.IsNull() {
                return position;
            }
            if !unsafe { &*descendants.CurrentItem() }
                .BoxFragment()
                .is_null()
            {
                let position = descendants.PositionForPointInInlineBox(&point, options);
                if !position.position.IsNull() {
                    return position;
                }
            }
        }
        PositionWithAffinity::default()
    }

    /// Source: core/layout/inline/inline_cursor.cc:595-710. The host supplies
    /// the platform editing behavior instead of Layout selecting an OS policy.
    pub fn PositionForPointInInlineFormattingContext(
        &mut self,
        point: &PhysicalOffset,
        container: &PhysicalBoxFragment,
        options: crate::EditingPositionOptions,
    ) -> PositionWithAffinity {
        let bidi_caret_affinity = options.bidi_caret_affinity;
        let move_to_boundary = options.move_caret_to_horizontal_boundary_when_past_top_or_bottom;
        let converter =
            WritingModeConverter::new(container.Style().GetWritingDirection(), container.Size());
        let block = converter
            .ToLogicalOffset(
                *point,
                PhysicalSize::new(LayoutUnit::from_signed(1), LayoutUnit::from_signed(1)),
            )
            .block_offset;
        let mut before: Option<(LayoutUnit, InlineCursorPosition)> = None;
        let mut after: Option<(LayoutUnit, InlineCursorPosition)> = None;
        while self.IsNotNull() {
            let child = unsafe { &*self.CurrentItem() };
            if child.Type() != FragmentItemType::kLine {
                self.MoveToNext();
                continue;
            }
            if ShouldIgnoreLineForPoint(self) {
                self.MoveToNextSkippingChildren();
                continue;
            }
            let start = converter
                .ToLogicalOffset(*child.OffsetInContainerFragment(), child.Size())
                .block_offset;
            if block < start {
                if before.as_ref().is_none_or(|(old, _)| start < *old) {
                    before = Some((start, self.Current().clone()));
                }
                self.MoveToNextSkippingChildren();
                continue;
            }
            let end = start + converter.ToLogicalSize(child.Size()).block_size;
            if block >= end {
                if after.as_ref().is_none_or(|(old, _)| end > *old) {
                    after = Some((end, self.Current().clone()));
                }
                self.MoveToNextSkippingChildren();
                continue;
            }
            let position = self.PositionForPointInInlineBox(point, options);
            if !position.position.IsNull() {
                return position;
            }
            self.MoveToNextSkippingChildren();
        }
        if let Some((_, candidate)) = before {
            self.MoveToPosition(candidate);
            let mut position = if move_to_boundary {
                self.PositionForStartOfLine(bidi_caret_affinity)
            } else {
                self.PositionForPointInInlineBox(point, options)
            };
            if !position.position.IsNull() {
                if move_to_boundary {
                    position.affinity = crate::EditingTextAffinity::Downstream;
                }
                return position;
            }
        }
        if let Some((_, candidate)) = after {
            self.MoveToPosition(candidate);
            let mut position = if move_to_boundary {
                self.PositionForEndOfLine(bidi_caret_affinity)
            } else {
                self.PositionForPointInInlineBox(point, options)
            };
            if !position.position.IsNull() {
                if move_to_boundary {
                    position.affinity = crate::EditingTextAffinity::Downstream;
                }
                return position;
            }
        }
        PositionWithAffinity::default()
    }
}
