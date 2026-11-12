//! Identity-bearing scalar input shared by an embedding and document owners.
//! This adapter contains no engine state, policy or lifecycle decisions.
use std::sync::{Arc, RwLock};
#[derive(Debug)]
pub struct SharedValue<T: Copy>(Arc<RwLock<T>>);
impl<T: Copy> Clone for SharedValue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Copy> SharedValue<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }
    pub fn get(&self) -> T {
        *self.0.read().expect("live scalar owner")
    }
    pub fn set(&self, value: T) {
        *self.0.write().expect("live scalar owner") = value;
    }
    pub fn PtrEq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T: Copy + Default> Default for SharedValue<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
