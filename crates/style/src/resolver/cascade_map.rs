// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/resolver/cascade_map.h
// cpp: third_party/blink/renderer/core/css/resolver/cascade_map.cc

use super::cascade_origin::CascadeOrigin;
use super::cascade_priority::CascadePriority;
use crate::css_property_name::CSSPropertyName;
use crate::properties::css_property::CSSProperty;
use foundation::{kNotFound, kNumCSSProperties, AtomicString, CSSBitset, CSSPropertyID, WtfSizeT};
use std::collections::HashMap;
use std::ops::Deref;

// cpp: cascade_map.h:119-127
#[derive(Clone, Copy, Debug)]
pub(crate) struct CascadePriorityNode {
    priority: CascadePriority,
    next_index: WtfSizeT,
}

impl CascadePriorityNode {
    fn new(priority: CascadePriority, next_index: WtfSizeT) -> Self {
        Self {
            priority,
            next_index,
        }
    }
}

// cpp: cascade_map.h:129-132
// C++ uses inline capacity kNumCSSProperties * 2 as an allocation optimization;
// Rust retains the same node ordering and indices in its Vec backing store.
pub(crate) type CascadePriorityBackingVector = Vec<CascadePriorityNode>;

// cpp: cascade_map.h:116-176
#[derive(Clone, Copy, Debug)]
pub struct CascadePriorityList {
    head_index_: WtfSizeT,
}

impl Default for CascadePriorityList {
    // cpp: cascade_map.h:135,175
    fn default() -> Self {
        Self {
            head_index_: kNotFound,
        }
    }
}

impl CascadePriorityList {
    // cpp: cascade_map.h:136-140
    pub(crate) fn FromPriority(
        backing_vector: &mut CascadePriorityBackingVector,
        priority: CascadePriority,
    ) -> Self {
        let head_index_ =
            WtfSizeT::try_from(backing_vector.len()).expect("WTF vector size exceeds 32 bits");
        backing_vector.push(CascadePriorityNode::new(priority, kNotFound));
        Self { head_index_ }
    }

    // cpp: cascade_map.h:253-260
    pub(crate) fn Begin<'a>(
        &self,
        backing_vector: &'a CascadePriorityBackingVector,
    ) -> CascadePriorityListIterator<'a> {
        if self.head_index_ == kNotFound {
            return CascadePriorityListIterator::new(backing_vector, kNotFound);
        }
        // The C++ implementation indexes the vector to obtain the node pointer.
        let _ = &backing_vector[self.head_index_ as usize];
        CascadePriorityListIterator::new(backing_vector, self.head_index_)
    }

    // cpp: cascade_map.h:262-266
    pub(crate) fn End<'a>(
        &self,
        backing_vector: &'a CascadePriorityBackingVector,
    ) -> CascadePriorityListIterator<'a> {
        CascadePriorityListIterator::new(backing_vector, kNotFound)
    }

    // cpp: cascade_map.h:268-272
    pub(crate) fn Top<'a>(
        &self,
        backing_vector: &'a CascadePriorityBackingVector,
    ) -> &'a CascadePriority {
        debug_assert!(!self.IsEmpty());
        &backing_vector[self.head_index_ as usize].priority
    }

    // cpp: cascade_map.h:274-278
    pub(crate) fn TopMut<'a>(
        &self,
        backing_vector: &'a mut CascadePriorityBackingVector,
    ) -> &'a mut CascadePriority {
        debug_assert!(!self.IsEmpty());
        &mut backing_vector[self.head_index_ as usize].priority
    }

    // cpp: cascade_map.h:280-284
    pub(crate) fn Push(
        &mut self,
        backing_vector: &mut CascadePriorityBackingVector,
        priority: CascadePriority,
    ) {
        backing_vector.push(CascadePriorityNode::new(priority, self.head_index_));
        self.head_index_ =
            WtfSizeT::try_from(backing_vector.len() - 1).expect("WTF vector size exceeds 32 bits");
    }

    // cpp: cascade_map.h:286-321
    pub(crate) fn InsertKeepingSorted(
        &mut self,
        backing_vector: &mut CascadePriorityBackingVector,
        priority: CascadePriority,
    ) {
        let mut prev_index = kNotFound;
        let mut curr_index = self.head_index_;
        while curr_index != kNotFound {
            let curr_node = &backing_vector[curr_index as usize];
            if priority >= curr_node.priority {
                break;
            }
            prev_index = curr_index;
            curr_index = curr_node.next_index;
        }
        let new_index =
            WtfSizeT::try_from(backing_vector.len()).expect("WTF vector size exceeds 32 bits");
        backing_vector.push(CascadePriorityNode::new(priority, curr_index));
        if prev_index == kNotFound {
            self.head_index_ = new_index;
        } else {
            backing_vector[prev_index as usize].next_index = new_index;
        }
    }

    // cpp: cascade_map.h:323-325
    pub const fn IsEmpty(&self) -> bool {
        self.head_index_ == kNotFound
    }
}

