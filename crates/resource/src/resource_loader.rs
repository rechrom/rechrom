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

#[derive(Default)]
pub struct ResourceLoadResult {
    pub response: Option<URLResponse>,
    pub error: String,
}

pub struct ResourceLoader {
    operation: Option<Box<dyn URLLoadOperation>>,
    result: ResourceLoadResult,
    status: ResourceLoadStatus,
}

impl ResourceLoader {
    pub fn Start(loader: &mut dyn URLLoader, request: &URLRequest) -> Self {
        match loader.Load(request) {
            Ok(operation) => Self {
                operation: Some(operation),
                result: ResourceLoadResult::default(),
                status: ResourceLoadStatus::kLoading,
            },
            Err(error) => Self::Failed(error),
        }
    }

    pub fn Failed(error: impl ToString) -> Self {
        Self {
            operation: None,
            result: ResourceLoadResult {
                response: None,
                error: error.to_string(),
            },
            status: ResourceLoadStatus::kFailed,
        }
    }

    pub fn Status(&self) -> ResourceLoadStatus {
        self.status
    }

    pub fn TakeResult(&mut self) -> ResourceLoadResult {
        std::mem::take(&mut self.result)
    }

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

    pub fn Cancel(&mut self) {
        if self.status != ResourceLoadStatus::kLoading {
            return;
        }
        self.operation.take();
        self.status = ResourceLoadStatus::kCancelled;
        self.result.error = "resource load cancelled".into();
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
