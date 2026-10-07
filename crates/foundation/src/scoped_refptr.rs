use std::ops::Deref;
use std::sync::Arc;

// cpp: foundation/base/memory/scoped_refptr.h:226-247,255-260,268-315
// The source's intrusive refcount is an Arc ownership edge in Rust. Null is
// represented by None, preserving the one-pointer nullable handle shape.
#[repr(transparent)]
pub struct ScopedRefPtr<T> {
    value: Option<Arc<T>>,
}

impl<T> Default for ScopedRefPtr<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T> Clone for ScopedRefPtr<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

#[allow(non_snake_case)]
impl<T> ScopedRefPtr<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(Arc::new(value)),
        }
    }

    pub fn from_arc(value: Arc<T>) -> Self {
        Self { value: Some(value) }
    }

    // cpp: foundation/base/memory/scoped_refptr.h:278-311
    pub fn get(&self) -> *const T {
        self.value.as_ref().map_or(std::ptr::null(), Arc::as_ptr)
    }

    pub fn reset(&mut self) {
        self.value = None;
    }

    pub fn release(&mut self) -> Option<Arc<T>> {
        self.value.take()
    }

    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.value, &mut other.value);
    }

    pub fn IsNull(&self) -> bool {
        self.value.is_none()
    }
}

impl<T> Deref for ScopedRefPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
            .as_deref()
            .expect("dereferencing null scoped_refptr")
    }
}

// cpp: foundation/base/memory/scoped_refptr.h:313-343
impl<T> PartialEq for ScopedRefPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}
impl<T> Eq for ScopedRefPtr<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_reset_and_release_preserve_identity() {
        assert_eq!(
            std::mem::size_of::<ScopedRefPtr<u32>>(),
            std::mem::size_of::<usize>()
        );
        let mut first = ScopedRefPtr::new(5u32);
        let second = first.clone();
        assert!(first == second);
        first.reset();
        assert!(first.IsNull());
        assert_eq!(*second, 5);
        let mut second = second;
        let owner = second.release().unwrap();
        assert!(second.IsNull());
        assert_eq!(*owner, 5);
    }
}
