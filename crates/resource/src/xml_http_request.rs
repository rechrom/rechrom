//! XMLHttpRequest's native request/error lifetime. Chromium 6c1d401f:
//! xml_http_request.cc565-604,649-726,728-747,982-1184,1187-1346,
//! 1648-1664,1680-1781,1802-1830,1874-1948,1986-2000;
//! xml_http_request_progress_event_throttle.cc42-189.
//! Script and EventTarget projection belongs to bindings, not this loader owner.
use crate::{
    RegisteredXMLHttpRequestOperation, ResourceCancellationClient, XMLHttpRequestData,
    XMLHttpResponseData,
};
use std::{
    cell::{Cell, RefCell},
    io,
    rc::{Rc, Weak},
    time::Duration,
};
use url_loader::URLRequest;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum XMLHttpRequestState {
    Unsent,
    Opened,
    HeadersReceived,
    Loading,
    Done,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XMLHttpRequestException {
    InvalidState,
    Network,
    Abort,
    Timeout,
}
#[derive(Clone, Copy, Debug)]
pub struct XMLHttpRequestProgress {
    pub length_computable: bool,
    pub loaded: u64,
    pub total: u64,
}
#[derive(Clone, Copy, Debug)]
pub enum XMLHttpRequestEvent {
    ReadyStateChange,
    Progress {
        kind: &'static str,
        data: XMLHttpRequestProgress,
    },
}
pub trait XMLHttpRequestTask {
    fn Cancel(&self);
}
pub trait XMLHttpRequestContext {
    fn IsContextDestroyed(&self) -> bool;
    fn Dispatch(&self, event: XMLHttpRequestEvent);
    fn PostNetworkingTask(
        &self,
        delay: Duration,
        run: Box<dyn FnOnce()>,
    ) -> Rc<dyn XMLHttpRequestTask>;
    /// The existing operation is registered before its first completion is
    /// delivered. The adapter keeps that same request identity in its IO task.
    fn StartRequest(
        &self,
        owner: &Rc<XMLHttpRequest>,
        request: XMLHttpRequestData,
        asynchronous: bool,
    ) -> io::Result<Rc<RegisteredXMLHttpRequestOperation>>;
    /// The synchronous transport waits at the embedding IO boundary; this
    /// source owner does not own an OS wait loop.
    fn FinishSynchronously(
        &self,
        loader: &RegisteredXMLHttpRequestOperation,
    ) -> io::Result<XMLHttpResponseData>;
    fn PendingActivityChanged(&self, active: bool);
}
pub struct XMLHttpRequest {
    self_: Weak<Self>,
    context: Rc<dyn XMLHttpRequestContext>,
    state: Cell<XMLHttpRequestState>,
    asynchronous: Cell<bool>,
    error: Cell<bool>,
    send_flag: Cell<bool>,
    upload_complete: Cell<bool>,
    exception: Cell<Option<XMLHttpRequestException>>,
    request: RefCell<XMLHttpRequestData>,
    loader: RefCell<Option<Rc<RegisteredXMLHttpRequestOperation>>>,
    response: RefCell<Option<XMLHttpResponseData>>,
    received_length: Cell<u64>,
    pending_abort: RefCell<Option<Rc<dyn XMLHttpRequestTask>>>,
    progress_timer: RefCell<Option<Rc<dyn XMLHttpRequestTask>>>,
    deferred_progress: Cell<Option<XMLHttpRequestProgress>>,
    has_dispatched_progress: Cell<bool>,
}
impl XMLHttpRequest {
    pub fn new(context: Rc<dyn XMLHttpRequestContext>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            context,
            state: Cell::new(XMLHttpRequestState::Unsent),
            asynchronous: Cell::new(true),
            error: Cell::new(false),
            send_flag: Cell::new(false),
            upload_complete: Cell::new(false),
            exception: Cell::new(None),
            request: Default::default(),
            loader: Default::default(),
            response: Default::default(),
            received_length: Cell::new(0),
            pending_abort: Default::default(),
            progress_timer: Default::default(),
            deferred_progress: Cell::new(None),
            has_dispatched_progress: Cell::new(false),
        })
    }
    pub fn ReadyState(&self) -> XMLHttpRequestState {
        self.state.get()
    }
    pub fn IsAsync(&self) -> bool {
        self.asynchronous.get()
    }
    pub fn SendFlag(&self) -> bool {
        self.send_flag.get()
    }
    pub fn HasError(&self) -> bool {
        self.error.get()
    }
    pub fn HasPendingActivity(&self) -> bool {
        self.loader.borrow().is_some()
    }
    pub fn WithResponse<R>(&self, read: impl FnOnce(Option<&XMLHttpResponseData>) -> R) -> R {
        read(self.response.borrow().as_ref())
    }
    pub fn OpenValidated(&self, request: XMLHttpRequestData, asynchronous: bool) {
        self.InternalAbort();
        let previous = self.state.replace(XMLHttpRequestState::Unsent);
        self.error.set(false);
        self.upload_complete.set(false);
        self.exception.set(None);
        *self.request.borrow_mut() = request;
        self.asynchronous.set(asynchronous);
        self.send_flag.set(false);
        if previous != XMLHttpRequestState::Opened {
            self.ChangeState(XMLHttpRequestState::Opened);
        } else {
            self.state.set(XMLHttpRequestState::Opened);
        }
    }
    pub fn SetRequestHeaderValidated(
        &self,
        name: String,
        value: String,
    ) -> Result<(), XMLHttpRequestException> {
        if self.state.get() != XMLHttpRequestState::Opened || self.send_flag.get() {
            return Err(XMLHttpRequestException::InvalidState);
        }
        let mut request = self.request.borrow_mut();
        if let Some((_, previous)) = request
            .headers
            .iter_mut()
            .find(|(key, _)| key.eq_ignore_ascii_case(&name))
        {
            previous.push_str(", ");
            previous.push_str(&value);
        } else {
            request.headers.push((name, value));
        }
        Ok(())
    }
    /// Validation of body type and synchronous-XHR policy is performed by the
    /// source binding before entering CreateRequest. No script state is copied.
    pub fn Send(self: &Rc<Self>, body: Vec<u8>) -> Result<(), XMLHttpRequestException> {
        if self.context.IsContextDestroyed() {
            self.HandleNetworkError();
            return Err(XMLHttpRequestException::Network);
        }
        if self.state.get() != XMLHttpRequestState::Opened || self.send_flag.get() {
            return Err(XMLHttpRequestException::InvalidState);
        }
        self.error.set(false);
        self.exception.set(None);
        {
            let mut request = self.request.borrow_mut();
            if request.method != "GET" && request.method != "HEAD" {
                request.body = body;
            }
        }
        self.send_flag.set(true);
        if self.asynchronous.get() {
            self.DispatchProgress("loadstart", 0, 0);
            if !self.send_flag.get() {
                return Ok(());
            }
        }
        let request = self.request.borrow().clone();
        match self
            .context
            .StartRequest(self, request, self.asynchronous.get())
        {
            Ok(loader) => {
                let client: Rc<dyn ResourceCancellationClient> = self.clone();
                loader.RegisterCancellationClient(&client);
                *self.loader.borrow_mut() = Some(loader.clone());
                self.context.PendingActivityChanged(true);
                if !self.asynchronous.get() {
                    match self.context.FinishSynchronously(&loader) {
                        Ok(response) => self.DeliverResponse(response),
                        Err(_) => self.HandleNetworkError(),
                    }
                }
            }
            Err(_) => self.HandleNetworkError(),
        }
        self.exception.get().map_or(Ok(()), Err)
    }
    pub fn Abort(&self) {
        self.InternalAbort();
        let state = self.state.get();
        if self.asynchronous.get()
            && ((state == XMLHttpRequestState::Opened && self.send_flag.get())
                || state == XMLHttpRequestState::HeadersReceived
                || state == XMLHttpRequestState::Loading)
        {
            self.HandleRequestError(None, "abort");
        }
        if self.state.get() == XMLHttpRequestState::Done {
            self.state.set(XMLHttpRequestState::Unsent);
        }
    }
    fn CancelTask(slot: &RefCell<Option<Rc<dyn XMLHttpRequestTask>>>) {
        let task = slot.borrow_mut().take();
        if let Some(task) = task {
            task.Cancel();
        }
    }
    pub fn InternalAbort(&self) {
        Self::CancelTask(&self.pending_abort);
        if self.error.get() && self.loader.borrow().is_none() {
            return;
        }
        self.error.set(true);
        self.received_length.set(0);
        self.response.borrow_mut().take();
        self.request.borrow_mut().headers.clear();
        let loader = self.loader.borrow_mut().take();
        if let Some(loader) = loader {
            loader.Cancel();
            self.context.PendingActivityChanged(false);
        }
    }
    pub fn ContextDestroyed(&self) {
        Self::CancelTask(&self.progress_timer);
        self.InternalAbort();
        self.send_flag.set(false);
    }
    pub fn HandleNetworkError(&self) {
        self.InternalAbort();
        self.HandleRequestError(Some(XMLHttpRequestException::Network), "error");
    }
    pub fn HandleTimeout(&self) {
        self.InternalAbort();
        self.HandleRequestError(Some(XMLHttpRequestException::Timeout), "timeout");
    }
    pub fn HandleDidCancel(&self) {
        self.InternalAbort();
        let owner = self.self_.upgrade().expect("live XHR notification");
        let task = self.context.PostNetworkingTask(
            Duration::ZERO,
            Box::new(move || {
                owner.pending_abort.borrow_mut().take();
                owner.HandleRequestError(Some(XMLHttpRequestException::Abort), "abort");
            }),
        );
        *self.pending_abort.borrow_mut() = Some(task);
    }
    fn HandleRequestError(&self, exception: Option<XMLHttpRequestException>, kind: &'static str) {
        self.send_flag.set(false);
        if !self.asynchronous.get() {
            assert!(exception.is_some());
            self.state.set(XMLHttpRequestState::Done);
            self.exception.set(exception);
            return;
        }
        debug_assert!(self.error.get());
        self.ChangeState(XMLHttpRequestState::Done);
        self.upload_complete.set(true);
        self.DispatchProgress(kind, 0, 0);
        self.DispatchProgress("loadend", 0, 0);
    }
    pub fn IsCurrentLoader(&self, identity: u64) -> bool {
        self.loader
            .borrow()
            .as_ref()
            .is_some_and(|loader| loader.Identity() == identity)
    }
    pub fn DidFail(&self, identity: u64, cancelled: bool, access_check: bool, timeout: bool) {
        if !self.IsCurrentLoader(identity) || self.error.get() {
            return;
        }
        if cancelled && !access_check {
            self.HandleDidCancel();
        } else if timeout {
            self.HandleTimeout();
        } else {
            self.HandleNetworkError();
        }
    }
    /// The transport's existing completion is projected as response/data/finish
    /// in original callback order. A state callback can abort this same loader.
    pub fn DeliverResponse(&self, mut response: XMLHttpResponseData) {
        if self.error.get() {
            return;
        }
        let body = std::mem::take(&mut response.body);
        *self.response.borrow_mut() = Some(response);
        if self.state.get() < XMLHttpRequestState::HeadersReceived {
            self.ChangeState(XMLHttpRequestState::HeadersReceived);
        }
        if self.error.get() {
            return;
        }
        if !body.is_empty() {
            let count = body.len() as u64;
            self.response
                .borrow_mut()
                .as_mut()
                .expect("original response")
                .body = body;
            self.received_length
                .set(self.received_length.get().saturating_add(count));
            self.ChangeState(XMLHttpRequestState::Loading);
            if self.asynchronous.get() {
                self.DispatchSnapshot("progress");
            }
        }
        if self.error.get() {
            return;
        }
        // EndLoading releases the ThreadableLoader before the DONE callback.
        let previous_error = self.error.replace(true);
        let loader = self.loader.borrow_mut().take();
        if let Some(loader) = loader {
            loader.Finish();
            self.context.PendingActivityChanged(false);
        }
        self.error.set(previous_error);
        self.send_flag.set(false);
        self.ChangeState(XMLHttpRequestState::Done);
    }
    fn ChangeState(&self, state: XMLHttpRequestState) {
        if self.state.replace(state) != state {
            self.DispatchReadyStateChange();
        }
    }
    fn DispatchReadyStateChange(&self) {
        if self.context.IsContextDestroyed() {
            return;
        }
        let state = self.state.get();
        if self.asynchronous.get()
            || state <= XMLHttpRequestState::Opened
            || state == XMLHttpRequestState::Done
        {
            if state == XMLHttpRequestState::Done {
                if self.error.get() {
                    self.deferred_progress.set(None);
                } else if let Some(progress) = self.deferred_progress.take() {
                    self.DispatchProgressProgressEvent(progress);
                }
                Self::CancelTask(&self.progress_timer);
            }
            self.has_dispatched_progress.set(false);
            if self.state.get() == state {
                self.context.Dispatch(XMLHttpRequestEvent::ReadyStateChange);
            }
        }
        if self.state.get() == XMLHttpRequestState::Done && !self.error.get() {
            self.DispatchSnapshot("load");
            self.DispatchSnapshot("loadend");
        }
    }
    fn DispatchSnapshot(&self, kind: &'static str) {
        let expected = self
            .response
            .borrow()
            .as_ref()
            .and_then(|response| response.expected_content_length);
        self.DispatchProgress(kind, self.received_length.get(), expected.unwrap_or(0));
    }
    fn DispatchProgress(&self, kind: &'static str, loaded: u64, expected: u64) {
        let length_computable = expected > 0 && loaded <= expected;
        let data = XMLHttpRequestProgress {
            length_computable,
            loaded,
            total: if length_computable { expected } else { 0 },
        };
        if kind != "progress" {
            self.context
                .Dispatch(XMLHttpRequestEvent::Progress { kind, data });
            return;
        }
        if self.progress_timer.borrow().is_some() {
            self.deferred_progress.set(Some(data));
        } else {
            self.DispatchProgressProgressEvent(data);
            self.StartProgressTimer();
        }
    }
    fn DispatchProgressProgressEvent(&self, data: XMLHttpRequestProgress) {
        let state = self.state.get();
        if state == XMLHttpRequestState::Loading && self.has_dispatched_progress.get() {
            self.context.Dispatch(XMLHttpRequestEvent::ReadyStateChange);
        }
        if self.state.get() != state {
            return;
        }
        self.has_dispatched_progress.set(true);
        self.context.Dispatch(XMLHttpRequestEvent::Progress {
            kind: "progress",
            data,
        });
    }
    fn StartProgressTimer(&self) {
        Self::CancelTask(&self.progress_timer);
        let weak = self.self_.clone();
        let task = self.context.PostNetworkingTask(
            Duration::from_millis(50),
            Box::new(move || {
                if let Some(owner) = weak.upgrade() {
                    owner.progress_timer.borrow_mut().take();
                    if let Some(data) = owner.deferred_progress.take() {
                        owner.DispatchProgressProgressEvent(data);
                        owner.StartProgressTimer();
                    }
                }
            }),
        );
        *self.progress_timer.borrow_mut() = Some(task);
    }
}
impl ResourceCancellationClient for XMLHttpRequest {
    fn ResourceCancelled(&self, identity: u64, _: &URLRequest) {
        self.DidFail(identity, true, false, false);
    }
}
impl Drop for XMLHttpRequest {
    fn drop(&mut self) {
        Self::CancelTask(&self.pending_abort);
        Self::CancelTask(&self.progress_timer);
        let loader = self.loader.get_mut().take();
        if let Some(loader) = loader {
            loader.Cancel();
        }
    }
}