// cpp: cascade_map.h:142-159
// The C++ iterator keeps a node pointer; the Rust iterator keeps its index in
// the same immutable backing vector. Its lifetime prevents vector mutation
// from invalidating that node while the iterator is used.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CascadePriorityListIterator<'a> {
    backing_vector_: &'a CascadePriorityBackingVector,
    backing_node_index_: WtfSizeT,
}

impl<'a> CascadePriorityListIterator<'a> {
    // cpp: cascade_map.h:221-224
    fn new(backing_vector: &'a CascadePriorityBackingVector, node_index: WtfSizeT) -> Self {
        Self {
            backing_vector_: backing_vector,
            backing_node_index_: node_index,
        }
    }

    // cpp: cascade_map.h:236-244
    pub(crate) fn Advance(&mut self) -> &mut Self {
        let next_index = self.backing_vector_[self.backing_node_index_ as usize].next_index;
        if next_index == kNotFound {
            self.backing_node_index_ = kNotFound;
        } else {
            let _ = &self.backing_vector_[next_index as usize];
            self.backing_node_index_ = next_index;
        }
        self
    }
}

// cpp: cascade_map.h:226-234
impl Deref for CascadePriorityListIterator<'_> {
    type Target = CascadePriority;
    fn deref(&self) -> &Self::Target {
        &self.backing_vector_[self.backing_node_index_ as usize].priority
    }
}

// cpp: cascade_map.h:246-251
impl PartialEq for CascadePriorityListIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        debug_assert!(std::ptr::eq(self.backing_vector_, other.backing_vector_));
        self.backing_node_index_ == other.backing_node_index_
    }
}
impl Eq for CascadePriorityListIterator<'_> {}

// cpp: cascade_map.h:26-105,178-217
// Chromium avoids constructing 824 empty lists by placing them in raw aligned
// storage and tracking live entries in CSSBitset. Rust uses Option for the same
// live-entry distinction; this changes storage mechanics, not cascade logic.
pub struct CascadeMap {
    inline_style_lost_: bool,
    native_properties_: Vec<Option<CascadePriorityList>>,
    native_bits_: CSSBitset,
    custom_properties_: HashMap<AtomicString, CascadePriorityList>,
    backing_vector_: CascadePriorityBackingVector,
    important_set_: Option<CSSBitset>,
    important_set_released_: bool,
}

impl Default for CascadeMap {
    fn default() -> Self {
        Self {
            inline_style_lost_: false,
            native_properties_: vec![None; kNumCSSProperties as usize],
            native_bits_: CSSBitset::new(),
            custom_properties_: HashMap::new(),
            backing_vector_: Vec::new(),
            important_set_: None,
            important_set_released_: false,
        }
    }
}

impl CascadeMap {
    pub fn new() -> Self {
        Self::default()
    }

    // cpp: cascade_map.cc:20-25
    pub fn At(&self, name: &CSSPropertyName) -> CascadePriority {
        self.Find(name).copied().unwrap_or_default()
    }

    // cpp: cascade_map.cc:27-42
    pub fn Find(&self, name: &CSSPropertyName) -> Option<&CascadePriority> {
        if name.IsCustomProperty() {
            return self
                .custom_properties_
                .get(name.ToAtomicString())
                .map(|list| list.Top(&self.backing_vector_));
        }
        let index = name.Id() as usize;
        debug_assert!(index < kNumCSSProperties as usize);
        self.native_properties_[index]
            .as_ref()
            .map(|list| list.Top(&self.backing_vector_))
    }

    // Mutable counterpart of the C++ Find overload (cascade_map.cc:44-47).
    pub fn FindMut(&mut self, name: &CSSPropertyName) -> Option<&mut CascadePriority> {
        let head = if name.IsCustomProperty() {
            self.custom_properties_
                .get(name.ToAtomicString())?
                .head_index_
        } else {
            let index = name.Id() as usize;
            debug_assert!(index < kNumCSSProperties as usize);
            self.native_properties_[index].as_ref()?.head_index_
        };
        Some(&mut self.backing_vector_[head as usize].priority)
    }

    // cpp: cascade_map.cc:49-75
    pub fn FindForOrigin(
        &self,
        name: &CSSPropertyName,
        origin: CascadeOrigin,
    ) -> Option<&CascadePriority> {
        let list = self.ListForName(name)?;
        let mut index = list.head_index_;
        while index != kNotFound {
            let node = &self.backing_vector_[index as usize];
            if origin as u8 >= node.priority.GetOrigin() as u8 {
                return Some(&node.priority);
            }
            index = node.next_index;
        }
        None
    }

    // cpp: cascade_map.cc:78-102
    pub fn FindRevertLayer(
        &self,
        name: &CSSPropertyName,
        revert_from: u64,
    ) -> Option<&CascadePriority> {
        let list = self.ListForName(name)?;
        let mut index = list.head_index_;
        while index != kNotFound {
            let node = &self.backing_vector_[index as usize];
            if node.priority.ForLayerComparison() < revert_from {
                return Some(&node.priority);
            }
            index = node.next_index;
        }
        None
    }

