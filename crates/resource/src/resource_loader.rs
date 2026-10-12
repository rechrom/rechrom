#![allow(non_snake_case)]

//! One resource request and its terminal result. The URL backend supplies bytes;
//! the resource owner decides how to decode, apply and schedule those bytes.

use std::thread;
use std::time::Duration;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLoadStatus {
    kLoading,
    kComplete,
    kFailed,
    kCancelled,
}

#[derive(Clone, Default)]
pub struct ResourceLoadResult {
    pub response: Option<URLResponse>,
    pub error: String,
}

pub(crate) struct SharedResourceLoad {
    operation: Option<Box<dyn URLLoadOperation>>,
    result: ResourceLoadResult,
    status: ResourceLoadStatus,
}

pub struct ResourceLoader {
    shared: Rc<RefCell<SharedResourceLoad>>,
    cancelled: bool,
    took_result: bool,
}

impl ResourceLoader {
    pub fn Start(loader: &mut dyn URLLoader, request: &URLRequest) -> Self {
        let shared = match loader.Load(request) {
            Ok(operation) => SharedResourceLoad {
                operation: Some(operation),
                result: ResourceLoadResult::default(),
                status: ResourceLoadStatus::kLoading,
            },
            Err(error) => SharedResourceLoad {
                operation: None,
                result: ResourceLoadResult {
                    response: None,
                    error: error.to_string(),
                },
                status: ResourceLoadStatus::kFailed,
            },
        };
        Self::FromShared(Rc::new(RefCell::new(shared)))
    }

    pub fn Failed(error: impl ToString) -> Self {
        Self::FromShared(Rc::new(RefCell::new(SharedResourceLoad {
            operation: None,
            result: ResourceLoadResult {
                response: None,
                error: error.to_string(),
            },
            status: ResourceLoadStatus::kFailed,
        })))
    }

    pub(crate) fn FromShared(shared: Rc<RefCell<SharedResourceLoad>>) -> Self {
        Self {
            shared,
            cancelled: false,
            took_result: false,
        }
    }

    pub(crate) fn Shared(&self) -> Rc<RefCell<SharedResourceLoad>> {
        self.shared.clone()
    }

    pub(crate) fn CancelShared(shared: &Rc<RefCell<SharedResourceLoad>>) {
        let mut shared = shared.borrow_mut();
        if shared.status != ResourceLoadStatus::kLoading {
            return;
        }
        shared.operation.take();
        shared.status = ResourceLoadStatus::kCancelled;
        shared.result.error = "resource load cancelled".into();
    }

    pub fn Status(&self) -> ResourceLoadStatus {
        if self.cancelled {
            ResourceLoadStatus::kCancelled
        } else {
            self.shared.borrow().status
        }
    }

    pub fn TakeResult(&mut self) -> ResourceLoadResult {
        if self.took_result {
            return ResourceLoadResult::default();
        }
        self.took_result = true;
        if self.cancelled {
            return ResourceLoadResult {
                response: None,
                error: "resource load cancelled".into(),
            };
        }
        self.shared.borrow().result.clone()
    }

    pub fn Poll(&mut self) -> bool {
        if self.cancelled {
            return true;
        }
        let mut shared = self.shared.borrow_mut();
        if shared.status != ResourceLoadStatus::kLoading {
            return true;
        }
        match shared
            .operation
            .as_mut()
            .expect("loading operation exists")
            .Poll()
        {
            Ok(None) => return false,
            Ok(Some(response)) => {
                shared.result.response = Some(response);
                shared.status = ResourceLoadStatus::kComplete;
            }
            Err(error) => {
                shared.result.error = error.to_string();
                shared.status = ResourceLoadStatus::kFailed;
            }
        }
        shared.operation.take();
        true
    }

    pub fn SetWakeCallback(&mut self, callback: url_loader::URLWakeCallback) {
        if self.cancelled {
            return;
        }
        let mut shared = self.shared.borrow_mut();
        if let Some(operation) = shared.operation.as_mut() {
            operation.SetWakeCallback(callback);
        } else if shared.status != ResourceLoadStatus::kLoading {
            callback();
        }
    }

    pub fn Cancel(&mut self) {
        self.cancelled = true;
    }
}

pub fn StartResource(loader: &mut dyn URLLoader, request: &URLRequest) -> ResourceLoader {
    ResourceLoader::Start(loader, request)
}

pub fn AwaitResource(pending: &mut ResourceLoader) -> ResourceLoadResult {
    while !pending.Poll() {
        thread::sleep(Duration::from_millis(1));
    }
    pending.TakeResult()
}

pub fn RequireResponse(result: ResourceLoadResult) -> io::Result<URLResponse> {
    result
        .response
        .ok_or_else(|| io::Error::other(result.error))
}

pub fn LoadResponse(loader: &mut dyn URLLoader, request: &URLRequest) -> io::Result<URLResponse> {
    RequireResponse(AwaitResource(&mut StartResource(loader, request)))
}
