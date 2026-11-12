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
// Source: core/editing/iterators/text_iterator_text_state.{h,cc}:46-261.
use super::text_iterator_behavior::TextIteratorBehavior;
use crate::core::editing::position::PositionTree;
use foundation::String;
use std::cell::RefCell;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PositionNodeType {
    None,
    AfterNode,
    AltText,
    AsNode,
    BeforeCharacter,
    BeforeChildren,
    BeforeNode,
    InText,
}

/// The source iterator's current emitted run. Its String shares the original
/// immutable buffer (normally resident OffsetMapping text), not copied DOM text.
pub struct TextIteratorTextState<N> {
    behavior: TextIteratorBehavior,
    text_length: usize,
    single_character: u16,
    text: String,
    text_start_offset: usize,
    position_node: Option<N>,
    position: RefCell<Option<(N, usize, usize)>>,
    pub kind: PositionNodeType,
    has_emitted: bool,
    last_character: u16,
}
impl<N: Clone + PartialEq> TextIteratorTextState<N> {
    pub fn new(behavior: TextIteratorBehavior) -> Self {
        Self {
            behavior,
            text_length: 0,
            single_character: 0,
            text: String::default(),
            text_start_offset: 0,
            position_node: None,
            position: RefCell::new(None),
            kind: PositionNodeType::None,
            has_emitted: false,
            last_character: 0,
        }
    }
    pub fn length(&self) -> usize {
        self.text_length
    }
    pub fn CharacterAt(&self, index: usize) -> u16 {
        if index >= self.length() {
            return 0;
        }
        if self.single_character != 0 {
            return self.single_character;
        }
        self.text.Span16().expect("emitted UTF-16 run")[self.text_start_offset + index]
    }
    pub fn AppendText(&self, builder: &mut Vec<u16>) {
        if self.text_length == 0 {
            return;
        }
        if self.single_character != 0 {
            builder.push(self.single_character);
        } else {
            builder.extend_from_slice(
                &self.text.Span16().expect("emitted UTF-16 run")
                    [self.text_start_offset..self.text_start_offset + self.text_length],
            );
        }
    }
    pub fn PositionNode(&self) -> Option<&N> {
        self.position_node.as_ref()
    }
    pub fn HasEmitted(&self) -> bool {
        self.has_emitted
    }
    pub fn LastCharacter(&self) -> u16 {
        self.last_character
    }
    pub fn ResetRunInformation(&mut self) {
        self.position_node = None;
        self.text_length = 0;
    }
    pub fn Position<D: PositionTree<Node = N>>(&self, dom: &D) -> (N, usize, usize) {
        if let Some(position) = self.position.borrow().as_ref() {
            return position.clone();
        }
        let node = self.position_node.as_ref().expect("emitted run position");
        let container = dom.Parent(node).expect("emitted node parent");
        let index = dom.NodeIndex(node);
        let (start, end) = match self.kind {
            PositionNodeType::AfterNode => (index + 1, index + 1),
            PositionNodeType::AltText | PositionNodeType::AsNode => (index, index + 1),
            PositionNodeType::BeforeNode => (index, index),
            _ => unreachable!("text/children run has an explicit container"),
        };
        let position = (container, start, end);
        *self.position.borrow_mut() = Some(position.clone());
        position
    }
    fn ResetPosition(&mut self, kind: PositionNodeType, node: &N) {
        self.kind = kind;
        self.position_node = Some(node.clone());
        *self.position.get_mut() = None;
    }
    fn PopulateChar(&mut self, code_unit: u16) {
        assert_ne!(code_unit, 0);
        self.has_emitted = true;
        self.single_character = code_unit;
        self.text_length = 1;
        self.text_start_offset = 0;
        self.last_character = code_unit;
    }
    fn PopulateString(&mut self, text: String, start: usize, end: usize) {
        assert!(start <= end && end <= text.length() as usize);
        self.single_character = 0;
        self.text_start_offset = start;
        self.text_length = end - start;
        self.last_character = if end == 0 {
            0
        } else {
            text.Span16().expect("emitted UTF-16 run")[end - 1]
        };
        self.text = text;
        self.has_emitted = true;
    }
    pub fn UpdateForReplacedElement(&mut self, node: &N) {
        self.ResetPosition(PositionNodeType::AsNode, node);
        self.PopulateString(String::default(), 0, 0);
    }
    pub fn EmitAltText(&mut self, node: &N, text: String) {
        self.ResetPosition(PositionNodeType::AltText, node);
        let end = text.length() as usize;
        self.PopulateString(text, 0, end);
    }
    pub fn EmitChar16AfterNode(&mut self, ch: u16, node: &N) {
        self.ResetPosition(PositionNodeType::AfterNode, node);
        self.PopulateChar(ch);
    }
    pub fn EmitChar16AsNode(&mut self, ch: u16, node: &N) {
        self.ResetPosition(PositionNodeType::AsNode, node);
        self.PopulateChar(ch);
    }
    pub fn EmitChar16BeforeNode(&mut self, ch: u16, node: &N) {
        self.ResetPosition(PositionNodeType::BeforeNode, node);
        self.PopulateChar(ch);
    }
    pub fn EmitChar16BeforeChildren(&mut self, ch: u16, node: &N) {
        self.ResetPosition(PositionNodeType::BeforeChildren, node);
        *self.position.get_mut() = Some((node.clone(), 0, 0));
        self.PopulateChar(ch);
    }
    pub fn EmitChar16Before(&mut self, ch: u16, node: &N, offset: usize) {
        self.ResetPosition(PositionNodeType::BeforeCharacter, node);
        *self.position.get_mut() = Some((node.clone(), offset, offset));
        self.PopulateChar(ch);
    }
    fn SetTextPosition(&mut self, node: &N, start: usize, end: usize) {
        assert!(start < end);
        self.ResetPosition(PositionNodeType::InText, node);
        *self.position.get_mut() = Some((node.clone(), start, end));
    }
    pub fn EmitReplacementCodeUnit(&mut self, ch: u16, node: &N, offset: usize) {
        self.SetTextPosition(node, offset, offset + 1);
        self.PopulateChar(ch);
    }
    pub fn EmitText(
        &mut self,
        node: &N,
        position_start: usize,
        position_end: usize,
        text: &String,
        text_start: usize,
        text_end: usize,
        is_text_security: bool,
    ) {
        let text = if self.behavior.EmitsSmallXForTextSecurity() && is_text_security {
            String::from_utf16(&vec![b'x' as u16; text.length() as usize])
        } else {
            text.clone()
        };
        assert!(!text.empty() && text_start < text.length() as usize);
        self.SetTextPosition(node, position_start, position_end);
        self.PopulateString(text, text_start, text_end);
    }
}
