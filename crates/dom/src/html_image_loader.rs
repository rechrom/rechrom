// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! ImageLoader and HTMLImageLoader's retained per-element request lifecycle.
//! Chromium image_loader.h:66-285; image_loader.cc:147-330,400-722,745-911,
//! 957-991,1042-1094; html_image_loader.cc:61-146.
use super::*;
use crate::load_event_delay_count::{LoadEventDelayCounter, LoadEventDelayLease};
use std::cell::{Cell, RefCell};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageLoaderUpdate {
    Normal,
    FromMicrotask,
    IgnorePreviousError,
    SizeChanged,
    ForcedReload,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageLoaderTaskKind {
    Update,
    Decode(u64),
    ResumeDeferred,
}
#[derive(Clone)]
pub struct HTMLImageLoaderTask {
    pub loader: Rc<HTMLImageLoader>,
    pub generation: u64,
    pub kind: ImageLoaderTaskKind,
    pub update: ImageLoaderUpdate,
    pub force_blocking: bool,
    pub synchronous: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageLoaderEvent {
    Load,
    Error,
}
#[derive(Clone)]
pub struct HTMLImageLoaderEventTicket {
    pub loader: Rc<HTMLImageLoader>,
    pub generation: u64,
    pub event: ImageLoaderEvent,
    delay: Rc<RefCell<Option<LoadEventDelayLease>>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageDecodeState {
    PendingMicrotask,
    PendingLoad,
    Dispatched,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageDecodeResult {
    Resolved,
    EncodingError,
}
pub enum ImageLoaderInputContent {
    NoChange,
    Primary,
    Fallback,
}
struct DecodeRequest {
    id: u64,
    state: ImageDecodeState,
}
struct LoaderState {
    image_content: Option<Rc<DOMImageResourceContent>>,
    complete: bool,
    lazy_deferred: bool,
    lazy_full_image: bool,
    source: Option<String>,
    failed_load_url: Option<String>,
    pending_task: Option<u64>,
    update_delay: Option<LoadEventDelayLease>,
    load_delay: Option<Rc<RefCell<Option<LoadEventDelayLease>>>>,
    error_delay: Option<Rc<RefCell<Option<LoadEventDelayLease>>>>,
    pending_load: bool,
    load_task_queued: bool,
    error_task_queued: bool,
    pending_error: bool,
    suppress_errors: bool,
    generation: u64,
    decodes: Vec<DecodeRequest>,
    decode_results: Vec<(u64, ImageDecodeResult)>,
}
pub struct HTMLImageLoader {
    self_: std::rc::Weak<HTMLImageLoader>,
    load_counter: RefCell<Rc<LoadEventDelayCounter>>,
    element: DOMOwnerHandle,
    disposed: Cell<bool>,
    state: RefCell<LoaderState>,
}
impl HTMLImageLoader {
    fn new(element: DOMOwnerHandle, counter: Rc<LoadEventDelayCounter>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            load_counter: RefCell::new(counter),
            element,
            disposed: Cell::new(false),
            state: RefCell::new(LoaderState {
                image_content: None,
                complete: true,
                lazy_deferred: false,
                lazy_full_image: false,
                source: None,
                failed_load_url: None,
                pending_task: None,
                update_delay: None,
                load_delay: None,
                error_delay: None,
                pending_load: false,
                load_task_queued: false,
                error_task_queued: false,
                pending_error: false,
                suppress_errors: false,
                generation: 0,
                decodes: vec![],
                decode_results: vec![],
            }),
        })
    }
    pub fn InputContentDisposition(
        &self,
        no_resource: bool,
        alt_empty: bool,
    ) -> ImageLoaderInputContent {
        if no_resource {
            return if alt_empty {
                ImageLoaderInputContent::NoChange
            } else {
                ImageLoaderInputContent::Fallback
            };
        }
        match self.GetContent() {
            Some(content) if content.ErrorOccurred() => ImageLoaderInputContent::Fallback,
            Some(_) => ImageLoaderInputContent::Primary,
            None => ImageLoaderInputContent::NoChange,
        }
    }
    pub fn Element(&self) -> DOMOwnerHandle {
        self.element
    }
    pub fn GetContent(&self) -> Option<Rc<DOMImageResourceContent>> {
        self.state.borrow().image_content.clone()
    }
    pub fn InitializeSource(&self, source: Option<String>) {
        let mut state = self.state.borrow_mut();
        assert_eq!(state.generation, 0);
        state.complete = source.as_ref().is_none_or(|url| url.is_empty());
        state.source = source;
    }
    pub fn HasStarted(&self) -> bool {
        self.state.borrow().generation != 0
    }
    pub fn ImageComplete(&self) -> bool {
        self.state.borrow().complete
    }
    pub fn CurrentSource(&self) -> Option<String> {
        self.state.borrow().source.clone()
    }
    pub fn HasPendingActivity(&self) -> bool {
        let s = self.state.borrow();
        s.pending_task.is_some()
            || s.pending_load
            || s.pending_error
            || !s.decodes.is_empty()
            || (s.image_content.is_some() && !s.complete && !s.lazy_deferred)
    }
    pub fn HasPendingError(&self) -> bool {
        self.state.borrow().pending_error
    }
    pub fn HadError(&self) -> bool {
        self.state
            .borrow()
            .failed_load_url
            .as_ref()
            .is_some_and(|url| !url.is_empty())
    }
    pub fn ImageIsPotentiallyAvailable(&self) -> bool {
        let s = self.state.borrow();
        s.lazy_deferred
            || s.image_content.as_ref().is_some_and(|content| {
                !content.IsLoading() && !content.ErrorOccurred() && content.HasImage()
            })
            || ((s.pending_task.is_some() || s.pending_load || s.pending_error)
                && !s.pending_error
                && s.source.as_ref().is_some_and(|url| !url.is_empty()))
    }
    /// UpdateFromElement cc:616-695. The pending task owns the loader; replacing
    /// its generation cancels that task without disposing another request.
    pub fn UpdateFromElement(
        self: &Rc<Self>,
        source: Option<String>,
        active: bool,
        update: ImageLoaderUpdate,
        immediately: bool,
        force_blocking: bool,
    ) -> Option<HTMLImageLoaderTask> {
        if !active || self.disposed.get() {
            return None;
        }
        let old = {
            let mut s = self.state.borrow_mut();
            s.suppress_errors = update == ImageLoaderUpdate::SizeChanged;
            if update == ImageLoaderUpdate::IgnorePreviousError {
                s.failed_load_url = None;
            }
            if s.failed_load_url
                .as_ref()
                .is_some_and(|failed| Some(failed) == source.as_ref())
            {
                return None;
            }
            s.generation = s
                .generation
                .checked_add(1)
                .expect("ImageLoader task identity overflow");
            s.pending_task = None;
            s.update_delay = None;
            s.source = source.clone();
            let synchronous = immediately && update != ImageLoaderUpdate::FromMicrotask;
            if synchronous {
                return Some(HTMLImageLoaderTask {
                    loader: self.clone(),
                    generation: s.generation,
                    kind: ImageLoaderTaskKind::Update,
                    update,
                    force_blocking,
                    synchronous: true,
                });
            }
            s.pending_task = Some(s.generation);
            s.update_delay = Some(self.load_counter.borrow().Acquire());
            if source.as_ref().is_none_or(|url| url.is_empty()) {
                s.complete = true;
                s.image_content.take()
            } else {
                s.complete = false;
                None
            }
        };
        if let Some(old) = old {
            old.RemoveObserver(self);
        }
        Some(HTMLImageLoaderTask {
            loader: self.clone(),
            generation: self.state.borrow().generation,
            kind: ImageLoaderTaskKind::Update,
            update,
            force_blocking,
            synchronous: false,
        })
    }
    pub fn TaskIsCurrent(&self, task: &HTMLImageLoaderTask) -> bool {
        !self.disposed.get()
            && task.loader.Element() == self.element
            && self.state.borrow().generation == task.generation
    }
    /// DoUpdateFromElement cc:574-607: the new pending content becomes the
    /// loader's content immediately. Layout's previous cached image survives.
    pub fn DoUpdateFromElement(
        self: &Rc<Self>,
        task: &HTMLImageLoaderTask,
        content: Option<Rc<DOMImageResourceContent>>,
        active: bool,
    ) -> bool {
        if !self.TaskIsCurrent(task) {
            return false;
        }
        if !active {
            self.ClearImage();
            return true;
        }
        let _update_delay = self.state.borrow_mut().update_delay.take();
        let old = {
            let mut s = self.state.borrow_mut();
            s.pending_task = None;
            let changed = match (&s.image_content, &content) {
                (Some(a), Some(b)) => !Rc::ptr_eq(a, b),
                (None, None) => false,
                _ => true,
            };
            if changed {
                Self::RejectDecodes(&mut s, !task.synchronous);
            }
            s.pending_load = false;
            Self::CancelDelay(&mut s.load_delay);
            if content.is_some() {
                s.pending_error = false;
                Self::CancelDelay(&mut s.error_delay);
            }
            let old = changed.then(|| s.image_content.clone()).flatten();
            s.failed_load_url = None;
            s.image_content = content.clone();
            s.complete = content.is_none();
            // Keep the previous observer until the new one has been attached.
            old
        };
        if let Some(content) = content {
            content.AddObserver(self);
        }
        if let Some(old) = old {
            old.RemoveObserver(self);
        }
        true
    }
    pub fn LayoutUpdate(&self) -> image_resource::ImageResourceContentUpdate {
        let state = self.state.borrow();
        image_resource::ImageResourceContentUpdate {
            content: state.image_content.as_ref().map(|content| {
                content.clone() as Rc<dyn image_resource::ImageResourceContentLease>
            }),
            complete: state.complete,
        }
    }
    pub fn SetLazyDeferred(&self) {
        let mut s = self.state.borrow_mut();
        if !s.lazy_full_image {
            s.lazy_deferred = true;
        }
    }
    /// ImageLoader.cc:1028-1040: commit FullImage before posting the original
    /// Agent's update task. The task retains this loader and its generation.
    pub fn ResumeDeferredImage(
        self: &Rc<Self>,
        active: bool,
        force_blocking: bool,
        from_microtask: bool,
    ) -> Option<HTMLImageLoaderTask> {
        if !self.IsLazyDeferred() {
            return None;
        }
        self.state.borrow_mut().lazy_deferred = false;
        self.state.borrow_mut().lazy_full_image = true;
        let mut task = self.UpdateFromElement(
            self.CurrentSource(),
            active,
            if from_microtask {
                ImageLoaderUpdate::FromMicrotask
            } else {
                ImageLoaderUpdate::Normal
            },
            !from_microtask,
            force_blocking,
        )?;
        task.kind = ImageLoaderTaskKind::ResumeDeferred;
        Some(task)
    }
    pub fn IsLazyDeferred(&self) -> bool {
        self.state.borrow().lazy_deferred
    }
    pub fn LoadDeferredImage(self: &Rc<Self>) {
        let content = {
            let mut s = self.state.borrow_mut();
            s.lazy_deferred = false;
            s.lazy_full_image = true;
            s.image_content.clone()
        };
        if let Some(content) = content.filter(|content| content.IsLoaded()) {
            self.ImageNotifyFinished(&content);
        }
    }
    pub fn ImageNotifyFinished(self: &Rc<Self>, content: &Rc<DOMImageResourceContent>) {
        let mut s = self.state.borrow_mut();
        if self.disposed.get()
            || !s
                .image_content
                .as_ref()
                .is_some_and(|current| Rc::ptr_eq(current, content))
            || s.complete
        {
            return;
        }
        if s.lazy_deferred {
            return;
        }
        s.complete = true;
        if content.ErrorOccurred() {
            s.pending_load = false;
            Self::CancelDelay(&mut s.load_delay);
            if content.AccessCheckFailed() {
                s.failed_load_url = s.source.clone();
            }
            if !s.suppress_errors {
                s.pending_error = true;
                s.error_task_queued = false;
                Self::CancelDelay(&mut s.error_delay);
                s.error_delay = Some(Rc::new(RefCell::new(Some(
                    self.load_counter.borrow().Acquire(),
                ))));
            }
        } else {
            s.pending_load = true;
            s.load_task_queued = false;
            s.load_delay = Some(Rc::new(RefCell::new(Some(
                self.load_counter.borrow().Acquire(),
            ))));
        }
        Self::DispatchDecodesIfComplete(&mut s, true);
    }
    pub fn QueueError(&self) {
        let mut s = self.state.borrow_mut();
        if !s.suppress_errors {
            s.pending_error = true;
            s.error_task_queued = false;
            Self::CancelDelay(&mut s.error_delay);
            s.error_delay = Some(Rc::new(RefCell::new(Some(
                self.load_counter.borrow().Acquire(),
            ))));
        }
    }
    pub fn EventTicket(self: &Rc<Self>) -> Option<HTMLImageLoaderEventTicket> {
        let s = self.state.borrow();
        let event = if s.pending_error {
            ImageLoaderEvent::Error
        } else if s.pending_load {
            ImageLoaderEvent::Load
        } else {
            return None;
        };
        Some(HTMLImageLoaderEventTicket {
            loader: self.clone(),
            generation: s.generation,
            event,
            delay: match event {
                ImageLoaderEvent::Load => s.load_delay.as_ref(),
                ImageLoaderEvent::Error => s.error_delay.as_ref(),
            }
            .expect("source pending event delay")
            .clone(),
        })
    }
    pub fn TakeEventTicket(self: &Rc<Self>) -> Option<HTMLImageLoaderEventTicket> {
        let ticket = self.EventTicket()?;
        let mut state = self.state.borrow_mut();
        let queued = match ticket.event {
            ImageLoaderEvent::Load => &mut state.load_task_queued,
            ImageLoaderEvent::Error => &mut state.error_task_queued,
        };
        if std::mem::replace(queued, true) {
            None
        } else {
            Some(ticket)
        }
    }
    pub fn ConsumeEvent(&self, ticket: &HTMLImageLoaderEventTicket) -> bool {
        if self.disposed.get() {
            return false;
        }
        let mut s = self.state.borrow_mut();
        if s.generation != ticket.generation {
            return false;
        }
        let delay = match ticket.event {
            ImageLoaderEvent::Load => s.load_delay.as_ref(),
            ImageLoaderEvent::Error => s.error_delay.as_ref(),
        };
        if !delay.is_some_and(|delay| Rc::ptr_eq(delay, &ticket.delay)) {
            return false;
        }

        let pending = match ticket.event {
            ImageLoaderEvent::Load => &mut s.pending_load,
            ImageLoaderEvent::Error => &mut s.pending_error,
        };
        std::mem::replace(pending, false)
    }
    fn CancelDelay(delay: &mut Option<Rc<RefCell<Option<LoadEventDelayLease>>>>) {
        if let Some(delay) = delay.take() {
            delay.borrow_mut().take();
        }
    }
    pub fn BeginEventDispatch(
        &self,
        ticket: &HTMLImageLoaderEventTicket,
    ) -> Option<LoadEventDelayLease> {
        if !self.ConsumeEvent(ticket) {
            return None;
        }
        ticket.delay.borrow_mut().take()
    }
    pub fn ElementDidMoveToNewDocument(self: &Rc<Self>, counter: Rc<LoadEventDelayCounter>) {
        self.state.borrow_mut().failed_load_url = None;
        self.ClearImage();
        *self.load_counter.borrow_mut() = counter;
    }
    pub fn ClearImage(self: &Rc<Self>) {
        let old = {
            let mut s = self.state.borrow_mut();
            s.pending_load = false;
            s.pending_error = false;
            s.pending_task = None;
            s.update_delay = None;
            Self::CancelDelay(&mut s.load_delay);
            Self::CancelDelay(&mut s.error_delay);
            s.complete = true;
            s.lazy_deferred = false;
            s.lazy_full_image = false;
            Self::RejectDecodes(&mut s, false);
            let old = s.image_content.take();
            old
        };
        if let Some(old) = old {
            old.RemoveObserver(self);
        }
    }
    pub fn Dispose(self: &Rc<Self>) {
        self.disposed.set(true);
        self.ClearImage();
    }
    fn RejectDecodes(s: &mut LoaderState, asynchronous: bool) {
        let mut retained = vec![];
        for request in s.decodes.drain(..) {
            if asynchronous && request.state == ImageDecodeState::PendingMicrotask {
                retained.push(request);
            } else {
                s.decode_results
                    .push((request.id, ImageDecodeResult::EncodingError));
            }
        }
        s.decodes = retained;
    }
    fn DispatchDecodesIfComplete(s: &mut LoaderState, active: bool) {
        if !s.complete {
            return;
        }
        if !active
            || !s
                .image_content
                .as_ref()
                .is_some_and(|content| content.HasImage() && !content.ErrorOccurred())
        {
            Self::RejectDecodes(s, false);
            return;
        }
        let svg = s
            .image_content
            .as_ref()
            .is_some_and(|content| content.IsSVGImage());
        if svg {
            let mut retained = Vec::new();
            for request in s.decodes.drain(..) {
                if request.state == ImageDecodeState::PendingLoad {
                    s.decode_results
                        .push((request.id, ImageDecodeResult::Resolved));
                } else {
                    retained.push(request);
                }
            }
            s.decodes = retained;
        } else {
            for request in &mut s.decodes {
                if request.state == ImageDecodeState::PendingLoad {
                    request.state = ImageDecodeState::Dispatched;
                }
            }
        }
    }
    pub fn Decode(self: &Rc<Self>, request_id: u64) -> HTMLImageLoaderTask {
        let mut s = self.state.borrow_mut();
        s.decodes.push(DecodeRequest {
            id: request_id,
            state: ImageDecodeState::PendingMicrotask,
        });
        HTMLImageLoaderTask {
            loader: self.clone(),
            generation: s.generation,
            kind: ImageLoaderTaskKind::Decode(request_id),
            update: ImageLoaderUpdate::Normal,
            force_blocking: false,
            synchronous: false,
        }
    }
    pub fn ProcessDecodeTask(&self, request_id: u64, active: bool) {
        let mut s = self.state.borrow_mut();
        if let Some(request) = s
            .decodes
            .iter_mut()
            .find(|request| request.id == request_id)
        {
            request.state = ImageDecodeState::PendingLoad;
        }
        Self::DispatchDecodesIfComplete(&mut s, active);
    }
    pub fn DispatchedDecodeRequests(&self) -> Vec<u64> {
        self.state
            .borrow()
            .decodes
            .iter()
            .filter(|r| r.state == ImageDecodeState::Dispatched)
            .map(|r| r.id)
            .collect()
    }
    pub fn DecodeRequestFinished(&self, id: u64, success: bool) {
        let mut s = self.state.borrow_mut();
        if let Some(index) = s.decodes.iter().position(|r| r.id == id) {
            s.decodes.remove(index);
            s.decode_results.push((
                id,
                if success {
                    ImageDecodeResult::Resolved
                } else {
                    ImageDecodeResult::EncodingError
                },
            ));
        }
    }
    pub fn TakeDecodeResults(&self) -> Vec<(u64, ImageDecodeResult)> {
        std::mem::take(&mut self.state.borrow_mut().decode_results)
    }
}
impl PersistentDocument {
    /// HTMLImageLoader's ShouldSkipEventDispatch helper, cc:37-43. The
    /// original virtual class decides; Page only routes the resulting fact.
    pub fn HTMLImageLoaderShouldSkipEventDispatch(&self, node: usize) -> bool {
        self.IsVideoElement(node)
    }
    pub fn EnsureHTMLImageLoader(&self, node: usize) -> Rc<HTMLImageLoader> {
        let mut slot = self.arena[node].image_loader.borrow_mut();
        slot.get_or_insert_with(|| {
            let loader = HTMLImageLoader::new(
                DOMOwnerHandle {
                    arena_id: self.arena_id,
                    node_id: self.arena[node].id,
                },
                self.LoadEventDelayCounterForDocument(
                    self.arena[node]
                        .OwnerDocumentNode()
                        .expect("image owner Document"),
                ),
            );
            let observer: Rc<
                dyn crate::native_execution_context_lifetime::NativeExecutionContextObserver,
            > = loader.clone();
            self.NativeExecutionContextObservers().Register(&observer);
            loader
        })
        .clone()
    }
    pub fn ExistingHTMLImageLoader(&self, node: usize) -> Option<Rc<HTMLImageLoader>> {
        self.arena[node].image_loader.borrow().clone()
    }
}

impl crate::native_execution_context_lifetime::NativeExecutionContextObserver for HTMLImageLoader {
    fn ContextDestroyed(&self) {
        if let Some(loader) = self.self_.upgrade() {
            loader.Dispose();
        }
    }
}
