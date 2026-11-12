// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Compatibility exports; algorithms live with their Chromium source owner.
pub use super::composite_edit_command::{BreakOutOfEmptyListItem, InsertNodeAt};
pub(super) use super::composite_edit_command::{
    DeleteInsignificant as DeleteInsignificantText, MoveRemainingSiblings,
    RebalanceWhitespaceAt as RebalanceWhitespace, SplitTreeToNode,
};
pub use super::delete_selection_command::DeleteSelectionCommand;
pub use super::insert_line_break_command::InsertLineBreak;
pub use super::insert_paragraph_separator_command::InsertParagraphSeparatorCommand;
pub use super::structural_editing_context::{
    StructuralBooleanQuery, StructuralEditingContext, StructuralNodeFacts, StructuralPositionQuery,
};
pub(super) use crate::core::editing::{
    commands::structural_editing_support::Ancestor, visible_units_paragraph::IsParagraphEdge,
};
