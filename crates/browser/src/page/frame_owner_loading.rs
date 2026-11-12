//! Native frame-owner loading assembly. DOM owns all source branch policy.
//! Page supplies one call's live parent Frame facts before DOM is borrowed.
//! Missing original policy/FrameLoader/LocalFrameClient owners stay required
//! ports; they never create an empty policy, a null child, or another Page.
use super::*;
use dom::{
    content_frames::{ContentFrameSource, LazyLoadFrameObserverSource},
    document_base_url::DocumentURLValue,
    frame_owner_loading::{
        FrameOwnerLoadEffect, FrameOwnerLoadError, FrameOwnerLoadingClient,
        FrameOwnerLoadingServices, FrameOwnerNavigation, FrameOwnerParentFacts,
        FrameTrustTokenParams,
    },
    persistent_document::{DOMOwnerHandle, SecurityOrigin},
};

pub(super) struct PageFrameOwnerLoadingClient {
    pub domain: std::rc::Weak<super::local_frame::LocalFrameDomain>,
    pub parent: u64,
}
struct PageFrameOwnerLoadingServices {
    document: DOMOwnerHandle,
    parent: Result<Option<FrameOwnerParentFacts>, FrameOwnerLoadError>,
    origin: Rc<SecurityOrigin>,
    domain: Rc<super::local_frame::LocalFrameDomain>,
    parent_identity: u64,
    has_origin_window: bool,
    public_urls: Rc<dom::public_url_manager::PublicURLManager>,
    intersection_target: Rc<dyn intersection_observer::IntersectionObserverTarget>,
}
impl FrameOwnerLoadingClient for PageFrameOwnerLoadingClient {
    fn ServicesForOwner(
        &self,
        node: DOMOwnerHandle,
    ) -> Result<Box<dyn FrameOwnerLoadingServices + '_>, FrameOwnerLoadError> {
        let domain = self
            .domain
            .upgrade()
            .ok_or(FrameOwnerLoadError::MissingClient)?;
        let document_owner = domain
            .Document(self.parent)
            .ok_or(FrameOwnerLoadError::ParentFrameFacts)?;
        let owner = document_owner.borrow();
        let document = owner.GetDocument();
        let load_type = domain.LoadType(self.parent);
        let target =
            domain.EnsureIntersectionTarget(self.parent, &document_owner, document, node)?;
        let parent = if !document.IsActive() {
            Ok(None)
        } else {
            load_type
                .ok_or(FrameOwnerLoadError::ParentFrameFacts)
                .map(|load_type| {
                    Some(FrameOwnerParentFacts {
                        subframe_count: domain.SubframeCount(),
                        // Source page.cc:117,899-904. This host exposes no testing override.
                        maximum_frames: 1000,
                        load_type,
                        // Source settings.json5:1059-1062; no host override exists yet.
                        lazy_load_enabled: true,
                        page_visible: domain.IsVisible(),
                        can_execute_scripts: document.ScriptingEnabled()
                            && !document
                                .SecurityContext()
                                .IsSandboxed(foundation::security::SandboxFlags::SCRIPTS),
                    })
                })
        };
        Ok(Box::new(PageFrameOwnerLoadingServices {
            document: document.RootHandle(),
            parent,
            origin: document.SecurityContext().Origin().clone(),
            domain: domain.clone(),
            parent_identity: self.parent,
            has_origin_window: document.HasDOMWindow(),
            public_urls: document.SecurityContext().PublicURLManager().clone(),
            intersection_target: target,
        }))
    }
    fn CreateLocalFrame(
        &self,
        owner: DOMOwnerHandle,
        name: &[u16],
        is_created_by_script: bool,
    ) -> Result<Option<Rc<dyn ContentFrameSource>>, FrameOwnerLoadError> {
        let domain = self
            .domain
            .upgrade()
            .ok_or(FrameOwnerLoadError::MissingClient)?;
        domain
            .Create(self.parent, owner, name, is_created_by_script)
            .map(|frame| Some(frame as Rc<dyn ContentFrameSource>))
    }
    fn RouteEffect(&self, effect: &FrameOwnerLoadEffect) -> Result<(), FrameOwnerLoadError> {
        let domain = self
            .domain
            .upgrade()
            .ok_or(FrameOwnerLoadError::MissingClient)?;
        domain.RouteEffect(effect)
    }
}
impl FrameOwnerLoadingServices for PageFrameOwnerLoadingServices {
    fn ParentFrame(
        &self,
        document: DOMOwnerHandle,
    ) -> Result<Option<FrameOwnerParentFacts>, FrameOwnerLoadError> {
        if document == self.document {
            self.parent
        } else {
            Ok(None)
        }
    }
    fn CanNavigate(
        &self,
        target: &dyn ContentFrameSource,
        url: &DocumentURLValue,
    ) -> Result<bool, FrameOwnerLoadError> {
        // HTMLFrameElementBase only asks about its own ContentFrame. Restrict
        // this port to that actual direct descendant; LocalFrame::CanNavigate
        // cc:2191-2224,2233-2243,2320-2340 then reaches CanAccessAncestor(self).
        let frame = self
            .domain
            .Frame(target.State().Identity())
            .ok_or(FrameOwnerLoadError::NavigationPolicy)?;
        if frame.parent != self.parent_identity {
            return Err(FrameOwnerLoadError::NavigationPolicy);
        }
        if url
            .ParsedURL()
            .is_some_and(|url| url.scheme() == "javascript")
        {
            let target = self
                .domain
                .Document(frame.State().Identity())
                .ok_or(FrameOwnerLoadError::NavigationPolicy)?;
            return Ok(self
                .origin
                .CanAccess(target.borrow().GetDocument().SecurityContext().Origin()));
        }
        Ok(true)
    }
    fn TrustTokenParams(
        &self,
        _: DOMOwnerHandle,
        _: &str,
    ) -> Result<Option<Rc<dyn FrameTrustTokenParams>>, FrameOwnerLoadError> {
        Err(FrameOwnerLoadError::TrustTokenParams)
    }
    fn MonotonicTime(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
    fn CanAccessURL(&self, url: &DocumentURLValue) -> bool {
        let target = SecurityOrigin::CreateForNavigation(
            url.AsStr(),
            None,
            Rc::new(self.origin.SchemeRegistryForRequests()),
        );
        self.origin.CanAccess(target.as_ref())
    }
    fn NewLazyLoadObserver(
        &self,
        owner: DOMOwnerHandle,
        frame: &dyn ContentFrameSource,
    ) -> Result<Rc<dyn LazyLoadFrameObserverSource>, FrameOwnerLoadError> {
        let frame = self
            .domain
            .Frame(frame.State().Identity())
            .ok_or(FrameOwnerLoadError::LazyLoadObserver)?;
        if frame.parent != self.parent_identity || frame.State().Owner() != Some(owner) {
            return Err(FrameOwnerLoadError::LazyLoadObserver);
        }
        Ok(dom::lazy_load_frame_observer::LazyLoadFrameObserver::new(
            owner,
            dom::lazy_load_frame_observer::LazyFrameLoadKind::Subsequent,
            Rc::new(NativeLazyLoadFrameClient {
                domain: Rc::downgrade(&self.domain),
                frame: Rc::downgrade(&frame),
                owner,
                target: self.intersection_target.clone(),
            }),
        ))
    }
    fn AllowRequestForThisFrame(
        &self,
        frame: &dyn ContentFrameSource,
        navigation: &FrameOwnerNavigation,
    ) -> Result<bool, FrameOwnerLoadError> {
        // FrameLoader.cc:601-631. The source origin Window is the owner
        // document; no independent Page or root document is substituted.
        if !self.has_origin_window {
            return Err(FrameOwnerLoadError::RequestAdmission);
        }
        let child = self
            .domain
            .Frame(frame.State().Identity())
            .ok_or(FrameOwnerLoadError::RequestAdmission)?;
        if child.parent != self.parent_identity || child.State().Owner() != Some(navigation.owner) {
            return Err(FrameOwnerLoadError::RequestAdmission);
        }
        let Some(url) = navigation.request.url.ParsedURL() else {
            return Ok(true);
        };
        if url.scheme() == "javascript" {
            // Inline Javascript URL admission is Window/ScriptController's
            // source capability; neither CSP nor sandbox is bypassed here.
            return Err(FrameOwnerLoadError::RequestAdmission);
        }
        let allowed = self
            .origin
            .CanDisplay(url, self.public_urls.OriginForURL(url.as_str()).as_ref());
        if !allowed {
            return Err(FrameOwnerLoadError::RequestAdmission);
        }
        Ok(true)
    }
}
struct NativeLazyLoadFrameClient {
    domain: Weak<super::local_frame::LocalFrameDomain>,
    frame: Weak<super::local_frame::LocalFrame>,
    owner: DOMOwnerHandle,
    target: Rc<dyn intersection_observer::IntersectionObserverTarget>,
}
impl dom::lazy_load_frame_observer::LazyLoadFrameClient for NativeLazyLoadFrameClient {
    fn ObserveNearViewport(
        &self,
        owner: DOMOwnerHandle,
        observer: Weak<dom::lazy_load_frame_observer::LazyLoadFrameObserver>,
    ) -> Result<
        Rc<dyn dom::lazy_load_frame_observer::LazyFrameIntersectionSubscription>,
        FrameOwnerLoadError,
    > {
        let domain = self
            .domain
            .upgrade()
            .ok_or(FrameOwnerLoadError::MissingClient)?;
        domain.ObserveLazyFrame(owner, self.target.clone(), observer)
    }
    fn Navigate(
        &self,
        kind: dom::lazy_load_frame_observer::LazyFrameLoadKind,
        request: dom::frame_owner_loading::FrameOwnerResourceRequest,
        load_type: navigation_history::FrameLoadType,
        observer: &dyn LazyLoadFrameObserverSource,
    ) -> Result<(), FrameOwnerLoadError> {
        let domain = self
            .domain
            .upgrade()
            .ok_or(FrameOwnerLoadError::MissingClient)?;
        let frame = self
            .frame
            .upgrade()
            .ok_or(FrameOwnerLoadError::UnexpectedCreatedFrame)?;
        if frame.State().Owner() != Some(self.owner) {
            return Err(FrameOwnerLoadError::UnexpectedCreatedFrame);
        }
        let policy = frame
            .pending_policy
            .borrow()
            .clone()
            .ok_or(FrameOwnerLoadError::ContainerPolicy)?;
        let navigation = FrameOwnerNavigation {
            owner: self.owner,
            request,
            policy,
            load_type,
            reason: dom::frame_owner_loading::FrameOwnerNavigationReason::None,
            is_container_initiated: false,
        };
        let frame: Rc<dyn ContentFrameSource> = frame;
        // FrameLoader/Frame::Navigate cancels this original observer before
        // dispatching the navigation; no parent DOM borrow is reacquired.
        observer.CancelPendingLazyLoad();
        domain.RouteEffect(&match kind {
            dom::lazy_load_frame_observer::LazyFrameLoadKind::First => {
                FrameOwnerLoadEffect::StartNavigation { frame, navigation }
            }
            dom::lazy_load_frame_observer::LazyFrameLoadKind::Subsequent => {
                FrameOwnerLoadEffect::Navigate { frame, navigation }
            }
        })
    }
}
