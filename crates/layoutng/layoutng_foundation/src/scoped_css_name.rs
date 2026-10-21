use std::hash::{Hash, Hasher};

use crate::{
    AddIntToHash, AtomicString, HashInt64, HeapVector, Member, Traceable, ValuesEquivalent,
    Visitor, WeakMember,
};

// cpp: foundation/style_values/style/scoped_css_name.h:21
// This source tree only forward-declares TreeScope. The DOM object stays at
// the external boundary; no local implementation or behavior is invented.
pub enum TreeScope {}

// cpp: foundation/style_values/style/scoped_css_name.h:23-55
pub struct ScopedCSSName {
    name_: AtomicString,
    tree_scope_: WeakMember<TreeScope>,
}

#[allow(non_snake_case)]
impl ScopedCSSName {
    // cpp: foundation/style_values/style/scoped_css_name.h:29-35
    pub fn new(name: &AtomicString, tree_scope: *const TreeScope) -> Self {
        debug_assert!(!name.IsNull());
        Self {
            name_: name.clone(),
            tree_scope_: WeakMember::from_ptr(tree_scope.cast_mut()),
        }
    }

    pub fn GetName(&self) -> &AtomicString {
        &self.name_
    }

    pub fn GetTreeScope(&self) -> *const TreeScope {
        self.tree_scope_.Get()
    }

    // cpp: foundation/style_values/style/scoped_css_name.h:41-45
    pub fn GetHash(&self) -> u32 {
        let mut hash = self.name_.Hash();
        AddIntToHash(&mut hash, HashInt64(self.GetTreeScope() as usize as u64));
        hash
    }

    // cpp: foundation/style_values/style/scoped_css_name.h:47
    // cpp: foundation/style_values/style/scoped_css_name.cc:6-8
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.tree_scope_);
    }
}

// cpp: foundation/style_values/style/scoped_css_name.h:37-39
impl PartialEq for ScopedCSSName {
    fn eq(&self, other: &Self) -> bool {
        self.name_ == other.name_ && self.tree_scope_ == other.tree_scope_
    }
}
impl Eq for ScopedCSSName {}

impl Hash for ScopedCSSName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.GetHash());
    }
}

impl Traceable for ScopedCSSName {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ScopedCSSName::Trace(self, visitor);
    }
}

// cpp: foundation/style_values/style/scoped_css_name.h:57-83
pub struct ScopedCSSNameList {
    names_: HeapVector<Member<ScopedCSSName>>,
}

#[allow(non_snake_case)]
impl ScopedCSSNameList {
    // cpp: foundation/style_values/style/scoped_css_name.h:64-70
    pub fn new(names: HeapVector<Member<ScopedCSSName>>) -> Self {
        Self { names_: names }
    }

    pub fn GetNames(&self) -> &HeapVector<Member<ScopedCSSName>> {
        &self.names_
    }

    // cpp: foundation/style_values/style/scoped_css_name.h:79
    // cpp: foundation/style_values/style/scoped_css_name.cc:10-12
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.names_);
    }
}

// cpp: foundation/style_values/style/scoped_css_name.h:72-77
impl PartialEq for ScopedCSSNameList {
    fn eq(&self, other: &Self) -> bool {
        self.names_.len() == other.names_.len()
            && self
                .names_
                .iter()
                .zip(other.names_.iter())
                .all(|(a, b)| ValuesEquivalent(a, b))
    }
}
impl Eq for ScopedCSSNameList {}

impl Traceable for ScopedCSSNameList {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ScopedCSSNameList::Trace(self, visitor);
    }
}

// cpp: foundation/style_values/style/scoped_css_name.h:85-108
// HashMap's trait-parameter storage policy is still partial. Keep the source
// value-based trait operations explicit for the later specialized hash table.
pub struct ScopedCSSNameWrapperPtrHashTraits;

#[allow(non_snake_case, non_upper_case_globals)]
impl ScopedCSSNameWrapperPtrHashTraits {
    pub const kSafeToCompareToEmptyOrDeleted: bool = false;

    pub fn GetHash(value: &Member<ScopedCSSName>) -> u32 {
        assert!(!value.Get().is_null());
        unsafe { &*value.Get() }.GetHash()
    }

    pub fn Equal(a: &Member<ScopedCSSName>, b: &Member<ScopedCSSName>) -> bool {
        ValuesEquivalent(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayoutHeapScope, MakeGarbageCollected};
    use std::ptr;

    #[test]
    fn scoped_names_compare_by_name_and_scope() {
        let _scope = LayoutHeapScope::new();
        let name = AtomicString::from_str("anchor");
        let a = MakeGarbageCollected(ScopedCSSName::new(&name, ptr::null()));
        let b = MakeGarbageCollected(ScopedCSSName::new(&name, ptr::null()));
        assert!(unsafe { &*a } == unsafe { &*b });
        assert_eq!(unsafe { &*a }.GetHash(), unsafe { &*b }.GetHash());

        let first = ScopedCSSNameList::new(HeapVector::from(vec![Member::from_ptr(a)]));
        let second = ScopedCSSNameList::new(HeapVector::from(vec![Member::from_ptr(b)]));
        assert!(first == second);
        assert!(ScopedCSSNameWrapperPtrHashTraits::Equal(
            &Member::from_ptr(a),
            &Member::from_ptr(b)
        ));
    }
}
