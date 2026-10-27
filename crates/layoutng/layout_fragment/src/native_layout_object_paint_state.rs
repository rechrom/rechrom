//! LayoutObject fixed-cost paint and compositing state.
//!
//! These accessors are physically fragment-owned but deliberately inlineable:
//! they are per-object scalar policy and should not create profiler noise.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:3446-3446
    pub(crate) fn SetIsBackgroundAttachmentFixedObject(&mut self, value: bool) {
        unsafe { LayoutObjectSetIsBackgroundAttachmentFixedObjectProvider(self, value) }
    }

    // cpp: layoutng/internal/layout_object.h:3447-3447
    pub(crate) fn SetCanCompositeBackgroundAttachmentFixed(&mut self, value: bool) {
        unsafe { LayoutObjectSetCanCompositeBackgroundAttachmentFixedProvider(self, value) }
    }

    // cpp: layoutng/internal/layout_object.h:3474-3474
    pub(crate) fn BackgroundIsKnownToBeObscured(&self) -> bool {
        unsafe { LayoutObjectBackgroundIsKnownToBeObscuredProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3475-3475
    pub(crate) fn SetBackgroundIsKnownToBeObscured(&mut self, value: bool) {
        unsafe { LayoutObjectSetBackgroundIsKnownToBeObscuredProvider(self, value) }
    }

    // cpp: layoutng/internal/layout_object.h:2725-2725
    pub fn GetImageOrientation(object: *const LayoutObject) -> RespectImageOrientationEnum {
        unsafe { LayoutObjectGetImageOrientationProvider(object) }
    }
}
