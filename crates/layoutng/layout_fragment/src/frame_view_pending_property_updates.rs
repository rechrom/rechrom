//! LocalFrameView.cc:5647-5682,5713-5726. The resident native view retains
//! the actual LayoutObject members, never DOM ids or copied property values.
use super::{LayoutObject, LayoutView};
use foundation::{Member, Visitor};
use std::{cell::RefCell, collections::HashMap};

type PendingSet = RefCell<Option<HashMap<usize, Member<LayoutObject>>>>;
#[derive(Default)]
pub(super) struct FrameViewPendingPropertyUpdates {
    transform: PendingSet,
    opacity: PendingSet,
}
impl FrameViewPendingPropertyUpdates {
    pub(super) fn Trace(&self, visitor: &mut Visitor) {
        for set in [&self.transform, &self.opacity] {
            if let Some(set) = set.borrow().as_ref() {
                for object in set.values() {
                    visitor.Trace(object);
                }
            }
        }
    }
    fn Add(set: &PendingSet, object: &LayoutObject) {
        let pointer = object as *const LayoutObject as *mut LayoutObject;
        set.borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(pointer as usize, Member::from_ptr(pointer));
    }
    fn Remove(set: &PendingSet, object: &LayoutObject) -> bool {
        set.borrow_mut().as_mut().is_some_and(|set| {
            set.remove(&(object as *const LayoutObject as usize))
                .is_some()
        })
    }
    fn RemoveAll(set: &PendingSet) {
        if let Some(set) = set.borrow().as_ref() {
            for object in set.values() {
                // Original GC Member identity remains alive throughout this
                // synchronous marking pass. SetNeedsPaintPropertyUpdate marks
                // native ancestors and does not execute Style or callbacks.
                unsafe { &mut *object.Get() }.SetNeedsPaintPropertyUpdate();
            }
        }
        if let Some(set) = set.borrow_mut().as_mut() {
            set.clear();
        }
    }
}
impl LayoutView {
    pub fn AddPendingTransformUpdate(&self, object: &LayoutObject) {
        FrameViewPendingPropertyUpdates::Add(
            &self.frame_view_pending_property_updates_.transform,
            object,
        );
    }
    pub fn RemovePendingTransformUpdate(&self, object: &LayoutObject) -> bool {
        FrameViewPendingPropertyUpdates::Remove(
            &self.frame_view_pending_property_updates_.transform,
            object,
        )
    }
    pub fn AddPendingOpacityUpdate(&self, object: &LayoutObject) {
        FrameViewPendingPropertyUpdates::Add(
            &self.frame_view_pending_property_updates_.opacity,
            object,
        );
    }
    pub fn RemovePendingOpacityUpdate(&self, object: &LayoutObject) -> bool {
        FrameViewPendingPropertyUpdates::Remove(
            &self.frame_view_pending_property_updates_.opacity,
            object,
        )
    }
    pub fn RemoveAllPendingUpdates(&self) {
        FrameViewPendingPropertyUpdates::RemoveAll(
            &self.frame_view_pending_property_updates_.opacity,
        );
        FrameViewPendingPropertyUpdates::RemoveAll(
            &self.frame_view_pending_property_updates_.transform,
        );
    }
}
