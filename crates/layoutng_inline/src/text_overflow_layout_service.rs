// C++: layoutng_inline/text_overflow_layout_service.cc.
#![allow(non_snake_case)]

use foundation::{DynamicTo, RuntimeEnabledFeatures};
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::text_overflow_post_layout_snapshot::TextOverflowPostLayoutSnapshot;

// cpp: layoutng_inline/text_overflow_layout_service.cc:41-71
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBlockFlowShouldTruncateOverflowingTextFromInline(
    flow: *const LayoutBlockFlow,
) -> bool {
    let flow = unsafe { &*flow };
    flow.CheckIsNotDestroyed();
    let mut object_to_check = flow as *const LayoutBlockFlow as *const LayoutObject;
    if flow.IsAnonymousBlockFlow() {
        let parent = flow.Parent();
        if parent.is_null() || !unsafe { &*parent }.BehavesLikeBlockContainer() {
            return false;
        }
        object_to_check = parent;
    }
    let object = unsafe { &*object_to_check };
    if !object.HasNonVisibleOverflow() || object.StyleRef().TextOverflow().IsClip() {
        return false;
    }
    if RuntimeEnabledFeatures::TextOverflowClipWithSelectionEnabled()
        && object.ContainsSelectionFocus()
    {
        return false;
    }
    if RuntimeEnabledFeatures::DisableEllipsisWhenScrolledEnabled() {
        let box_ = DynamicTo::<LayoutBox>(object_to_check);
        if !box_.is_null() {
            let scrollable_area = unsafe { &*box_ }.GetScrollableArea();
            if !scrollable_area.is_null() {
                let area = unsafe { &mut *scrollable_area };
                let mut snapshot = area.GetTextOverflowPostLayoutSnapshot();
                if snapshot.is_null() {
                    snapshot = TextOverflowPostLayoutSnapshot::new(area);
                }
                return !unsafe { &*snapshot }.IsScrolled();
            }
        }
    }
    true
}
