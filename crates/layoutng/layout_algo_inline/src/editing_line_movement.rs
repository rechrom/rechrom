//! Native AbstractLineBox portion of selection_modifier_line.cc:54-234.
//!
//! DOM traversal, editable-root fallback and editing-boundary adjustment stay
//! in the editing owner. This module only traverses the resident InlineCursor
//! and returns the source PositionForPoint result for an adjacent line.
use super::*;
use crate::internal::editing::forward::{
    Position, PositionAnchorType, PositionWithAffinity, TextAffinity,
};
use crate::internal::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
use foundation::{LayoutUnit, PhysicalOffset, PhysicalSize, TextDirection, WritingDirectionMode};
use layout_fragment::inline_cursor::InlineCursor;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditingLinePositionResult {
    NoLine,
    Line(Option<PositionWithAffinity>),
}

fn CanBeCaretContainer(line: &InlineCursor) -> bool {
    if line.IsNull() || line.Current().IsEmptyLineBox() {
        return false;
    }
    let logical = WritingModeConverter::new(
        WritingDirectionMode::new(line.Current().Style().GetWritingMode(), TextDirection::kLtr),
        line.Current().Size(),
    )
    .ToLogicalSize(line.Current().Size());
    if logical.block_size == LayoutUnit::default() {
        return false;
    }
    let mut cursor = line.CursorForDescendants();
    while cursor.IsNotNull() {
        if cursor.Current().IsInlineLeaf() {
            let object = cursor.Current().GetLayoutObject();
            if unsafe { object.as_ref() }.is_some_and(|object| !object.NonPseudoNode().is_null()) {
                return true;
            }
        }
        cursor.MoveToNext();
    }
    false
}

fn AdjacentLine(
    mut line: InlineCursor,
    next: bool,
    editing_line_movement_with_ruby_fix: bool,
) -> Option<InlineCursor> {
    loop {
        if next {
            line.MoveToNextIncludingFragmentainer();
        } else {
            line.MoveToPreviousIncludingFragmentainer();
        }
        if line.IsNull() {
            return None;
        }
        if line.Current().IsLineBox()
            && !(editing_line_movement_with_ruby_fix && line.Current().IsRubyAnnotationLine())
        {
            break;
        }
    }
    (!unsafe { &*line.CurrentItem() }.IsBlockInInline() && CanBeCaretContainer(&line))
        .then_some(line)
}

fn PointInContainingBlock(line: &InlineCursor, absolute: LayoutUnit) -> Option<PhysicalOffset> {
    let block = unsafe { line.GetLayoutBlockFlow().as_ref() }?;
    let flags = MapCoordinatesFlags::from_mode(MapCoordinatesMode::kIgnoreTransforms);
    let mut absolute_block = block.LocalToAbsolutePointPhysical(&PhysicalOffset::default(), flags);
    if block.IsScrollContainer() {
        absolute_block -= block.ScrolledContentOffset();
    }
    let line_offset = line.Current().OffsetInContainerFragment();
    Some(if block.IsHorizontalWritingMode() {
        PhysicalOffset::new(absolute - absolute_block.left, line_offset.top)
    } else {
        PhysicalOffset::new(line_offset.left, absolute - absolute_block.top)
    })
}

