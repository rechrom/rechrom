/*
 * Copyright (C) 2014 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Source ledger: comments and blank lines stripped; braces retained.
// Effective = mapped + omitted + production pending. All non-omitted
// effective source lines are mapped; omissions are listed exactly below.
// Omitted: includes/guards/access/forward/cast/allocation scaffolding,
// deleted copy/move/default destructors, GC Trace and inspector tracing.
// invalidation_set.h: physical 651, effective 383, mapped 311,
// omitted 72, production pending 0.
// invalidation_set.h omitted effective lines:31-32,34,36-44,46,48-49,57,59-61,97-100,188,258-259,265,267-268
// 285,289,292,317,346,355,360,373,382,385-387,389-392,463,468,474,501,505,547,552,626-631,633-638,640-645,649
// 651
// invalidation_set.cc: physical 607, effective 485, mapped 394,
// omitted 91, production pending 0.
// invalidation_set.cc omitted effective lines:31,33-34,36-44,46,48,66,68-72,75-77,122-123,128-130,135-137
// 143-144,153-154,162-163,169-170,181-183,211,292-299,409,431,449-450,452,454-471,473-475,477-479,481-483
// 485-487,489-492,607
// cpp: third_party/blink/renderer/core/css/invalidation/invalidation_set.h:51-55,248-380,563-624
// cpp: third_party/blink/renderer/core/css/invalidation/invalidation_set.cc:50-64
// Complete operational mapping, including descendant/sibling/nth storage,
// matching, combination and singleton sets. Rust Rc replaces scoped_refptr;
// the DOM owns element data through the required InvalidationElement adapter.
// TRACE/inspector hooks and allocation/casting scaffolding
// are omitted; no production invalidation logic remains pending.

use super::invalidation_flags::InvalidationFlags;
use foundation::{g_null_atom, AtomicString, HashSet};
use std::cell::{RefCell, RefMut};
use std::collections::hash_set;
use std::ops::Deref;
use std::rc::Rc;

// cpp: invalidation_set.h:51-55
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum InvalidationType {
    kInvalidateDescendants,
    kInvalidateSiblings,
    kInvalidateNthSiblings,
}

// cpp: invalidation_set.h:248-256
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BackingType {
    kClasses,
    kIds,
    kTagNames,
    kCustomPseudoNames,
    kAttributes,
}

// cpp: invalidation_set.h:264-269
#[derive(Clone, Default)]
pub struct BackingFlags {
    bits_: u8,
}

// Rust's active enum variant safely owns the C++ union's active allocation.
// The external BackingFlags bits retain their original shared semantics.
#[derive(Clone)]
enum BackingStorage {
    String(AtomicString),
    HashSet(Box<HashSet<AtomicString>>),
}

// cpp: invalidation_set.h:283-380
// TYPE is the original BackingType template's bit index, not a replacement
// interface for a missing dependency. AtomicString and HashSet are mapped
// foundation types.
#[derive(Clone)]
pub struct Backing<const TYPE: u8> {
    storage_: BackingStorage,
}

impl<const TYPE: u8> Default for Backing<TYPE> {
    fn default() -> Self {
        let _ = Self::CHECK_TYPE;
        Self {
            storage_: BackingStorage::String(AtomicString::default()),
        }
    }
}

impl<const TYPE: u8> Backing<TYPE> {
    // cpp: invalidation_set.h:286-287
    const CHECK_TYPE: () = assert!(TYPE < u8::BITS as u8, "Enough bits in BackingFlags");

    // cpp: invalidation_set.h:563-582
    pub fn Add(&mut self, flags: &mut BackingFlags, string: &AtomicString) {
        debug_assert!(!string.IsNull());
        if self.IsHashSet(flags) {
            match &mut self.storage_ {
                BackingStorage::HashSet(set) => {
                    set.insert(string.clone());
                }
                _ => panic!("backing flags do not describe the active storage"),
            }
        } else {
            let BackingStorage::String(existing) = &self.storage_ else {
                panic!("backing flags do not describe the active storage");
            };
            if !existing.IsNull() {
                if existing == string {
                    return;
                }
                let old = std::mem::replace(
                    &mut self.storage_,
                    BackingStorage::String(AtomicString::default()),
                );
                let BackingStorage::String(atomic_string) = old else {
                    unreachable!()
                };
                let mut set = Box::new(HashSet::new());
                set.insert(atomic_string);
                set.insert(string.clone());
                self.storage_ = BackingStorage::HashSet(set);
                Self::SetIsHashSet(flags);
            } else {
                self.storage_ = BackingStorage::String(string.clone());
            }
        }
    }

    // cpp: invalidation_set.h:584-596
    pub fn Clear(&mut self, flags: &mut BackingFlags) {
        // Replacing the active variant deletes a HashSet or drops an atom,
        // then reconstructs the null AtomicString just as Clear() does in C++.
        self.storage_ = BackingStorage::String(AtomicString::default());
        Self::SetIsString(flags);
    }

    // cpp: invalidation_set.h:598-606
    pub fn Contains(&self, flags: &BackingFlags, string: &AtomicString) -> bool {
        if let Some(set) = self.GetHashSet(flags) {
            return set.Contains(string);
        }
        string == self.GetString(flags).expect("string backing")
    }

    // cpp: invalidation_set.h:608-612
    pub fn IsEmpty(&self, flags: &BackingFlags) -> bool {
        !self.IsHashSet(flags) && self.GetString(flags).expect("string backing").IsNull()
    }

    // cpp: invalidation_set.h:614-624
    pub fn Size(&self, flags: &BackingFlags) -> usize {
        if let Some(set) = self.GetHashSet(flags) {
            return set.size() as usize;
        }
        // C++ GetString() returns the string slot even when its atom is null.
        if self.GetString(flags).is_some() {
            return 1;
        }
        0
    }

    // cpp: invalidation_set.h:305
    pub const fn IsHashSet(&self, flags: &BackingFlags) -> bool {
        flags.bits_ & Self::GetMask() != 0
    }

    // cpp: invalidation_set.h:307-312
    pub fn GetString(&self, flags: &BackingFlags) -> Option<&AtomicString> {
        if self.IsHashSet(flags) {
            return None;
        }
        match &self.storage_ {
            BackingStorage::String(value) => Some(value),
            _ => panic!("backing flags do not describe the active storage"),
        }
    }
    pub fn GetHashSet(&self, flags: &BackingFlags) -> Option<&HashSet<AtomicString>> {
        if !self.IsHashSet(flags) {
            return None;
        }
        match &self.storage_ {
            BackingStorage::HashSet(value) => Some(value),
            _ => panic!("backing flags do not describe the active storage"),
        }
    }

    // cpp: invalidation_set.h:365-371
    pub fn Items(&self, flags: &BackingFlags) -> BackingRange<'_> {
        if let Some(set) = self.GetHashSet(flags) {
            BackingRange::new(
                BackingIterator::FromHashSet(set.iter()),
                BackingIterator::HashSetEnd(),
            )
        } else {
            BackingRange::new(
                BackingIterator::FromString(self.GetString(flags).expect("string backing")),
                BackingIterator::FromString(&g_null_atom),
            )
        }
    }

    // cpp: invalidation_set.h:374-376
    const fn GetMask() -> u8 {
        1u8 << TYPE
    }
    fn SetIsString(flags: &mut BackingFlags) {
        flags.bits_ &= !Self::GetMask();
    }
    fn SetIsHashSet(flags: &mut BackingFlags) {
        flags.bits_ |= Self::GetMask();
    }
}

// cpp: invalidation_set.h:316-352
// Borrowed atoms/iterators retain the original values and prevent mutation of
// a backing while a range refers to it. This replaces the C++ atom copies and
// hash table cursors without changing iteration or equality semantics.
#[derive(Clone)]
pub enum BackingIterator<'a> {
    String(&'a AtomicString),
    HashSet {
        current: Option<&'a AtomicString>,
        remaining: Option<hash_set::Iter<'a, AtomicString>>,
    },
}

impl<'a> BackingIterator<'a> {
    // cpp: invalidation_set.h:320-323
    pub fn FromString(string: &'a AtomicString) -> Self {
        Self::String(string)
    }
    pub fn FromHashSet(mut iterator: hash_set::Iter<'a, AtomicString>) -> Self {
        Self::HashSet {
            current: iterator.next(),
            remaining: Some(iterator),
        }
    }
    pub fn HashSetEnd() -> Self {
        Self::HashSet {
            current: None,
            remaining: None,
        }
    }

    // cpp: invalidation_set.h:334-340
    pub fn Advance(&mut self) {
        match self {
            Self::String(string) => *string = &g_null_atom,
            Self::HashSet { current, remaining } => {
                *current = remaining
                    .as_mut()
                    .expect("cannot advance a hash set end iterator")
                    .next();
            }
        }
    }

    // cpp: invalidation_set.h:342-344
    fn Value(&self) -> &'a AtomicString {
        match self {
            Self::String(string) => string,
            Self::HashSet { current, .. } => {
                current.expect("cannot dereference a hash set end iterator")
            }
        }
    }
}

// cpp: invalidation_set.h:325-333
impl PartialEq for BackingIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(a), Self::String(b)) => a == b,
            (Self::HashSet { current: a, .. }, Self::HashSet { current: b, .. }) => match (a, b) {
                (Some(a), Some(b)) => std::ptr::eq(*a, *b),
                (None, None) => true,
                _ => false,
            },
            _ => false,
        }
    }
}
impl Eq for BackingIterator<'_> {}
impl Deref for BackingIterator<'_> {
    type Target = AtomicString;
    fn deref(&self) -> &Self::Target {
        self.Value()
    }
}

impl<'a> Iterator for BackingIterator<'a> {
    type Item = &'a AtomicString;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::String(value) if value.IsNull() => return None,
            Self::HashSet { current: None, .. } => return None,
            _ => {}
        }
        let value = self.Value();
        self.Advance();
        Some(value)
    }
}

// cpp: invalidation_set.h:354-363
#[derive(Clone)]
pub struct BackingRange<'a> {
    begin_: BackingIterator<'a>,
    end_: BackingIterator<'a>,
}
impl<'a> BackingRange<'a> {
    pub fn new(begin: BackingIterator<'a>, end: BackingIterator<'a>) -> Self {
        Self {
            begin_: begin,
            end_: end,
        }
    }
    pub fn begin(&self) -> BackingIterator<'a> {
        self.begin_.clone()
    }
    pub fn end(&self) -> BackingIterator<'a> {
        self.end_.clone()
    }
}
impl<'a> IntoIterator for BackingRange<'a> {
    type Item = &'a AtomicString;
    type IntoIter = BackingIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.begin_
    }
}

// cpp: invalidation_set.cc:50-64
pub(crate) fn BackingEqual<const TYPE: u8>(
    a_flags: &BackingFlags,
    a: &Backing<TYPE>,
    b_flags: &BackingFlags,
    b: &Backing<TYPE>,
) -> bool {
    if a.Size(a_flags) != b.Size(b_flags) {
        return false;
    }
    for value in a.Items(a_flags) {
        if !b.Contains(b_flags, value) {
            return false;
        }
    }
    true
}

/// Required Blink Element/ComputedStyle/ContainerQueryEvaluator reads.
/// PseudoElementStylesDependOnSiblingFunctions must examine all cached pseudo
/// styles just as PseudoElementStylesDependOnFunc does in the C++ owner.
pub trait InvalidationElement {
    fn LocalNameForSelectorMatching(&self) -> &AtomicString;
    fn IdForStyleResolution(&self) -> Option<&AtomicString>;
    fn ClassNames(&self) -> &[AtomicString];
    fn ShadowPseudoId(&self) -> &AtomicString;
    fn HasAttributes(&self) -> bool;
    fn HasAttributeIgnoringNamespace(&self, name: &AtomicString) -> bool;
    fn HasPart(&self) -> bool;
    fn HasComputedStyle(&self) -> bool;
    fn HasSiblingFunctions(&self) -> bool;
    fn PseudoElementStylesDependOnSiblingFunctions(&self) -> bool;
    fn ContainerQueryDependsOnTreeCounting(&self) -> bool;
}

// cpp: invalidation_set.h:97-246,382-563
// A tagged owner replaces C++ nonvirtual inheritance. Only sibling variants
// carry the corresponding state; callers cannot create a mismatched tag.
#[derive(Clone)]
pub struct InvalidationSet {
    type_: InvalidationType,
    classes_: Backing<0>,
    ids_: Backing<1>,
    tag_names_: Backing<2>,
    custom_pseudo_names_: Backing<3>,
    attributes_: Backing<4>,
    invalidation_flags_: InvalidationFlags,
    backing_flags_: BackingFlags,
    invalidates_self_: bool,
    invalidates_nth_: bool,
    sibling_: Option<SiblingState>,
}

#[derive(Clone)]
struct SiblingState {
    max_direct_adjacent_selectors_: u32,
    sibling_descendant_invalidation_set_: Option<InvalidationSetRef>,
    descendant_invalidation_set_: Option<InvalidationSetRef>,
}

pub type InvalidationSetRef = Rc<RefCell<InvalidationSet>>;
pub type InvalidationSetVector = Vec<InvalidationSetRef>;
#[derive(Clone, Default)]
pub struct InvalidationLists {
    pub descendants: InvalidationSetVector,
    pub siblings: InvalidationSetVector,
}

// Factory names retain C++ call sites while sharing the tagged Rc owner.
pub struct DescendantInvalidationSet;
pub struct SiblingInvalidationSet;
pub struct NthSiblingInvalidationSet;
impl DescendantInvalidationSet {
    pub fn Create() -> InvalidationSetRef {
        Rc::new(RefCell::new(InvalidationSet::new(
            InvalidationType::kInvalidateDescendants,
            None,
        )))
    }
}
impl SiblingInvalidationSet {
    pub const kDirectAdjacentMax: u32 = u32::MAX;
    pub fn Create(descendants: Option<InvalidationSetRef>) -> InvalidationSetRef {
        debug_assert!(descendants
            .as_ref()
            .is_none_or(|d| d.borrow().IsDescendantInvalidationSet()));
        Rc::new(RefCell::new(InvalidationSet::new(
            InvalidationType::kInvalidateSiblings,
            descendants,
        )))
    }
}
impl NthSiblingInvalidationSet {
    pub fn Create() -> InvalidationSetRef {
        Rc::new(RefCell::new(InvalidationSet::new(
            InvalidationType::kInvalidateNthSiblings,
            None,
        )))
    }
}

thread_local! {
    // cpp: invalidation_set.cc:411-447; Rust style owners are Rc/thread-local.
    static SELF_SET: InvalidationSetRef = {
        let set = DescendantInvalidationSet::Create();
        set.borrow_mut().SetInvalidatesSelf();
        set
    };
    static PART_SET: InvalidationSetRef = {
        let set = DescendantInvalidationSet::Create();
        let mut value = set.borrow_mut();
        value.SetInvalidatesParts();
        value.SetTreeBoundaryCrossing();
        drop(value);
        set
    };
    static TREE_COUNTING_SET: InvalidationSetRef = {
        let set = NthSiblingInvalidationSet::Create();
        let mut value = set.borrow_mut();
        value.SetInvalidatesTreeCounting();
        value.SetInvalidatesSelf();
        drop(value);
        set
    };
}

impl InvalidationSet {
    /// `Combine` for shared handles, including the permitted case where both
    /// handles point to the SelfInvalidationSet singleton.
    pub fn CombineSets(target: &InvalidationSetRef, other: &InvalidationSetRef) {
        if Rc::ptr_eq(target, other) {
            assert!(
                target.borrow().IsSelfInvalidationSet(),
                "cannot combine a set with itself"
            );
            return;
        }
        target.borrow_mut().Combine(&other.borrow());
    }
    // cpp: invalidation_set.cc:114-118,579-587
    fn new(type_: InvalidationType, descendants: Option<InvalidationSetRef>) -> Self {
        Self {
            type_,
            classes_: Backing::default(),
            ids_: Backing::default(),
            tag_names_: Backing::default(),
            custom_pseudo_names_: Backing::default(),
            attributes_: Backing::default(),
            invalidation_flags_: InvalidationFlags::default(),
            backing_flags_: BackingFlags::default(),
            invalidates_self_: false,
            invalidates_nth_: false,
            sibling_: (type_ != InvalidationType::kInvalidateDescendants).then(|| SiblingState {
                max_direct_adjacent_selectors_: if type_ == InvalidationType::kInvalidateNthSiblings
                {
                    u32::MAX
                } else {
                    1
                },
                sibling_descendant_invalidation_set_: None,
                descendant_invalidation_set_: descendants,
            }),
        }
    }

    pub fn GetType(&self) -> InvalidationType {
        self.type_
    }
    pub fn IsDescendantInvalidationSet(&self) -> bool {
        self.type_ == InvalidationType::kInvalidateDescendants
    }
    pub fn IsSiblingInvalidationSet(&self) -> bool {
        !self.IsDescendantInvalidationSet()
    }
    pub fn IsNthSiblingInvalidationSet(&self) -> bool {
        self.type_ == InvalidationType::kInvalidateNthSiblings
    }
    pub fn SetInvalidationFlags(&mut self, flags: InvalidationFlags) {
        self.invalidation_flags_ = flags;
    }
    pub fn GetInvalidationFlags(&self) -> InvalidationFlags {
        self.invalidation_flags_
    }
    pub fn SetInvalidatesSelf(&mut self) {
        self.invalidates_self_ = true;
    }
    pub fn InvalidatesSelf(&self) -> bool {
        self.invalidates_self_
    }
    pub fn SetInvalidatesNth(&mut self) {
        debug_assert!(!self.IsSelfInvalidationSet());
        self.invalidates_nth_ = true;
    }
    pub fn InvalidatesNth(&self) -> bool {
        self.invalidates_nth_
    }
    pub fn WholeSubtreeInvalid(&self) -> bool {
        self.invalidation_flags_.WholeSubtreeInvalid()
    }
    pub fn SetTreeBoundaryCrossing(&mut self) {
        self.invalidation_flags_.SetTreeBoundaryCrossing(true);
    }
    pub fn TreeBoundaryCrossing(&self) -> bool {
        self.invalidation_flags_.TreeBoundaryCrossing()
    }
    pub fn SetInsertionPointCrossing(&mut self) {
        self.invalidation_flags_.SetInsertionPointCrossing(true);
    }
    pub fn InsertionPointCrossing(&self) -> bool {
        self.invalidation_flags_.InsertionPointCrossing()
    }
    pub fn SetInvalidatesSlotted(&mut self) {
        self.invalidation_flags_.SetInvalidatesSlotted(true);
    }
    pub fn InvalidatesSlotted(&self) -> bool {
        self.invalidation_flags_.InvalidatesSlotted()
    }
    pub fn SetInvalidatesParts(&mut self) {
        self.invalidation_flags_.SetInvalidatesParts(true);
    }
    pub fn InvalidatesParts(&self) -> bool {
        self.invalidation_flags_.InvalidatesParts()
    }
    pub fn SetInvalidatesTreeCounting(&mut self) {
        self.invalidation_flags_.SetInvalidatesTreeCounting(true);
    }
    pub fn InvalidatesTreeCounting(&self) -> bool {
        self.invalidation_flags_.InvalidatesTreeCounting()
    }
    pub fn IsEmpty(&self) -> bool {
        self.HasEmptyBackings()
            && !self.InsertionPointCrossing()
            && !self.InvalidatesSlotted()
            && !self.InvalidatesParts()
            && !self.InvalidatesTreeCounting()
    }

    pub fn HasClasses(&self) -> bool {
        !self.classes_.IsEmpty(&self.backing_flags_)
    }
    pub fn HasIds(&self) -> bool {
        !self.ids_.IsEmpty(&self.backing_flags_)
    }
    pub fn HasTagNames(&self) -> bool {
        !self.tag_names_.IsEmpty(&self.backing_flags_)
    }
    pub fn HasCustomPseudoNames(&self) -> bool {
        !self.custom_pseudo_names_.IsEmpty(&self.backing_flags_)
    }
    pub fn HasAttributes(&self) -> bool {
        !self.attributes_.IsEmpty(&self.backing_flags_)
    }
    pub fn HasId(&self, value: &AtomicString) -> bool {
        self.ids_.Contains(&self.backing_flags_, value)
    }
    pub fn HasTagName(&self, value: &AtomicString) -> bool {
        self.tag_names_.Contains(&self.backing_flags_, value)
    }
    pub fn HasCustomPseudoName(&self, value: &AtomicString) -> bool {
        self.custom_pseudo_names_
            .Contains(&self.backing_flags_, value)
    }
    pub fn Classes(&self) -> BackingRange<'_> {
        self.classes_.Items(&self.backing_flags_)
    }
    pub fn Ids(&self) -> BackingRange<'_> {
        self.ids_.Items(&self.backing_flags_)
    }
    pub fn TagNames(&self) -> BackingRange<'_> {
        self.tag_names_.Items(&self.backing_flags_)
    }
    pub fn CustomPseudoNames(&self) -> BackingRange<'_> {
        self.custom_pseudo_names_.Items(&self.backing_flags_)
    }
    pub fn Attributes(&self) -> BackingRange<'_> {
        self.attributes_.Items(&self.backing_flags_)
    }

    // cpp: invalidation_set.cc:319-354
    pub fn FindAnyClass<E: InvalidationElement>(&self, element: &E) -> Option<&AtomicString> {
        if let Some(string) = self.classes_.GetString(&self.backing_flags_) {
            if element.ClassNames().iter().any(|name| name == string) {
                return Some(string);
            }
        }
        if let Some(set) = self.classes_.GetHashSet(&self.backing_flags_) {
            for name in element.ClassNames() {
                if let Some(item) = set.get(name) {
                    return Some(item);
                }
            }
        }
        None
    }
    pub fn FindAnyAttribute<E: InvalidationElement>(&self, element: &E) -> Option<&AtomicString> {
        if let Some(string) = self.attributes_.GetString(&self.backing_flags_) {
            if element.HasAttributeIgnoringNamespace(string) {
                return Some(string);
            }
        }
        if let Some(set) = self.attributes_.GetHashSet(&self.backing_flags_) {
            for name in set.iter() {
                if element.HasAttributeIgnoringNamespace(name) {
                    return Some(name);
                }
            }
        }
        None
    }

    // cpp: invalidation_set.cc:120-195; trace-only selector reasons omitted.
    pub fn InvalidatesElement<E: InvalidationElement>(&self, element: &E) -> bool {
        if self.WholeSubtreeInvalid() {
            return true;
        }
        if self.HasTagNames() && self.HasTagName(element.LocalNameForSelectorMatching()) {
            return true;
        }
        if let Some(id) = element.IdForStyleResolution() {
            if self.HasIds() && self.HasId(id) {
                return true;
            }
        }
        if !element.ClassNames().is_empty()
            && self.HasClasses()
            && self.FindAnyClass(element).is_some()
        {
            return true;
        }
        if self.HasCustomPseudoNames()
            && !element.ShadowPseudoId().IsNull()
            && self.HasCustomPseudoName(element.ShadowPseudoId())
        {
            return true;
        }
        if element.HasAttributes()
            && self.HasAttributes()
            && self.FindAnyAttribute(element).is_some()
        {
            return true;
        }
        if element.HasPart() && self.InvalidatesParts() {
            return true;
        }
        if self.InvalidatesTreeCounting() && element.HasComputedStyle() {
            if element.HasSiblingFunctions()
                || element.PseudoElementStylesDependOnSiblingFunctions()
            {
                return true;
            }
            if element.ContainerQueryDependsOnTreeCounting() {
                return true;
            }
        }
        false
    }

    // cpp: invalidation_set.cc:301-317,356-407
    fn ClearAllBackings(&mut self) {
        self.classes_.Clear(&mut self.backing_flags_);
        self.ids_.Clear(&mut self.backing_flags_);
        self.tag_names_.Clear(&mut self.backing_flags_);
        self.custom_pseudo_names_.Clear(&mut self.backing_flags_);
        self.attributes_.Clear(&mut self.backing_flags_);
    }
    fn HasEmptyBackings(&self) -> bool {
        !self.HasClasses()
            && !self.HasIds()
            && !self.HasTagNames()
            && !self.HasCustomPseudoNames()
            && !self.HasAttributes()
    }
    pub fn AddClass(&mut self, value: &AtomicString) {
        if self.WholeSubtreeInvalid() {
            return;
        }
        assert!(!value.empty());
        self.classes_.Add(&mut self.backing_flags_, value);
    }
    pub fn AddId(&mut self, value: &AtomicString) {
        if self.WholeSubtreeInvalid() {
            return;
        }
        assert!(!value.empty());
        self.ids_.Add(&mut self.backing_flags_, value);
    }
    pub fn AddTagName(&mut self, value: &AtomicString) {
        if self.WholeSubtreeInvalid() {
            return;
        }
        assert!(!value.empty());
        self.tag_names_.Add(&mut self.backing_flags_, value);
    }
    pub fn AddCustomPseudoName(&mut self, value: &AtomicString) {
        if self.WholeSubtreeInvalid() {
            return;
        }
        assert!(!value.empty());
        self.custom_pseudo_names_
            .Add(&mut self.backing_flags_, value);
    }
    pub fn AddAttribute(&mut self, value: &AtomicString) {
        if self.WholeSubtreeInvalid() {
            return;
        }
        assert!(!value.empty());
        self.attributes_.Add(&mut self.backing_flags_, value);
    }
    pub fn SetWholeSubtreeInvalid(&mut self) {
        if self.WholeSubtreeInvalid() {
            return;
        }
        self.invalidation_flags_.SetWholeSubtreeInvalid(true);
        self.invalidation_flags_.SetTreeBoundaryCrossing(false);
        self.invalidation_flags_.SetInsertionPointCrossing(false);
        self.invalidation_flags_.SetInvalidatesSlotted(false);
        self.invalidation_flags_.SetInvalidatesParts(false);
        self.invalidation_flags_.SetInvalidatesTreeCounting(false);
        self.ClearAllBackings();
    }

    pub fn SelfInvalidationSet() -> InvalidationSetRef {
        SELF_SET.with(Clone::clone)
    }
    pub fn PartInvalidationSet() -> InvalidationSetRef {
        PART_SET.with(Clone::clone)
    }
    pub fn TreeCountingInvalidationSet() -> InvalidationSetRef {
        TREE_COUNTING_SET.with(Clone::clone)
    }
    pub fn IsSelfInvalidationSet(&self) -> bool {
        SELF_SET.with(|set| std::ptr::eq(self, set.as_ptr()))
    }
    pub fn MaxDirectAdjacentSelectors(&self) -> u32 {
        self.sibling_
            .as_ref()
            .expect("sibling set")
            .max_direct_adjacent_selectors_
    }
    pub fn UpdateMaxDirectAdjacentSelectors(&mut self, value: u32) {
        let state = self.sibling_.as_mut().expect("sibling set");
        state.max_direct_adjacent_selectors_ = state.max_direct_adjacent_selectors_.max(value);
    }
    pub fn SiblingDescendants(&self) -> Option<&InvalidationSetRef> {
        self.sibling_
            .as_ref()
            .expect("sibling set")
            .sibling_descendant_invalidation_set_
            .as_ref()
    }
    pub fn Descendants(&self) -> Option<&InvalidationSetRef> {
        self.sibling_
            .as_ref()
            .expect("sibling set")
            .descendant_invalidation_set_
            .as_ref()
    }
    // RefCell preserves scoped_refptr shared mutation and stable allocation identity.
    pub fn EnsureSiblingDescendants(&mut self) -> RefMut<'_, Self> {
        let set = self
            .sibling_
            .as_mut()
            .expect("sibling set")
            .sibling_descendant_invalidation_set_
            .get_or_insert_with(DescendantInvalidationSet::Create);
        set.borrow_mut()
    }
    pub fn EnsureDescendants(&mut self) -> RefMut<'_, Self> {
        let set = self
            .sibling_
            .as_mut()
            .expect("sibling set")
            .descendant_invalidation_set_
            .get_or_insert_with(DescendantInvalidationSet::Create);
        set.borrow_mut()
    }

    // cpp: invalidation_set.cc:197-290
    pub fn Combine(&mut self, other: &Self) {
        assert_eq!(self.GetType(), other.GetType());
        if self.IsSelfInvalidationSet() {
            debug_assert!(other.IsSelfInvalidationSet());
            return;
        }
        assert!(!std::ptr::eq(self, other));
        if self.IsSiblingInvalidationSet() {
            self.UpdateMaxDirectAdjacentSelectors(other.MaxDirectAdjacentSelectors());
            if let Some(descendants) = other.SiblingDescendants() {
                let target = self
                    .sibling_
                    .as_mut()
                    .unwrap()
                    .sibling_descendant_invalidation_set_
                    .get_or_insert_with(DescendantInvalidationSet::Create);
                Self::CombineSets(target, descendants);
            }
            if let Some(descendants) = other.Descendants() {
                let target = self
                    .sibling_
                    .as_mut()
                    .unwrap()
                    .descendant_invalidation_set_
                    .get_or_insert_with(DescendantInvalidationSet::Create);
                Self::CombineSets(target, descendants);
            }
        }
        if other.InvalidatesNth() {
            self.SetInvalidatesNth();
        }
        if other.InvalidatesSelf() {
            self.SetInvalidatesSelf();
            if other.IsSelfInvalidationSet() {
                return;
            }
        }
        if self.WholeSubtreeInvalid() {
            return;
        }
        if other.WholeSubtreeInvalid() {
            self.SetWholeSubtreeInvalid();
            return;
        }
        if other.TreeBoundaryCrossing() {
            self.SetTreeBoundaryCrossing();
        }
        if other.InsertionPointCrossing() {
            self.SetInsertionPointCrossing();
        }
        if other.InvalidatesSlotted() {
            self.SetInvalidatesSlotted();
        }
        if other.InvalidatesParts() {
            self.SetInvalidatesParts();
        }
        if other.InvalidatesTreeCounting() {
            self.SetInvalidatesTreeCounting();
        }
        for value in other.Classes() {
            self.AddClass(value);
        }
        for value in other.Ids() {
            self.AddId(value);
        }
        for value in other.TagNames() {
            self.AddTagName(value);
        }
        for value in other.CustomPseudoNames() {
            self.AddCustomPseudoName(value);
        }
        for value in other.Attributes() {
            self.AddAttribute(value);
        }
    }

    // cpp: invalidation_set.cc:494-577; sort by UTF-16 code units, including
    // non-BMP names, rather than by UTF-8 or atomic allocation addresses.
    pub fn ToString(&self) -> foundation::String {
        fn format_backing(range: BackingRange<'_>, prefix: &str, suffix: &str) -> Vec<u16> {
            let mut names: Vec<_> = range.into_iter().collect();
            names.sort_by(|a, b| a.utf16_units().cmp(&b.utf16_units()));
            let mut result = Vec::new();
            for name in names {
                if !result.is_empty() {
                    result.push(b' ' as u16);
                }
                result.extend(prefix.encode_utf16());
                result.extend_from_slice(name.utf16_units().unwrap_or_default());
                result.extend(suffix.encode_utf16());
            }
            result
        }
        let mut features = Vec::new();
        for (has, range, prefix, suffix) in [
            (self.HasIds(), self.Ids(), "#", ""),
            (self.HasClasses(), self.Classes(), ".", ""),
            (self.HasTagNames(), self.TagNames(), "", ""),
            (
                self.HasCustomPseudoNames(),
                self.CustomPseudoNames(),
                "",
                "",
            ),
            (self.HasAttributes(), self.Attributes(), "[", "]"),
        ] {
            if has {
                if !features.is_empty() {
                    features.push(b' ' as u16);
                }
                features.extend(format_backing(range, prefix, suffix));
            }
        }
        let mut metadata = Vec::new();
        for (flag, marker) in [
            (self.InvalidatesSelf(), '$'),
            (self.InvalidatesNth(), 'N'),
            (self.WholeSubtreeInvalid(), 'W'),
            (self.TreeBoundaryCrossing(), 'T'),
            (self.InsertionPointCrossing(), 'I'),
            (self.InvalidatesSlotted(), 'S'),
            (self.InvalidatesParts(), 'P'),
            (self.InvalidatesTreeCounting(), 't'),
        ] {
            if flag {
                metadata.push(marker as u16);
            }
        }
        if self.IsSiblingInvalidationSet() {
            let maximum = self.MaxDirectAdjacentSelectors();
            if maximum == u32::MAX {
                metadata.push(b'~' as u16);
            } else if maximum != 1 {
                metadata.extend(maximum.to_string().encode_utf16());
            }
        }
        let mut result = vec![b'{' as u16];
        if !features.is_empty() {
            result.push(b' ' as u16);
            result.extend(features);
        }
        if !metadata.is_empty() {
            result.push(b' ' as u16);
            result.extend(metadata);
        }
        result.extend([b' ' as u16, b'}' as u16]);
        foundation::String::from_utf16(&result)
    }
}

// cpp: invalidation_set.cc:79-112. Chromium deliberately does not compare
// invalidates_nth_, and only compares nested sibling state for kSiblings.
impl PartialEq for InvalidationSet {
    fn eq(&self, other: &Self) -> bool {
        if self.GetType() != other.GetType() {
            return false;
        }
        if self.GetType() == InvalidationType::kInvalidateSiblings {
            if self.MaxDirectAdjacentSelectors() != other.MaxDirectAdjacentSelectors()
                || self.Descendants() != other.Descendants()
                || self.SiblingDescendants() != other.SiblingDescendants()
            {
                return false;
            }
        }
        self.invalidation_flags_ == other.invalidation_flags_
            && self.invalidates_self_ == other.invalidates_self_
            && BackingEqual(
                &self.backing_flags_,
                &self.classes_,
                &other.backing_flags_,
                &other.classes_,
            )
            && BackingEqual(
                &self.backing_flags_,
                &self.ids_,
                &other.backing_flags_,
                &other.ids_,
            )
            && BackingEqual(
                &self.backing_flags_,
                &self.tag_names_,
                &other.backing_flags_,
                &other.tag_names_,
            )
            && BackingEqual(
                &self.backing_flags_,
                &self.custom_pseudo_names_,
                &other.backing_flags_,
                &other.custom_pseudo_names_,
            )
            && BackingEqual(
                &self.backing_flags_,
                &self.attributes_,
                &other.backing_flags_,
                &other.attributes_,
            )
    }
}
impl Eq for InvalidationSet {}

// cpp: invalidation_set.cc:603-605
impl std::fmt::Display for InvalidationSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.ToString().Utf8())
    }
}

#[cfg(test)]
#[path = "invalidation_set_test.rs"]
mod tests;
