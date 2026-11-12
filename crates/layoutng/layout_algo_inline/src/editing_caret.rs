//! Source: core/layout/inline/inline_caret_position.cc:492-539 and
//! core/editing/bidi_adjustment.cc:22-51,98-156,169-258,346-391,521-571,755-766.
//! Caret cursors borrow the canonical FragmentItems, including visual bidi order.
use crate::inline_cursor::InlineCursor;
use crate::internal::editing::forward::{
    Position, PositionAnchorType, PositionWithAffinity, TextAffinity,
};
use crate::offset_mapping::OffsetMapping;
use foundation::TextDirection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InlineCaretPositionType {
    BeforeBox,
    AfterBox,
    AtTextOffset,
}
#[derive(Clone)]
pub(crate) struct InlineCaretPosition {
    pub cursor: InlineCursor,
    pub position_type: InlineCaretPositionType,
    pub text_offset: Option<u32>,
}
impl InlineCaretPosition {
    pub fn ToPositionInDOMTreeWithAffinity(&self) -> PositionWithAffinity {
        if self.cursor.IsNull() {
            return PositionWithAffinity::default();
        }
        match self.position_type {
            InlineCaretPositionType::BeforeBox | InlineCaretPositionType::AfterBox => {
                let Some(node) = (unsafe { self.cursor.Current().GetNode().as_ref() }) else {
                    return PositionWithAffinity::default();
                };
                let before = self.position_type == InlineCaretPositionType::BeforeBox;
                PositionWithAffinity {
                    position: Position::WithAnchor(
                        node.InputId(),
                        0,
                        if before {
                            PositionAnchorType::BeforeAnchor
                        } else {
                            PositionAnchorType::AfterAnchor
                        },
                    ),
                    affinity: if before {
                        TextAffinity::Downstream
                    } else {
                        TextAffinity::Upstream
                    },
                }
            }
            InlineCaretPositionType::AtTextOffset => {
                let offset = self.text_offset.expect("text caret requires a text offset");
                let Some(mapping) = (unsafe {
                    OffsetMapping::GetForLayoutObject(self.cursor.Current().GetLayoutObject())
                        .as_ref()
                }) else {
                    return PositionWithAffinity::default();
                };
                let affinity = if offset == self.cursor.Current().TextEndOffset() {
                    TextAffinity::Upstream
                } else {
                    TextAffinity::Downstream
                };
                let position = if affinity == TextAffinity::Downstream {
                    mapping.GetLastPosition(offset)
                } else {
                    mapping.GetFirstPosition(offset)
                };
                if position.IsNull() {
                    PositionWithAffinity::default()
                } else {
                    PositionWithAffinity { position, affinity }
                }
            }
        }
    }
    pub(crate) fn IsAtFragmentStart(&self) -> bool {
        match self.position_type {
            InlineCaretPositionType::BeforeBox => true,
            InlineCaretPositionType::AfterBox => false,
            InlineCaretPositionType::AtTextOffset => {
                self.text_offset == Some(self.cursor.Current().TextStartOffset())
            }
        }
    }
    pub(crate) fn IsAtFragmentEnd(&self) -> bool {
        match self.position_type {
            InlineCaretPositionType::BeforeBox => false,
            InlineCaretPositionType::AfterBox => true,
            InlineCaretPositionType::AtTextOffset => {
                self.text_offset == Some(self.cursor.Current().TextEndOffset())
            }
        }
    }
}
pub(crate) fn ResolvedDirection(cursor: &InlineCursor) -> TextDirection {
    if cursor.Current().IsText() || cursor.Current().IsAtomicInline() {
        return cursor.Current().ResolvedDirection();
    }
    let mut line = cursor.CursorForMovingAcrossFragmentainer();
    line.MoveToContainingLine();
    line.Current().BaseDirection()
}
pub(crate) fn BidiLevel(cursor: &InlineCursor) -> u8 {
    if cursor.Current().IsText() || cursor.Current().IsAtomicInline() {
        return cursor.Current().BidiLevel();
    }
    u8::from(ResolvedDirection(cursor) != TextDirection::kLtr)
}
pub(crate) fn StepIgnoringLineBreak(cursor: &InlineCursor, right: bool) -> InlineCursor {
    let mut result = cursor.clone();
    if right {
        result.MoveToNextInlineLeafIgnoringLineBreak();
    } else {
        result.MoveToPreviousInlineLeafIgnoringLineBreak();
    }
    result
}
pub(crate) fn AtSide(cursor: InlineCursor, at_left: bool) -> InlineCaretPosition {
    let cursor = cursor.CursorForMovingAcrossFragmentainer();
    let at_start = (ResolvedDirection(&cursor) == TextDirection::kLtr) == at_left;
    if cursor.Current().IsText() {
        let offset = if at_start {
            cursor.Current().TextStartOffset()
        } else {
            cursor.Current().TextEndOffset()
        };
        InlineCaretPosition {
            cursor,
            position_type: InlineCaretPositionType::AtTextOffset,
            text_offset: Some(offset),
        }
    } else {
        InlineCaretPosition {
            cursor,
            position_type: if at_start {
                InlineCaretPositionType::BeforeBox
            } else {
                InlineCaretPositionType::AfterBox
            },
            text_offset: None,
        }
    }
}
/// Chromium's HitTestAdjuster, traversing the resident line in visual order.
/// This is separate from caret-resolution and range-selection adjustment.
pub(crate) fn AdjustForHitTest(caret: InlineCaretPosition) -> InlineCaretPosition {
    debug_assert!(caret.IsAtFragmentStart() || caret.IsAtFragmentEnd());
    let at_left =
        caret.IsAtFragmentStart() == (ResolvedDirection(&caret.cursor) == TextDirection::kLtr);
    let mut line = caret.cursor.CursorForMovingAcrossFragmentainer();
    line.MoveToContainingLine();
    let primary = line.Current().BaseDirection();
    let mut leaf = line.CursorForDescendants();
    leaf.MoveToCursor(&caret.cursor);
    if ResolvedDirection(&leaf) == primary {
        return caret;
    }
    let level = BidiLevel(&leaf);
    // A left-side hit uses TraverseRight; a right-side hit uses TraverseLeft.
    let forward_right = at_left;
    let backward = StepIgnoringLineBreak(&leaf, !forward_right);
    if backward.IsNotNull() && BidiLevel(&backward) == level {
        return caret;
    }
    if backward.IsNotNull() && BidiLevel(&backward) > level {
        let mut boundary = backward;
        loop {
            let next = StepIgnoringLineBreak(&boundary, !forward_right);
            if next.IsNull() || BidiLevel(&next) <= level {
                break;
            }
            boundary = next;
        }
        return AtSide(boundary, !forward_right);
    }
    let mut boundary = leaf.clone();
    loop {
        let next = StepIgnoringLineBreak(&boundary, forward_right);
        if next.IsNull() || BidiLevel(&next) < level {
            break;
        }
        boundary = next;
    }
    let same_direction = ResolvedDirection(&leaf) == ResolvedDirection(&boundary);
    AtSide(
        boundary,
        if same_direction {
            !forward_right
        } else {
            forward_right
        },
    )
}

