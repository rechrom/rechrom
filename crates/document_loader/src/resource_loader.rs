#![allow(non_snake_case)]
//! One resource request and its terminal result. The URL backend supplies bytes;
//! the resource owner decides how to decode, apply and schedule those bytes.
use std::io;
use std::thread;
use std::time::Duration;
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLoadStatus {
    kLoading,
    kComplete,
    kFailed,
    kCancelled,
}

// cpp: browser/browser.cc:35-39
#[derive(Default)]
pub struct ResourceLoadResult {
    pub response: Option<URLResponse>,
    pub error: String,
}

// cpp: browser/browser.cc:40-52
pub struct ResourceLoader {
    operation: Option<Box<dyn URLLoadOperation>>,
    result: ResourceLoadResult,
    status: ResourceLoadStatus,
}
impl ResourceLoader {
    /// Start one request without waiting for its response.
    pub fn Start(loader: &mut dyn URLLoader, request: &URLRequest) -> Self {
        match loader.Load(request) {
            Ok(operation) => Self {
                operation: Some(operation),
                result: ResourceLoadResult::default(),
                status: ResourceLoadStatus::kLoading,
            },
            Err(error) => Self {
                operation: None,
                result: ResourceLoadResult {
                    response: None,
                    error: error.to_string(),
                },
                status: ResourceLoadStatus::kFailed,
            },
        }
    }
    pub fn Status(&self) -> ResourceLoadStatus {
        self.status
    }
    /// Consume the result once. Terminal state persists after consumption.
    pub fn TakeResult(&mut self) -> ResourceLoadResult {
        std::mem::take(&mut self.result)
    }
    /// Poll once; true means terminal, including failure or cancellation.
    pub fn Poll(&mut self) -> bool {
        if self.status != ResourceLoadStatus::kLoading {
            return true;
        }
        match self
            .operation
            .as_mut()
            .expect("loading operation exists")
            .Poll()
        {
            Ok(None) => return false,
            Ok(Some(response)) => {
                self.result.response = Some(response);
                self.status = ResourceLoadStatus::kComplete;
            }
            Err(error) => {
                self.result.error = error.to_string();
                self.status = ResourceLoadStatus::kFailed;
            }
        }
        self.operation.take();
        true
    }
    /// Releasing the operation invokes the URL backend's cancellation.
    pub fn Cancel(&mut self) {
        if self.status != ResourceLoadStatus::kLoading {
            return;
        }
        self.operation.take();
        self.status = ResourceLoadStatus::kCancelled;
        self.result.error = "resource load cancelled".into();
    }
}

// cpp: browser/browser.cc:53-60
pub fn StartResource(loader: &mut dyn URLLoader, request: &URLRequest) -> ResourceLoader {
    ResourceLoader::Start(loader, request)
}

// cpp: browser/browser.cc:61-64
/// Compatibility wait for synchronous callers; ordinary Poll never waits.
pub fn AwaitResource(pending: &mut ResourceLoader) -> ResourceLoadResult {
    while !pending.Poll() {
        thread::sleep(Duration::from_millis(1));
    }
    pending.TakeResult()
}

// cpp: browser/browser.cc:65-69
pub fn RequireResponse(result: ResourceLoadResult) -> io::Result<URLResponse> {
    result
        .response
        .ok_or_else(|| io::Error::other(result.error))
}

pub fn LoadResponse(loader: &mut dyn URLLoader, request: &URLRequest) -> io::Result<URLResponse> {
    RequireResponse(AwaitResource(&mut StartResource(loader, request)))
}
