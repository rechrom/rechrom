//! Compatibility imports for the source-owned editing implementations.
pub use super::composite_edit_command::{
    PositionAvoidingSpecialElementBoundary, PositionOutsideTabSpan,
    PrepareWhitespaceAtPositionForSplit, RebalanceWhitespaceAt, RemovePlaceholderAt,
    RemovePreservingChildren,
};
pub use super::replace_selection_command::PositionAvoidingPrecedingNodes;
pub use crate::core::editing::editing_utilities::ComputePlaceholderToCollapseAt;
