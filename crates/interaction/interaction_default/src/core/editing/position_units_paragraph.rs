/*
 * Copyright (C) 2004, 2005, 2006, 2007, 2008, 2009 Apple Inc. All rights
 * reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/position_units_paragraph.cc:41-296.
use super::{
    position::{Position, PositionAnchorType, PositionDOM, PositionTree},
    text_offset_mapping::{NextNode, NextSkippingChildren},
    visible_units::EditingBoundaryCrossingRule as Rule,
};
use foundation::String;
pub struct ParagraphTextFacts {
    pub transformed_text: String,
    pub preserve_breaks: bool,
    pub resolved_text_length: usize,
    pub caret_max_offset: usize,
    pub text_start_offset: usize,
}
pub struct ParagraphLayoutFacts {
    pub visible: bool,
    pub br: bool,
    pub selectable: bool,
    pub user_select_all: bool,
    pub text: Option<ParagraphTextFacts>,
}
pub trait ParagraphContext: PositionDOM {
    fn ParagraphLayout(&self, node: &Self::Node) -> Option<ParagraphLayoutFacts>;
    fn IsRenderedAsNonInlineTableImageOrHR(&self, node: &Self::Node) -> bool;
    fn EnclosingBlock(&self, position: &Position<Self::Node>, rule: Rule) -> Option<Self::Node>;
    fn HighestEditableRoot(&self, position: &Position<Self::Node>) -> Option<Self::Node>;
    fn IsEditable(&self, node: &Self::Node) -> bool;
    fn IsEnclosingBlock(&self, node: &Self::Node) -> bool;
    fn IsDOMDescendantOf(&self, node: &Self::Node, ancestor: Option<&Self::Node>) -> bool;
}
fn LastChild<C: PositionTree>(context: &C, node: &C::Node) -> Option<C::Node> {
    context
        .ChildCount(node)
        .checked_sub(1)
        .and_then(|index| context.ChildAt(node, index))
}
fn PreviousSkippingChildren<C: PositionTree>(
    context: &C,
    node: &C::Node,
    stay: Option<&C::Node>,
) -> Option<C::Node> {
    let mut parent = Some(node.clone());
    while let Some(node) = parent {
        if Some(&node) == stay {
            return None;
        }
        if let Some(previous) = context.PreviousSibling(&node) {
            return Some(previous);
        }
        parent = context.Parent(&node);
    }
    None
}
fn Previous<C: ParagraphContext>(
    context: &C,
    node: &C::Node,
    start: &C::Node,
    position: &Position<C::Node>,
    block: Option<&C::Node>,
) -> Option<C::Node> {
    if node == start {
        let before = position.ComputeNodeBeforePosition(context);
        let Some(before) = before else {
            return PreviousSkippingChildren(context, node, block);
        };
        if &before != node {
            return Some(before);
        }
    }
    LastChild(context, node).or_else(|| PreviousSkippingChildren(context, node, block))
}
fn Next<C: ParagraphContext>(
    context: &C,
    node: &C::Node,
    start: &C::Node,
    candidate: &C::Node,
    position: &Position<C::Node>,
    block: Option<&C::Node>,
) -> Option<C::Node> {
    if node == start {
        let after = position.ComputeNodeAfterPosition(context);
        let Some(after) = after else {
            return NextSkippingChildren(context, node, block);
        };
        if &after != candidate {
            return Some(after);
        }
    }
    NextNode(context, node, block)
}
fn Candidate<C: PositionTree>(
    context: &C,
    node: C::Node,
    kind: PositionAnchorType,
    offset: usize,
) -> Position<C::Node> {
    match kind {
        PositionAnchorType::OffsetInAnchor => Position::new(node, offset),
        PositionAnchorType::BeforeAnchor => Position::BeforeNode(node),
        PositionAnchorType::AfterAnchor => Position::AfterNode(node),
        PositionAnchorType::AfterChildren => Position::LastPositionInNode(context, &node),
    }
}
pub fn StartOfParagraph<C: ParagraphContext>(
    context: &C,
    position: &Position<C::Node>,
    rule: Rule,
) -> Position<C::Node> {
    let Some(start) = position.AnchorNode() else {
        return Position::default();
    };
    if context.IsRenderedAsNonInlineTableImageOrHR(start) {
        return Position::BeforeNode(start.clone());
    }
    let block = context.EnclosingBlock(
        &Position::FirstPositionInOrBeforeNode(context, start),
        Rule::CannotCross,
    );
    let root = context.HighestEditableRoot(position);
    let editable = context.IsEditable(start);
    let mut candidate = start.clone();
    let mut kind = position.AnchorType();
    let mut offset = position.ComputeEditingOffset(context);
    let mut previous = Some(start.clone());
    while let Some(node) = previous.clone() {
        let all = context
            .ParagraphLayout(&node)
            .is_some_and(|facts| facts.user_select_all);
        if rule == Rule::CannotCross && !all && context.IsEditable(&node) != editable {
            break;
        }
        if rule == Rule::CanSkipOver {
            while previous
                .as_ref()
                .is_some_and(|node| context.IsEditable(node) != editable)
            {
                let node = previous.take().unwrap();
                previous = Previous(context, &node, start, position, block.as_ref());
            }
            if previous
                .as_ref()
                .is_none_or(|node| !context.IsDOMDescendantOf(node, root.as_ref()))
            {
                break;
            }
        }
        let node = previous.as_ref().unwrap().clone();
        let Some(facts) = context.ParagraphLayout(&node) else {
            previous = Previous(context, &node, start, position, block.as_ref());
            continue;
        };
        if !facts.visible {
            previous = Previous(context, &node, start, position, block.as_ref());
            continue;
        }
        if facts.br || context.IsEnclosingBlock(&node) {
            break;
        }
        if let Some(text) = facts.text.filter(|text| text.resolved_text_length != 0) {
            if text.preserve_breaks {
                let text = text.transformed_text.Span16().unwrap_or_default();
                let mut index = text.len();
                if &node == start && offset < index {
                    index = offset;
                }
                while index != 0 {
                    index -= 1;
                    if text[index] == 10 {
                        return Position::new(node, index + 1);
                    }
                }
            }
            candidate = node.clone();
            kind = PositionAnchorType::OffsetInAnchor;
            offset = 0;
            previous = Previous(context, &node, start, position, block.as_ref());
        } else if context.EditingIgnoresContent(&node) || context.IsDisplayInsideTable(&node) {
            if facts.selectable {
                candidate = node.clone();
                kind = PositionAnchorType::BeforeAnchor;
            }
            previous = PreviousSkippingChildren(context, &node, block.as_ref());
        } else {
            previous = Previous(context, &node, start, position, block.as_ref());
        }
    }
    Candidate(context, candidate, kind, offset)
}
pub fn EndOfParagraph<C: ParagraphContext>(
    context: &C,
    position: &Position<C::Node>,
    rule: Rule,
) -> Position<C::Node> {
    let Some(start) = position.AnchorNode() else {
        return Position::default();
    };
    if context.IsRenderedAsNonInlineTableImageOrHR(start) {
        return Position::AfterNode(start.clone());
    }
    let block = context.EnclosingBlock(
        &Position::FirstPositionInOrBeforeNode(context, start),
        Rule::CannotCross,
    );
    let root = context.HighestEditableRoot(position);
    let editable = context.IsEditable(start);
    let mut candidate = start.clone();
    let mut kind = position.AnchorType();
    let mut offset = position.ComputeEditingOffset(context);
    let mut next = if block.as_ref() == Some(start) {
        Next(context, start, start, &candidate, position, block.as_ref())
    } else {
        Some(start.clone())
    };
    while let Some(node) = next.clone() {
        let all = context
            .ParagraphLayout(&node)
            .is_some_and(|facts| facts.user_select_all);
        if rule == Rule::CannotCross && !all && context.IsEditable(&node) != editable {
            break;
        }
        if rule == Rule::CanSkipOver {
            while next
                .as_ref()
                .is_some_and(|node| context.IsEditable(node) != editable)
            {
                let node = next.as_ref().unwrap();
                if !context.IsDOMDescendantOf(node, root.as_ref()) {
                    break;
                }
                candidate = node.clone();
                kind = PositionAnchorType::AfterAnchor;
                next = NextSkippingChildren(context, node, block.as_ref());
            }
            if next
                .as_ref()
                .is_none_or(|node| !context.IsDOMDescendantOf(node, root.as_ref()))
            {
                break;
            }
        }
        let node = next.as_ref().unwrap().clone();
        let Some(facts) = context.ParagraphLayout(&node) else {
            next = Next(context, &node, start, &candidate, position, block.as_ref());
            continue;
        };
        if !facts.visible {
            candidate = node.clone();
            kind = PositionAnchorType::AfterAnchor;
            next = NextSkippingChildren(context, &node, block.as_ref());
            continue;
        }
        if facts.br || context.IsEnclosingBlock(&node) {
            break;
        }
        if let Some(text) = facts.text.filter(|text| text.resolved_text_length != 0) {
            if text.preserve_breaks {
                let units = text.transformed_text.Span16().unwrap_or_default();
                let begin = if &node == start { offset } else { 0 };
                for index in begin..units.len() {
                    if units[index] == 10 {
                        return Position::new(node, index + text.text_start_offset);
                    }
                }
            }
            candidate = node.clone();
            kind = PositionAnchorType::OffsetInAnchor;
            offset = text.caret_max_offset + text.text_start_offset;
            next = Next(context, &node, start, &candidate, position, block.as_ref());
        } else if context.EditingIgnoresContent(&node) || context.IsDisplayInsideTable(&node) {
            if facts.selectable {
                candidate = node.clone();
                kind = PositionAnchorType::AfterAnchor;
            }
            next = NextSkippingChildren(context, &node, block.as_ref());
        } else {
            next = Next(context, &node, start, &candidate, position, block.as_ref());
        }
    }
    Candidate(context, candidate, kind, offset)
}

/// Source position_units_paragraph.cc:321-326. This is the raw-position
/// predicate; callers selecting that source API must not canonicalize via VP.
pub fn IsStartOfParagraph<C: ParagraphContext>(
    context: &C,
    position: &Position<C::Node>,
    rule: Rule,
) -> bool {
    !position.IsNull() && position == &StartOfParagraph(context, position, rule)
}
/// Source position_units_paragraph.cc:328-333.
pub fn IsEndOfParagraph<C: ParagraphContext>(
    context: &C,
    position: &Position<C::Node>,
    rule: Rule,
) -> bool {
    !position.IsNull() && position == &EndOfParagraph(context, position, rule)
}

// Source position_units_paragraph.cc:344-365, including the source guard for
// paragraph boundaries which unexpectedly reverse under editing boundaries.
pub fn ExpandToParagraphBoundary<
    C: ParagraphContext + super::position_conversion::PositionTreeScope,
>(
    context: &C,
    range: &super::ephemeral_range::EphemeralRange<C::Node>,
) -> super::ephemeral_range::EphemeralRange<C::Node> {
    use std::cmp::Ordering;
    let start = StartOfParagraph(context, range.StartPosition(), Rule::CannotCross);
    let end = EndOfParagraph(context, range.EndPosition(), Rule::CannotCross);
    let start = if !start.IsNull()
        && super::compare_positions::CompareDOMPositions(context, &start, range.StartPosition())
            .is_some_and(|c| c != Ordering::Greater)
    {
        start
    } else {
        range.StartPosition().clone()
    };
    let end = if !end.IsNull()
        && super::compare_positions::CompareDOMPositions(context, &end, range.EndPosition())
            .is_some_and(|c| c != Ordering::Less)
    {
        end
    } else {
        range.EndPosition().clone()
    };
    super::ephemeral_range::EphemeralRange::new(start, end)
}
