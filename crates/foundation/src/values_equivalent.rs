// C++: foundation/base/memory/values_equivalent.h:22-70.
// Keep pointer identity, null handling, then pointee comparison in source
// order. A borrow of Member/WeakMember is the Rust form of the C++ Get()
// wrapper overload; raw pointers use the primary C++ overload.
use std::ptr::NonNull;

use crate::{Member, WeakMember};

pub trait ValuePointer<T: ?Sized> {
    fn pointer(self) -> Option<NonNull<T>>;
}

impl<T: ?Sized> ValuePointer<T> for *const T {
    fn pointer(self) -> Option<NonNull<T>> {
        NonNull::new(self as *mut T)
    }
}

impl<T: ?Sized> ValuePointer<T> for *mut T {
    fn pointer(self) -> Option<NonNull<T>> {
        NonNull::new(self)
    }
}

impl<'a, T: ?Sized> ValuePointer<T> for &'a Member<T> {
    fn pointer(self) -> Option<NonNull<T>> {
        self.GetNonNull()
    }
}

impl<'a, T: ?Sized> ValuePointer<T> for &'a WeakMember<T> {
    fn pointer(self) -> Option<NonNull<T>> {
        self.GetNonNull()
    }
}

// cpp: foundation/base/memory/values_equivalent.h:22-37,49-70
pub fn ValuesEquivalent<T: PartialEq + ?Sized, A: ValuePointer<T>, B: ValuePointer<T>>(
    a: A,
    b: B,
) -> bool {
    let a = a.pointer();
    let b = b.pointer();
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            if std::ptr::eq(a.as_ptr(), b.as_ptr()) {
                true
            } else {
                // SAFETY: these are source-level non-owning pointers. As in C++,
                // callers must keep both live for the comparison.
                unsafe { a.as_ref() == b.as_ref() }
            }
        }
        _ => false,
    }
}
