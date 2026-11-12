// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Live host inputs for source-owned structural editing commands.
//! Interfaces retain native node and position identity.
use super::paragraph_editing_context::EditResult;
use crate::core::editing::position::{Position, PositionDOM};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub enum StructuralPositionQuery {
    Visible,
    Backward,
    Forward,
    Previous,
    PreviousWithinEditable,
    PreviousCandidate,
    Next,
    NextWithinEditable,
    VisibleUpstream,
    ParagraphStart,
    ParagraphEnd,
    BlockStart,
    BlockEnd,
}
#[derive(Clone, Copy)]
pub enum StructuralBooleanQuery {
    RenderedCharacter,
    VisuallyEquivalentCandidate,
    RendersInDifferentPosition,
    StartOfLine,
    LineBreakExists,
}
#[derive(Clone, Copy, Default)]
pub struct StructuralNodeFacts {
    pub has_layout: bool,
    pub special_element: bool,
    pub richly_editable: bool,
    pub insignificant_text: bool,
    pub block: bool,
    pub block_flow: bool,
    pub list_item: bool,
    pub inline: bool,
    pub table_cell: bool,
    pub table_structure: bool,
    pub br: bool,
    pub atomic_block: bool,
    pub preserve_breaks: bool,
    pub preserve_spaces: bool,
    pub caret_min: usize,
    pub caret_max: usize,
    pub visible: bool,
}
pub trait StructuralEditingContext: PositionDOM<Node = u64> {
    type Tracked;
    type Style: Clone;
    fn CaptureStyle(&self, position: &Position<u64>, in_effect: bool) -> EditResult<Self::Style>;
    fn MergeTypingStyle(&self, style: Self::Style) -> EditResult<Self::Style>;
    fn PrepareStyle(
        &self,
        style: &Self::Style,
        position: &Position<u64>,
    ) -> EditResult<Option<Self::Style>>;
    fn RemoveStyleAddedByElement(
        &self,
        style: &Self::Style,
        element: u64,
    ) -> EditResult<Self::Style>;
    fn SetTypingStyle(&self, style: Option<Self::Style>);
    fn ApplyStyle(
        &self,
        style: &Self::Style,
        position: &Position<u64>,
    ) -> EditResult<Position<u64>>;
    fn AttributeEquals(&self, node: u64, name: &str, value: &str) -> bool;
    fn IsElement(&self, node: u64) -> bool;
    fn PositionBoolean(
        &self,
        position: &Position<u64>,
        other: Option<&Position<u64>>,
        query: StructuralBooleanQuery,
    ) -> bool;
    fn TextCodeUnit(&self, node: u64, offset: usize) -> Option<u16>;
    fn OriginalPlainText(&self, start: &Position<u64>, end: &Position<u64>)
        -> EditResult<Vec<u16>>;
    fn ReplaceText(&self, node: u64, offset: usize, count: usize, text: &[u16]) -> EditResult;
    fn SetStartingSelection(&self, start: &Position<u64>, end: &Position<u64>);
    fn UpdateStyleAndLayout(&self);
    fn QueryPosition(
        &self,
        position: &Position<u64>,
        query: StructuralPositionQuery,
    ) -> Position<u64>;
    fn NodeFacts(&self, node: u64) -> StructuralNodeFacts;
    fn EnclosingBlock(&self, position: &Position<u64>) -> Option<u64>;
    fn RootEditable(&self, node: u64) -> Option<u64>;
    fn Editable(&self, node: u64) -> bool;
    fn Tag(&self, node: u64, tag: &str) -> bool;
    fn HasAttributes(&self, node: u64) -> bool;
    fn Compare(&self, a: &Position<u64>, b: &Position<u64>) -> Option<Ordering>;
    fn Track(&self, position: &Position<u64>) -> EditResult<Self::Tracked>;
    fn Tracked(&self, position: &Self::Tracked) -> Position<u64>;
    fn CreateElement(&self, tag: &str) -> EditResult<u64>;
    fn CreateDefaultParagraphElement(&self) -> EditResult<u64>;
    fn CloneWithoutChildren(&self, node: u64) -> EditResult<u64>;
    fn RemoveId(&self, node: u64);
    fn Append(&self, node: u64, parent: u64) -> EditResult;
    fn InsertBefore(&self, node: u64, reference: u64) -> EditResult;
    fn Remove(&self, node: u64) -> EditResult;
    fn SplitText(&self, right: u64, offset: usize) -> EditResult;
    fn SplitElement(&self, right: u64, at: u64) -> EditResult;
    fn DeleteText(&self, node: u64, offset: usize, count: usize) -> EditResult;
    fn InsertText(&self, node: u64, offset: usize, text: &[u16]) -> EditResult;
    fn SetEndingPosition(&self, position: &Position<u64>);
}
