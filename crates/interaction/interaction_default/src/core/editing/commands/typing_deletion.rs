//! Compatibility imports for the source-owned typing command implementation.
pub use super::editing_commands_utilities::DeletionInputTypeFromTextGranularity;
pub use super::typing_command::{
    AfterBlockIfBeforeAnonymousPlaceholder, AnonymousPlaceholderContext, AnonymousPlaceholderFacts,
    BackwardDeletionGeometry, BackwardDeletionNeedsEmptyRoot, BeginBackwardDeletionGeometry,
    DeletionGeometryAction, FinishBackwardDeletionGeometry, ForwardDeletionGeometry,
    ModifierForDeletion, TypingDeletionContext, TypingDeletionGeometryContext,
};
