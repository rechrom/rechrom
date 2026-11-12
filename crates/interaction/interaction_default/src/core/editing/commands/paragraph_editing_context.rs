// Copyright (C) 2005, 2006 Apple Computer, Inc. All rights reserved.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/commands/insert_paragraph_separator_command.cc:92-697.
//! Source owners used by the paragraph command. Handles retain live DOM identity;
//! visible positions borrow the current layout generation. No copied editing tree
//! or flattened text representation crosses this interface.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditingAborted;
pub type EditResult<T = ()> = Result<T, EditingAborted>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditingNamespace {
    Html,
    MathMl,
    Svg,
    Other,
}
pub use super::super::position::PositionAnchorType;

/// The existing DOM owner supplies identity-preserving node and position access.
/// Position must include its source anchor kind, not just a Range offset pair.
pub trait ParagraphDOM {
    type Node: Clone + PartialEq;
    type Position: Clone + PartialEq;
    type Affinity: Copy;
    fn Parent(&self, node: &Self::Node) -> Option<Self::Node>;
    fn ParentElement(&self, node: &Self::Node) -> Option<Self::Node>;
    fn NonShadowBoundaryParent(&self, node: &Self::Node) -> Option<Self::Node>;
    fn FirstChild(&self, node: &Self::Node) -> Option<Self::Node>;
    fn NextSibling(&self, node: &Self::Node) -> Option<Self::Node>;
    fn PreviousSibling(&self, node: &Self::Node) -> Option<Self::Node>;
    fn NextNode(&self, node: &Self::Node, stay_within: &Self::Node) -> Option<Self::Node>;
    fn ChildAt(&self, node: &Self::Node, offset: usize) -> Option<Self::Node>;
    fn IsElement(&self, node: &Self::Node) -> bool;
    fn IsLink(&self, node: &Self::Node) -> bool;
    fn IsText(&self, node: &Self::Node) -> bool;
    fn Namespace(&self, node: &Self::Node) -> EditingNamespace;
    fn HasTagName(&self, node: &Self::Node, namespace: EditingNamespace, local_name: &str) -> bool;
    fn HasAttributes(&self, node: &Self::Node) -> bool;
    fn IsDescendantOf(&self, node: &Self::Node, ancestor: &Self::Node) -> bool;
    fn TextLength(&self, node: &Self::Node) -> usize;
    fn TextCodeUnit(&self, node: &Self::Node, offset: usize) -> Option<u16>;
    fn AnchorNode(&self, position: &Self::Position) -> Self::Node;
    fn ContainerNode(&self, position: &Self::Position) -> Option<Self::Node>;
    fn AnchorType(&self, position: &Self::Position) -> PositionAnchorType;
    fn EditingOffset(&self, position: &Self::Position) -> usize;
    fn AtStartOfTree(&self, position: &Self::Position) -> bool;
    fn AtFirstEditingPosition(&self, position: &Self::Position) -> bool;
    fn AtLastEditingPosition(&self, position: &Self::Position) -> bool;
    fn ParentAnchoredEquivalent(&self, position: &Self::Position) -> Self::Position;
    fn FirstPositionInNode(&self, node: &Self::Node) -> Self::Position;
    fn LastPositionInNode(&self, node: &Self::Node) -> Self::Position;
    fn FirstPositionInOrBeforeNode(&self, node: &Self::Node) -> Self::Position;
    fn LastPositionInOrAfterNode(&self, node: &Self::Node) -> Self::Position;
    fn InParentBeforeNode(&self, node: &Self::Node) -> Self::Position;
    fn InParentAfterNode(&self, node: &Self::Node) -> Self::Position;
    fn OffsetInNode(&self, node: &Self::Node, offset: usize) -> Self::Position;
    fn NodeAfterPosition(&self, position: &Self::Position) -> Option<Self::Node>;
    fn PreviousCodeUnitPosition(&self, position: &Self::Position) -> Self::Position;
    fn HighestEditableRoot(&self, position: &Self::Position) -> Option<Self::Node>;
    fn RootEditableElement(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsEditable(&self, node: &Self::Node) -> bool;
    fn IsEditablePosition(&self, position: &Self::Position) -> bool;
    fn EditingIgnoresContent(&self, node: &Self::Node) -> bool;
    fn IsTabSpanText(&self, node: &Self::Node) -> bool;
    fn TabSpanElement(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsMailBlockquote(&self, node: &Self::Node) -> bool;
}

/// Geometry queries use the resident layout output, including white-space used
/// values and visually equivalent caret positions. They are not DOM heuristics.
pub trait ParagraphVisibleUnits: ParagraphDOM {
    type VisiblePosition;
    fn UpdateStyleAndLayout(&mut self);
    fn DefaultAffinity(&self) -> Self::Affinity;
    fn CreateVisiblePosition(
        &self,
        position: &Self::Position,
        affinity: Self::Affinity,
    ) -> Option<Self::VisiblePosition>;
    fn DeepEquivalent(&self, position: &Self::VisiblePosition) -> Self::Position;
    fn PreviousVisiblePosition(
        &self,
        position: &Self::VisiblePosition,
    ) -> Option<Self::VisiblePosition>;
    fn NextVisiblePosition(
        &self,
        position: &Self::VisiblePosition,
    ) -> Option<Self::VisiblePosition>;
    fn IsStartOfBlock(&self, position: &Self::VisiblePosition) -> bool;
    fn IsEndOfBlock(&self, position: &Self::VisiblePosition) -> bool;
    fn IsStartOfParagraph(&self, position: &Self::VisiblePosition) -> bool;
    fn IsEndOfParagraph(&self, position: &Self::VisiblePosition) -> bool;
    fn IsStartOfLine(&self, position: &Self::VisiblePosition) -> bool;
    fn EnclosingBlock(&self, node: &Self::Node) -> Option<Self::Node>;
    fn EnclosingBlockFlowElement(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsEnclosingBlock(&self, node: &Self::Node) -> bool;
    fn IsDisplayInlineType(&self, node: &Self::Node) -> bool;
    fn IsTableCell(&self, node: &Self::Node) -> bool;
    fn IsDisplayInsideTable(&self, node: &Self::Node) -> bool;
    fn IsLayoutBR(&self, node: &Self::Node) -> bool;
    fn ShouldPreserveWhiteSpaces(&self, node: &Self::Node) -> bool;
    fn CaretMinOffset(&self, node: &Self::Node) -> usize;
    fn CaretMaxOffset(&self, node: &Self::Node) -> usize;
    fn MostBackwardCaretPosition(&self, position: &Self::Position) -> Self::Position;
    fn MostForwardCaretPosition(&self, position: &Self::Position) -> Self::Position;
    fn IsVisuallyEquivalentCandidate(&self, position: &Self::Position) -> bool;
    fn RendersInDifferentPosition(&self, first: &Self::Position, second: &Self::Position) -> bool;
    fn ComparePositions(
        &self,
        first: &Self::VisiblePosition,
        second: &Self::VisiblePosition,
    ) -> std::cmp::Ordering;
    fn LineBreakExistsAtPosition(&self, position: &Self::Position) -> bool;
    fn IsRenderedCharacter(&self, position: &Self::Position) -> bool;
}

/// The CompositeEditCommand owner executes each elementary command immediately,
/// retaining its undo item and honoring synchronous DOM reentry before return.
/// The paragraph algorithm owns ordering; the embedding must not batch commands.
pub trait ParagraphComposite: ParagraphVisibleUnits {
    fn EndingSelectionIsValid(&self) -> bool;
    fn EndingSelectionIsRange(&self) -> bool;
    fn EndingSelectionStart(&self) -> Option<Self::Position>;
    fn EndingVisibleSelectionStart(&self) -> Option<Self::Position>;
    fn EndingSelectionAffinity(&self) -> Self::Affinity;
    fn DeleteSelectionNormal(&mut self) -> EditResult;
    fn InsertLineBreak(&mut self) -> EditResult;
    fn CreateDefaultParagraphElement(&mut self) -> Self::Node;
    fn CreateBR(&mut self) -> Self::Node;
    fn CreateLI(&mut self) -> Self::Node;
    fn CloneWithoutChildren(&mut self, node: &Self::Node) -> Self::Node;
    fn RemoveIdAttribute(&mut self, node: &Self::Node);
    fn AppendNode(&mut self, node: &Self::Node, parent: &Self::Node) -> EditResult;
    fn InsertNodeBefore(&mut self, node: &Self::Node, reference: &Self::Node) -> EditResult;
    fn InsertNodeAfter(&mut self, node: &Self::Node, reference: &Self::Node) -> EditResult;
    fn InsertNodeAt(&mut self, node: &Self::Node, position: &Self::Position) -> EditResult;
    fn RemoveNode(&mut self, node: &Self::Node) -> EditResult;
    fn RemoveNodePreservingChildren(&mut self, node: &Self::Node) -> EditResult;
    fn SplitElement(&mut self, element: &Self::Node, at_child: &Self::Node) -> EditResult;
    /// SplitTextNode keeps the original text node as the right half.
    fn SplitTextNode(&mut self, node: &Self::Node, offset: usize) -> EditResult;
    fn SplitTextNodeContainingElement(&mut self, node: &Self::Node, offset: usize) -> EditResult;
    fn PushAnchorElementDown(&mut self, anchor: &Self::Node) -> EditResult;
    fn AppendBlockPlaceholder(&mut self, block: &Self::Node) -> EditResult;
    fn ReplaceTextInNode(
        &mut self,
        text: &Self::Node,
        offset: usize,
        count: usize,
        data: &[u16],
    ) -> EditResult;
    fn InsertTextIntoNode(&mut self, text: &Self::Node, offset: usize, data: &[u16]) -> EditResult;
    fn DeleteInsignificantText(
        &mut self,
        start: &Self::Position,
        end: &Self::Position,
    ) -> EditResult;
    /// Updates both SelectionForUndoStep and the DOM position API ending owner.
    fn SetEndingSelection(&mut self, position: &Self::Position);
    fn EditingUseDomPositionApiEnabled(&self) -> bool;
    fn PositionOutsideTabSpanCheckSiblingNodeEnabled(&self) -> bool;
}

/// EditingStyle owns the same style declaration and typing-style merge as Blink.
/// It is independent of ParagraphDOM and never projects a complete layout style.
pub trait ParagraphEditingStyle: ParagraphComposite {
    type Style;
    fn CreateEditingStyle(&self, position: &Self::Position) -> EditResult<Self::Style>;
    fn MergeTypingStyle(&self, style: &mut Self::Style);
    fn PrepareStyleToApplyAt(
        &self,
        style: &mut Self::Style,
        position: &Self::Position,
    ) -> EditResult;
    fn IsStyleEmpty(&self, style: &Self::Style) -> bool;
    fn ApplyStyle(&mut self, style: &Self::Style) -> EditResult;
}
