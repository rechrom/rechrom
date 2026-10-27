use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:3540-3540
    #[inline(never)]
    fn AddAsImageObserver(&mut self, image: *mut StyleImage) {
        unsafe { LayoutObjectAddAsImageObserverProvider(self, image) }
    }

    // cpp: layoutng/internal/layout_object.h:3541-3541
    #[inline(never)]
    fn RemoveAsImageObserver(&mut self, image: *mut StyleImage) {
        unsafe { LayoutObjectRemoveAsImageObserverProvider(self, image) }
    }

    // cpp: layoutng/internal/layout_object.h:3543-3543
    #[inline(never)]
    fn UpdateImage(&mut self, old_image: *mut StyleImage, new_image: *mut StyleImage) {
        unsafe { LayoutObjectUpdateImageProvider(self, old_image, new_image) }
    }

    // cpp: layoutng/internal/layout_object.h:3544-3544
    #[inline(never)]
    fn UpdateShapeImage(&mut self, old_shape: *const ShapeValue, new_shape: *const ShapeValue) {
        unsafe { LayoutObjectUpdateShapeImageProvider(self, old_shape, new_shape) }
    }

    // cpp: layoutng/internal/layout_object.h:3545-3546
    #[inline(never)]
    fn UpdateFillImages(&mut self, old_layers: *const FillLayer, new_layers: *const FillLayer) {
        unsafe { LayoutObjectUpdateFillImagesProvider(self, old_layers, new_layers) }
    }

    // cpp: layoutng/internal/layout_object.h:3547-3548
    #[inline(never)]
    fn UpdateCursorImages(
        &mut self,
        old_cursors: *const CursorList,
        new_cursors: *const CursorList,
    ) {
        unsafe { LayoutObjectUpdateCursorImagesProvider(self, old_cursors, new_cursors) }
    }

    // cpp: layoutng/internal/layout_object.h:3572-3573
    #[inline(never)]
    fn UpdateImageObservers(
        &mut self,
        old_style: *const ComputedStyle,
        new_style: *const ComputedStyle,
    ) {
        unsafe { LayoutObjectUpdateImageObserversProvider(self, old_style, new_style) }
    }

    // cpp: layoutng/internal/layout_object.h:3574-3574
    #[inline(never)]
    fn UpdateFirstLineImageObservers(&mut self, new_style: *const ComputedStyle) {
        unsafe { LayoutObjectUpdateFirstLineImageObserversProvider(self, new_style) }
    }

    // Source UpdateImageObservers/UpdateFillImages subset for real native
    // mask StyleImages. Add new subscriptions before removing old ones.
    // cpp: core/layout/layout_object.cc:3109-3116,3200,3703-3722
    pub(crate) fn UpdateMaskImageObservers(
        &mut self,
        old: *const ComputedStyle,
        new: *const ComputedStyle,
    ) {
        LayoutObjectUpdateMaskImageObservers(self, old, new)
    }
}

// Source UpdateImageObservers/UpdateFillImages subset for real native
// mask StyleImages. Add new subscriptions before removing old ones.
// cpp: core/layout/layout_object.cc:3109-3116,3200,3703-3722
#[inline(never)]
pub(super) fn LayoutObjectUpdateMaskImageObservers(
    object: &mut LayoutObject,
    old_style: *const ComputedStyle,
    new_style: *const ComputedStyle,
) {
    if object.IsText() {
        return;
    }
    let old_layers = if old_style.is_null() {
        std::ptr::null()
    } else {
        unsafe { &*old_style }.MaskLayers() as *const FillLayer
    };
    let new_layers = if new_style.is_null() {
        std::ptr::null()
    } else {
        unsafe { &*new_style }.MaskLayers() as *const FillLayer
    };
    let mut old_cursor = old_layers;
    let mut new_cursor = new_layers;
    let mut identical = true;
    while !old_cursor.is_null() || !new_cursor.is_null() {
        if old_cursor.is_null()
            || new_cursor.is_null()
            || unsafe { &*old_cursor }.GetImage() != unsafe { &*new_cursor }.GetImage()
        {
            identical = false;
            break;
        }
        old_cursor = unsafe { &*old_cursor }.Next();
        new_cursor = unsafe { &*new_cursor }.Next();
    }
    if identical {
        return;
    }
    let receiver = (object as *mut LayoutObject).cast();
    object.image_resource_observer_.BindReceiver(receiver);
    let observer = &mut object.image_resource_observer_ as *mut ImageResourceObserver;
    let mut layer = new_layers;
    while !layer.is_null() {
        let current = unsafe { &*layer };
        let image = current.GetImage();
        if !image.is_null() {
            unsafe { &mut *image }.AddClient(observer);
        }
        layer = current.Next();
    }
    layer = old_layers;
    while !layer.is_null() {
        let current = unsafe { &*layer };
        let image = current.GetImage();
        if !image.is_null() {
            unsafe { &mut *image }.RemoveClient(observer);
        }
        layer = current.Next();
    }
}
