//! This Document's alias of its actual Window's navigation_ owner. The Window
//! retains the source across ClearForReuse; Document::Shutdown releases only
//! this alias. No URL/token value decides Window reuse.
use super::*;
pub trait WindowNavigationApiOwner {
    fn WindowIdentity(&self) -> u64;
    fn FocusedElementChanged(&self) {}
}
impl PersistentDocument {
    pub fn WindowNavigationApiOwner(&self) -> Option<Rc<dyn WindowNavigationApiOwner>> {
        self.window_navigation_api.borrow().clone()
    }
    pub fn InstallWindowNavigationApiOwner(&self, owner: Rc<dyn WindowNavigationApiOwner>) {
        assert!(
            self.window_navigation_api
                .borrow_mut()
                .replace(owner)
                .is_none(),
            "one current Window Navigation source"
        );
    }
    pub fn ClearDocumentWindowNavigationApiAlias(
        &self,
    ) -> Option<Rc<dyn WindowNavigationApiOwner>> {
        self.window_navigation_api.borrow_mut().take()
    }
}
