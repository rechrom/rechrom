use foundation::{Member, ScopedCSSName, Visitor};

// cpp: layoutng_style/style/style_position_anchor.h:21
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Type {
    kNone,
    kAuto,
    kName,
    kNormal,
}

// cpp: layoutng_style/style/style_position_anchor.h:17-49
#[derive(Clone)]
pub struct StylePositionAnchor {
    type_: Type,
    name_: Member<ScopedCSSName>,
}

#[allow(non_snake_case)]
impl StylePositionAnchor {
    // cpp: layoutng_style/style/style_position_anchor.h:23
    pub fn new_type(type_: Type) -> Self {
        Self {
            type_,
            name_: Member::default(),
        }
    }

    // cpp: layoutng_style/style/style_position_anchor.h:24-25
    pub fn new_name(name: Member<ScopedCSSName>) -> Self {
        Self {
            type_: Type::kName,
            name_: name,
        }
    }

    // cpp: layoutng_style/style/style_position_anchor.h:27-29
    pub fn Initial() -> Self {
        Self::new_type(Type::kNormal)
    }

    // cpp: layoutng_style/style/style_position_anchor.h:35-36
    pub fn GetType(&self) -> Type {
        self.type_
    }
    pub fn IsName(&self) -> bool {
        self.type_ == Type::kName
    }

    // cpp: layoutng_style/style/style_position_anchor.h:38-42
    pub fn GetName(&self) -> &ScopedCSSName {
        debug_assert!(self.type_ == Type::kName);
        let name = self.name_.Get();
        debug_assert!(!name.is_null());
        // SAFETY: the source's two DCHECKs guard this dereference in debug
        // builds; a null name violates the same C++ caller precondition.
        unsafe { &*name }
    }

    // cpp: layoutng_style/style/style_position_anchor.h:44
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.name_);
    }
}

// cpp: layoutng_style/style/style_position_anchor.h:31-33
impl PartialEq for StylePositionAnchor {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && foundation::ValuesEquivalent(&self.name_, &other.name_)
    }
}
