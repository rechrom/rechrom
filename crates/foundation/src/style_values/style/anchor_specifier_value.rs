#![allow(non_snake_case, non_camel_case_types)]

use crate::{
    AddIntToHash, HashInt, MakeGarbageCollected, Member, Persistent, ScopedCSSName, Traceable,
    ValuesEquivalent, Visitor,
};

// cpp: foundation/style_values/style/anchor_specifier_value.h:24-27
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    kDefault,
    kNamed,
}

// cpp: foundation/style_values/style/anchor_specifier_value.h:18-57
pub struct AnchorSpecifierValue {
    type_: Type,
    name_: Member<ScopedCSSName>,
}

#[allow(non_upper_case_globals)]
impl AnchorSpecifierValue {
    // cpp: foundation/style_values/style/anchor_specifier_value.h:30-30
    // cpp: foundation/style_values/style/anchor_specifier_value.cc:27-28
    pub fn new_named(name: &ScopedCSSName) -> Self {
        Self {
            type_: Type::kNamed,
            name_: Member::from_ptr(name as *const ScopedCSSName as *mut ScopedCSSName),
        }
    }

    // cpp: foundation/style_values/style/anchor_specifier_value.h:36-37
    // cpp: foundation/style_values/style/anchor_specifier_value.cc:21-24
    fn new_keyword(type_: Type) -> Self {
        debug_assert_ne!(type_, Type::kNamed);
        Self {
            type_,
            name_: Member::default(),
        }
    }

    // cpp: foundation/style_values/style/anchor_specifier_value.h:32-33
    // cpp: foundation/style_values/style/anchor_specifier_value.cc:14-18
    pub fn Default() -> *mut Self {
        std::thread_local! {
            static INSTANCE: Persistent<AnchorSpecifierValue> = Persistent::from_ptr(
                MakeGarbageCollected(AnchorSpecifierValue::new_keyword(Type::kDefault))
            );
        }
        INSTANCE.with(Persistent::Get)
    }

    // cpp: foundation/style_values/style/anchor_specifier_value.h:39-45
    pub fn IsDefault(&self) -> bool {
        self.type_ == Type::kDefault
    }

    pub fn IsNamed(&self) -> bool {
        self.type_ == Type::kNamed
    }

    pub fn GetName(&self) -> &ScopedCSSName {
        debug_assert!(self.IsNamed());
        debug_assert!(!self.name_.Get().is_null());
        unsafe { &*self.name_.Get() }
    }

    // cpp: foundation/style_values/style/anchor_specifier_value.h:49-49
    // cpp: foundation/style_values/style/anchor_specifier_value.cc:34-39
    pub fn GetHash(&self) -> u32 {
        let mut hash = 0;
        AddIntToHash(&mut hash, HashInt(self.type_ as u32));
        AddIntToHash(
            &mut hash,
            if self.name_.Get().is_null() {
                0
            } else {
                self.GetName().GetHash()
            },
        );
        hash
    }

    // cpp: foundation/style_values/style/anchor_specifier_value.h:51-51
    // cpp: foundation/style_values/style/anchor_specifier_value.cc:41-43
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.name_);
    }
}

// cpp: foundation/style_values/style/anchor_specifier_value.h:47-47
// cpp: foundation/style_values/style/anchor_specifier_value.cc:30-32
impl PartialEq for AnchorSpecifierValue {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && ValuesEquivalent(&self.name_, &other.name_)
    }
}

impl Eq for AnchorSpecifierValue {}

impl Traceable for AnchorSpecifierValue {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        AnchorSpecifierValue::Trace(self, visitor);
    }
}