/// Source: core/layout/layout_text.cc:661-713; layout_inline.cc:699-708;
/// layout_box.cc:3181-3217. The returned value remains a source DOM position;
/// the editing owner performs AdjustForEditingBoundary after projection.
pub(crate) fn PositionForPoint(
    object: &crate::internal::layout_object::LayoutObject,
    point: &foundation::PhysicalOffset,
    options: crate::EditingPositionOptions,
) -> PositionWithAffinity {
    use crate::internal::layout_box::LayoutBox;
    use foundation::{DynamicTo, LayoutUnit, PhysicalOffset};
    use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
    if object.IsBR() {
        return object.PositionBeforeThis();
    }
    if object.IsLayoutReplaced() {
        let replaced = DynamicTo::<crate::internal::layout_replaced::LayoutReplaced>(object);
        if let Some(replaced) = unsafe { replaced.as_ref() } {
            return replaced.PositionForPoint(point);
        }
    }
    // Source: core/layout/svg/layout_svg_inline_text.cc:129-167.
    if object.IsSVGInlineText() {
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(object);
        let mut closest = None;
        let mut shortest = LayoutUnit::Max();
        while cursor.IsNotNull() {
            let item = unsafe { &*cursor.CurrentItem() };
            let transformed = item.MapPointInContainer(*point);
            let rect = cursor.Current().RectInContainerFragment();
            let contains = transformed.left >= rect.X()
                && transformed.left < rect.Right()
                && transformed.top >= rect.Y()
                && transformed.top < rect.Bottom();
            let hit = contains
                && !cursor
                    .PositionForPointInChild(&transformed, options)
                    .position
                    .IsNull();
            let distance = if hit {
                LayoutUnit::default()
            } else {
                rect.SquaredDistanceTo(&transformed)
            };
            if distance <= shortest {
                shortest = distance;
                closest = Some((cursor.clone(), transformed));
            }
            cursor.MoveToNextForSameLayoutObject();
        }
        return closest.map_or_else(
            || object.CreatePositionWithAffinityDefault(0),
            |(cursor, point)| cursor.PositionForPointInChild(&point, options),
        );
    }
    if object.IsText() && object.IsInLayoutNGInlineFormattingContext() {
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(object);
        if cursor.HasRoot() {
            let container = unsafe { &*cursor.GetLayoutBlockFlow() };
            let contents_point = if container.IsScrollContainer() {
                *point
                    + PhysicalOffset::FromVector2d(&container.PixelSnappedScrolledContentOffset())
            } else {
                *point
            };
            let combine = DynamicTo::<crate::internal::layout_text_combine::LayoutTextCombine>(
                object.Parent(),
            );
            while cursor.IsNotNull() {
                let mut local =
                    contents_point - cursor.ContainerFragment().OffsetFromOwnerLayoutBox();
                if !combine.is_null() {
                    local = unsafe { &*combine }.AdjustOffsetForHitTest(&local);
                }
                let rect = cursor.Current().RectInContainerFragment();
                // Source: ToEnclosingRect(rect).Contains(ToFlooredPoint(point)).
                if local.left.Floor() >= rect.offset.left.Floor()
                    && local.top.Floor() >= rect.offset.top.Floor()
                    && local.left.Floor() < (rect.offset.left + rect.size.width).Ceil()
                    && local.top.Floor() < (rect.offset.top + rect.size.height).Ceil()
                {
                    let result = cursor.PositionForPointInChild(&local, options);
                    if !result.position.IsNull() {
                        return result;
                    }
                }
                cursor.MoveToNextForSameLayoutObject();
            }
            return PositionForPoint(
                unsafe {
                    &*(container as *const _ as *const crate::internal::layout_object::LayoutObject)
                },
                point,
                options,
            );
        }
        return object.CreatePositionWithAffinityDefault(0);
    }
    if object.IsLayoutInline() {
        if let Some(container) = unsafe { object.FragmentItemsContainer().as_ref() } {
            return PositionForPoint(
                unsafe {
                    &*(container as *const _ as *const crate::internal::layout_object::LayoutObject)
                },
                point,
                options,
            );
        }
    }
    let box_object = DynamicTo::<LayoutBox>(object);
    if let Some(box_object) = unsafe { box_object.as_ref() }.filter(|_| object.IsLayoutBlock()) {
        // Source layout_block.cc:405-441; layout_block_flow.cc:746-765.
        // An atomic inline block resolves points outside its logical box to
        // the corresponding DOM boundary before examining resident fragments.
        if object.IsInline() {
            let converter =
                layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter::new(
                    foundation::WritingDirectionMode::new(
                        box_object.StyleRef().GetWritingMode(),
                        box_object.ResolvedDirection(),
                    ),
                    box_object.StitchedSize(),
                );
            let logical = converter.ToLogicalOffset(*point, foundation::PhysicalSize::default());
            if logical.inline_offset < LayoutUnit::default() {
                return object.FirstPositionInOrBeforeThis();
            }
            if logical.inline_offset >= box_object.LogicalWidth() {
                return object.LastPositionInOrAfterThis();
            }
            if logical.block_offset < LayoutUnit::default() {
                return object.FirstPositionInOrBeforeThis();
            }
            if logical.block_offset >= box_object.LogicalHeight() {
                return object.LastPositionInOrAfterThis();
            }
        }
        if box_object.PhysicalFragmentCount() != 0 {
            // Reuse the source LayoutBox entry rather than an adapter walk.
            return if options == crate::EditingPositionOptions::default() {
                box_object.PositionForPointInFragments(point)
            } else {
                PositionForPointInFragments(box_object, point, options)
            };
        }
    }
    // Source LayoutObject::PositionForPoint, layout_object.cc:4504-4512;
    // also LayoutBlock's no-fragment LayoutBox/base fallback.
    object.CreatePositionWithAffinityDefault(0)
}

