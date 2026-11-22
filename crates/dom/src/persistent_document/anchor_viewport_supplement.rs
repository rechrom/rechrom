//! Document's one AnchorElementViewportPositionTracker supplement. The native
//! owner retains its source observation state; DOM retains its original life.
use super::*;
pub trait DocumentAnchorViewportOwner {
    fn OnFirstContentfulPaint(&self);
    fn OnScrollEnd(&self);
}
impl PersistentDocument {
    pub fn InstallAnchorViewportTracker(&self, owner: Rc<dyn DocumentAnchorViewportOwner>) {
        assert!(
            self.anchor_viewport_tracker
                .borrow_mut()
                .replace(owner)
                .is_none(),
            "one Document viewport tracker supplement"
        );
    }
    pub fn ExistingAnchorViewportTracker(&self) -> Option<Rc<dyn DocumentAnchorViewportOwner>> {
        self.anchor_viewport_tracker.borrow().clone()
    }
}