    // cpp: cascade_map.cc:105-133
    pub fn FindRevertRule(
        &self,
        name: &CSSPropertyName,
        revert_from: CascadePriority,
    ) -> Option<&CascadePriority> {
        let list = self.ListForName(name)?;
        let mut index = list.head_index_;
        while index != kNotFound {
            let node = &self.backing_vector_[index as usize];
            if node.priority < revert_from
                && node.priority.GetRuleIndex() != revert_from.GetRuleIndex()
            {
                return Some(&node.priority);
            }
            index = node.next_index;
        }
        None
    }

    // cpp: cascade_map.cc:75-77
    pub fn Top(&mut self, list: &CascadePriorityList) -> &mut CascadePriority {
        list.TopMut(&mut self.backing_vector_)
    }

    // cpp: cascade_map.cc:136-147
    pub fn AddCustom(&mut self, custom_property_name: AtomicString, priority: CascadePriority) {
        let list = self
            .custom_properties_
            .entry(custom_property_name)
            .or_default();
        if list.IsEmpty() {
            list.Push(&mut self.backing_vector_, priority);
            return;
        }
        Self::AddToList(
            &mut self.inline_style_lost_,
            list,
            &mut self.backing_vector_,
            priority,
        );
    }

    // cpp: cascade_map.cc:149-181
    pub fn Add(&mut self, id: CSSPropertyID, priority: CascadePriority) {
        debug_assert_ne!(id, CSSPropertyID::kInvalid);
        debug_assert_ne!(id, CSSPropertyID::kVariable);
        debug_assert!(!CSSProperty::Get(id).IsSurrogate());
        let index = id as usize;
        debug_assert!(index < kNumCSSProperties as usize);

        if priority.IsImportant() {
            let important_set = self.important_set_.get_or_insert_with(CSSBitset::new);
            let unvisited_id = CSSProperty::UnvisitedID(index);
            important_set.Set(if unvisited_id == CSSPropertyID::kInvalid {
                id
            } else {
                unvisited_id
            });
        }

        if !self.native_bits_.Has(id) {
            self.native_bits_.Set(id);
            self.native_properties_[index] = Some(CascadePriorityList::FromPriority(
                &mut self.backing_vector_,
                priority,
            ));
            return;
        }
        let list = self.native_properties_[index]
            .as_mut()
            .expect("native bit and list must agree");
        Self::AddToList(
            &mut self.inline_style_lost_,
            list,
            &mut self.backing_vector_,
            priority,
        );
    }

    // cpp: cascade_map.cc:183-207
    fn AddToList(
        inline_style_lost: &mut bool,
        list: &mut CascadePriorityList,
        backing_vector: &mut CascadePriorityBackingVector,
        priority: CascadePriority,
    ) {
        let top = *list.Top(backing_vector);
        debug_assert!(priority.ForLayerComparison() >= top.ForLayerComparison());
        if top >= priority {
            if priority.IsInlineStyle() {
                *inline_style_lost = true;
            }
            list.InsertKeepingSorted(backing_vector, priority);
            return;
        }
        if top.IsInlineStyle() {
            *inline_style_lost = true;
        }
        list.Push(backing_vector, priority);
    }

    // cpp: cascade_map.cc:209-218
    pub fn Reset(&mut self) {
        self.inline_style_lost_ = false;
        self.native_bits_.Reset();
        self.native_properties_.fill(None);
        self.custom_properties_.clear();
        self.backing_vector_.clear();
        self.important_set_ = None;
        self.important_set_released_ = false;
    }

    // cpp: cascade_map.cc:220-224
    pub fn ClearAppliedFlags(&mut self) {
        for node in &mut self.backing_vector_ {
            node.priority = CascadePriority::WithAlreadyApplied(node.priority, false);
        }
    }

    // cpp: cascade_map.h:79-92,94-100,203-204
    pub fn HighPriorityBits(&self) -> u64 {
        self.native_bits_.HighPriorityBits()
    }

    pub fn ReleaseImportantSet(&mut self) -> Option<CSSBitset> {
        debug_assert!(!self.important_set_released_);
        self.important_set_released_ = true;
        self.important_set_.take()
    }

    pub const fn InlineStyleLost(&self) -> bool {
        self.inline_style_lost_
    }

    pub const fn NativeBitset(&self) -> &CSSBitset {
        &self.native_bits_
    }

    pub const fn GetCustomMap(&self) -> &HashMap<AtomicString, CascadePriorityList> {
        &self.custom_properties_
    }

    pub fn GetCustomMapMut(&mut self) -> &mut HashMap<AtomicString, CascadePriorityList> {
        &mut self.custom_properties_
    }

    fn ListForName(&self, name: &CSSPropertyName) -> Option<&CascadePriorityList> {
        if name.IsCustomProperty() {
            self.custom_properties_.get(name.ToAtomicString())
        } else {
            self.native_properties_[name.Id() as usize].as_ref()
        }
    }
}

#[cfg(test)]
#[path = "cascade_map_test.rs"]
mod tests;