fn PositionForPoint(
    line: &InlineCursor,
    point: &PhysicalOffset,
    only_editable: bool,
    options: crate::EditingPositionOptions,
) -> PositionWithAffinity {
    let converter = WritingModeConverter::new(
        WritingDirectionMode::new(
            line.Current().Style().GetWritingMode(),
            line.Current().BaseDirection(),
        ),
        line.Current().Size(),
    );
    let inline_offset = converter
        .ToLogicalOffset(
            *point,
            PhysicalSize::new(LayoutUnit::from_signed(1), LayoutUnit::from_signed(1)),
        )
        .inline_offset;
    let mut closest: Option<(InlineCursor, LayoutUnit)> = None;
    let mut cursor = line.CursorForDescendants();
    while cursor.IsNotNull() {
        let object = unsafe { cursor.Current().GetLayoutObject().as_ref() };
        if !cursor.Current().IsInlineLeaf() || object.is_none() {
            cursor.MoveToNext();
            continue;
        }
        let object = object.unwrap();
        let editable = unsafe { object.GetNode().as_ref() }
            .is_some_and(|node| node.InputEditingFacts().is_editable);
        if only_editable && !editable {
            cursor.MoveToNext();
            continue;
        }
        let rect: LogicalRect = line
            .Current()
            .ConvertChildToLogical(cursor.Current().RectInContainerFragment());
        let min = rect.offset.inline_offset;
        let max = min + rect.size.inline_size;
        if inline_offset >= min && inline_offset < max {
            closest = Some((cursor.clone(), LayoutUnit::default()));
            break;
        }
        let distance = if inline_offset < min {
            min - inline_offset
        } else {
            inline_offset - max + LayoutUnit::from_signed(1)
        };
        if closest
            .as_ref()
            .is_none_or(|(_, current)| distance < *current)
        {
            closest = Some((cursor.clone(), distance));
        }
        cursor.MoveToNext();
    }
    let Some((cursor, _)) = closest else {
        return PositionWithAffinity::default();
    };
    let Some(node) = (unsafe { cursor.Current().GetNode().as_ref() }) else {
        return PositionWithAffinity::default();
    };
    if node.InputEditingFacts().ignores_content {
        return PositionWithAffinity {
            position: Position::WithAnchor(node.InputId(), 0, PositionAnchorType::BeforeAnchor),
            affinity: TextAffinity::Downstream,
        };
    }
    cursor.PositionForPointInChild(point, options)
}

impl LayoutNode<'_> {
    /// AbstractLineBox::CreateFor(candidate)->PositionForPoint used after the
    /// DOM root-inline candidate fallback has selected a line. It intentionally
    /// does not advance again or reapply CanBeCaretContainer.
    pub fn EditingPositionOnLine(
        &self,
        position: &PositionWithAffinity,
        options: crate::EditingPositionOptions,
        absolute_line_direction_point: LayoutUnit,
        only_editable_leaves: bool,
    ) -> EditingLinePositionResult {
        let _scope = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let Some(caret) = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            options,
        ) else {
            return EditingLinePositionResult::NoLine;
        };
        let mut line = caret.cursor;
        line.MoveToContainingLine();
        if line.IsNull() {
            return EditingLinePositionResult::NoLine;
        }
        let Some(point) = PointInContainingBlock(&line, absolute_line_direction_point) else {
            return EditingLinePositionResult::Line(None);
        };
        let candidate = PositionForPoint(&line, &point, only_editable_leaves, options);
        EditingLinePositionResult::Line((!candidate.position.IsNull()).then_some(candidate))
    }

    /// Returns only the native adjacent-line candidate. `None` asks the DOM
    /// editing owner to perform Chromium's root-inline-candidate and
    /// first/last-content fallback paths.
    pub fn EditingPositionOnAdjacentLine(
        &self,
        position: &PositionWithAffinity,
        options: crate::EditingPositionOptions,
        absolute_line_direction_point: LayoutUnit,
        next: bool,
        only_editable_leaves: bool,
    ) -> EditingLinePositionResult {
        let _scope = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let Some(caret) = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            options,
        ) else {
            return EditingLinePositionResult::NoLine;
        };
        let mut line = caret.cursor;
        line.MoveToContainingLine();
        if line.IsNull() {
            return EditingLinePositionResult::NoLine;
        }
        let Some(line) = AdjacentLine(line, next, options.editing_line_movement_with_ruby_fix)
        else {
            return EditingLinePositionResult::NoLine;
        };
        let Some(point) = PointInContainingBlock(&line, absolute_line_direction_point) else {
            return EditingLinePositionResult::Line(None);
        };
        let candidate = PositionForPoint(&line, &point, only_editable_leaves, options);
        EditingLinePositionResult::Line((!candidate.position.IsNull()).then_some(candidate))
    }
}
