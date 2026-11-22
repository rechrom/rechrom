//! Opaque aliases into the resident LayoutObject invalidation owner.
//! Chromium event_handler_registry.cc:298-308; layout_object.cc:5468-5512.
use crate::{internal::layout_object::LayoutObject, layout_engine::LayoutEngine};
use foundation::Persistent;

pub struct NativeEventRegistryLayout {
    object: Persistent<LayoutObject>,
}
impl NativeEventRegistryLayout {
    /// WebFrameWidgetImpl's focused LayoutEmbeddedObject gate; this is the
    /// existing resident LayoutObject identity, not a DOM tag approximation.
    pub fn IsEmbeddedObject(&self) -> bool {
        let object = unsafe { &*self.object.Get() };
        !object.WasDestroyedForTreeUpdate() && object.IsEmbeddedObject()
    }
}
impl LayoutEngine {
    /// None names Window::ContentLayoutObject (the same native LayoutView).
    pub fn EventRegistryLayout(&self, node: Option<u64>) -> Option<NativeEventRegistryLayout> {
        let tree = self.GetLayoutTree()?;
        let object = match node {
            Some(id) => tree.OutputNode(id)?,
            None => tree.Root(),
        };
        Some(NativeEventRegistryLayout {
            object: Persistent::from_ptr(object as *const LayoutObject as *mut LayoutObject),
        })
    }
    pub fn InvalidateEventRegistryLayout(
        &mut self,
        target: &NativeEventRegistryLayout,
        blocking_wheel: bool,
    ) {
        let object = unsafe { &mut *target.object.Get() };
        if object.WasDestroyedForTreeUpdate() {
            return;
        }
        let _scope = self.NativeScrollPassScope();
        if blocking_wheel {
            object.MarkBlockingWheelEventHandlerChanged();
        } else {
            object.MarkEffectiveAllowedTouchActionChanged();
        }
    }
}
