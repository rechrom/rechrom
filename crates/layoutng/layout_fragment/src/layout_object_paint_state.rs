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

    pub fn GetImageOrientation(object: *const LayoutObject) -> RespectImageOrientationEnum {
        unsafe { LayoutObjectGetImageOrientationProvider(object) }
    }
}
