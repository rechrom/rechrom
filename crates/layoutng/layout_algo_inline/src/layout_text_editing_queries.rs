//! Source: core/layout/layout_text.cc:1166-1353;
//! layout_text_fragment.cc:245-283; layout_br.cc:51-54,66-84;
//! layout_word_break.cc:42-63. Offsets retain DOM/fragment ownership.
use super::*;
use crate::internal::editing::forward::{Position, PositionAnchorType};
use crate::layout_text_fragment::LayoutTextFragment;
use crate::offset_mapping::OffsetMapping;

fn Fragment(text: &LayoutText) -> Option<&LayoutTextFragment> {
    unsafe { DynamicTo::<LayoutTextFragment>(text as *const _).as_ref() }
}
fn AssociatedNode(text: &LayoutText) -> Option<&Node> {
    if let Some(fragment) = Fragment(text) {
        unsafe { fragment.AssociatedTextNode().cast::<Node>().as_ref() }
    } else {
        unsafe { text.GetNode().as_ref() }
    }
}
fn Mapping(text: &LayoutText) -> Option<&OffsetMapping> {
    unsafe { OffsetMapping::GetForLayoutObject(text as *const _ as *const LayoutObject).as_ref() }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextPositionForCaretOffset(text: &LayoutText, offset: u32) -> Position {
    if let Some(fragment) = Fragment(text) {
        return LayoutTextFragmentPositionForCaretOffset(fragment, offset);
    }
    let Some(node) = AssociatedNode(text) else {
        return Position::default();
    };
    if text.IsBR() || text.IsWordBreak() {
        debug_assert!(offset <= u32::from(text.IsBR()));
        return Position::WithAnchor(
            node.InputId(),
            0,
            if offset == 0 {
                PositionAnchorType::BeforeAnchor
            } else {
                PositionAnchorType::AfterAnchor
            },
        );
    }
    Position::new(
        node.InputId(),
        offset.min(text.OriginalTextLength()) as usize,
    )
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCaretOffsetForPosition(
    text: &LayoutText,
    position: &Position,
) -> Option<u32> {
    if let Some(fragment) = Fragment(text) {
        return LayoutTextFragmentCaretOffsetForPosition(fragment, position);
    }
    let node = AssociatedNode(text)?;
    if position.node_id != Some(node.InputId()) {
        return None;
    }
    if text.IsWordBreak() {
        return Some(0);
    }
    match position.anchor {
        PositionAnchorType::BeforeAnchor => Some(0),
        PositionAnchorType::AfterAnchor => Some(if text.IsBR() {
            1
        } else {
            text.OriginalTextLength()
        }),
        PositionAnchorType::OffsetInAnchor if !text.IsBR() => u32::try_from(position.offset).ok(),
        _ => None,
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextFragmentPositionForCaretOffset(
    fragment: &LayoutTextFragment,
    offset: u32,
) -> Position {
    let Some(node) = (unsafe { fragment.AssociatedTextNode().as_ref() }) else {
        return Position::default();
    };
    Position::new(
        node.InputId(),
        (fragment.Start() + offset.min(fragment.FragmentLength())) as usize,
    )
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextFragmentCaretOffsetForPosition(
    fragment: &LayoutTextFragment,
    position: &Position,
) -> Option<u32> {
    let node = unsafe { fragment.AssociatedTextNode().as_ref() }?;
    if position.node_id != Some(node.InputId()) {
        return None;
    }
    let offset = match position.anchor {
        PositionAnchorType::BeforeAnchor => 0,
        PositionAnchorType::AfterAnchor => fragment.Start() + fragment.FragmentLength(),
        PositionAnchorType::OffsetInAnchor => u32::try_from(position.offset).ok()?,
        _ => return None,
    };
    (fragment.Start() <= offset && offset <= fragment.Start() + fragment.FragmentLength())
        .then(|| offset - fragment.Start())
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextNonCollapsedCaretMaxOffset(text: &LayoutText) -> u32 {
    if text.IsBR() {
        return 1;
    }
    text.OriginalTextLength()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCaretMinOffset(text: &LayoutText) -> usize {
    if text.IsBR() {
        return 0;
    }
    let (Some(mapping), Some(node)) = (Mapping(text), AssociatedNode(text)) else {
        return 0;
    };
    let first = text.PositionForCaretOffset(0);
    if first.IsNull() {
        return 0;
    }
    let next = mapping.NonCollapsedBoundaryForNode(node, &first, true);
    match text.CaretOffsetForPosition(&next) {
        Some(candidate) if candidate != text.TransformedTextLength() => candidate as usize,
        _ => 0,
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCaretMaxOffset(text: &LayoutText) -> usize {
    if text.IsBR() {
        return 1;
    }
    let length = text.OriginalTextLength();
    let (Some(mapping), Some(node)) = (Mapping(text), AssociatedNode(text)) else {
        return length as usize;
    };
    let last = text.PositionForCaretOffset(length);
    if last.IsNull() {
        return length as usize;
    }
    let previous = mapping.NonCollapsedBoundaryForNode(node, &last, false);
    match text.CaretOffsetForPosition(&previous) {
        Some(candidate) if candidate != 0 => candidate as usize,
        _ => length as usize,
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextResolvedTextLength(text: &LayoutText) -> u32 {
    let (Some(mapping), Some(node)) = (Mapping(text), AssociatedNode(text)) else {
        return 0;
    };
    let start = text.PositionForCaretOffset(0);
    let end = text.PositionForCaretOffset(text.NonCollapsedCaretMaxOffset());
    let (Some(start), Some(end)) = (
        mapping.GetTextContentOffsetForNode(node, &start),
        mapping.GetTextContentOffsetForNode(node, &end),
    ) else {
        return 0;
    };
    debug_assert!(start <= end);
    end - start
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextHasNonCollapsedText(text: &LayoutText) -> bool {
    text.ResolvedTextLength() != 0
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextIsBeforeNonCollapsedCharacter(
    text: &LayoutText,
    offset: u32,
) -> bool {
    let (Some(mapping), Some(node)) = (Mapping(text), AssociatedNode(text)) else {
        return false;
    };
    if offset >= text.NonCollapsedCaretMaxOffset() {
        return false;
    }
    let position = text.PositionForCaretOffset(offset);
    !position.IsNull() && mapping.IsAdjacentToNonCollapsedContentForNode(node, &position, true)
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextIsAfterNonCollapsedCharacter(
    text: &LayoutText,
    offset: u32,
) -> bool {
    let (Some(mapping), Some(node)) = (Mapping(text), AssociatedNode(text)) else {
        return false;
    };
    if offset == 0 {
        return false;
    }
    let position = text.PositionForCaretOffset(offset);
    !position.IsNull() && mapping.IsAdjacentToNonCollapsedContentForNode(node, &position, false)
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextContainsCaretOffset(text: &LayoutText, offset: i32) -> bool {
    if offset < 0 || offset as u32 > text.NonCollapsedCaretMaxOffset() {
        return false;
    }
    let (Some(mapping), Some(node)) = (Mapping(text), AssociatedNode(text)) else {
        return false;
    };
    let position = text.PositionForCaretOffset(offset as u32);
    if position.IsNull() {
        return false;
    }
    if (offset as u32) < text.NonCollapsedCaretMaxOffset()
        && mapping.IsAdjacentToNonCollapsedContentForNode(node, &position, true)
    {
        return true;
    }
    if offset == 0 || !mapping.IsAdjacentToNonCollapsedContentForNode(node, &position, false) {
        return false;
    }
    mapping
        .GetTextContentOffsetForNode(node, &position)
        .is_some_and(|offset| offset != 0 && mapping.GetText().CodeUnitAt(offset - 1) != 0x0a)
}
