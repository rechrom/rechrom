use crate::{Member, ScopedCSSNameList, Traceable, TreeScope, ValuesEquivalent, Visitor};

// cpp: foundation/style_values/style/style_name_scope.h:23
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StyleNameScopeType {
    #[default]
    kNone,
    kAll,
    kNames,
}

// cpp: foundation/style_values/style/style_name_scope.h:19-49
#[derive(Clone)]
pub struct StyleNameScope {
    type_: StyleNameScopeType,
    all_tree_scope_: Member<TreeScope>,
    names_: Member<ScopedCSSNameList>,
}

impl Default for StyleNameScope {
    fn default() -> Self {
        Self {
            type_: StyleNameScopeType::kNone,
            all_tree_scope_: Member::default(),
            names_: Member::default(),
        }
    }
}

#[allow(non_snake_case)]
impl StyleNameScope {
    // cpp: foundation/style_values/style/style_name_scope.h:25-29
    pub fn new(
        type_: StyleNameScopeType,
        all_tree_scope: *const TreeScope,
        names: *const ScopedCSSNameList,
    ) -> Self {
        Self {
            type_,
            all_tree_scope_: Member::from_ptr(all_tree_scope.cast_mut()),
            names_: Member::from_ptr(names.cast_mut()),
        }
    }

    // cpp: foundation/style_values/style/style_name_scope.h:33-36
    pub fn IsNone(&self) -> bool {
        self.type_ == StyleNameScopeType::kNone
    }
    pub fn IsAll(&self) -> bool {
        self.type_ == StyleNameScopeType::kAll
    }
    pub fn AllTreeScope(&self) -> *const TreeScope {
        self.all_tree_scope_.Get()
    }
    pub fn Names(&self) -> *const ScopedCSSNameList {
        self.names_.Get()
    }

    // cpp: foundation/style_values/style/style_name_scope.h:38-41
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.all_tree_scope_);
        visitor.Trace(&self.names_);
    }
}

// cpp: foundation/style_values/style/style_name_scope.h:31
// cpp: foundation/style_values/style/style_name_scope.cc:7-14
impl PartialEq for StyleNameScope {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
            && self.all_tree_scope_ == other.all_tree_scope_
            && ValuesEquivalent(&self.names_, &other.names_)
    }
}
impl Eq for StyleNameScope {}

impl Traceable for StyleNameScope {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        StyleNameScope::Trace(self, visitor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtomicString, HeapVector, LayoutHeapScope, MakeGarbageCollected, ScopedCSSName};

    #[test]
    fn default_and_names_compare_as_source() {
        assert!(StyleNameScope::default().IsNone());
        let _scope = LayoutHeapScope::new();
        let name = AtomicString::from_str("anchor");
        let first = MakeGarbageCollected(ScopedCSSName::new(&name, std::ptr::null()));
        let second = MakeGarbageCollected(ScopedCSSName::new(&name, std::ptr::null()));
        let first_names = MakeGarbageCollected(ScopedCSSNameList::new(HeapVector::from(vec![
            Member::from_ptr(first),
        ])));
        let second_names = MakeGarbageCollected(ScopedCSSNameList::new(HeapVector::from(vec![
            Member::from_ptr(second),
        ])));
        let a = StyleNameScope::new(StyleNameScopeType::kNames, std::ptr::null(), first_names);
        let b = StyleNameScope::new(StyleNameScopeType::kNames, std::ptr::null(), second_names);
        assert!(a == b);
        assert!(!a.IsAll());
    }
}
