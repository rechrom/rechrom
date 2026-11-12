//! Original LazyLoadFrameObserver request/observer lifetime. Source
//! lazy_load_frame_observer.cc:68-168. Viewport observation is a typed native
//! owner, never an iframe-tag test or an invented eager-load fallback.
use crate::{
    content_frames::LazyLoadFrameObserverSource,
    frame_owner_loading::{FrameOwnerLoadError, FrameOwnerResourceRequest},
    persistent_document::DOMOwnerHandle,
};
use navigation_history::FrameLoadType;
/// LazyLoadFrameObserver.cc:30-52; Settings.json5:1067-1095. The embedding
/// supplies NetworkStateNotifier's measured effective type (initially Unknown).
pub fn IntersectionParams(
    connection: foundation::network_connection_type::WebEffectiveConnectionType,
) -> intersection_observer::IntersectionObserverParams {
    use foundation::network_connection_type::WebEffectiveConnectionType as E;
    let margin = match connection {
        E::Unknown => 4000,
        E::Offline | E::Slow2G => 8000,
        E::Type2G => 6000,
        E::Type3G => 3500,
        E::Type4G => 2500,
    };
    intersection_observer::IntersectionObserverParams {
        scroll_margin: vec![foundation::Length::Fixed(margin)],
        thresholds: vec![intersection_observer::MINIMUM_THRESHOLD],
        ..Default::default()
    }
}
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy)]
pub enum LazyFrameLoadKind {
    First,
    Subsequent,
}
pub trait LazyFrameIntersectionSubscription {
    fn Disconnect(&self);
}
pub trait LazyLoadFrameClient {
    fn ObserveNearViewport(
        &self,
        owner: DOMOwnerHandle,
        observer: Weak<LazyLoadFrameObserver>,
    ) -> Result<Rc<dyn LazyFrameIntersectionSubscription>, FrameOwnerLoadError>;
    fn Navigate(
        &self,
        kind: LazyFrameLoadKind,
        request: FrameOwnerResourceRequest,
        load_type: FrameLoadType,
        observer: &dyn LazyLoadFrameObserverSource,
    ) -> Result<(), FrameOwnerLoadError>;
}
struct LazyLoadRequestInfo {
    request: FrameOwnerResourceRequest,
    load_type: FrameLoadType,
}
pub struct LazyLoadFrameObserver {
    owner: DOMOwnerHandle,
    kind: LazyFrameLoadKind,
    client: Rc<dyn LazyLoadFrameClient>,
    weak: Weak<Self>,
    request: RefCell<Option<LazyLoadRequestInfo>>,
    intersection: RefCell<Option<Rc<dyn LazyFrameIntersectionSubscription>>>,
}
impl LazyLoadFrameObserver {
    pub fn new(
        owner: DOMOwnerHandle,
        kind: LazyFrameLoadKind,
        client: Rc<dyn LazyLoadFrameClient>,
    ) -> Rc<Self> {
        Rc::new_cyclic(|weak| Self {
            owner,
            kind,
            client,
            weak: weak.clone(),
            request: Default::default(),
            intersection: Default::default(),
        })
    }
    /// The native IntersectionObserver supplies the last entry for this owner.
    pub fn LoadIfNearViewport(
        &self,
        owner: DOMOwnerHandle,
        is_intersecting: bool,
    ) -> Result<(), FrameOwnerLoadError> {
        assert_eq!(owner, self.owner);
        if is_intersecting {
            self.LoadImmediately()?;
        }
        Ok(())
    }
}
impl LazyLoadFrameObserverSource for LazyLoadFrameObserver {
    fn CancelPendingLazyLoad(&self) {
        self.request.borrow_mut().take();
        let observer = self.intersection.borrow_mut().take();
        if let Some(observer) = observer {
            observer.Disconnect();
        }
    }
    fn IsLazyLoadPending(&self) -> bool {
        self.intersection.borrow().is_some()
    }
    fn LoadImmediately(&self) -> Result<(), FrameOwnerLoadError> {
        assert!(self.IsLazyLoadPending());
        let pending = self
            .request
            .borrow_mut()
            .take()
            .expect("source pending lazy frame request");
        // Move out before navigation, allowing the original synchronous
        // callback to cancel this same observer during loader reentry.
        if let Err(error) =
            self.client
                .Navigate(self.kind, pending.request.clone(), pending.load_type, self)
        {
            *self.request.borrow_mut() = Some(pending);
            return Err(error);
        }
        assert!(!self.IsLazyLoadPending());
        Ok(())
    }
    fn DeferLoadUntilNearViewport(
        &self,
        request: FrameOwnerResourceRequest,
        load_type: FrameLoadType,
    ) -> Result<(), FrameOwnerLoadError> {
        assert!(self.intersection.borrow().is_none() && self.request.borrow().is_none());
        *self.request.borrow_mut() = Some(LazyLoadRequestInfo { request, load_type });
        let observer = self
            .client
            .ObserveNearViewport(self.owner, self.weak.clone())?;
        *self.intersection.borrow_mut() = Some(observer);
        Ok(())
    }
}
