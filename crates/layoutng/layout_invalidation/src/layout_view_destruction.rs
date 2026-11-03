//! LayoutView-owned anonymous page teardown belongs to tree invalidation.

use super::*;

// cpp: layoutng/internal/layout_view_layout.cc:194-202
#[inline(never)]
pub(super) fn LayoutViewWillBeDestroyed(view: &mut LayoutView) {
    view.CheckIsNotDestroyed();
    for object in view.anonymous_page_objects_.iter() {
        let object = object.Get();
        if !unsafe { &*object }.BeingDestroyed() {
            unsafe { &mut *object }.Destroy();
        }
    }
    view.anonymous_page_objects_.clear();
    view.base_.WillBeDestroyed();
}
