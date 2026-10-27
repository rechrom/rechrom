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
    debug_assert!(!object.HasInlineFragments());
    let parent = object.Parent();
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
    if !parent.is_null() && unsafe { &*parent }.ChildrenInline() {
        unsafe { &mut *parent }.DirtyLinesFromChangedChild(object);
    }
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
    object.Remove();
}
