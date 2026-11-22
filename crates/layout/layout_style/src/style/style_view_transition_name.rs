use foundation::{AtomicString, MakeGarbageCollected, TreeScope, Visitor, WeakMember};

impl foundation::Traceable for StyleViewTransitionName {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        StyleViewTransitionName::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/style_view_transition_name.h:19
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StyleViewTransitionNameType {
    kAuto,
    kCustom,
    kMatchElement,
}

// cpp: layoutng_style/style/style_view_transition_name.h:16-18
// cpp: layoutng_style/style/style_view_transition_name.h:68-72
pub struct StyleViewTransitionName {
    type_: StyleViewTransitionNameType,
    custom_name_: AtomicString,
    tree_scope_: WeakMember<TreeScope>,
}

#[allow(non_snake_case)]
impl StyleViewTransitionName {
    // cpp: layoutng_style/style/style_view_transition_name.h:59-63
    pub fn new_with_custom_name(custom_name: &AtomicString, tree_scope: *const TreeScope) -> Self {
        Self {
            type_: StyleViewTransitionNameType::kCustom,
            custom_name_: custom_name.clone(),
            tree_scope_: WeakMember::from_ptr(tree_scope.cast_mut()),
        }
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:65-66
    pub fn new_with_type(type_: StyleViewTransitionNameType, tree_scope: *const TreeScope) -> Self {
        Self {
            type_,
            custom_name_: AtomicString::default(),
            tree_scope_: WeakMember::from_ptr(tree_scope.cast_mut()),
        }
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:21
    pub fn IsAuto(&self) -> bool {
        self.type_ == StyleViewTransitionNameType::kAuto
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:22
    pub fn IsMatchElement(&self) -> bool {
        self.type_ == StyleViewTransitionNameType::kMatchElement
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:23
    pub fn IsCustom(&self) -> bool {
        self.type_ == StyleViewTransitionNameType::kCustom
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:25
    pub fn GetType(&self) -> StyleViewTransitionNameType {
        self.type_
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:27-30
    pub fn Auto(tree_scope: *const TreeScope) -> *mut Self {
        MakeGarbageCollected(Self::new_with_type(
            StyleViewTransitionNameType::kAuto,
            tree_scope,
        ))
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:32-35
    pub fn MatchElement(tree_scope: *const TreeScope) -> *mut Self {
        MakeGarbageCollected(Self::new_with_type(
            StyleViewTransitionNameType::kMatchElement,
            tree_scope,
        ))
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:37-43
    pub fn Create(name: &AtomicString, tree_scope: *const TreeScope) -> *mut Self {
        assert!(!name.IsNull());
        assert!(name != "none");
        assert!(name != "auto");
        MakeGarbageCollected(Self::new_with_custom_name(name, tree_scope))
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:45-48
    pub fn CustomName(&self) -> AtomicString {
        assert!(self.type_ == StyleViewTransitionNameType::kCustom);
        self.custom_name_.clone()
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:50
    pub fn GetTreeScope(&self) -> *const TreeScope {
        self.tree_scope_.Get()
    }

    // cpp: layoutng_style/style/style_view_transition_name.h:57
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.tree_scope_);
    }
}

// cpp: layoutng_style/style/style_view_transition_name.h:52-55
impl PartialEq for StyleViewTransitionName {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
            && self.custom_name_ == other.custom_name_
            && self.tree_scope_ == other.tree_scope_
    }
}
