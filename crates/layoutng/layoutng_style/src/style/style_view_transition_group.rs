use foundation::AtomicString;

// cpp: layoutng_style/style/style_view_transition_group.h:18
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GroupType {
    kNormal,
    kCustom,
    kNearest,
    kContain,
}

// cpp: layoutng_style/style/style_view_transition_group.h:14-58
#[derive(Clone)]
pub struct StyleViewTransitionGroup {
    type_: GroupType,
    custom_name_: AtomicString,
}

// cpp: layoutng_style/style/style_view_transition_group.h:47-49
impl PartialEq for StyleViewTransitionGroup {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.custom_name_ == other.custom_name_
    }
}

impl Eq for StyleViewTransitionGroup {}

#[allow(non_snake_case)]
impl StyleViewTransitionGroup {
    // cpp: layoutng_style/style/style_view_transition_group.h:20-23
    pub fn IsNearest(&self) -> bool {
        self.type_ == GroupType::kNearest
    }
    pub fn IsCustom(&self) -> bool {
        self.type_ == GroupType::kCustom
    }
    pub fn IsNormal(&self) -> bool {
        self.type_ == GroupType::kNormal
    }
    pub fn IsContain(&self) -> bool {
        self.type_ == GroupType::kContain
    }

    // cpp: layoutng_style/style/style_view_transition_group.h:25-27
    pub fn Nearest() -> Self {
        Self::with_type(GroupType::kNearest)
    }

    // cpp: layoutng_style/style/style_view_transition_group.h:28-30
    pub fn Normal() -> Self {
        Self::with_type(GroupType::kNormal)
    }

    // cpp: layoutng_style/style/style_view_transition_group.h:31-33
    pub fn Contain() -> Self {
        Self::with_type(GroupType::kContain)
    }

    // cpp: layoutng_style/style/style_view_transition_group.h:34-40
    pub fn Create(name: &AtomicString) -> Self {
        assert!(!name.IsNull());
        assert!(name != "nearest");
        assert!(name != "normal");
        assert!(name != "contain");
        Self::with_custom_name(name)
    }

    // cpp: layoutng_style/style/style_view_transition_group.h:42-45
    pub fn CustomName(&self) -> AtomicString {
        assert!(self.type_ == GroupType::kCustom);
        self.custom_name_.clone()
    }

    // cpp: layoutng_style/style/style_view_transition_group.h:52-53
    fn with_custom_name(name: &AtomicString) -> Self {
        Self {
            type_: GroupType::kCustom,
            custom_name_: name.clone(),
        }
    }

    // cpp: layoutng_style/style/style_view_transition_group.h:55-57
    fn with_type(type_: GroupType) -> Self {
        Self {
            type_,
            custom_name_: AtomicString::default(),
        }
    }
}