fn Step(cursor: &InlineCursor, right: bool) -> InlineCursor {
    let mut result = cursor.clone();
    if right {
        result.MoveToNextInlineLeaf();
    } else {
        result.MoveToPreviousInlineLeaf();
    }
    result
}
/// Source: core/editing/bidi_adjustment.cc:393-520,742-753.
fn AdjustForCaretResolution(caret: InlineCaretPosition) -> InlineCaretPosition {
    let at_left =
        caret.IsAtFragmentStart() == (ResolvedDirection(&caret.cursor) == TextDirection::kLtr);
    let forward_right = at_left;
    let mut line = caret.cursor.CursorForMovingAcrossFragmentainer();
    line.MoveToContainingLine();
    let primary = line.Current().BaseDirection();
    let mut leaf = line.CursorForDescendants();
    leaf.MoveToCursor(&caret.cursor);
    if ResolvedDirection(&leaf) == primary {
        let backward = Step(&leaf, !forward_right);
        let different = backward.IsNull()
            || if !forward_right {
                BidiLevel(&backward) >= BidiLevel(&leaf)
            } else {
                ResolvedDirection(&backward) == ResolvedDirection(&leaf)
                    || BidiLevel(&backward) > BidiLevel(&leaf)
            };
        if different {
            return caret;
        }
        let level = BidiLevel(&backward);
        let mut runner = Step(&leaf, forward_right);
        while runner.IsNotNull() && BidiLevel(&runner) > level {
            runner = Step(&runner, forward_right);
        }
        if runner.IsNotNull() && BidiLevel(&runner) == level {
            return caret;
        }
        let mut boundary = leaf;
        loop {
            let next = Step(&boundary, !forward_right);
            if next.IsNull() || BidiLevel(&next) < level {
                break;
            }
            boundary = next;
        }
        return AtSide(boundary, forward_right);
    }
    let level = BidiLevel(&leaf);
    let backward = StepIgnoringLineBreak(&leaf, !forward_right);
    if backward.IsNull() || BidiLevel(&backward) < level {
        let mut boundary = leaf;
        loop {
            let next = StepIgnoringLineBreak(&boundary, forward_right);
            if next.IsNull() || BidiLevel(&next) < level {
                break;
            }
            boundary = next;
        }
        return AtSide(boundary, !forward_right);
    }
    if BidiLevel(&backward) <= level {
        return caret;
    }
    let mut boundary = leaf;
    loop {
        let next = StepIgnoringLineBreak(&boundary, !forward_right);
        if next.IsNull() || BidiLevel(&next) <= level {
            break;
        }
        boundary = next;
    }
    AtSide(boundary, forward_right)
}
fn AdjustResolved(caret: InlineCaretPosition, bidi_caret_affinity: bool) -> InlineCaretPosition {
    if bidi_caret_affinity
        || (caret.position_type == InlineCaretPositionType::AtTextOffset
            && !caret.IsAtFragmentStart()
            && !caret.IsAtFragmentEnd())
    {
        caret
    } else {
        AdjustForCaretResolution(caret)
    }
}
fn CanResolveBefore(
    cursor: &InlineCursor,
    affinity: TextAffinity,
    bidi_caret_affinity: bool,
) -> bool {
    if affinity == TextAffinity::Downstream {
        return true;
    }
    if bidi_caret_affinity {
        return false;
    }
    let mut line = cursor.clone();
    line.MoveToContainingLine();
    let mut first = line.clone();
    first.MoveToFirstLogicalLeaf();
    if first != *cursor {
        return true;
    }
    line.MoveToPreviousLine();
    line.IsNull() || !line.Current().HasSoftWrapToNextLine()
}
fn CanResolveAfter(
    cursor: &InlineCursor,
    affinity: TextAffinity,
    bidi_caret_affinity: bool,
) -> bool {
    if affinity == TextAffinity::Upstream {
        return true;
    }
    if bidi_caret_affinity {
        return false;
    }
    let mut line = cursor.clone();
    line.MoveToContainingLine();
    let mut last = line.clone();
    last.MoveToLastLogicalLeaf();
    *cursor != last || !line.Current().HasSoftWrapToNextLine()
}
fn IsAfterLineBreak(caret: &InlineCaretPosition) -> bool {
    caret.position_type == InlineCaretPositionType::AtTextOffset
        && caret.cursor.Current().IsLineBreak()
        && caret.IsAtFragmentEnd()
}

