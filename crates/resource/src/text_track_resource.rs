//! RawResource::FetchTextTrack (raw_resource.cc:80-87) over the original
//! Fetch/CORS transport, with RawResourceClient streaming and cancellation.
use crate::{
    raw_resource::RawRequestKind, resource_loader::NextResourceLoadIdentity, RawResource,
    ResourceEngine, ResourceLoadStatus,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::Arc,
};
use url_loader::{URLLoadEvent, URLRequest, URLStreamOperation, URLWakeCallback};
pub trait TextTrackResourceClient {
    fn DataReceived(&self, resource_id: u64, data: &[u8]);
    fn NotifyFinished(&self, resource_id: u64, error: bool);
}
pub struct TextTrackResource {
    id: u64,
    request: Arc<URLRequest>,
    stream: RefCell<Option<Box<dyn URLStreamOperation>>>,
    client: Weak<dyn TextTrackResourceClient>,
    status: Cell<ResourceLoadStatus>,
    notified: Cell<bool>,
    _blocked: Option<RawResource>,
}
impl TextTrackResource {
    pub fn Identity(&self) -> u64 {
        self.id
    }
    pub fn Request(&self) -> &Arc<URLRequest> {
        &self.request
    }
    pub fn Status(&self) -> ResourceLoadStatus {
        self.status.get()
    }
    /// ResourceFetcher::ResourceForBlockedRequest cc1144-1156 binds its client
    /// before FinishAsError. The document adapter invokes this original second
    /// stage immediately after binding, rather than delaying a policy failure
    /// behind a later transport poll or dispatching the track event itself.
    pub fn FinishBlockedRequestAfterClientBinding(&self) {
        if self._blocked.is_some() {
            self.NotifyFinished();
        }
    }
    pub fn Cancel(&self) {
        self.stream.borrow_mut().take();
        if self.status.get() == ResourceLoadStatus::kLoading {
            self.status.set(ResourceLoadStatus::kCancelled);
        }
    }
    pub(crate) fn CancelForFetcher(&self) {
        if self.status.get() == ResourceLoadStatus::kLoading {
            self.Cancel();
            self.NotifyFinished();
        }
    }
    pub(crate) fn SetWakeCallback(&self, wake: URLWakeCallback) {
        if let Some(stream) = self.stream.borrow_mut().as_mut() {
            stream.SetWakeCallback(wake);
        }
    }
    fn NotifyFinished(&self) {
        self.stream.borrow_mut().take();
        if !self.notified.replace(true) {
            if let Some(client) = self.client.upgrade() {
                client.NotifyFinished(self.id, self.status.get() != ResourceLoadStatus::kComplete);
            }
        }
    }
    fn Poll(&self) -> (bool, bool) {
        if self.status.get() == ResourceLoadStatus::kCancelled {
            return (true, false);
        }
        if self.status.get() != ResourceLoadStatus::kLoading {
            self.NotifyFinished();
            return (true, false);
        }
        // Release the transport borrow before a native parser/client can reenter
        // cancellation or resource selection on the same original resource.
        let event = {
            let mut stream = self.stream.borrow_mut();
            stream
                .as_mut()
                .expect("active text-track stream")
                .Poll(65536)
        };
        match event {
            Ok(None) => (false, false),
            Ok(Some(URLLoadEvent::Response(head))) => {
                if head.status_code >= 400 {
                    self.status.set(ResourceLoadStatus::kFailed);
                    self.NotifyFinished();
                    return (true, true);
                }
                (false, true)
            }
            Ok(Some(URLLoadEvent::Data(bytes))) => {
                if let Some(client) = self.client.upgrade() {
                    client.DataReceived(self.id, &bytes);
                }
                (false, true)
            }
            Ok(Some(URLLoadEvent::Finished)) => {
                self.status.set(ResourceLoadStatus::kComplete);
                self.NotifyFinished();
                (true, true)
            }
            Err(_) => {
                self.status.set(ResourceLoadStatus::kFailed);
                self.NotifyFinished();
                (true, true)
            }
        }
    }
}
impl ResourceEngine {
    pub fn FetchTextTrack(
        &self,
        request: URLRequest,
        client: Weak<dyn TextTrackResourceClient>,
    ) -> Rc<TextTrackResource> {
        let _request = crate::fetcher_lifecycle::RequestResourceScope::new(
            self.is_in_request_resource.clone(),
        );
        // ResourceFetcher::ResourceForBlockedRequest, cc:1144-1156, creates
        // the original failed Resource and binds its client too. A policy
        // rejection is not a null FetchTextTrack result or a direct track event.
        let (id, request, stream, status, blocked_owner) =
            match self.PrepareRawResourceRequest(request, RawRequestKind::TextTrack) {
                Ok(request) => {
                    let id = NextResourceLoadIdentity();
                    let request = Arc::new(request);
                    if let Some(observer) = self.raw_observer.borrow().clone() {
                        observer.WillSendRequest(id, &request);
                    }
                    match self.loader.borrow_mut().LoadStream(&request) {
                        Ok(stream) => (
                            id,
                            request,
                            Some(stream),
                            ResourceLoadStatus::kLoading,
                            None,
                        ),
                        Err(_) => (id, request, None, ResourceLoadStatus::kFailed, None),
                    }
                }
                Err(blocked) => {
                    let id = blocked.Identity();
                    let request = blocked
                        .Request()
                        .expect("blocked request retains its original ResourceRequest");
                    let status = blocked.GetStatus();
                    (id, request, None, status, Some(blocked))
                }
            };
        let resource = Rc::new(TextTrackResource {
            id,
            request,
            stream: RefCell::new(stream),
            client,
            status: Cell::new(status),
            notified: Cell::new(false),
            _blocked: blocked_owner,
        });
        if let Some(wake) = self.wake_callback.borrow().clone() {
            resource.SetWakeCallback(wake);
        }
        self.text_track_resources
            .borrow_mut()
            .insert(id, resource.clone());
        self.RequestWake();
        resource
    }
    pub(crate) fn PollTextTrackResources(&self) -> usize {
        let resources = self
            .text_track_resources
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let mut completed = 0;
        for resource in resources {
            let (finished, progress) = resource.Poll();
            if finished {
                self.text_track_resources.borrow_mut().remove(&resource.id);
                completed += 1;
                if let Some(observer) = self.raw_observer.borrow().clone() {
                    observer.DidFinishLoading(resource.id, resource.Status());
                }
            } else if progress {
                // A bounded stream may already contain its next record. Keep
                // the existing resource wake/turn owner responsible for it.
                self.RequestWake();
            }
        }
        completed
    }
    pub(crate) fn CancelTextTrackResources(&self) {
        let resources = std::mem::take(&mut *self.text_track_resources.borrow_mut());
        for resource in resources.into_values() {
            resource.Cancel();
        }
    }
}
