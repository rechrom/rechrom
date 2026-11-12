//! FontResource's retained network/decode owner and finish-observer lifetime.
//! Chromium platform/loader/fetch/resource.{h,cc}, font_resource.{h,cc};
//! PendingLinkPreload.cc:18-76. CSSFontFace and preload use this SAME resource.
use crate::ResourceLoader;
use resource::ResourceEngine;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    rc::{Rc, Weak},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
static NEXT_OBSERVER: AtomicU64 = AtomicU64::new(1);
pub trait FontResourceFinishObserver {
    fn NotifyFinished(&self, resource: &Rc<FontResource>);
}
pub struct FontResource {
    identity: u64,
    context: Weak<dom::font_loading::FontLoadingDocument>,
    engine: Rc<ResourceEngine>,
    loader: RefCell<Option<ResourceLoader>>,
    network_finished: Cell<bool>,
    network_error: RefCell<Option<String>>,
    encoded: RefCell<Option<Vec<u8>>>,
    decode_requested: Cell<bool>,
    decode: RefCell<Option<decode::DecodeJob<Vec<u8>>>>,
    decoded: RefCell<Option<Result<font_engine::SharedFontBytes, String>>>,
    observers: RefCell<HashMap<u64, Weak<dyn FontResourceFinishObserver>>>,
}
impl FontResource {
    pub fn new(
        engine: Rc<ResourceEngine>,
        loader: ResourceLoader,
        context: &Rc<dom::font_loading::FontLoadingDocument>,
    ) -> Rc<Self> {
        Rc::new(Self {
            identity: loader.Identity(),
            context: Rc::downgrade(context),
            engine,
            loader: RefCell::new(Some(loader)),
            network_finished: Cell::new(false),
            network_error: RefCell::new(None),
            encoded: RefCell::new(None),
            decode_requested: Cell::new(false),
            decode: RefCell::new(None),
            decoded: RefCell::new(None),
            observers: RefCell::new(HashMap::new()),
        })
    }
    pub fn Identity(&self) -> u64 {
        self.identity
    }
    pub fn NetworkFinished(&self) -> bool {
        self.network_finished.get()
    }
    pub fn NetworkFailed(&self) -> bool {
        self.network_error.borrow().is_some()
    }
    pub fn Decoded(&self) -> Option<Result<font_engine::SharedFontBytes, String>> {
        self.decoded.borrow().clone()
    }
    pub fn RequestDecode(&self) {
        self.decode_requested.set(true);
    }
    pub fn AddFinishObserver(
        self: &Rc<Self>,
        observer: &Rc<dyn FontResourceFinishObserver>,
    ) -> u64 {
        let token = NEXT_OBSERVER.fetch_add(1, Ordering::Relaxed);
        self.observers
            .borrow_mut()
            .insert(token, Rc::downgrade(observer));
        if self.NetworkFinished() {
            self.NotifyObserver(observer.clone());
        }
        token
    }
    pub fn RemoveFinishObserver(&self, token: u64) {
        self.observers.borrow_mut().remove(&token);
    }
    fn NotifyObserver(self: &Rc<Self>, observer: Rc<dyn FontResourceFinishObserver>) {
        if let Some(context) = self.context.upgrade() {
            let resource = self.clone();
            context.QueueTask(3, Box::new(move || observer.NotifyFinished(&resource)));
        }
    }
    pub fn Poll(self: &Rc<Self>) -> bool {
        let mut changed = false;
        if !self.network_finished.get() {
            let finished = self
                .loader
                .borrow_mut()
                .as_mut()
                .is_some_and(|loader| loader.Poll());
            if finished {
                let mut loader = self
                    .loader
                    .borrow_mut()
                    .take()
                    .expect("source font resource loader");
                let result = loader.TakeResult();
                if let Some(response) = result.response {
                    *self.encoded.borrow_mut() = Some(response.body);
                } else {
                    *self.network_error.borrow_mut() = Some(result.error);
                }
                self.network_finished.set(true);
                changed = true;
                // Source ResourceFinishObservers run after the resource has
                // committed its result, with no transport borrow held.
                let observers = self
                    .observers
                    .borrow()
                    .iter()
                    .filter_map(|(_, weak)| weak.upgrade())
                    .collect::<Vec<_>>();
                for observer in observers {
                    self.NotifyObserver(observer);
                }
            }
        }
        if self.decode_requested.get()
            && self.network_finished.get()
            && self.decoded.borrow().is_none()
        {
            if let Some(error) = self.network_error.borrow().clone() {
                *self.decoded.borrow_mut() = Some(Err(error));
                return true;
            }
            if self.decode.borrow().is_none() {
                if let Some(encoded) = self.encoded.borrow_mut().take() {
                    *self.decode.borrow_mut() =
                        Some(self.engine.GetDecodeEngine().BeginFontDecode(encoded));
                }
            }
            let result = self.decode.borrow_mut().as_mut().and_then(|job| job.poll());
            if let Some(result) = result {
                self.decode.borrow_mut().take();
                *self.decoded.borrow_mut() = Some(
                    result
                        .and_then(font_engine::AcceptDecodedWebFont)
                        .map(Into::into)
                        .map_err(|error| error.to_string()),
                );
                changed = true;
            }
        }
        changed
    }
}
/// One PendingLinkPreload, not the element id: href changes dispose the old
/// observer and unblock exactly its old registration, leaving a new token live.
pub struct PendingLinkPreload {
    identity: dom::render_blocking_resource_manager::FontPreloadOwner,
    manager:
        Option<Rc<RefCell<dom::render_blocking_resource_manager::RenderBlockingResourceManager>>>,
    resource: RefCell<Option<Rc<FontResource>>>,
    observer_token: Cell<Option<u64>>,
    matches_media: Cell<bool>,
    finished: Cell<Option<bool>>,
    completion: Rc<dyn Fn(bool)>,
    integrity: RefCell<String>,
}
static NEXT_PRELOAD: AtomicU64 = AtomicU64::new(1);
impl PendingLinkPreload {
    pub fn new(
        manager: Option<
            Rc<RefCell<dom::render_blocking_resource_manager::RenderBlockingResourceManager>>,
        >,
        completion: Rc<dyn Fn(bool)>,
    ) -> Rc<Self> {
        Rc::new(Self {
            identity: dom::render_blocking_resource_manager::FontPreloadOwner(
                NEXT_PRELOAD.fetch_add(1, Ordering::Relaxed),
            ),
            manager,
            resource: RefCell::new(None),
            observer_token: Cell::new(None),
            matches_media: Cell::new(false),
            finished: Cell::new(None),
            completion,
            integrity: RefCell::new(String::new()),
        })
    }
    pub fn Identity(&self) -> dom::render_blocking_resource_manager::FontPreloadOwner {
        self.identity
    }
    pub fn MatchesMedia(&self) -> bool {
        self.matches_media.get()
    }
    pub fn SetMatchesMedia(&self, value: bool) {
        self.matches_media.set(value);
    }
    pub fn HasResource(&self) -> bool {
        self.resource.borrow().is_some()
    }
    pub fn SetIntegrity(&self, integrity: String) {
        *self.integrity.borrow_mut() = integrity;
    }
    pub fn RegisterRenderBlocking(&self, has_body: bool) {
        if let Some(manager) = &self.manager {
            let mut manager = manager.borrow_mut();
            manager.AddPendingFontPreload(self.identity, has_body);
        }
    }
    pub fn AddResource(self: &Rc<Self>, resource: Rc<FontResource>) {
        assert!(self.resource.borrow().is_none());
        let observer: Rc<dyn FontResourceFinishObserver> = self.clone();
        let token = resource.AddFinishObserver(&observer);
        *self.resource.borrow_mut() = Some(resource.clone());
        self.observer_token.set(Some(token));
        // AddFinishObserver on an already completed resource schedules its
        // finish callback on kNetworking; PollFinished is invoked there.
    }
    fn UnblockRendering(&self) {
        if let Some(manager) = &self.manager {
            let mut manager = manager.borrow_mut();
            let body = manager.DocumentHasBody();
            manager.RemovePendingFontPreload(self.identity, body);
        }
    }
    pub fn Dispose(&self) {
        self.UnblockRendering();
        if let Some(resource) = self.resource.borrow_mut().take() {
            if let Some(token) = self.observer_token.take() {
                resource.RemoveFinishObserver(token);
            }
        }
    }
    pub fn Finished(&self) -> Option<bool> {
        self.finished.get()
    }
}
impl FontResourceFinishObserver for PendingLinkPreload {
    fn NotifyFinished(&self, resource: &Rc<FontResource>) {
        if self.resource.borrow().is_none() || self.finished.get().is_some() {
            return;
        }
        self.UnblockRendering();
        let succeeded = !resource.NetworkFailed();
        self.finished.set(Some(succeeded));
        (self.completion)(succeeded);
        if let Some(token) = self.observer_token.take() {
            resource.RemoveFinishObserver(token);
        }
    }
}
impl Drop for PendingLinkPreload {
    fn drop(&mut self) {
        self.Dispose();
    }
}
