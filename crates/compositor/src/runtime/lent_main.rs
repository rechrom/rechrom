//! The main half lent to the impl half for one in-process call (S1b-6 of
//! S1B_PROXY_DESIGN.md).
//!
//! Since S1b-6 the Page owns the main half (`LayerizeEngine` and
//! `ProxyMain`, `crate::engine::CompositorMainEngine`) next to the impl half
//! (`CompositorEngine`). Both are still driven on one sequence and the
//! in-process Proxy transport (`EngineCore::RunProxyTasks`, decision D1)
//! still runs ProxyMain tasks from inside Scheduler callbacks, which only
//! receive `&mut EngineCore`. Every `CompositorEngine` entry therefore takes
//! the main half by `&mut` and moves its boxes into the core for exactly that
//! call (`CompositorEngine::Lend`); outside a call the core holds nothing of
//! the main half. S1b-7 replaces the lending with posted tasks across the
//! document and compositor threads.
use std::ops::{Deref, DerefMut};

pub(crate) struct LentMain<T>(Option<Box<T>>);

impl<T> LentMain<T> {
    pub(crate) fn new(value: Box<T>) -> Self {
        Self(Some(value))
    }
    pub(crate) fn Lend(&mut self, value: Box<T>) {
        assert!(self.0.is_none(), "main half already lent");
        self.0 = Some(value);
    }
    pub(crate) fn Return(&mut self) -> Box<T> {
        self.0.take().expect("main half lent for this call")
    }
    pub(crate) fn IsLent(&self) -> bool {
        self.0.is_some()
    }
}

impl<T> Deref for LentMain<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
            .as_deref()
            .expect("the main half is lent only inside a CompositorEngine call")
    }
}

impl<T> DerefMut for LentMain<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
            .as_deref_mut()
            .expect("the main half is lent only inside a CompositorEngine call")
    }
}
