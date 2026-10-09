// Copyright 2020 The Chromium Authors. All rights reserved.
// BSD-style license; see the LICENSE file.
// cpp: third_party/blink/renderer/core/css/resolver/cascade_resolver.h
// cpp: third_party/blink/renderer/core/css/resolver/cascade_resolver.cc
// Source ledger (physical/effective/mapped/omitted/pending):
//   cascade_resolver.h:  239 / 139 / 99 / 40 / 0
//   cascade_resolver.cc:  75 /  61 / 47 / 14 / 0
//   total:              314 / 200 /146 / 54 / 0
// Effective removes comments/blank lines but retains source delimiters,
// declarations and preprocessing; mapped + omitted + pending = effective.
// All production h:37-232 and cc:20-73 are mapped. Omitted header lines:
// 5-6,8-19,21,23-26,28,30,32,38,40,50,52,74,165,167,175,179-182,
// 225,227,234,236-237,239. Omitted implementation: 5,7-12,14,16-18,
// 37,59,75. These are includes/guards, namespaces/forwards, allocator/access/
// friendship/GC-memory annotations, Trace and diagnostic-only assertions.
// Rust owned names, Rc identity and AutoLock's mutable-borrow guard preserve
// the original custom-property equality, rule identity and nested cycle range;
// required external metadata adapters do not inflate this source ledger.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
use super::cascade_filter::CascadeFilter;
use super::cascade_origin::CascadeOrigin;
use crate::css_property_name::CSSPropertyName;
use crate::css_property_value::{CSSPropertyValue, CSSPropertyValueBackend};
use crate::css_value::CSSValue;
use crate::properties::css_property::{CSSProperty, Flags};
use foundation::AtomicString;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub const kNotFound: usize = usize::MAX;
/// Only the real CSSVariableData accessor and CSSAnimations metadata query.
/// The taint/context decision and all cycle/locking decisions stay in Rust.
pub trait CascadeResolverBackend {
    type VariableData;
    fn IsAnimationTainted(&self, data: &Self::VariableData) -> bool;
    fn IsAnimationAffectingProperty(&self, property: &CSSProperty) -> bool;
}
// cpp: cascade_resolver.h:53-63.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleNodeType {
    kProperty,
    kAttribute,
    kLocalVariable,
    kFunction,
}
// cpp: cascade_resolver.h:49-89. Existing CSSPropertyName retains the custom
// property's real name alongside its erased base metadata; it is not a new
// property model. Function identity is the actual shared rule object's address.
pub struct CycleNode<F> {
    pub node_type: CycleNodeType,
    property: Option<CSSProperty>,
    property_name: Option<CSSPropertyName>,
    pub name: AtomicString,
    pub function: Option<Rc<F>>,
    random_value_count: usize,
}
impl<F> Clone for CycleNode<F> {
    fn clone(&self) -> Self {
        Self {
            node_type: self.node_type,
            property: self.property,
            property_name: self.property_name.clone(),
            name: self.name.clone(),
            function: self.function.clone(),
            random_value_count: self.random_value_count,
        }
    }
}
impl<F> PartialEq for CycleNode<F> {
    // cpp: cascade_resolver.h:65-73. The source ignores random_value_count.
    fn eq(&self, other: &Self) -> bool {
        self.node_type == other.node_type
            && self.property_name == other.property_name
            && self.name == other.name
            && match (&self.function, &other.function) {
                (None, None) => true,
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                _ => false,
            }
    }
}
impl<F> Eq for CycleNode<F> {}
impl<F> CycleNode<F> {
    pub fn ForProperty(property: CSSProperty, name: CSSPropertyName) -> Self {
        Self {
            node_type: CycleNodeType::kProperty,
            property: Some(property),
            property_name: Some(name),
            name: AtomicString::default(),
            function: None,
            random_value_count: 0,
        }
    }
    pub fn ForNativeProperty(property: &CSSProperty) -> Self {
        Self::ForProperty(*property, property.GetCSSPropertyName())
    }
    pub fn ForAttribute(name: AtomicString) -> Self {
        Self {
            node_type: CycleNodeType::kAttribute,
            property: None,
            property_name: None,
            name,
            function: None,
            random_value_count: 0,
        }
    }
    pub fn ForLocalVariable(name: AtomicString, function: Option<Rc<F>>) -> Self {
        Self {
            node_type: CycleNodeType::kLocalVariable,
            property: None,
            property_name: None,
            name,
            function,
            random_value_count: 0,
        }
    }
    pub fn ForFunction(function: Rc<F>) -> Self {
        Self {
            node_type: CycleNodeType::kFunction,
            property: None,
            property_name: None,
            name: AtomicString::default(),
            function: Some(function),
            random_value_count: 0,
        }
    }
}
// cpp: cascade_resolver.h:224-230. Stores the actual CSSPropertyValue objects
// and the original CSSValue pointer; no alternate parsed-property representation.
pub struct ShorthandCache<D: CSSPropertyValueBackend> {
    pub value: Option<Rc<CSSValue<D>>>,
    pub parsed_properties: Vec<CSSPropertyValue<D>>,
}
// cpp: cascade_resolver.h:207-232.
pub struct CascadeResolver<D: CSSPropertyValueBackend, F> {
    stack: Vec<CycleNode<F>>,
    cycle_start: usize,
    cycle_end: usize,
    filter: CascadeFilter,
    author_flags: Flags,
    flags: Flags,
    rejected_flags: Flags,
    pub shorthand_cache: ShorthandCache<D>,
    function_invocation_count: usize,
}
impl<D: CSSPropertyValueBackend, F> CascadeResolver<D, F> {
    // cpp: cascade_resolver.h:184 and explicit field initializers 210-231.
    pub fn new(filter: CascadeFilter) -> Self {
        Self {
            stack: Vec::new(),
            cycle_start: kNotFound,
            cycle_end: kNotFound,
            filter,
            author_flags: 0,
            flags: 0,
            rejected_flags: 0,
            shorthand_cache: ShorthandCache {
                value: None,
                parsed_properties: Vec::new(),
            },
            function_invocation_count: 0,
        }
    }
    pub fn Filter(&self) -> CascadeFilter {
        self.filter
    }
    // cpp: cascade_resolver.h:98-102.
    pub fn IsLockedProperty(&self, property: &CSSProperty) -> bool {
        self.IsLocked(&CycleNode::ForNativeProperty(property))
    }
    pub fn IsLocked(&self, node: &CycleNode<F>) -> bool {
        self.Find(node) != kNotFound
    }
    // cpp: cascade_resolver.h:105-112.
    pub fn CurrentProperty(&self) -> Option<&CSSProperty> {
        self.stack
            .iter()
            .rev()
            .find(|node| node.node_type == CycleNodeType::kProperty)
            .and_then(|node| node.property.as_ref())
    }
    pub fn CurrentPropertyName(&self) -> Option<&CSSPropertyName> {
        self.stack
            .iter()
            .rev()
            .find(|node| node.node_type == CycleNodeType::kProperty)
            .and_then(|node| node.property_name.as_ref())
    }
    // cpp: cascade_resolver.cc:20-29.
    pub fn AllowSubstitution<B: CascadeResolverBackend>(
        &self,
        data: Option<&B::VariableData>,
        backend: &B,
    ) -> bool {
        if data.is_some_and(|data| backend.IsAnimationTainted(data)) && !self.stack.is_empty() {
            let property = self
                .CurrentProperty()
                .expect("source animation-taint path requires current property");
            if self
                .CurrentPropertyName()
                .expect("current property name")
                .IsCustomProperty()
            {
                return true;
            }
            return !backend.IsAnimationAffectingProperty(property);
        }
        true
    }
    // cpp: cascade_resolver.h:120-126.
    pub fn Rejects(&mut self, property: &CSSProperty) -> bool {
        if self.filter.Accepts(property) {
            return false;
        }
        self.rejected_flags |= property.GetFlags();
        true
    }
    // cpp: cascade_resolver.h:130-134.
    pub fn CollectFlags(&mut self, property: &CSSProperty, origin: CascadeOrigin) {
        let flags = property.GetFlags();
        if origin == CascadeOrigin::kAuthor {
            self.author_flags |= flags;
        }
        self.flags |= flags;
    }
    pub fn Flags(&self) -> Flags {
        self.flags
    }
    pub fn AuthorFlags(&self) -> Flags {
        self.author_flags
    }
    pub fn RejectedFlags(&self) -> Flags {
        self.rejected_flags
    }
    // cpp: cascade_resolver.h:146-154.
    pub fn RandomValueCount(&self) -> usize {
        self.stack.last().map_or(0, |node| node.random_value_count)
    }
    pub fn SetRandomValueCount(&mut self, count: usize) {
        if let Some(node) = self.stack.last_mut() {
            node.random_value_count = count;
        }
    }
    pub fn FunctionInvocationCount(&self) -> usize {
        self.function_invocation_count
    }
    pub fn NextFunctionInvocationCount(&mut self) -> usize {
        self.function_invocation_count = self.function_invocation_count.wrapping_add(1);
        self.function_invocation_count
    }
    // cpp: cascade_resolver.h:192-196; cc:31-39.
    pub fn DetectCycleProperty(&mut self, property: &CSSProperty) -> bool {
        self.DetectCycle(&CycleNode::ForNativeProperty(property))
    }
    pub fn DetectCycle(&mut self, node: &CycleNode<F>) -> bool {
        self.DetectCycleAt(self.Find(node))
    }
    pub fn DetectCycleAt(&mut self, index: usize) -> bool {
        if index == kNotFound {
            return false;
        }
        self.cycle_start = self.cycle_start.min(index);
        self.cycle_end = self.stack.len();
        true
    }
    // cpp: cascade_resolver.cc:41-43.
    pub fn InCycle(&self) -> bool {
        self.stack.len() > self.cycle_start && self.stack.len() <= self.cycle_end
    }
    // cpp: cascade_resolver.cc:45-54.
    pub fn Find(&self, node: &CycleNode<F>) -> usize {
        self.stack
            .iter()
            .position(|candidate| candidate == node)
            .unwrap_or(kNotFound)
    }
    // Source AutoLock constructor and destructor bodies. The guard below owns
    // the mutable borrow, making proper nesting and unwind cleanup explicit.
    fn Lock(&mut self, node: CycleNode<F>) {
        self.stack.push(node);
    }
    fn Unlock(&mut self) {
        self.stack.pop().expect("source pop_back requires lock");
        if self.cycle_end != kNotFound {
            self.cycle_end = self.cycle_end.min(self.stack.len());
        }
        if self.cycle_end <= self.cycle_start {
            self.cycle_start = kNotFound;
            self.cycle_end = kNotFound;
        }
    }
    pub fn WithLock<R>(&mut self, node: CycleNode<F>, f: impl FnOnce(&mut Self) -> R) -> R {
        let mut lock = AutoLock::new(node, self);
        f(&mut lock)
    }
}
// cpp: cascade_resolver.h:161-176; cc:56-73.
pub struct AutoLock<'a, D: CSSPropertyValueBackend, F> {
    resolver: &'a mut CascadeResolver<D, F>,
}
impl<'a, D: CSSPropertyValueBackend, F> AutoLock<'a, D, F> {
    pub fn new(node: CycleNode<F>, resolver: &'a mut CascadeResolver<D, F>) -> Self {
        resolver.Lock(node);
        Self { resolver }
    }
    pub fn ForProperty(property: &CSSProperty, resolver: &'a mut CascadeResolver<D, F>) -> Self {
        Self::new(CycleNode::ForNativeProperty(property), resolver)
    }
}
impl<D: CSSPropertyValueBackend, F> Deref for AutoLock<'_, D, F> {
    type Target = CascadeResolver<D, F>;
    fn deref(&self) -> &Self::Target {
        self.resolver
    }
}
impl<D: CSSPropertyValueBackend, F> DerefMut for AutoLock<'_, D, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resolver
    }
}
impl<D: CSSPropertyValueBackend, F> Drop for AutoLock<'_, D, F> {
    fn drop(&mut self) {
        self.resolver.Unlock();
    }
}
