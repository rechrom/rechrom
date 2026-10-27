use super::*;

// cpp: layoutng/internal/layout_box_lifecycle.cc:33-38
#[inline(never)]
pub(super) fn LayoutBoxWillBeDestroyed(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    ShapeOutsideInfo::RemoveInfo(box_);
    box_.DisassociatePhysicalFragments();
    box_.model_object_base_mut().WillBeDestroyed();
}

// cpp: layoutng/internal/layout_box_lifecycle.cc:40-44
#[inline(never)]
pub(super) fn LayoutBoxInsertedIntoTree(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    box_.model_object_base_mut().InsertedIntoTreeBase();
    box_.AddCustomLayoutChildIfNeeded();
}

// cpp: layoutng/internal/layout_box_lifecycle.cc:46-50
#[inline(never)]
pub(super) fn LayoutBoxWillBeRemovedFromTree(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    box_.ClearCustomLayoutChild();
    box_.model_object_base_mut().WillBeRemovedFromTreeBase();
}
