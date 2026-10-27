//! Cache allocation owned by the actual immutable Paint node value. Cache
//! algorithms and concrete cache types belong to their canonical compositor
//! source modules; this slot only breaks the Rust crate-dependency cycle.
use std::{
    any::Any,
    fmt,
    sync::{Arc, OnceLock},
};
#[derive(Default)]
pub struct OpaquePaintCacheSlot {
    value: OnceLock<Arc<dyn Any + Send + Sync>>,
}
impl OpaquePaintCacheSlot {
    /// The node has one native cache allocation. A different concrete type is
    /// a programming error, never an invitation to replace a live cache slot.
    pub fn get_or_init<T: Any + Send + Sync>(&self, init: impl FnOnce() -> T) -> Arc<T> {
        Arc::clone(self.value.get_or_init(|| Arc::new(init())))
            .downcast::<T>()
            .unwrap_or_else(|_| {
                panic!(
                    "Paint cache slot type mismatch: {}",
                    std::any::type_name::<T>()
                )
            })
    }
}
// A newly cloned node is a distinct immutable value revision, just as a new
// C++ node starts without a lazily created mutable cache. It must not share the
// old node's cache allocation or copy any mutable cached state.
impl Clone for OpaquePaintCacheSlot {
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl fmt::Debug for OpaquePaintCacheSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OpaquePaintCacheSlot")
    }
}
impl PartialEq for OpaquePaintCacheSlot {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for OpaquePaintCacheSlot {}
