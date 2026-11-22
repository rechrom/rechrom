use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:745-745
    #[inline(never)]
    pub fn NotifyPriorityScrollAnchorStatusChanged(&mut self) {
        unsafe { LayoutObjectNotifyPriorityScrollAnchorStatusChangedProvider(self) }
    }
}

// cpp: layoutng/internal/layout_object_tree.cc:146-160
#[inline(never)]
pub(super) fn LayoutObjectDepthSlow(object: &LayoutObject) -> u32 {
    object.CheckIsNotDestroyed();
    debug_assert_eq!(u32::from(object.bitfields_.depth_), kMaxLayoutObjectDepth);
    let mut extra = 0u32;
    let mut current: *const LayoutObject = object;
    while !current.is_null() {
        let current_ref = unsafe { &*current };
        if u32::from(current_ref.bitfields_.depth_) < kMaxLayoutObjectDepth {
            return u32::from(current_ref.bitfields_.depth_).wrapping_add(extra);
        }
        extra = extra.wrapping_add(1);
        current = current_ref.Parent();
    }
    std::process::abort();
}

// cpp: layoutng/internal/layout_object_tree.cc:162-180
#[inline(never)]
pub(super) fn LayoutObjectSetDepthIncludingDescendants(object: &mut LayoutObject, depth: u32) {
    object.CheckIsNotDestroyed();
    let clamped_depth = depth.min(kMaxLayoutObjectDepth);
    if u32::from(object.bitfields_.depth_) == clamped_depth {
        return;
    }
    object.bitfields_.depth_ = clamped_depth as u16;
    let mut child = object.SlowFirstChild();
    while !child.is_null() {
        LayoutObjectSetDepthIncludingDescendants(unsafe { &mut *child }, clamped_depth + 1);
        child = unsafe { &*child }.NextSibling();
    }
}

// cpp: layoutng/internal/layout_node_data.cc:33-48
#[inline(never)]
pub(super) fn LayoutObjectSetScrollAnchorDisablingStyleChangedOnAncestor(
    object: &mut LayoutObject,
) {
    object.CheckIsNotDestroyed();
    let mut ancestor = object.Parent();
    let viewport_defining_element = object.ViewportDefiningElementForLayout();
    while !ancestor.is_null() {
        let block = DynamicTo::<LayoutBlock>(ancestor);
        if !block.is_null()
            && (unsafe { &*block }.IsScrollContainer()
                || unsafe { &*block }.GetNode() == viewport_defining_element as *mut Node)
        {
            unsafe { &mut *block }.SetScrollAnchorDisablingStyleChanged(true);
            return;
        }
        ancestor = unsafe { &*ancestor }.Parent();
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:271-279
#[inline(never)]
pub(super) fn LayoutObjectSetDescendantNeedsPaintPropertyUpdate(object: &mut LayoutObject) {
    object.CheckIsNotDestroyed();
    let mut ancestor: *mut LayoutObject = object;
    while !ancestor.is_null()
        && !unsafe { &*ancestor }
            .bitfields_
            .descendant_needs_paint_property_update_
    {
        let current = unsafe { &mut *ancestor };
        current.bitfields_.descendant_needs_paint_property_update_ = true;
        ancestor = current.Parent();
    }
}
