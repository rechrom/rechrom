/*
 * Copyright (C) 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2012 Apple Inc. All
 * rights reserved.
 * Copyright (C) 2005 Alexey Proskuryakov.
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

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/iterators/text_iterator.cc:77-145,170-205,207-1120;
// fully_clipped_state_stack.cc:18-100. DOM and flat strategies share this
// source traversal; only DOM enters/exits explicit shadow roots.
use super::{
    text_iterator_behavior::TextIteratorBehavior,
    text_iterator_text_state::{PositionNodeType, TextIteratorTextState},
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
use crate::core::editing::{
    position::{Position, PositionDOM, PositionWithAffinity, TextAffinity},
    text_offset_mapping::{IsDescendantOf, NextSkippingChildren},
    visible_position::{VisiblePosition, VisiblePositionContext},
};
use foundation::String;
use std::cmp::Ordering;

/// Current LayoutObject predicates used by the source iterator. These are
/// ephemeral scalar reads of one native owner, not retained style/geometry.
pub struct TextIteratorLayoutFacts {
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
/// Output of the original per-text-node handler, sharing OffsetMapping text.
pub struct TextIteratorTextRun {
    pub dom_start: usize,
    pub dom_end: usize,
    pub text: String,
    pub text_start: usize,
    pub text_end: usize,
    pub is_text_security: bool,
}
pub trait TextIteratorContext: PositionDOM {
    type VisibleContext: VisiblePositionContext<Node = Self::Node>;
    fn VisibleContext(&self) -> &Self::VisibleContext;
    type TextCursor;
    fn AssertTextIteratorLifecyclePrepared(&self);
    fn IsFlatTreeStrategy(&self) -> bool;
    fn OpenShadowRoot(&self, node: &Self::Node) -> Option<Self::Node>;
    fn UserAgentShadowRoot(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsOpenShadowRoot(&self, node: &Self::Node) -> bool;
    fn TextTreeScopeRoot(&self, node: &Self::Node) -> Self::Node;
    fn IsAtomicInline(&self, node: &Self::Node) -> bool;
    fn CountShadowTreeUse(&self, selection_to_string: bool, window_find: bool);

    fn LayoutFacts(&self, node: &Self::Node) -> Option<TextIteratorLayoutFacts>;
    fn IsHTMLElement(&self, node: &Self::Node, tag: &str) -> bool;
    fn HasDisplayContents(&self, node: &Self::Node) -> bool;
    fn IsShadowRoot(&self, node: &Self::Node) -> bool;
    fn ShadowHost(&self, node: &Self::Node) -> Option<Self::Node>;
    fn LockedInclusiveAncestorPreventingLayout(&self, node: &Self::Node) -> bool;
    fn DisplayLockAllowsSelectionTraversal(&self, node: &Self::Node) -> bool;
    fn IsFormControl(&self, node: &Self::Node) -> bool;
    fn IsTextControl(&self, node: &Self::Node) -> bool;
    fn EnclosingAutofilledOrPreviewedTextControl(&self, node: &Self::Node) -> bool;
    fn AltText(&self, node: &Self::Node) -> String;
    fn IsImageInput(&self, node: &Self::Node) -> bool;
    fn CreateTextCursor(
        &self,
        node: &Self::Node,
        start: usize,
        end: usize,
        behavior: TextIteratorBehavior,
    ) -> Option<Self::TextCursor>;
    fn NextTextRun(&self, cursor: &mut Self::TextCursor) -> Option<TextIteratorTextRun>;
}

pub fn ShouldEmitNewlineBeforeNode<C: TextIteratorContext>(context: &C, node: &C::Node) -> bool {
    let Some(f) = context.LayoutFacts(node) else {
        return !context.HasDisplayContents(node)
            && [
                "blockquote",
                "dd",
                "div",
                "dl",
                "dt",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "hr",
                "li",
                "listing",
                "ol",
                "p",
                "pre",
                "tr",
                "ul",
            ]
            .iter()
            .any(|tag| context.IsHTMLElement(node, tag));
    };
    if context.IsHTMLElement(node, "option")
        || context.IsHTMLElement(node, "optgroup")
        || f.table_cell
    {
        return false;
    }
    if f.table_row && f.table_exists && !f.table_inline {
        return true;
    }
    !f.inline && f.block && !f.floating_or_out_of_flow && !f.body
}
pub fn ShouldEmitNewlineAfterNode<C: TextIteratorContext>(context: &C, node: &C::Node) -> bool {
    if !ShouldEmitNewlineBeforeNode(context, node) {
        return false;
    }
    let mut next = NextSkippingChildren(context, node, None);
    while let Some(node) = next {
        if context.LayoutFacts(&node).is_some() {
            return true;
        }
        next = NextSkippingChildren(context, &node, None);
    }
    false
}
pub fn ShouldEmitTabBeforeNode<C: TextIteratorContext>(context: &C, node: &C::Node) -> bool {
    context
        .LayoutFacts(node)
        .is_some_and(|f| f.table_cell && f.table_exists && !f.table_first_cell)
}
pub fn SupportsAltText<C: TextIteratorContext>(context: &C, node: &C::Node) -> bool {
    context.IsHTMLElement(node, "img") || context.IsImageInput(node)
}
pub fn ShouldEmitNewlineForNode<C: TextIteratorContext>(
    context: &C,
    node: &C::Node,
    emits_original_text: bool,
) -> bool {
    let br = context
        .LayoutFacts(node)
        .map_or_else(|| context.IsHTMLElement(node, "br"), |f| f.br);
    br && (emits_original_text
        || !context
            .ShadowHost(node)
            .is_some_and(|host| context.IsHTMLElement(&host, "input")))
}
#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Progress {
    None,
    OpenShadowRoots,
    UserAgentShadowRoot,
    Node,
    Children,
}
pub struct TextIterator<'a, C: TextIteratorContext> {
    context: &'a C,
    start_container: C::Node,
    start_offset: usize,
    end_container: C::Node,
    end_offset: usize,
    end_node: Option<C::Node>,
    past_end_node: Option<C::Node>,
    node: Option<C::Node>,
    progress: Progress,
    clipped: Vec<bool>,
    needs_another_newline: bool,
    should_stop: bool,
    last_text_node: Option<C::Node>,
    text_cursor: Option<C::TextCursor>,
    behavior: TextIteratorBehavior,
    state: TextIteratorTextState<C::Node>,
    shadow_depth: usize,
    handled_shadow_root: bool,
}
impl<'a, C: TextIteratorContext> TextIterator<'a, C> {
    fn ParentCrossingShadow(context: &C, node: &C::Node) -> Option<C::Node> {
        context.Parent(node).or_else(|| {
            (!context.IsFlatTreeStrategy())
                .then(|| context.ShadowHost(node))
                .flatten()
        })
    }
    fn ShadowDepth(context: &C, start: &C::Node, end: &C::Node) -> usize {
        if context.IsFlatTreeStrategy() {
            return 0;
        }
        let mut end_scope = Some(context.TextTreeScopeRoot(end));
        while let Some(scope) = end_scope {
            let mut depth = 0;
            let mut start_scope = Some(context.TextTreeScopeRoot(start));
            while let Some(current) = start_scope {
                if current == scope {
                    return depth;
                }
                start_scope = context
                    .ShadowHost(&current)
                    .map(|host| context.TextTreeScopeRoot(&host));
                depth += 1;
            }
            end_scope = context
                .ShadowHost(&scope)
                .map(|host| context.TextTreeScopeRoot(&host));
        }
        panic!("text range scopes have a common ancestor")
    }
    fn NotSkipping(context: &C, node: &C::Node) -> bool {
        context.LayoutFacts(node).is_some()
            || context.HasDisplayContents(node)
            || (context.IsShadowRoot(node)
                && context
                    .ShadowHost(node)
                    .is_some_and(|host| context.LayoutFacts(&host).is_some()))
    }
    fn PastLastNode(context: &C, container: &C::Node, offset: usize) -> Option<C::Node> {
        if context.CharacterDataLength(container).is_none() && Self::NotSkipping(context, container)
        {
            let mut next = context.ChildAt(container, offset);
            while let Some(node) = next {
                if Self::NotSkipping(context, &node) {
                    return Some(node);
                }
                next = context.NextSibling(&node);
            }
        }
        let mut current = Some(container.clone());
        while let Some(node) = current {
            let parent = Self::ParentCrossingShadow(context, &node);
            if parent
                .as_ref()
                .is_some_and(|parent| Self::NotSkipping(context, parent))
            {
                if let Some(next) = context.NextSibling(&node) {
                    return Some(next);
                }
            }
            current = parent;
        }
        None
    }
    pub fn new(
        context: &'a C,
        start: &Position<C::Node>,
        end: &Position<C::Node>,
        mut behavior: TextIteratorBehavior,
    ) -> Self {
        context.AssertTextIteratorLifecyclePrepared();
        assert!(
            context
                .VisibleContext()
                .CompareFlatPositions(start, end)
                .is_some_and(|order| order != Ordering::Greater),
            "ordered source text range"
        );
        let start_container = start
            .ComputeContainerNode(context)
            .expect("text range start");
        let end_container = end.ComputeContainerNode(context).expect("text range end");
        let start_offset = start.ComputeOffsetInContainerNode(context);
        let end_offset = end.ComputeOffsetInContainerNode(context);
        let node = if context.CharacterDataLength(&start_container).is_some() {
            Some(start_container.clone())
        } else {
            context.ChildAt(&start_container, start_offset).or_else(|| {
                if start_offset == 0 {
                    Some(start_container.clone())
                } else {
                    NextSkippingChildren(context, &start_container, None)
                }
            })
        };
        let end_node = if context.CharacterDataLength(&end_container).is_none() && end_offset != 0 {
            context.ChildAt(&end_container, end_offset - 1)
        } else {
            None
        };
        let past_end_node = Self::PastLastNode(context, &end_container, end_offset);
        behavior.SetExcludeAutofilledValue(
            behavior.ForSelectionToString() || behavior.ExcludeAutofilledValue(),
        );
        if context.IsFlatTreeStrategy() {
            behavior.SetEntersOpenShadowRoots(false);
        }
        let shadow_depth = Self::ShadowDepth(context, &start_container, &end_container);
        let mut it = Self {
            context,
            start_container,
            start_offset,
            end_container,
            end_offset,
            end_node,
            past_end_node,
            node,
            progress: Progress::None,
            clipped: Vec::new(),
            needs_another_newline: false,
            should_stop: false,
            last_text_node: None,
            text_cursor: None,
            behavior,
            state: TextIteratorTextState::new(behavior),
            shadow_depth,
            handled_shadow_root: false,
        };
        if let Some(node) = it.node.clone() {
            let mut ancestry = Vec::new();
            let mut parent = Self::ParentCrossingShadow(context, &node);
            while let Some(node) = parent {
                parent = Self::ParentCrossingShadow(context, &node);
                ancestry.push(node);
            }
            for node in ancestry.into_iter().rev() {
                it.PushClipped(&node);
            }
            it.PushClipped(&node);
            it.Advance();
        }
        it
    }
    fn PushClipped(&mut self, node: &C::Node) {
        let facts = self.context.LayoutFacts(node);
        let clips = facts.as_ref().is_some_and(|f| {
            f.is_box && f.is_scroll_container && !f.is_layout_view && f.size_empty
        });
        let ignores = facts.as_ref().is_some_and(|f| !f.is_text && f.out_of_flow);
        self.clipped
            .push(clips || (self.clipped.last().copied().unwrap_or(false) && !ignores));
    }
    fn FullyClipped(&self) -> bool {
        self.clipped.last().copied().unwrap_or(false)
    }
    pub fn AtEnd(&self) -> bool {
        self.state.PositionNode().is_none() || self.should_stop
    }
    pub fn length(&self) -> usize {
        self.state.length()
    }
    pub fn CharacterAt(&self, index: usize) -> u16 {
        self.state.CharacterAt(index)
    }
    pub fn IsInsideAtomicInlineElement(&self) -> bool {
        !self.AtEnd()
            && self.length() == 1
            && self
                .node
                .as_ref()
                .is_some_and(|node| self.context.IsAtomicInline(node))
    }
    pub fn GetTextState(&self) -> &TextIteratorTextState<C::Node> {
        &self.state
    }
    pub fn BreaksAtReplacedElement(&self) -> bool {
        !self.behavior.DoesNotBreakAtReplacedElement()
    }
    fn RemainingText(&mut self) -> bool {
        let Some(cursor) = self.text_cursor.as_mut() else {
            return false;
        };
        let Some(run) = self.context.NextTextRun(cursor) else {
            self.text_cursor = None;
            return false;
        };
        self.state.EmitText(
            self.last_text_node.as_ref().expect("text cursor node"),
            run.dom_start,
            run.dom_end,
            &run.text,
            run.text_start,
            run.text_end,
            run.is_text_security,
        );
        true
    }
    fn HandleRememberedProgress(&mut self) -> bool {
        if self.needs_another_newline {
            let node = self.node.as_ref().expect("remembered block");
            let base = self
                .context
                .ChildCount(node)
                .checked_sub(1)
                .and_then(|index| self.context.ChildAt(node, index))
                .unwrap_or_else(|| node.clone());
            self.state.EmitChar16AfterNode(10, &base);
            self.needs_another_newline = false;
            return true;
        }
        self.RemainingText()
    }
    fn ShouldHandleChildren(&self, node: &C::Node) -> bool {
        !self.context.IsHTMLElement(node, "img")
            && (self.behavior.EntersTextControls() || !self.context.IsTextControl(node))
            && (self.behavior.IgnoresDisplayLock()
                || self.context.DisplayLockAllowsSelectionTraversal(node))
    }
    pub fn Advance(&mut self) {
        if self.should_stop {
            return;
        }
        self.context.AssertTextIteratorLifecyclePrepared();
        self.state.ResetRunInformation();
        if self.HandleRememberedProgress() {
            return;
        }
        while self.node.is_some() && self.node != self.past_end_node {
            let current = self.node.clone().expect("current text iterator node");
            if !self.should_stop && self.behavior.StopsOnFormControls() {
                let mut control = Some(current.clone());
                while let Some(node) = control {
                    if self.context.IsFormControl(&node) {
                        self.should_stop = true;
                        break;
                    }
                    control = self.context.Parent(&node);
                }
            }
            if current == self.end_container && self.end_offset == 0 {
                self.RepresentNodeOffsetZero();
                self.node = None;
                return;
            }
            let locked = !self.behavior.IgnoresDisplayLock()
                && self
                    .context
                    .LockedInclusiveAncestorPreventingLayout(&current);
            let facts = self.context.LayoutFacts(&current);
            if facts.is_none() || locked {
                if !locked
                    && (self.context.IsShadowRoot(&current)
                        || self.context.HasDisplayContents(&current))
                {
                    if self.progress < Progress::Node {
                        self.progress = Progress::Node;
                    }
                    self.handled_shadow_root = self.context.IsShadowRoot(&current);
                } else {
                    self.progress = Progress::Children;
                }
            } else {
                if self.progress < Progress::OpenShadowRoots {
                    if !self.context.IsFlatTreeStrategy() && self.behavior.EntersOpenShadowRoots() {
                        if let Some(root) = self.context.OpenShadowRoot(&current) {
                            self.node = Some(root.clone());
                            self.progress = Progress::None;
                            self.shadow_depth += 1;
                            self.PushClipped(&root);
                            continue;
                        }
                    }
                    self.progress = Progress::OpenShadowRoots;
                }
                if self.progress < Progress::UserAgentShadowRoot {
                    if !self.context.IsFlatTreeStrategy()
                        && self.behavior.EntersTextControls()
                        && self.context.IsTextControl(&current)
                    {
                        let root = self
                            .context
                            .UserAgentShadowRoot(&current)
                            .expect("laid out text-control UA shadow root");
                        self.node = Some(root.clone());
                        self.progress = Progress::None;
                        self.shadow_depth += 1;
                        self.PushClipped(&root);
                        continue;
                    }
                    self.progress = Progress::UserAgentShadowRoot;
                }
                if self.progress < Progress::Node {
                    let f = facts.as_ref().expect("native layout owner");
                    if !self.behavior.SkipsUnselectableContent() || f.selectable {
                        if f.is_text && self.context.IsText(&current) {
                            if !self.FullyClipped() || self.behavior.IgnoresStyleVisibility() {
                                self.HandleTextNode();
                            }
                        } else if f.image
                            || f.embedded
                            || self.context.IsFormControl(&current)
                            || ["legend", "img", "meter", "progress"]
                                .iter()
                                .any(|tag| self.context.IsHTMLElement(&current, tag))
                        {
                            self.HandleReplacedElement();
                        } else {
                            self.HandleNonTextNode();
                        }
                    }
                    self.progress = Progress::Node;
                    if self.state.PositionNode().is_some() {
                        return;
                    }
                }
            }
            let mut next =
                if self.progress < Progress::Children && self.ShouldHandleChildren(&current) {
                    self.context.ChildAt(&current, 0)
                } else {
                    None
                };
            if next.is_none() {
                if self
                    .past_end_node
                    .as_ref()
                    .is_some_and(|end| IsDescendantOf(self.context, end, &current))
                {
                    self.node = self.past_end_node.clone();
                    self.progress = Progress::None;
                    self.clipped.pop();
                    return;
                }
                next = self.context.NextSibling(&current);
                if next.is_none() {
                    let mut parent = self.context.Parent(&current);
                    while next.is_none() && parent.is_some() {
                        let old = self.node.as_ref().expect("current native node");
                        let p = parent.take().unwrap();
                        if self.end_node.as_ref() == Some(old)
                            || IsDescendantOf(self.context, &self.end_container, &p)
                        {
                            return;
                        }
                        let had_layout = self.context.LayoutFacts(old).is_some();
                        self.node = Some(p.clone());
                        self.clipped.pop();
                        parent = self.context.Parent(&p);
                        let nf = self.context.LayoutFacts(&p);
                        let pf = parent.as_ref().and_then(|p| self.context.LayoutFacts(p));
                        if had_layout
                            || nf
                                .as_ref()
                                .zip(pf.as_ref())
                                .is_some_and(|(n, p)| n.block && !p.inline)
                        {
                            self.ExitNode();
                        }
                        if self.state.PositionNode().is_some() {
                            self.progress = Progress::Children;
                            return;
                        }
                        next = self.context.NextSibling(&p);
                    }
                }
                if next.is_none() && self.shadow_depth != 0 {
                    let root = self.node.as_ref().expect("shadow root exit");
                    if !self.context.IsShadowRoot(root) {
                        self.should_stop = true;
                        return;
                    }
                    let open = self.context.IsOpenShadowRoot(root);
                    self.node = self.context.ShadowHost(root);
                    self.progress = if open {
                        Progress::OpenShadowRoots
                    } else {
                        Progress::UserAgentShadowRoot
                    };
                    self.shadow_depth -= 1;
                    self.clipped.pop();
                    continue;
                }
                self.clipped.pop();
            }
            self.node = next;
            if let Some(node) = self.node.clone() {
                self.PushClipped(&node);
            }
            self.progress = Progress::None;
            if self.state.PositionNode().is_some() {
                return;
            }
        }
    }
    fn HandleTextNode(&mut self) {
        let node = self.node.as_ref().unwrap();
        if self.behavior.ExcludeAutofilledValue()
            && self.context.EnclosingAutofilledOrPreviewedTextControl(node)
        {
            return;
        }
        assert!(
            self.last_text_node.as_ref() != Some(node),
            "text node visited once"
        );
        let start = if node == &self.start_container {
            self.start_offset
        } else {
            0
        };
        let end = if node == &self.end_container {
            self.end_offset
        } else {
            self.context.CharacterDataLength(node).expect("text length")
        };
        self.last_text_node = Some(node.clone());
        self.text_cursor = self
            .context
            .CreateTextCursor(node, start, end, self.behavior);
        self.RemainingText();
    }
    fn HandleReplacedElement(&mut self) {
        if self.FullyClipped() {
            return;
        }
        let node = self.node.as_ref().unwrap();
        let f = self.context.LayoutFacts(node).expect("replaced layout");
        if !f.visible && !self.behavior.IgnoresStyleVisibility() {
            return;
        }
        if self.behavior.EmitsObjectReplacementCharacter() {
            self.state.EmitChar16AsNode(0xfffc, node);
            return;
        }
        if self.behavior.EntersTextControls() && f.text_control {
            return;
        }
        if self.behavior.EmitsCharactersBetweenAllVisiblePositions() {
            self.state.EmitChar16AsNode(b',' as u16, node);
            return;
        }
        if self.behavior.EmitsImageAltText()
            && (self.context.IsHTMLElement(node, "img") || self.context.IsImageInput(node))
        {
            self.state.EmitAltText(node, self.context.AltText(node));
            return;
        }
        self.state.UpdateForReplacedElement(node);
    }
    fn NewlinesBeforeAndAfter(&self, node: &C::Node) -> bool {
        ShouldEmitNewlineBeforeNode(self.context, node)
    }
    fn NewlineAfter(&self, node: &C::Node) -> bool {
        ShouldEmitNewlineAfterNode(self.context, node)
    }
    fn SpaceBeforeAndAfter(&self, node: &C::Node) -> bool {
        self.context.LayoutFacts(node).is_some_and(|f| {
            f.table && (f.inline || self.behavior.EmitsCharactersBetweenAllVisiblePositions())
        })
    }
    fn ShouldRepresentNodeOffsetZero(&self) -> bool {
        let node = self.node.as_ref().unwrap();
        if self.behavior.EmitsCharactersBetweenAllVisiblePositions()
            && self.context.LayoutFacts(node).is_some_and(|f| f.table)
        {
            return true;
        }
        if self.state.LastCharacter() == 10 {
            return false;
        }
        if self.state.HasEmitted() {
            return true;
        }
        if node == &self.start_container {
            return false;
        }
        if !IsDescendantOf(self.context, node, &self.start_container) {
            return true;
        }
        if self.start_offset == 0 {
            return false;
        }
        let Some(f) = self.context.LayoutFacts(node) else {
            return false;
        };
        if !f.visible || (f.block_flow && !f.has_height && !f.body) {
            return false;
        }
        let start = VisiblePosition::Create(
            self.context.VisibleContext(),
            PositionWithAffinity {
                position: Position::new(self.start_container.clone(), self.start_offset),
                affinity: TextAffinity::Downstream,
            },
        );
        let current = VisiblePosition::Create(
            self.context.VisibleContext(),
            PositionWithAffinity {
                position: Position::BeforeNode(node.clone()),
                affinity: TextAffinity::Downstream,
            },
        );
        !start.IsNull()
            && !current.IsNull()
            && !self.context.VisibleContext().InSameLine(
                start.ToPositionWithAffinity(),
                current.ToPositionWithAffinity(),
            )
    }
    fn RepresentNodeOffsetZero(&mut self) {
        let node = self.node.as_ref().unwrap();
        let facts = self.context.LayoutFacts(node);
        let ch = if facts
            .as_ref()
            .is_some_and(|f| f.table_cell && f.table_exists && !f.table_first_cell)
        {
            Some(9)
        } else if self.NewlinesBeforeAndAfter(node) {
            Some(10)
        } else if self.SpaceBeforeAndAfter(node) {
            Some(32)
        } else {
            None
        };
        if let Some(ch) = ch {
            if self.ShouldRepresentNodeOffsetZero() {
                self.state.EmitChar16BeforeNode(ch, node);
            }
        }
    }
    fn HandleNonTextNode(&mut self) {
        let node = self.node.as_ref().unwrap();
        let facts = self.context.LayoutFacts(node);
        let br = facts
            .as_ref()
            .map_or_else(|| self.context.IsHTMLElement(node, "br"), |f| f.br);
        let input_shadow = self
            .context
            .ShadowHost(node)
            .is_some_and(|host| self.context.IsHTMLElement(&host, "input"));
        if br && (self.behavior.EmitsOriginalText() || !input_shadow) {
            self.state.EmitChar16AsNode(10, node);
        } else if self.behavior.EmitsCharactersBetweenAllVisiblePositions()
            && facts.is_some_and(|f| f.hr)
        {
            self.state.EmitChar16AsNode(32, node);
        } else {
            self.RepresentNodeOffsetZero();
        }
    }
    fn ExitNode(&mut self) {
        if !self.state.HasEmitted() {
            return;
        }
        let node = self.node.as_ref().unwrap();
        let base = self
            .context
            .ChildCount(node)
            .checked_sub(1)
            .and_then(|index| self.context.ChildAt(node, index))
            .unwrap_or_else(|| node.clone());
        if self.last_text_node.is_some() && self.NewlineAfter(node) {
            let extra = !self.behavior.SuppressesExtraNewlineEmission()
                && self.context.LayoutFacts(node).is_some_and(|f| f.is_box)
                && self.context.IsHTMLElement(node, "p");
            if self.state.LastCharacter() != 10 {
                self.state.EmitChar16AfterNode(10, &base);
                assert!(!self.needs_another_newline);
                self.needs_another_newline = extra;
            } else if extra {
                self.state.EmitChar16AfterNode(10, &base);
            }
        }
        if self.state.PositionNode().is_none() && self.SpaceBeforeAndAfter(node) {
            self.state.EmitChar16AfterNode(32, &base);
        }
    }
    pub fn CurrentContainer(&self) -> C::Node {
        if self.state.PositionNode().is_none() {
            self.end_container.clone()
        } else {
            self.state.Position(self.context).0
        }
    }
    pub fn StartOffsetInCurrentContainer(&self) -> usize {
        if self.state.PositionNode().is_none() {
            self.end_offset
        } else {
            self.state.Position(self.context).1
        }
    }
    pub fn EndOffsetInCurrentContainer(&self) -> usize {
        if self.state.PositionNode().is_none() {
            self.end_offset
        } else {
            self.state.Position(self.context).2
        }
    }
    pub fn StartPositionInCurrentContainer(&self) -> Position<C::Node> {
        Position::EditingPositionOf(
            self.context,
            self.CurrentContainer(),
            self.StartOffsetInCurrentContainer(),
        )
    }
    pub fn EndPositionInCurrentContainer(&self) -> Position<C::Node> {
        Position::EditingPositionOf(
            self.context,
            self.CurrentContainer(),
            self.EndOffsetInCurrentContainer(),
        )
    }
    pub fn GetPositionBefore(&self, offset: usize) -> Position<C::Node> {
        if self.AtEnd() {
            assert_eq!(offset, 0);
            return Position::new(
                self.CurrentContainer(),
                self.StartOffsetInCurrentContainer(),
            );
        }
        assert!(offset < self.length());
        let node = self.state.PositionNode().unwrap();
        if matches!(
            self.state.kind,
            PositionNodeType::InText | PositionNodeType::BeforeCharacter
        ) {
            return Position::new(node.clone(), self.state.Position(self.context).1 + offset);
        }
        if self.context.IsText(node) {
            return Position::new(
                node.clone(),
                if self.state.kind == PositionNodeType::AfterNode {
                    self.context.CharacterDataLength(node).unwrap()
                } else {
                    0
                },
            );
        }
        if self.state.kind == PositionNodeType::AfterNode {
            Position::AfterNode(node.clone())
        } else {
            assert!(self.state.kind != PositionNodeType::BeforeChildren);
            Position::BeforeNode(node.clone())
        }
    }
    pub fn GetPositionAfter(&self, offset: usize) -> Position<C::Node> {
        if self.AtEnd() {
            assert_eq!(offset, 0);
            return Position::new(self.CurrentContainer(), self.EndOffsetInCurrentContainer());
        }
        assert!(offset < self.length());
        let node = self.state.PositionNode().unwrap();
        if self.state.kind == PositionNodeType::BeforeCharacter {
            return Position::new(node.clone(), self.state.Position(self.context).1 + offset);
        }
        if self.state.kind == PositionNodeType::InText {
            return Position::new(
                node.clone(),
                self.state.Position(self.context).1 + offset + 1,
            );
        }
        if self.context.IsText(node) {
            return Position::new(
                node.clone(),
                if self.state.kind == PositionNodeType::BeforeNode {
                    0
                } else {
                    self.context.CharacterDataLength(node).unwrap()
                },
            );
        }
        if self.state.kind == PositionNodeType::BeforeNode {
            Position::BeforeNode(node.clone())
        } else {
            assert!(self.state.kind != PositionNodeType::BeforeChildren);
            Position::AfterNode(node.clone())
        }
    }
}
pub fn RangeLength<C: TextIteratorContext>(
    context: &C,
    start: &Position<C::Node>,
    end: &Position<C::Node>,
    behavior: TextIteratorBehavior,
) -> usize {
    let mut iterator = TextIterator::new(context, start, end, behavior);
    let mut length = 0;
    while !iterator.AtEnd() {
        length += iterator.length();
        iterator.Advance();
    }
    length
}
pub fn PlainText<C: TextIteratorContext>(
    context: &C,
    start: &Position<C::Node>,
    end: &Position<C::Node>,
    behavior: TextIteratorBehavior,
) -> Vec<u16> {
    if start.IsNull() || end.IsNull() {
        return Vec::new();
    }
    let mut iterator = TextIterator::new(context, start, end, behavior);
    if iterator.AtEnd() {
        return Vec::new();
    }
    let mut text = Vec::with_capacity(1 << 15);
    while !iterator.AtEnd() {
        iterator.GetTextState().AppendText(&mut text);
        iterator.Advance();
    }
    text
}

impl<C: TextIteratorContext> Drop for TextIterator<'_, C> {
    fn drop(&mut self) {
        if self.handled_shadow_root {
            self.context.CountShadowTreeUse(
                self.behavior.ForSelectionToString(),
                self.behavior.ForWindowFind(),
            );
        }
    }
}
