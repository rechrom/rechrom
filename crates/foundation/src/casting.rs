#![allow(non_snake_case)]

// The C++ DowncastTraits specialization belongs to the target type. Rust
// implementations must check the source tag before casting a base pointer.
// cpp: foundation/blink_base/wtf/casting.h:48-105
pub trait DowncastFrom<Base: ?Sized>: Sized {
    fn AllowFrom(base: &Base) -> bool;
}

impl<T> DowncastFrom<T> for T {
    fn AllowFrom(_: &T) -> bool {
        true
    }
}

// A returned raw pointer has the same lifetime as the input. In particular,
// no reference to a derived object is formed until the caller dereferences it.
// Each DowncastFrom implementation is responsible for a base-at-offset-zero
// representation, matching the source inheritance edge.
pub trait CastSource<Derived>: Sized {
    fn GetBase(self) -> *const Self::Base;
    type Base: ?Sized;
    fn AsDerived(base: *const Self::Base) -> *mut Derived;
    fn Allowed(base: &Self::Base) -> bool;

    fn Cast(self, checked: bool) -> *mut Derived {
        let base = self.GetBase();
        if base.is_null() {
            return std::ptr::null_mut();
        }
        let allowed = unsafe { Self::Allowed(&*base) };
        if checked {
            assert!(allowed, "invalid checked downcast");
        }
        if allowed {
            Self::AsDerived(base)
        } else {
            std::ptr::null_mut()
        }
    }

    fn IsA(self) -> bool {
        let base = self.GetBase();
        !base.is_null() && unsafe { Self::Allowed(&*base) }
    }
}

impl<Derived, Base> CastSource<Derived> for *const Base
where
    Derived: DowncastFrom<Base>,
{
    type Base = Base;
    fn GetBase(self) -> *const Base {
        self
    }
    fn AsDerived(base: *const Base) -> *mut Derived {
        base.cast::<Derived>() as *mut Derived
    }
    fn Allowed(base: &Base) -> bool {
        Derived::AllowFrom(base)
    }
}

impl<Derived, Base> CastSource<Derived> for *mut Base
where
    Derived: DowncastFrom<Base>,
{
    type Base = Base;
    fn GetBase(self) -> *const Base {
        self
    }
    fn AsDerived(base: *const Base) -> *mut Derived {
        base.cast::<Derived>() as *mut Derived
    }
    fn Allowed(base: &Base) -> bool {
        Derived::AllowFrom(base)
    }
}

impl<Derived, Base> CastSource<Derived> for &Base
where
    Derived: DowncastFrom<Base>,
{
    type Base = Base;
    fn GetBase(self) -> *const Base {
        self
    }
    fn AsDerived(base: *const Base) -> *mut Derived {
        base.cast::<Derived>() as *mut Derived
    }
    fn Allowed(base: &Base) -> bool {
        Derived::AllowFrom(base)
    }
}

impl<Derived, Base> CastSource<Derived> for &mut Base
where
    Derived: DowncastFrom<Base>,
{
    type Base = Base;
    fn GetBase(self) -> *const Base {
        self
    }
    fn AsDerived(base: *const Base) -> *mut Derived {
        base.cast::<Derived>() as *mut Derived
    }
    fn Allowed(base: &Base) -> bool {
        Derived::AllowFrom(base)
    }
}

// cpp: foundation/blink_base/wtf/casting.h:108-125
pub fn IsA<Derived>(source: impl CastSource<Derived>) -> bool {
    source.IsA()
}

// cpp: foundation/blink_base/wtf/casting.h:149-168
pub fn To<Derived>(source: impl CastSource<Derived>) -> *mut Derived {
    source.Cast(true)
}

// cpp: foundation/blink_base/wtf/casting.h:176-212
pub fn DynamicTo<Derived>(source: impl CastSource<Derived>) -> *mut Derived {
    source.Cast(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    struct Base {
        is_derived: bool,
    }
    #[repr(C)]
    struct Derived {
        base: Base,
        value: u32,
    }
    impl DowncastFrom<Base> for Derived {
        fn AllowFrom(base: &Base) -> bool {
            base.is_derived
        }
    }

    #[test]
    fn checked_casts_obey_tag_and_null() {
        let derived = Derived {
            base: Base { is_derived: true },
            value: 7,
        };
        let base = Base { is_derived: false };
        assert_eq!(
            DynamicTo::<Derived>(&derived.base),
            &derived as *const _ as *mut _
        );
        assert!(DynamicTo::<Derived>(&base).is_null());
        assert!(DynamicTo::<Derived>(std::ptr::null::<Base>()).is_null());
        assert!(IsA::<Derived>(&derived.base));
        assert!(!IsA::<Derived>(&base));
        assert_eq!(To::<Derived>(&derived.base), &derived as *const _ as *mut _);
        assert!(To::<Derived>(std::ptr::null::<Base>()).is_null());
    }

    #[test]
    #[should_panic(expected = "invalid checked downcast")]
    fn checked_to_rejects_wrong_tag() {
        let base = Base { is_derived: false };
        To::<Derived>(&base);
    }
}
