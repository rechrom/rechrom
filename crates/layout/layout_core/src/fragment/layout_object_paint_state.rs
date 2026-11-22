//! LayoutObject fixed-cost paint and compositing state.
//!
//! These accessors are physically fragment-owned but deliberately inlineable:
//! they are per-object scalar policy and should not create profiler noise.

use super::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectBackgroundIsKnownToBeObscuredProvider(
    object: *const LayoutObject,
) -> bool {
    let object = unsafe { &*object };
    object.CheckIsNotDestroyed();
    object.bitfields_.background_is_known_to_be_obscured_
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectSetBackgroundIsKnownToBeObscuredProvider(
    object: *mut LayoutObject,
    value: bool,
) {
    let object = unsafe { &mut *object };
    object.CheckIsNotDestroyed();
    object.bitfields_.background_is_known_to_be_obscured_ = value;
}

// Source: LayoutObject::SetIsBackgroundAttachmentFixedObject
// (core/layout/layout_object.cc). The LayoutView carries LocalFrameView's
// fixed-background object set in this assembly.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectSetIsBackgroundAttachmentFixedObjectProvider(
    object: *mut LayoutObject,
    value: bool,
) {
    let object = unsafe { &mut *object };
    object.CheckIsNotDestroyed();
    if object.bitfields_.is_background_attachment_fixed_object_ == value {
        return;
    }
    object.bitfields_.is_background_attachment_fixed_object_ = value;
    let view = object.View();
    debug_assert!(!view.is_null());
    if let Some(view) = unsafe { view.as_ref() } {
        if value {
            view.AddBackgroundAttachmentFixedObject(object);
        } else {
            view.RemoveBackgroundAttachmentFixedObject(object);
        }
    }
}

impl LayoutObject {
    pub(crate) fn SetIsBackgroundAttachmentFixedObject(&mut self, value: bool) {
        unsafe { LayoutObjectSetIsBackgroundAttachmentFixedObjectProvider(self, value) }
    }

    pub(crate) fn SetCanCompositeBackgroundAttachmentFixed(&mut self, value: bool) {
        unsafe { LayoutObjectSetCanCompositeBackgroundAttachmentFixedProvider(self, value) }
    }

    pub(crate) fn BackgroundIsKnownToBeObscured(&self) -> bool {
        unsafe { LayoutObjectBackgroundIsKnownToBeObscuredProvider(self) }
    }

    pub(crate) fn SetBackgroundIsKnownToBeObscured(&mut self, value: bool) {
        unsafe { LayoutObjectSetBackgroundIsKnownToBeObscuredProvider(self, value) }
    }
}