/// Source: core/layout/inline/inline_caret_position.cc:98-235,383-421.
/// The caller supplies the source-resolved context/offset, preserving the
/// original OffsetMapping and fragment traversal order.
pub(crate) fn ComputeInlineCaretPosition(
    context: &crate::internal::layout_block_flow::LayoutBlockFlow,
    offset: u32,
    affinity: TextAffinity,
    layout_text: Option<&crate::internal::layout_object::LayoutObject>,
    options: crate::EditingPositionOptions,
) -> Option<InlineCaretPosition> {
    let mut cursor = InlineCursor::from_container(context);
    if let Some(text) = layout_text {
        let text_ptr = foundation::DynamicTo::<crate::internal::layout_text::LayoutText>(text);
        if let Some(text_object) = unsafe { text_ptr.as_ref() } {
            if text_object.HasInlineFragments() {
                cursor.MoveToLayoutObject(text);
            }
        }
    }
    let mut candidate: Option<InlineCaretPosition> = None;
    while cursor.IsNotNull() {
        let mut resolved = false;
        let mut current = None;
        if cursor.Current().IsText() && !cursor.Current().IsGeneratedText() {
            let mapping = unsafe {
                OffsetMapping::GetForLayoutObject(cursor.Current().GetLayoutObject()).as_ref()
            }?;
            let start = cursor.Current().TextStartOffset();
            let end = cursor.Current().TextEndOffset();
            if offset < start && !mapping.HasBidiControlCharactersOnly(offset, start) {
                cursor.MoveToNextIncludingFragmentainer();
                continue;
            }
            let units = mapping.GetText().Span16().unwrap_or_default();
            if affinity == TextAffinity::Upstream
                && offset == end + 1
                && cursor.Current().Style().NeedsTrailingSpace()
                && units.get((offset - 1) as usize).is_some_and(|character| {
                    cursor.Current().Style().IsCollapsibleWhiteSpace(*character)
                })
            {
                current = Some(InlineCaretPosition {
                    cursor: cursor.clone(),
                    position_type: InlineCaretPositionType::AtTextOffset,
                    text_offset: Some(offset - 1),
                });
                resolved = true;
            } else if offset <= end || mapping.HasBidiControlCharactersOnly(end, offset) {
                let offset = offset.clamp(start, end);
                resolved = (start < offset && offset < end)
                    || (offset == start
                        && CanResolveBefore(&cursor, affinity, options.bidi_caret_affinity))
                    || (offset == end
                        && !cursor.Current().IsLineBreak()
                        && CanResolveAfter(&cursor, affinity, options.bidi_caret_affinity));
                current = Some(InlineCaretPosition {
                    cursor: cursor.clone(),
                    position_type: InlineCaretPositionType::AtTextOffset,
                    text_offset: Some(offset),
                });
            }
        } else if cursor.Current().IsAtomicInline() {
            if let Some(node) = unsafe { cursor.Current().GetNode().as_ref() } {
                let object = unsafe { &*cursor.Current().GetLayoutObject() };
                if !object.NonPseudoNode().is_null() {
                    if let Some(mapping) =
                        unsafe { OffsetMapping::GetForLayoutObject(object).as_ref() }
                    {
                        let units = mapping.GetMappingUnitsForNode(node);
                        if let Some(unit) = units.first() {
                            let before = unit.ConvertDOMOffsetToTextContent(0);
                            if offset == before || offset == before + 1 {
                                let is_before = offset == before;
                                resolved = if is_before {
                                    CanResolveBefore(&cursor, affinity, options.bidi_caret_affinity)
                                } else {
                                    CanResolveAfter(&cursor, affinity, options.bidi_caret_affinity)
                                };
                                current = Some(InlineCaretPosition {
                                    cursor: cursor.clone(),
                                    position_type: if is_before {
                                        InlineCaretPositionType::BeforeBox
                                    } else {
                                        InlineCaretPositionType::AfterBox
                                    },
                                    text_offset: None,
                                });
                            }
                        }
                    }
                }
            }
        }
        if let Some(current) = current {
            if resolved {
                if layout_text.is_none_or(|text| {
                    std::ptr::eq(current.cursor.Current().GetLayoutObject(), text)
                }) {
                    return Some(AdjustResolved(current, options.bidi_caret_affinity));
                }
                candidate = Some(current);
            } else if candidate.as_ref().is_none_or(IsAfterLineBreak) {
                candidate = Some(current);
            }
        }
        cursor.MoveToNextIncludingFragmentainer();
    }
    candidate.map(|caret| AdjustResolved(caret, options.bidi_caret_affinity))
}

