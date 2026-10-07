// C++: foundation/blink_base/heap/member.h. These are non-owning pointer edges;
// liveness, tracing, and weak clearing belong to the layout heap and Visitor.
use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;

// cpp: foundation/blink_base/heap/member.h:13-25
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceKind {
    kStrong,
    kWeak,
    kUntraced,
}

pub const kMemberDeletedValue: usize = 0b10;

// cpp: foundation/blink_base/heap/member.h:27-75
#[repr(transparent)]
#[derive(Debug)]
pub struct Member<T: ?Sized> {
    pointer: Option<NonNull<T>>,
    _type: PhantomData<T>,
}

impl<T: ?Sized> Copy for Member<T> {}

impl<T: ?Sized> Clone for Member<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Default for Member<T> {
    fn default() -> Self {
        Self {
            pointer: None,
            _type: PhantomData,
        }
    }
}

impl<T: ?Sized> Member<T> {
    pub fn from_ptr(pointer: *mut T) -> Self {
        Self {
            pointer: NonNull::new(pointer),
            _type: PhantomData,
        }
    }

    pub fn from_ref(value: &mut T) -> Self {
        Self::from_ptr(value)
    }

    pub fn GetNonNull(&self) -> Option<NonNull<T>> {
        self.pointer
    }

    pub fn Clear(&mut self) {
        self.pointer = None;
    }

    pub fn ReleaseNonNull(&mut self) -> Option<NonNull<T>> {
        self.pointer.take()
    }

    pub fn Swap(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.pointer, &mut other.pointer);
    }

    pub fn IsHashTableDeletedValue(&self) -> bool {
        self.pointer
            .is_some_and(|pointer| pointer.as_ptr() as *mut () as usize == kMemberDeletedValue)
    }
}

impl<T> Member<T> {
    pub fn Get(&self) -> *mut T {
        self.pointer.map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    pub fn Release(&mut self) -> *mut T {
        self.ReleaseNonNull()
            .map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    pub fn deleted() -> Self {
        Self::from_ptr(kMemberDeletedValue as *mut T)
    }
}

impl<T: ?Sized> PartialEq for Member<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }
}

impl<T: ?Sized> Eq for Member<T> {}

// cpp: foundation/blink_base/heap/member_hash_traits.h:28-40,49-54
impl<T: ?Sized> Hash for Member<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.GetNonNull()
            .map(|pointer| pointer.as_ptr() as *mut () as usize)
            .hash(state);
    }
}

// cpp: foundation/blink_base/heap/member.h:21-25,27-75
#[repr(transparent)]
#[derive(Debug)]
pub struct WeakMember<T: ?Sized> {
    pointer: Cell<Option<NonNull<T>>>,
}

impl<T: ?Sized> Clone for WeakMember<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: Cell::new(self.pointer.get()),
        }
    }
}

impl<T: ?Sized> Default for WeakMember<T> {
    fn default() -> Self {
        Self {
            pointer: Cell::new(None),
        }
    }
}

impl<T: ?Sized> WeakMember<T> {
    pub fn from_ptr(pointer: *mut T) -> Self {
        Self {
            pointer: Cell::new(NonNull::new(pointer)),
        }
    }

    pub fn GetNonNull(&self) -> Option<NonNull<T>> {
        self.pointer.get()
    }

    pub fn Clear(&self) {
        self.pointer.set(None);
    }

    pub fn ReleaseNonNull(&self) -> Option<NonNull<T>> {
        self.pointer.replace(None)
    }

    pub fn Swap(&self, other: &Self) {
        self.pointer.swap(&other.pointer);
    }

    pub fn IsHashTableDeletedValue(&self) -> bool {
        self.pointer
            .get()
            .is_some_and(|pointer| pointer.as_ptr() as *mut () as usize == kMemberDeletedValue)
    }

    pub(crate) fn ClearForCollection(&self) {
        self.Clear();
    }
}

impl<T> WeakMember<T> {
    pub fn Get(&self) -> *mut T {
        self.pointer
            .get()
            .map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    pub fn Release(&self) -> *mut T {
        self.ReleaseNonNull()
            .map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    pub fn deleted() -> Self {
        Self::from_ptr(kMemberDeletedValue as *mut T)
    }
}

impl<T: ?Sized> PartialEq for WeakMember<T> {
    fn eq(&self, other: &Self) -> bool {
        self.GetNonNull() == other.GetNonNull()
    }
}

impl<T: ?Sized> Eq for WeakMember<T> {}

// cpp: foundation/blink_base/heap/member.h:79-85
#[repr(transparent)]
#[derive(Debug)]
pub struct UntracedMember<T: ?Sized>(Member<T>);

impl<T: ?Sized> Copy for UntracedMember<T> {}

impl<T: ?Sized> Clone for UntracedMember<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Default for UntracedMember<T> {
    fn default() -> Self {
        Self(Member::default())
    }
}

impl<T: ?Sized> UntracedMember<T> {
    pub fn from_ptr(pointer: *mut T) -> Self {
        Self(Member::from_ptr(pointer))
    }
    pub fn GetNonNull(&self) -> Option<NonNull<T>> {
        self.0.GetNonNull()
    }
    pub fn Clear(&mut self) {
        self.0.Clear();
    }
    pub fn ReleaseNonNull(&mut self) -> Option<NonNull<T>> {
        self.0.ReleaseNonNull()
    }
    pub fn Swap(&mut self, other: &mut Self) {
        self.0.Swap(&mut other.0);
    }
    pub fn IsHashTableDeletedValue(&self) -> bool {
        self.0.IsHashTableDeletedValue()
    }
}

impl<T> UntracedMember<T> {
    pub fn Get(&self) -> *mut T {
        self.0.Get()
    }
    pub fn Release(&mut self) -> *mut T {
        self.0.Release()
    }
    pub fn deleted() -> Self {
        Self(Member::deleted())
    }
}

impl<T: ?Sized> PartialEq for UntracedMember<T> {
    fn eq(&self, other: &Self) -> bool {
        self.GetNonNull() == other.GetNonNull()
    }
}

impl<T: ?Sized> Eq for UntracedMember<T> {}

// cpp: foundation/blink_base/heap/member_hash_traits.h:61-64
impl<T: ?Sized> Hash for UntracedMember<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.GetNonNull()
            .map(|pointer| pointer.as_ptr() as *mut () as usize)
            .hash(state);
    }
}

const _: () = assert!(std::mem::size_of::<Member<u8>>() == std::mem::size_of::<*mut u8>());
const _: () = assert!(std::mem::size_of::<WeakMember<u8>>() == std::mem::size_of::<*mut u8>());
const _: () = assert!(std::mem::size_of::<UntracedMember<u8>>() == std::mem::size_of::<*mut u8>());

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn strong_member_keys_use_pointer_identity() {
        let mut a = 7_u8;
        let mut b = 7_u8;
        let a_ptr = Member::from_ref(&mut a);
        let b_ptr = Member::from_ref(&mut b);
        let mut keys = HashSet::new();
        assert!(keys.insert(a_ptr));
        assert!(!keys.insert(a_ptr));
        assert!(keys.insert(b_ptr));
        assert_eq!(keys.len(), 2);
    }
}
