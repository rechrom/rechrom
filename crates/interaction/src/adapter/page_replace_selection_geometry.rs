//! Scalar resident leaves for ReplaceSelectionCommand and smart replacement.
use super::*;
pub(super) fn CharacterCodePointAtVisibleBoundary(
    document: &Document,
    layout: LayoutOutput<'_>,
    position: EditingPositionWithAffinity,
    before: bool,
) -> u32 {
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let mut visible = VisiblePosition::Create(&context, SourcePosition(position));
    if before {
        visible = visible_navigation::PreviousPositionOf(
            &context,
            visible.DeepEquivalent(),
            Rule::CanCross,
        );
    }
    let canonical = visible_units::MostForwardCaretPosition(
        &context,
        visible.DeepEquivalent(),
        Rule::CannotCross,
        SnapToClient::Editing,
    );
    if !canonical.IsOffsetInAnchor() {
        return 0;
    }
    let Some(node) = canonical
        .ComputeContainerNode(&context)
        .filter(|n| context.IsText(n))
    else {
        return 0;
    };
    let Some(index) = context.Index(&node) else {
        return 0;
    };
    let text = document.CharacterDataUTF16Slice(index);
    let offset = canonical.ComputeOffsetInContainerNode(&context);
    let Some(&first) = text.get(offset) else {
        return 0;
    };
    // WTF String::CodePointAtOrZero. The normalization wrapper narrows to UChar;
    // smart replacement retains this complete code point. Lone units survive.
    if (0xD800..=0xDBFF).contains(&first) {
        if let Some(&second) = text
            .get(offset + 1)
            .filter(|n| (0xDC00..=0xDFFF).contains(*n))
        {
            return 0x10000u32 + (((first as u32 - 0xD800) << 10) | (second as u32 - 0xDC00));
        }
    }
    u32::from(first)
}

pub(super) fn CharacterAtVisibleBoundary(
    document: &Document,
    layout: LayoutOutput<'_>,
    position: EditingPositionWithAffinity,
    before: bool,
) -> u16 {
    CharacterCodePointAtVisibleBoundary(document, layout, position, before) as u16
}
