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
// Source: core/editing/iterators/character_iterator.cc:39-187.
use super::{
    text_iterator::{TextIterator, TextIteratorContext},
    text_iterator_behavior::TextIteratorBehavior,
};
use crate::core::editing::position::Position;
pub struct CharacterIterator<'a, C: TextIteratorContext> {
    context: &'a C,
    text_iterator: TextIterator<'a, C>,
    offset: usize,
    run_offset: usize,
    at_break: bool,
}
impl<'a, C: TextIteratorContext> CharacterIterator<'a, C> {
    pub fn new(
        context: &'a C,
        start: &Position<C::Node>,
        end: &Position<C::Node>,
        behavior: TextIteratorBehavior,
    ) -> Self {
        let mut iterator = Self {
            context,
            text_iterator: TextIterator::new(context, start, end, behavior),
            offset: 0,
            run_offset: 0,
            at_break: true,
        };
        while !iterator.AtEnd() && iterator.text_iterator.length() == 0 {
            iterator.text_iterator.Advance();
        }
        iterator
    }
    pub fn AtEnd(&self) -> bool {
        self.text_iterator.AtEnd()
    }
    pub fn AtBreak(&self) -> bool {
        self.at_break
    }
    pub fn length(&self) -> usize {
        self.text_iterator.length() - self.run_offset
    }
    pub fn CharacterAt(&self, index: usize) -> u16 {
        self.text_iterator.CharacterAt(self.run_offset + index)
    }
    pub fn CharacterOffset(&self) -> usize {
        self.offset
    }
    pub fn CurrentContainer(&self) -> C::Node {
        self.text_iterator.CurrentContainer()
    }
    pub fn StartOffset(&self) -> usize {
        self.text_iterator.StartOffsetInCurrentContainer()
            + if !self.AtEnd() && self.text_iterator.length() > 1 {
                self.run_offset
            } else {
                0
            }
    }
    pub fn EndOffset(&self) -> usize {
        if !self.AtEnd() && self.text_iterator.length() > 1 {
            self.text_iterator.StartOffsetInCurrentContainer() + self.run_offset + 1
        } else {
            self.text_iterator.EndOffsetInCurrentContainer()
        }
    }
    pub fn GetPositionBefore(&self) -> Position<C::Node> {
        self.text_iterator.GetPositionBefore(self.run_offset)
    }
    pub fn GetPositionAfter(&self) -> Position<C::Node> {
        self.text_iterator.GetPositionAfter(self.run_offset)
    }
    pub fn StartPosition(&self) -> Position<C::Node> {
        if !self.AtEnd() && self.text_iterator.length() > 1 {
            Position::EditingPositionOf(self.context, self.CurrentContainer(), self.StartOffset())
        } else {
            self.text_iterator.StartPositionInCurrentContainer()
        }
    }
    pub fn EndPosition(&self) -> Position<C::Node> {
        if !self.AtEnd() && self.text_iterator.length() > 1 {
            Position::EditingPositionOf(self.context, self.CurrentContainer(), self.EndOffset())
        } else {
            self.text_iterator.EndPositionInCurrentContainer()
        }
    }
    pub fn Advance(&mut self, mut count: usize) {
        if count == 0 {
            return;
        }
        assert!(!self.AtEnd());
        self.at_break = false;
        let remaining = self.text_iterator.length() - self.run_offset;
        if count < remaining {
            self.run_offset += count;
            self.offset += count;
            return;
        }
        count -= remaining;
        self.offset += remaining;
        self.text_iterator.Advance();
        while !self.AtEnd() {
            let length = self.text_iterator.length();
            if length == 0 {
                self.at_break = self.text_iterator.BreaksAtReplacedElement();
            } else {
                if count < length {
                    self.run_offset = count;
                    self.offset += count;
                    return;
                }
                count -= length;
                self.offset += length;
            }
            self.text_iterator.Advance();
        }
        self.at_break = true;
        self.run_offset = 0;
    }
    pub fn CalculateCharacterSubrange(
        &mut self,
        offset: usize,
        length: usize,
    ) -> (Position<C::Node>, Position<C::Node>) {
        self.Advance(offset);
        let start = self.StartPosition();
        if length == 0 {
            return (start.clone(), start);
        }
        if length > 1 {
            self.Advance(length - 1);
        }
        (start, self.EndPosition())
    }
}