/// Source: core/layout/inline/inline_caret_position.cc:293-379,423-490.
/// The DOM anchor is resolved by the owning LayoutOutput, not a global tree.
pub(crate) fn ResolveForObject(
    object: &crate::internal::layout_object::LayoutObject,
    position: &PositionWithAffinity,
    options: crate::EditingPositionOptions,
) -> Option<InlineCaretPosition> {
    use crate::internal::layout_object::AssociatedLayoutObjectOfDefault;
    let node = unsafe { object.NonPseudoNode().as_ref() }?;
    if position.position.node_id != Some(node.InputId())
        || !object.IsInLayoutNGInlineFormattingContext()
    {
        return None;
    }
    if object.IsLayoutInline() {
        let after = match position.position.anchor {
            PositionAnchorType::BeforeAnchor => false,
            PositionAnchorType::AfterAnchor => true,
            _ => return None,
        };
        let mut cursor = InlineCursor::default();
        cursor.MoveToIncludingCulledInline(object);
        if cursor.IsNull() {
            return None;
        }
        let mut line = cursor.clone();
        line.MoveToContainingLine();
        let ltr = line.Current().BaseDirection() == TextDirection::kLtr;
        if ltr == after {
            cursor.MoveToVisualLastForSameLayoutObject();
        } else {
            cursor.MoveToVisualFirstForSameLayoutObject();
        }
        let same = line.Current().BaseDirection() == cursor.Current().ResolvedDirection();
        let caret = if cursor.Current().IsText() {
            let offset = if same == after {
                cursor.Current().TextEndOffset()
            } else {
                cursor.Current().TextStartOffset()
            };
            InlineCaretPosition {
                cursor,
                position_type: InlineCaretPositionType::AtTextOffset,
                text_offset: Some(offset),
            }
        } else {
            let after = if cursor.Current().IsAtomicInline() {
                same == after
            } else {
                after
            };
            InlineCaretPosition {
                cursor,
                position_type: if after {
                    InlineCaretPositionType::AfterBox
                } else {
                    InlineCaretPositionType::BeforeBox
                },
                text_offset: None,
            }
        };
        return Some(AdjustResolved(caret, options.bidi_caret_affinity));
    }
    let context = unsafe { object.FragmentItemsContainer().as_ref() }?;
    let mapping = unsafe { OffsetMapping::GetForLayoutObject(object).as_ref() }?;
    let offset = mapping
        .GetTextContentOffsetForNode(node, &position.position)
        .unwrap_or(0);
    let units = mapping.GetText().Span16().unwrap_or_default();
    let adjusted_offset = if position.affinity == TextAffinity::Upstream
        && offset != 0
        && units.get((offset - 1) as usize) == Some(&0x200b)
    {
        offset - 1
    } else {
        offset
    };
    let layout_text =
        if node.IsTextNode() && position.position.anchor == PositionAnchorType::OffsetInAnchor {
            let offset = i32::try_from(position.position.offset).ok()?;
            unsafe { AssociatedLayoutObjectOfDefault(node, offset).as_ref() }
        } else {
            None
        };
    ComputeInlineCaretPosition(
        context,
        adjusted_offset,
        position.affinity,
        layout_text,
        options,
    )
}

