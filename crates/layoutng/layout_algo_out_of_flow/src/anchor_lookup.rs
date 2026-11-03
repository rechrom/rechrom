use super::*;

// cpp: layoutng/internal/layout_box_core_services.cc:54-79
#[inline(never)]
fn for_each_anchor_map_on_container(box_: &LayoutBox, mut function: impl FnMut(&AnchorMap)) {
    let container = box_.Container();
    if container.is_null() {
        return;
    }
    if unsafe { &*container }.IsLayoutBlock() {
        let block = DynamicTo::<LayoutBlock>(container);
        assert!(!block.is_null());
        for fragment in unsafe { &*block }.PhysicalFragments() {
            let map = fragment.GetAnchorMap();
            if !map.is_null() {
                function(unsafe { &*map });
            }
        }
        return;
    }
    assert!(unsafe { &*container }.IsLayoutInline());
    let inline = DynamicTo::<LayoutInline>(container);
    assert!(!inline.is_null());
    if !unsafe { &*inline }.HasInlineFragments() {
        return;
    }
    let mut cursor = InlineCursor::default();
    cursor.MoveToLayoutObject(unsafe { &*container });
    while cursor.IsNotNull() {
        let fragment = cursor.Current().BoxFragment();
        if !fragment.is_null() {
            let map = unsafe { &*fragment }.GetAnchorMap();
            if !map.is_null() {
                function(unsafe { &*map });
            }
        }
        cursor.MoveToNextForSameLayoutObject();
    }
}

// cpp: layoutng/internal/layout_box_core_services.cc:117-134
#[inline(never)]
pub fn LayoutBoxFindTargetAnchor(box_: &LayoutBox, name: &ScopedCSSName) -> *const LayoutObject {
    box_.CheckIsNotDestroyed();
    if !box_.IsOutOfFlowPositioned() {
        return std::ptr::null();
    }
    let scoped_name = ToAnchorScopedName(name, unsafe {
        &*(box_ as *const LayoutBox as *const LayoutObject)
    });
    let mut anchor: *const LayoutObject = std::ptr::null();
    for_each_anchor_map_on_container(box_, |map| {
        let candidate = map.AnchorLayoutObject(box_, scoped_name);
        if !candidate.is_null()
            && (anchor.is_null()
                || (anchor != candidate
                    && unsafe { &*anchor }.IsBeforeInPreOrderDefault(unsafe { &*candidate })))
        {
            anchor = candidate;
        }
    });
    anchor
}

// cpp: layoutng/internal/layout_box_core_services.cc:136-151
#[inline(never)]
pub fn LayoutBoxAcceptableImplicitAnchor(box_: &LayoutBox) -> *const LayoutObject {
    box_.CheckIsNotDestroyed();
    if !box_.IsOutOfFlowPositioned() {
        return std::ptr::null();
    }
    let element = DynamicTo::<Element>(box_.GetNode());
    let anchor_element = if element.is_null() {
        std::ptr::null_mut()
    } else {
        unsafe { &*element }.container.node.ImplicitAnchorElement()
    };
    let anchor = if anchor_element.is_null() {
        std::ptr::null_mut()
    } else {
        unsafe { &*anchor_element }.container.node.GetLayoutObject()
    };
    if anchor.is_null() {
        return std::ptr::null();
    }
    let mut acceptable = false;
    for_each_anchor_map_on_container(box_, |map| {
        acceptable |= !map.AnchorLayoutObject(box_, anchor_element).is_null();
    });
    if acceptable {
        anchor
    } else {
        std::ptr::null()
    }
}
