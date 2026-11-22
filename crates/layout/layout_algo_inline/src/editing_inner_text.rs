//! ElementInnerTextCollector's resident LayoutText/FragmentItems queries.
//! Source element_inner_text.cc:161-211,219-289,451-468. No text geometry is
//! reconstructed from DOM offsets; each run borrows the actual shaped owner.
use super::*;
use crate::internal::layout_text::LayoutText;
use crate::layout_text_fragment::LayoutTextFragment;
use crate::offset_mapping::OffsetMapping;
use foundation::{DynamicTo, EDisplay, EVisibility, String};
use layout_fragment::inline_cursor::InlineCursor;
unsafe extern "Rust" {
    fn LayoutInnerTextTableRowNeedsNewline(object: *const LayoutObject) -> bool;
}
#[derive(Clone, Copy)]
pub struct InnerTextLayoutFacts {
    pub visible: bool,
    pub block_level: bool,
    pub table_cell: bool,
    pub table_row: bool,
    pub cell_has_next: bool,
    pub row_needs_newline: bool,
}
impl LayoutNode<'_> {
    pub fn InnerTextLayoutFacts(&self) -> InnerTextLayoutFacts {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let object = self.object;
        InnerTextLayoutFacts {
            visible: object.StyleRef().Visibility() == EVisibility::kVisible,
            block_level: !object.IsInline() && !object.IsTableSection(),
            table_cell: object.StyleRef().Display() == EDisplay::kTableCell,
            table_row: object.StyleRef().Display() == EDisplay::kTableRow,
            cell_has_next: object.IsTableCell() && !object.NextSibling().is_null(),
            row_needs_newline: object.IsTableRow()
                && unsafe { LayoutInnerTextTableRowNeedsNewline(object) },
        }
    }
    /// Each callback is the source Result::EmitText(StringView). String clones
    /// retain native immutable text storage; emitted slices preserve UTF-16.
    pub fn ForEachInnerTextRun(&self, emit: &mut dyn FnMut(&String, u32, u32)) {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let Some(text) = (unsafe { DynamicTo::<LayoutText>(self.object as *const _).as_ref() })
        else {
            return;
        };
        fn process(text: &LayoutText, start: u32, emit: &mut dyn FnMut(&String, u32, u32)) {
            if text.HasEmptyText() || text.StyleRef().Visibility() != EVisibility::kVisible {
                return;
            }
            let mut first_line_offset = 0;
            if text.StyleRef().TextTransform() != text.FirstLineStyleRef().TextTransform() {
                let block = OffsetMapping::GetInlineFormattingContextOfObject(text);
                assert!(
                    !block.is_null(),
                    "source rendered first-line text has inline container"
                );
                let mut cursor = InlineCursor::from_container(unsafe { &*block });
                while cursor.IsNotNull() && cursor.Current().UsesFirstLineStyle() {
                    if unsafe { &*cursor.CurrentItem() }.IsText()
                        && std::ptr::eq(
                            cursor.Current().GetLayoutObject(),
                            text as *const _ as *const LayoutObject,
                        )
                    {
                        let run = cursor.Current().Text(&cursor).ToString();
                        emit(&run, 0, run.length());
                        first_line_offset = cursor.Current().TextEndOffset();
                    }
                    cursor.MoveToNext();
                }
            }
            let adjusted = if first_line_offset != 0 {
                first_line_offset
            } else {
                start
            };
            let plain = text.PlainText();
            if adjusted < plain.length() {
                emit(&plain, adjusted, plain.length());
            }
        }
        // GetFirstLetterPart is virtual; the base LayoutText implementation is
        // null, while the remaining-text LayoutTextFragment owns its real part.
        let first = unsafe { DynamicTo::<LayoutTextFragment>(text as *const _).as_ref() }
            .and_then(|fragment| unsafe { fragment.GetFirstLetterPart().as_ref() });
        if let Some(first) = first {
            if text.HasEmptyText()
                || OffsetMapping::GetInlineFormattingContextOfObject(text)
                    != OffsetMapping::GetInlineFormattingContextOfObject(first)
            {
                process(first, 0, emit);
                process(text, first.PlainText().length(), emit);
                return;
            }
        }
        process(text, 0, emit);
    }
}