fn PositionForPointInFragments(
    box_object: &crate::internal::layout_box::LayoutBox,
    point: &foundation::PhysicalOffset,
    options: crate::EditingPositionOptions,
) -> PositionWithAffinity {
    use foundation::{LayoutUnit, PhysicalRect};
    use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
    let count = box_object.PhysicalFragmentCount();
    if count == 1 {
        return unsafe { &*box_object.GetPhysicalFragment(0) }
            .EditingPositionForPoint(point, options);
    }
    let mut closest = None;
    let mut shortest = LayoutUnit::Max();
    for index in 0..count {
        let fragment = unsafe { &*box_object.GetPhysicalFragment(index) };
        let offset = fragment.OffsetFromOwnerLayoutBox();
        let distance = PhysicalRect::new(offset, fragment.Size()).DistanceAsSize(*point);
        if distance.IsZero() {
            return fragment.EditingPositionForPoint(&(*point - offset), options);
        }
        let squared = distance.width * distance.width + distance.height * distance.height;
        if closest.is_none() || squared < shortest {
            shortest = squared;
            closest = Some((fragment, offset));
        }
    }
    if let Some((fragment, offset)) = closest {
        return fragment.EditingPositionForPoint(&(*point - offset), options);
    }

    box_object.CreatePositionWithAffinityDefault(0)
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxPositionForPointInFragmentsProvider(
    box_object: &crate::internal::layout_box::LayoutBox,
    point: &foundation::PhysicalOffset,
) -> PositionWithAffinity {
    PositionForPointInFragments(box_object, point, crate::EditingPositionOptions::default())
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextPositionForPoint(
    text: &crate::internal::layout_text::LayoutText,
    point: &foundation::PhysicalOffset,
) -> PositionWithAffinity {
    PositionForPoint(text, point, crate::EditingPositionOptions::default())
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlinePositionForPoint(
    inline: &crate::internal::layout_inline::LayoutInline,
    point: &foundation::PhysicalOffset,
) -> PositionWithAffinity {
    PositionForPoint(inline, point, crate::EditingPositionOptions::default())
}

/// Source: core/layout/inline/line_utils.cc:12-24.
pub(crate) fn ContainingLine(
    cursor: &InlineCursor,
    options: crate::EditingPositionOptions,
) -> InlineCursor {
    let mut line = cursor.clone();
    line.MoveToContainingLine();
    if options.editing_line_movement_with_ruby_fix
        && line.IsNotNull()
        && line.Current().IsRubyAnnotationLine()
    {
        line.MoveToPreviousLine();
    }
    line
}
