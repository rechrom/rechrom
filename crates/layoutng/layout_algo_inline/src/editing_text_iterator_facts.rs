//! Source TextIterator's predicates on the current resident LayoutObject.
//! These values are consumed immediately; no style/geometry model is retained.
use super::*;
use foundation::LayoutUnit;
unsafe extern "Rust" {
    // Implemented by layout_algo_table's owning LayoutTable/Cell/Row classes.
    // Assembly cannot depend backward on that package's internal objects.
    fn LayoutTextIteratorTableFacts(object: *const LayoutObject) -> (bool, bool, bool);
}
pub struct EditingTextIteratorObjectFacts {
    pub visible: bool,
    pub selectable: bool,
    pub is_text: bool,
    pub is_box: bool,
    pub is_scroll_container: bool,
    pub is_layout_view: bool,
    pub size_empty: bool,
    pub out_of_flow: bool,
    pub image: bool,
    pub embedded: bool,
    pub text_control: bool,
    pub br: bool,
    pub hr: bool,
    pub table: bool,
    pub table_cell: bool,
    pub table_row: bool,
    pub inline: bool,
    pub block: bool,
    pub block_flow: bool,
    pub floating_or_out_of_flow: bool,
    pub body: bool,
    pub has_height: bool,
    pub table_exists: bool,
    pub table_first_cell: bool,
    pub table_inline: bool,
}
pub struct EditingParagraphTextFacts {
    pub transformed_text: foundation::String,
    pub preserve_breaks: bool,
    pub resolved_text_length: usize,
    pub caret_max_offset: usize,
    pub text_start_offset: usize,
    pub has_inline_fragments: bool,
}
/// One borrowed LayoutText read for SimplifiedBackwardsTextIterator. The text
/// shares the native transformed String; the style input is one source enum.
pub struct EditingBackwardsTextFacts {
    pub transformed_text: foundation::String,
    pub caret_max_offset: usize,
    pub text_start_offset: usize,
    pub has_inline_fragments: bool,
    pub is_secure: bool,
    pub is_text_fragment: bool,
    pub is_remaining_text: bool,
    pub white_space_collapse: layoutng_style::css::white_space::WhiteSpaceCollapse,
}
pub struct EditingParagraphLayoutFacts {
    pub is_table_structure_node: bool,
    pub can_have_children: bool,
    pub is_list_item: bool,
    pub is_floating: bool,
    pub display_table: bool,
    pub content_box_height_is_zero: bool,
    pub is_text: bool,
    pub table_cell: bool,
    pub display_inline_type: bool,
    pub preserve_white_spaces: bool,
    pub collapse_breaks: bool,
    pub visible: bool,
    pub br: bool,
    pub selectable: bool,
    pub user_select_all: bool,
    pub text: Option<EditingParagraphTextFacts>,
    pub is_enclosing_block: bool,
    pub is_rendered_as_non_inline_table_image_or_hr: bool,
}
impl LayoutNode<'_> {
    pub fn EditingBackwardsTextFacts(
        &self,
        first_letter: bool,
    ) -> Option<EditingBackwardsTextFacts> {
        use crate::{internal::layout_text::LayoutText, layout_text_fragment::LayoutTextFragment};
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let original = unsafe { DynamicTo::<LayoutText>(self.object).as_ref() }?;
        let fragment = unsafe { DynamicTo::<LayoutTextFragment>(original as *const _).as_ref() };
        let text = if first_letter {
            let fragment = fragment.expect("source remaining first-letter fragment");
            assert!(fragment.IsRemainingTextLayoutObject());
            let pseudo = unsafe { fragment.GetFirstLetterPseudoElement().as_ref() }
                .expect("source first-letter pseudo owner");
            let layout = unsafe { pseudo.GetLayoutObject().as_ref() }
                .expect("source first-letter LayoutObject");
            unsafe { DynamicTo::<LayoutText>(layout.SlowFirstChild()).as_ref() }
                .expect("source first-letter LayoutText")
        } else {
            original
        };
        let selected_fragment =
            unsafe { DynamicTo::<LayoutTextFragment>(text as *const _).as_ref() };
        Some(EditingBackwardsTextFacts {
            transformed_text: text.TransformedText().clone(),
            caret_max_offset: text.CaretMaxOffset(),
            text_start_offset: text.TextStartOffset() as usize,
            has_inline_fragments: text.HasInlineFragments(),
            is_secure: text.IsSecure(),
            is_text_fragment: selected_fragment.is_some(),
            is_remaining_text: selected_fragment
                .is_some_and(|fragment| fragment.IsRemainingTextLayoutObject()),
            white_space_collapse: text.StyleRef().GetWhiteSpaceCollapse(),
        })
    }
    /// Source simplified_backwards_text_iterator.cc:66-93. Return the two
    /// original mapping offsets; the iterator owns their subtraction/fallback.
    pub fn EditingSecureTextContentOffsets(&self, offset: usize) -> Option<(usize, usize)> {
        use crate::{internal::layout_text::LayoutText, offset_mapping::OffsetMapping};
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let node = unsafe { self.object.NonPseudoNode().as_ref() }?;
        let text = unsafe { DynamicTo::<LayoutText>(self.object).as_ref() }?;
        if !node.IsTextNode() || !text.IsSecure() {
            return None;
        }
        let position = crate::internal::editing::forward::Position::new(node.InputId(), offset);
        let mapping = unsafe { OffsetMapping::GetForPosition(&position).as_ref() }?;
        let start = crate::internal::editing::forward::Position::new(node.InputId(), 0);
        Some((
            mapping.GetTextContentOffsetForNode(node, &position)? as usize,
            mapping.GetTextContentOffsetForNode(node, &start)? as usize,
        ))
    }
    // Source: selection_adjuster.cc:64-105. The caller follows source DOM
    // ancestors only while there is no LayoutObject; the rest is native layout.
    pub fn IsEmptyTableCellForSelection(&self) -> bool {
        let mut object = self.object;
        if object.IsBR() {
            let Some(parent) = (unsafe { object.Parent().as_ref() }) else {
                return false;
            };
            object = parent;
        }
        if !object.IsTableCell() {
            return false;
        }
        let Some(child) = (unsafe { object.SlowFirstChild().as_ref() }) else {
            return true;
        };
        child.IsBR() && child.NextSibling().is_null()
    }
    pub fn EditingParagraphLayoutFacts(&self) -> EditingParagraphLayoutFacts {
        use crate::internal::layout_text::LayoutText;
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let object = self.object;
        let text = unsafe { DynamicTo::<LayoutText>(object as *const _).as_ref() }.map(|text| {
            EditingParagraphTextFacts {
                transformed_text: text.TransformedText().clone(),
                preserve_breaks: text.StyleRef().ShouldPreserveBreaks(),
                resolved_text_length: text.ResolvedTextLength() as usize,
                caret_max_offset: text.CaretMaxOffset(),
                text_start_offset: text.TextStartOffset() as usize,
                has_inline_fragments: text.HasInlineFragments(),
            }
        });
        EditingParagraphLayoutFacts {
            is_table_structure_node: object.IsTableCell()
                || object.IsTableRow()
                || object.IsTableSection()
                || object.IsLayoutTableCol(),
            can_have_children: object.CanHaveChildren(),
            is_list_item: object.IsListItem(),
            is_floating: object.IsFloating(),
            display_table: object.StyleRef().IsDisplayTable(),
            // Source: delete_selection_command.cc:620,780. This is the
            // physical content-box height, irrespective of writing mode.
            content_box_height_is_zero: unsafe {
                DynamicTo::<LayoutBox>(object as *const _).as_ref()
            }
            .is_some_and(|box_| box_.PhysicalContentBoxRect().size.height == LayoutUnit::default()),
            is_text: object.IsText(),
            table_cell: object.IsTableCell(),
            display_inline_type: object.StyleRef().IsDisplayInlineType(),
            preserve_white_spaces: object.StyleRef().ShouldPreserveWhiteSpaces(),
            collapse_breaks: object.StyleRef().ShouldCollapseBreaks(),
            visible: object.StyleRef().Visibility() == foundation::EVisibility::kVisible,
            br: object.IsBR(),
            selectable: object.IsSelectable(),
            user_select_all: object.StyleRef().UsedUserSelect() == foundation::EUserSelect::kAll,
            text,
            is_enclosing_block: !object.IsInline(),
            is_rendered_as_non_inline_table_image_or_hr: !object.IsInline()
                && (object.IsTable() || object.IsImage() || object.IsHR()),
        }
    }
    // Source: text_iterator.cc:628-741,742-811;
    // fully_clipped_state_stack.cc:18-36.
    pub fn EditingTextIteratorObjectFacts(&self) -> EditingTextIteratorObjectFacts {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let object = self.object;
        let box_ = unsafe { DynamicTo::<LayoutBox>(object as *const _).as_ref() };
        let (table_exists, table_first_cell, table_inline) =
            if object.IsTableCell() || object.IsTableRow() {
                unsafe { LayoutTextIteratorTableFacts(object) }
            } else {
                (false, false, false)
            };
        EditingTextIteratorObjectFacts {
            visible: object.StyleRef().Visibility() == foundation::EVisibility::kVisible,
            selectable: object.IsSelectable(),
            is_text: object.IsText(),
            is_box: object.IsBox(),
            is_scroll_container: object.IsScrollContainer(),
            is_layout_view: object.IsLayoutView(),
            size_empty: box_.is_some_and(|box_| box_.StitchedSize().IsEmpty()),
            out_of_flow: object.StyleRef().HasOutOfFlowPosition(),
            image: object.IsImage(),
            embedded: object.IsLayoutEmbeddedContent(),
            text_control: object.IsTextControl(),
            br: object.IsBR(),
            hr: object.IsHR(),
            table: object.IsTable(),
            table_cell: object.IsTableCell(),
            table_row: object.IsTableRow(),
            inline: object.IsInline(),
            block: object.IsLayoutBlock(),
            block_flow: object.IsLayoutBlockFlow(),
            floating_or_out_of_flow: object.IsFloatingOrOutOfFlowPositioned(),
            body: object.IsBody(),
            has_height: box_
                .is_some_and(|box_| box_.StitchedSize().height != LayoutUnit::default()),
            table_exists,
            table_first_cell,
            table_inline,
        }
    }
}

