#![allow(non_snake_case)]

use foundation::To;
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng_assembly::internal::layout_object::LayoutObject;

// cpp: layoutng_forms/inner_editor_tree.h:8-9
// cpp: layoutng_forms/inner_editor_tree.cc:8-22
pub fn RemoveInnerEditorChild(block: &mut LayoutBlockFlow, old_child: &mut LayoutObject) {
    if old_child.IsBR() && !block.FirstChild().is_null() {
        let next = To::<LayoutBlockFlow>(block.NextSibling());
        if !next.is_null() && unsafe { &*next }.IsAnonymous() {
            block.MoveAllChildrenToBefore(
                next.cast::<LayoutBoxModelObject>(),
                unsafe { &*next }.FirstChild(),
                true,
            );
        }
    }
    if block.FirstChild().is_null() {
        block.Destroy();
    }
}
