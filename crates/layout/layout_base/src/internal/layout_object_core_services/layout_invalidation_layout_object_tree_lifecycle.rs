use super::*;

impl LayoutObject {
    // Keep the virtual-dispatch edge visible independently from the base
    // invalidation work below. This makes derived tree-insertion cost easy to
    // distinguish from line dirtying and anchor propagation in profiles.
    // cpp: layoutng/internal/layout_object_core_services.cc:205-217
    #[inline(never)]
    pub fn InsertedIntoTree(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectInsertedIntoTree(self) }
    }

    #[inline]
    pub fn InsertedIntoTreeBase(&mut self) {
        LayoutObjectInsertedIntoTreeBase(self)
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:218-223
    #[inline(never)]
    pub fn WillBeRemovedFromTree(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectWillBeRemovedFromTree(self) }
    }

    #[inline]
    pub fn WillBeRemovedFromTreeBase(&mut self) {
        LayoutObjectWillBeRemovedFromTreeBase(self)
    }

    // This is the base implementation. Destroy() invokes the pending typed
    // virtual dispatcher so concrete derived cleanup runs before this method.
    // cpp: layoutng/internal/layout_object_core_services.cc:224-230
    #[inline]
    pub fn WillBeDestroyed(&mut self) {
        LayoutObjectWillBeDestroyed(self)
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:205-217
#[inline(never)]
pub(super) fn LayoutObjectInsertedIntoTreeBase(object: &mut LayoutObject) {
    object.CheckIsNotDestroyed();
    let parent = object.Parent();
    if !parent.is_null() {
        // Source: core/layout/layout_object.cc:4136-4153. Keep our layer
        // hierarchy updated. Optimize for the common case where we don't have
        // any children and don't have a layer attached to ourselves.
        let mut layer = std::ptr::null_mut();
        if !object.SlowFirstChild().is_null() || object.HasLayer() {
            layer = unsafe { &*parent }.EnclosingLayer();
            object.AddLayers(layer);
        }

        // If |this| is visible but this object was not, tell the layer it has
        // some visible content that needs to be drawn and layer visibility
        // optimization can't be used.
        if unsafe { &*parent }.StyleRef().Visibility() != foundation::EVisibility::kVisible
            && object.StyleRef().Visibility() == foundation::EVisibility::kVisible
            && !object.HasLayer()
        {
            if layer.is_null() {
                layer = unsafe { &*parent }.EnclosingLayer();
            }
            if !layer.is_null() {
                unsafe { &mut *layer }.DirtyVisibleContentStatus();
            }
        }
    }
    debug_assert!(!object.HasInlineFragments());
    if !parent.is_null() && unsafe { &*parent }.ChildrenInline() {
        unsafe { &mut *parent }.DirtyLinesFromChangedChild(object);
    }
    let element = DynamicTo::<Element>(object.GetNode());
    if !element.is_null() && unsafe { &*element }.InputMayBeImplicitAnchor() {
        object.MarkMayContainAnchor();
    } else if object.MayContainAnchor() && !parent.is_null() {
        unsafe { &mut *parent }.MarkMayContainAnchor();
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:218-223
#[inline(never)]
pub(super) fn LayoutObjectWillBeRemovedFromTreeBase(object: &mut LayoutObject) {
    object.CheckIsNotDestroyed();
    let parent = object.Parent();
    if !parent.is_null() {
        // Source: core/layout/layout_object.cc:4203-4218. If we remove a
        // visible child from an invisible parent, we don't know the layer
        // visibility any more.
        let mut layer = std::ptr::null_mut();
        if unsafe { &*parent }.StyleRef().Visibility() != foundation::EVisibility::kVisible
            && object.StyleRef().Visibility() == foundation::EVisibility::kVisible
            && !object.HasLayer()
        {
            layer = unsafe { &*parent }.EnclosingLayer();
            if !layer.is_null() {
                unsafe { &mut *layer }.DirtyVisibleContentStatus();
            }
        }

        // Keep our layer hierarchy updated. This runs before
        // LayoutObjectChildList::RemoveChildNode unlinks the object, so the
        // derived layer links still name `layer` as the parent.
        if !object.SlowFirstChild().is_null() || object.HasLayer() {
            if layer.is_null() {
                layer = unsafe { &*parent }.EnclosingLayer();
            }
            object.RemoveLayers(layer);
        }
    }
    if !parent.is_null() && unsafe { &*parent }.ChildrenInline() {
        unsafe { &mut *parent }.DirtyLinesFromChangedChild(object);
    }
    object.ClearScrollAnchorReferencesBeforeRemoval();
}

// This is the base implementation. Destroy() invokes the pending typed
// virtual dispatcher so concrete derived cleanup runs before this method.
// cpp: layoutng/internal/layout_object_core_services.cc:224-230
#[inline(never)]
pub(super) fn LayoutObjectWillBeDestroyed(object: &mut LayoutObject) {
    object.CheckIsNotDestroyed();
    debug_assert!(!object.IsText());
    let children = object.VirtualChildren();
    if !children.is_null() {
        unsafe { &mut *children }.DestroyLeftoverChildren();
    }
    let view = object.View();
    object.Remove();
    // LayoutObject.cc:4260-4264: FrameView retains these original members;
    // remove both before this native object can be destroyed.
    if let Some(view) = unsafe { view.as_ref() } {
        view.RemovePendingTransformUpdate(object);
        view.RemovePendingOpacityUpdate(object);
        if object.IsBackgroundAttachmentFixedObject() {
            view.RemoveBackgroundAttachmentFixedObject(object);
        }
    }
}