/// Source MoveCommands::VerticalScrollDistance, move_commands.cc:74-90.
/// Only the actual LayoutBox supplies this physical padding-box scalar.
#[derive(Clone, Copy, Debug)]
pub struct EditingMovePageFacts {
    pub padding_box_height: i32,
    pub overflow_y: foundation::EOverflow,
}
impl LayoutNode<'_> {
    pub fn EditingMovePageFacts(&self) -> Option<EditingMovePageFacts> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object as *const _).as_ref() }?;
        Some(EditingMovePageFacts {
            padding_box_height: box_.PhysicalPaddingBoxRect().size.height.ToInt(),
            overflow_y: box_.StyleRef().OverflowY(),
        })
    }
}

/// Source composite_edit_command.cc:1072-1098. Facts are sampled on this actual
/// LayoutBlockFlow, including layout children rather than DOM child presence.
#[derive(Clone, Copy, Debug)]
pub struct EditingBlockFlowPlaceholderFacts {
    pub stitched_height_is_zero: bool,
    pub is_layout_list_item: bool,
    pub has_layout_child: bool,
}
impl LayoutNode<'_> {
    pub fn EditingBlockFlowPlaceholderFacts(&self) -> Option<EditingBlockFlowPlaceholderFacts> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let block = unsafe {
            DynamicTo::<crate::internal::layout_block_flow::LayoutBlockFlow>(
                self.object as *const _,
            )
            .as_ref()
        }?;
        Some(EditingBlockFlowPlaceholderFacts {
            stitched_height_is_zero: block.StitchedSize().height == LayoutUnit::default(),
            is_layout_list_item: block.IsLayoutListItem(),
            has_layout_child: !block.FirstChild().is_null(),
        })
    }
}

/// Source typing_command.cc:179-193, facts of the actual BR/anonymous parent.
#[derive(Clone, Copy, Debug)]
pub struct EditingAnonymousPlaceholderFacts {
    pub is_br: bool,
    pub has_layout_sibling: bool,
    pub parent_is_anonymous: bool,
    pub previous_parent_non_pseudo_node: Option<u64>,
}
impl LayoutNode<'_> {
    pub fn EditingAnonymousPlaceholderFacts(&self) -> EditingAnonymousPlaceholderFacts {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let object = self.object;
        let parent = unsafe { object.Parent().as_ref() };
        let previous = parent.and_then(|parent| unsafe { parent.PreviousSibling().as_ref() });
        EditingAnonymousPlaceholderFacts {
            is_br: object.IsBR(),
            has_layout_sibling: !object.NextSibling().is_null()
                || !object.PreviousSibling().is_null(),
            parent_is_anonymous: parent.is_some_and(|parent| parent.IsAnonymous()),
            previous_parent_non_pseudo_node: previous
                .and_then(|previous| unsafe { previous.NonPseudoNode().as_ref() })
                .map(|node| node.InputId()),
        }
    }
}
