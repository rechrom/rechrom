/*
 * Copyright (C) 2008, 2012 Apple Inc. All rights reserved.
 * Copyright (C) 2009 Google Inc. All rights reserved.
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
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// cpp: third_party/blink/renderer/core/css/css_selector_list.h:71-218
// cpp: third_party/blink/renderer/core/css/css_selector_list.cc:42-187
#![allow(non_snake_case)]
use crate::css_selector::SelectorStringUnits;
use crate::css_selector::{CSSSelector, CSSSelectorComplex, CSSSelectorParentRule, MatchType};
use crate::style_rule::StyleRuleSelectorList;
use foundation::String;
use std::cell::RefCell;
use std::rc::Rc;

// The storage is the same flat array of simple selectors as Blink. Traversal
// reads end bits; Vec's length is solely the Rust memory ownership boundary.
#[derive(Clone)]
pub struct CSSSelectorList {
    selectors_: Vec<CSSSelector>,
}
thread_local! { static EMPTY: RefCell<Option<Rc<CSSSelectorList>>> = const { RefCell::new(None) }; }
impl CSSSelectorList {
    // cpp: css_selector_list.cc:42-55
    pub fn Empty() -> Rc<Self> {
        EMPTY.with(|slot| {
            let mut slot = slot.borrow_mut();
            slot.get_or_insert_with(|| {
                let mut first = CSSSelector::default();
                first.SetMatch(MatchType::kInvalidList);
                Rc::new(Self {
                    selectors_: vec![first],
                })
            })
            .clone()
        })
    }
    // cpp: css_selector_list.cc:93-123; ownership move replaces memcpy/memset.
    pub fn AdoptSelectorVector(mut selectors: Vec<CSSSelector>) -> Rc<Self> {
        if selectors.is_empty() {
            return Self::Empty();
        }
        selectors.last_mut().unwrap().SetLastInSelectorList(true);
        Rc::new(Self {
            selectors_: selectors,
        })
    }
    pub fn AdoptSelectorVectorInto(mut selectors: Vec<CSSSelector>, array: &mut Vec<CSSSelector>) {
        assert!(!selectors.is_empty());
        selectors.last_mut().unwrap().SetLastInSelectorList(true);
        *array = selectors;
    }
    // cpp: css_selector_list.h:95-108,176-181
    pub fn IsValid(&self) -> bool {
        self.selectors_[0].Match() != MatchType::kInvalidList
    }
    fn IsInvalidWithoutUnparsed(&self) -> bool {
        !self.IsValid() && !self.selectors_[0].IsUnparsedInvalid()
    }
    pub fn First(&self) -> Option<CSSSelectorComplex<'_>> {
        let first = self.FirstIncludingUnparsedInvalid()?;
        if first.IsUnparsedInvalid() {
            self.Next(0)
        } else {
            Some(first)
        }
    }
    pub fn FirstIncludingUnparsedInvalid(&self) -> Option<CSSSelectorComplex<'_>> {
        (!self.IsInvalidWithoutUnparsed()).then(|| self.ComplexAt(0))
    }
    pub fn ComplexAt(&self, index: usize) -> CSSSelectorComplex<'_> {
        CSSSelectorComplex::new(&self.selectors_[index..])
    }
    // cpp: css_selector_list.h:191-218
    pub fn NextIncludingUnparsedInvalid(&self, index: usize) -> Option<CSSSelectorComplex<'_>> {
        self.ComplexAt(index)
            .NextComplexSelectorIncludingUnparsedInvalid()
    }
    pub fn Next(&self, index: usize) -> Option<CSSSelectorComplex<'_>> {
        self.ComplexAt(index).NextComplexSelector()
    }
    // cpp: css_selector_list.h:116-139
    pub fn IsSingleComplexSelector(&self) -> bool {
        self.IsValid() && self.Next(0).is_none()
    }
    pub fn SelectorAt(&self, index: usize) -> &CSSSelector {
        assert!(!self.IsInvalidWithoutUnparsed());
        &self.selectors_[index]
    }
    pub fn MutableSelectorAt(&mut self, index: usize) -> &mut CSSSelector {
        assert!(!self.IsInvalidWithoutUnparsed());
        &mut self.selectors_[index]
    }
    pub fn SelectorIndex(&self, selector: &CSSSelector) -> usize {
        assert!(!self.IsInvalidWithoutUnparsed());
        let base = self.selectors_.as_ptr() as usize;
        let address = std::ptr::from_ref(selector) as usize;
        let offset = address
            .checked_sub(base)
            .expect("selector is not in this list");
        let size = std::mem::size_of::<CSSSelector>();
        assert!(
            offset % size == 0 && offset / size < self.selectors_.len(),
            "selector is not in this list"
        );
        offset / size
    }
    pub fn IndexOfNextSelectorAfter(&self, index: usize) -> Option<usize> {
        self.Next(index).map(|s| self.SelectorIndex(&s))
    }
    pub fn NextSimpleSelector(&self, index: usize) -> Option<CSSSelectorComplex<'_>> {
        if self.SelectorAt(index).IsLastInComplexSelector() {
            None
        } else {
            Some(self.ComplexAt(index + 1))
        }
    }
    pub fn MutableNextSimpleSelector(&mut self, index: usize) -> Option<&mut CSSSelector> {
        if self.SelectorAt(index).IsLastInComplexSelector() {
            None
        } else {
            Some(self.MutableSelectorAt(index + 1))
        }
    }
    // cpp: css_selector_list.cc:57-91
    pub fn Copy(&self) -> Rc<Self> {
        if self.IsInvalidWithoutUnparsed() {
            return Self::Empty();
        }
        Rc::new(Self {
            selectors_: self.CopySelectors(),
        })
    }
    pub fn CopySelectors(&self) -> Vec<CSSSelector> {
        Self::CopyFromSelectors(Some(&self.selectors_))
    }
    pub fn CopyFromSelectors(selectors: Option<&[CSSSelector]>) -> Vec<CSSSelector> {
        let Some(selectors) = selectors else {
            return Vec::new();
        };
        let first = &selectors[0];
        if first.Match() == MatchType::kInvalidList && !first.IsUnparsedInvalid() {
            return Vec::new();
        }
        let mut result = Vec::new();
        for s in selectors {
            result.push(s.clone());
            if s.IsLastInSelectorList() {
                return result;
            }
        }
        panic!("missing selector list end bit")
    }
    // cpp: css_selector_list.cc:125-144
    pub fn ComputeLength(&self) -> usize {
        if self.IsInvalidWithoutUnparsed() {
            return 0;
        }
        self.selectors_
            .iter()
            .position(|s| s.IsLastInSelectorList())
            .expect("missing selector list end bit")
            + 1
    }
    pub fn MaximumSpecificity(&self) -> u32 {
        self.ComplexSelectors()
            .map(|s| s.Specificity())
            .max()
            .unwrap_or(0)
    }
    pub fn ComplexSelectors(&self) -> ComplexSelectors<'_> {
        ComplexSelectors {
            list: self,
            current: self.First().map(|s| self.SelectorIndex(&s)),
            include_invalid: false,
        }
    }
    pub fn ComplexSelectorsIncludingUnparsedInvalid(&self) -> ComplexSelectors<'_> {
        ComplexSelectors {
            list: self,
            current: self
                .FirstIncludingUnparsedInvalid()
                .map(|s| self.SelectorIndex(&s)),
            include_invalid: true,
        }
    }
    // cpp: css_selector_list.cc:146-174. Option is the source identity comparison:
    // None means return this; Some means adopt the actual re-nested vector.
    pub fn RenestChanged(&self, parent: Option<Rc<dyn CSSSelectorParentRule>>) -> Option<Self> {
        if !self.IsValid() {
            return None;
        }
        let mut selectors = Vec::new();
        Self::RenestSelectors(self.First(), parent, &mut selectors).then(|| Self {
            selectors_: selectors,
        })
    }
    pub fn RenestSelectors(
        first: Option<CSSSelectorComplex<'_>>,
        parent: Option<Rc<dyn CSSSelectorParentRule>>,
        result: &mut Vec<CSSSelector>,
    ) -> bool {
        let Some(first) = first else {
            return false;
        };
        let mut changed = false;
        for current in first.ArrayTail() {
            let renested = current.Renest(parent.clone());
            changed |= renested.is_some();
            result.push(renested.unwrap_or_else(|| current.clone()));
            if current.IsLastInSelectorList() {
                return changed;
            }
        }
        panic!("missing selector list end bit")
    }
    pub fn MutableNext(&mut self, index: usize) -> Option<&mut CSSSelector> {
        let next = self.Next(index).map(|s| self.SelectorIndex(&s))?;
        Some(&mut self.selectors_[next])
    }
    pub fn MutableNextIncludingUnparsedInvalid(
        &mut self,
        index: usize,
    ) -> Option<&mut CSSSelector> {
        let next = self
            .NextIncludingUnparsedInvalid(index)
            .map(|s| self.SelectorIndex(&s))?;
        Some(&mut self.selectors_[next])
    }
    pub fn Renest(self: &Rc<Self>, parent: Option<Rc<dyn CSSSelectorParentRule>>) -> Rc<Self> {
        self.RenestChanged(parent)
            .map_or_else(|| self.clone(), Rc::new)
    }
    // cpp: css_selector_list.cc:176-187
    pub fn SelectorsText(&self) -> String {
        self.SelectorsTextInternal(false, 0)
    }
    pub fn SelectorsTextInternal(&self, expand: bool, scope_id: usize) -> String {
        let mut result = Vec::new();
        self.SerializeTo(&mut result, expand, scope_id);
        String::from_utf16(&result)
    }
    pub fn SelectorsTextFromFirst(first: Option<CSSSelectorComplex<'_>>) -> String {
        let mut result = Vec::new();
        let mut current = first;
        let mut is_first = true;
        while let Some(selector) = current {
            if !is_first {
                result.extend(", ".encode_utf16());
            }
            is_first = false;
            result.extend(selector.SelectorText().EncodeForSelector());
            current = selector.NextComplexSelectorIncludingUnparsedInvalid();
        }
        String::from_utf16(&result)
    }
    pub fn SerializeTo(&self, output: &mut Vec<u16>, expand: bool, scope_id: usize) {
        for (i, selector) in self.ComplexSelectorsIncludingUnparsedInvalid().enumerate() {
            if i != 0 {
                output.extend(", ".encode_utf16());
            }
            output.extend(
                selector
                    .SelectorTextInternal(expand, scope_id)
                    .EncodeForSelector(),
            );
        }
    }
}
pub struct ComplexSelectors<'a> {
    list: &'a CSSSelectorList,
    current: Option<usize>,
    include_invalid: bool,
}
impl<'a> Iterator for ComplexSelectors<'a> {
    type Item = CSSSelectorComplex<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current?;
        self.current = if self.include_invalid {
            self.list.NextIncludingUnparsedInvalid(index)
        } else {
            self.list.Next(index)
        }
        .map(|s| self.list.SelectorIndex(&s));
        Some(self.list.ComplexAt(index))
    }
}
impl StyleRuleSelectorList for CSSSelectorList {
    type CSSSelector = CSSSelector;
    fn FirstSelector(&self) -> &CSSSelector {
        self.First()
            .map(|s| s.First())
            .expect("StyleRule requires a valid selector list")
    }
    fn SelectorAt(&self, index: usize) -> &CSSSelector {
        self.SelectorAt(index)
    }
    fn MutableSelectorAt(&mut self, index: usize) -> &mut CSSSelector {
        self.MutableSelectorAt(index)
    }
    fn SelectorIndex(&self, selector: &CSSSelector) -> usize {
        self.SelectorIndex(selector)
    }
    fn NextSelector(&self, selector: &CSSSelector) -> Option<&CSSSelector> {
        self.Next(self.SelectorIndex(selector)).map(|s| s.First())
    }
    fn SelectorsText(&self) -> String {
        self.SelectorsText()
    }
}
