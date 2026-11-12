//! ResourceFetcher's nonblocking loader membership for the original Window
//! network operation. No copied XHR state and no second transport is created.
//! Chromium resource_fetcher.cc2812-2826,3172-3201; resource_loader.cc490-493.
use crate::{
    ResourceCancellationClient, XMLHttpRequestData, XMLHttpRequestOperation,
    XMLHttpRequestTransport, XMLHttpResponseData,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    rc::{Rc, Weak},
    sync::Arc,
};
use url_loader::{RequestDestination, URLRequest};
pub struct XMLHttpRequestRegistry {
    in_request_resource: Rc<Cell<bool>>,
    operations: RefCell<HashMap<u64, Rc<RegisteredXMLHttpRequestOperation>>>,
}
pub struct RegisteredXMLHttpRequestOperation {
    identity: u64,
    request: Arc<URLRequest>,
    operation: RefCell<Option<Box<dyn XMLHttpRequestOperation>>>,
    registry: Weak<XMLHttpRequestRegistry>,
    cancellation_clients: RefCell<Vec<Weak<dyn ResourceCancellationClient>>>,
}
impl XMLHttpRequestRegistry {
    pub(crate) fn new(in_request_resource: Rc<Cell<bool>>) -> Self {
        Self {
            in_request_resource,
            operations: Default::default(),
        }
    }
    pub fn Start(
        self: &Rc<Self>,
        identity: u64,
        request: XMLHttpRequestData,
        transport: &mut dyn XMLHttpRequestTransport,
    ) -> io::Result<Rc<RegisteredXMLHttpRequestOperation>> {
        assert_ne!(identity, 0, "original Window network request identity");
        assert!(
            !self.operations.borrow().contains_key(&identity),
            "same original operation registers once"
        );
        let _scope =
            crate::fetcher_lifecycle::RequestResourceScope::new(self.in_request_resource.clone());
        let operation = transport.Start(&request)?;
        let operation = Rc::new(RegisteredXMLHttpRequestOperation {
            identity,
            request: Arc::new(URLRequest {
                method: request.method,
                url: request.url,
                referrer: request.referrer,
                body: request.body,
                headers: request.headers,
                destination: RequestDestination::kFetch,
                ..Default::default()
            }),
            operation: RefCell::new(Some(operation)),
            registry: Rc::downgrade(self),
            cancellation_clients: RefCell::new(Vec::new()),
        });
        self.operations
            .borrow_mut()
            .insert(identity, operation.clone());
        Ok(operation)
    }
    pub(crate) fn Snapshot(&self) -> Vec<Rc<RegisteredXMLHttpRequestOperation>> {
        self.operations.borrow().values().cloned().collect()
    }
    pub(crate) fn Contains(&self, operation: &Rc<RegisteredXMLHttpRequestOperation>) -> bool {
        self.operations
            .borrow()
            .get(&operation.identity)
            .is_some_and(|actual| Rc::ptr_eq(actual, operation))
    }
    pub fn IsEmpty(&self) -> bool {
        self.operations.borrow().is_empty()
    }
}
impl RegisteredXMLHttpRequestOperation {
    pub fn Identity(&self) -> u64 {
        self.identity
    }
    pub fn Request(&self) -> &Arc<URLRequest> {
        &self.request
    }
    pub fn Poll(&self) -> io::Result<Option<XMLHttpResponseData>> {
        let mut operation = self.operation.borrow_mut();
        let Some(operation) = operation.as_mut() else {
            return Ok(None);
        };
        operation.Poll()
    }
    pub fn IsCancelled(&self) -> bool {
        self.operation.borrow().is_none()
    }
    pub fn RegisterCancellationClient(&self, client: &Rc<dyn ResourceCancellationClient>) {
        let weak = Rc::downgrade(client);
        let mut clients = self.cancellation_clients.borrow_mut();
        clients.retain(|client| client.strong_count() != 0);
        if !clients.iter().any(|client| Weak::ptr_eq(client, &weak)) {
            clients.push(weak);
        }
    }
    pub fn UnregisterCancellationClient(&self, client: &Rc<dyn ResourceCancellationClient>) {
        let weak = Rc::downgrade(client);
        self.cancellation_clients
            .borrow_mut()
            .retain(|client| !Weak::ptr_eq(client, &weak));
    }
    fn RemoveMembership(&self) {
        if let Some(registry) = self.registry.upgrade() {
            registry.operations.borrow_mut().remove(&self.identity);
        }
    }
    /// Native ThreadableLoader completion precedes XHR's DONE/load callbacks.
    /// Header/data callbacks can still cancel this original registered loader.
    pub fn Finish(&self) {
        self.RemoveMembership();
        self.operation.borrow_mut().take();
    }
    pub fn Cancel(&self) {
        let operation = self.operation.borrow_mut().take();
        if operation.is_none() {
            return;
        }
        self.RemoveMembership();
        drop(operation);
        let clients = self.cancellation_clients.borrow().clone();
        for weak in clients {
            if !self
                .cancellation_clients
                .borrow()
                .iter()
                .any(|client| Weak::ptr_eq(client, &weak))
            {
                continue;
            }
            if let Some(client) = weak.upgrade() {
                client.ResourceCancelled(self.identity, &self.request);
            }
        }
    }
}
