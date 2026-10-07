//! Response headers, bounded body chunks, and a single terminal notification.
#![allow(non_snake_case)]
use crate::{URLLoadOperation, URLResponse};
use std::{io, sync::Arc};

pub type URLWakeCallback = Arc<dyn Fn() + Send + Sync>;
#[derive(Clone, Debug, Default)]
pub struct URLResponseHead {
    pub final_url: String,
    pub status_code: i64,
    pub mime_type: String,
    pub text_encoding: String,
}
#[derive(Debug)]
pub enum URLLoadEvent {
    Response(URLResponseHead),
    Data(Vec<u8>),
    Finished,
}
pub trait URLStreamOperation {
    /// Never waits. Data events contain at most max_bytes bytes. Dropping the
    /// operation cancels delivery, including a producer blocked by backpressure.
    fn Poll(&mut self, max_bytes: usize) -> io::Result<Option<URLLoadEvent>>;
    fn SetWakeCallback(&mut self, _callback: URLWakeCallback) {}
}
impl URLResponse {
    pub fn Head(&self) -> URLResponseHead {
        URLResponseHead {
            final_url: self.final_url.clone(),
            status_code: self.status_code,
            mime_type: self.mime_type.clone(),
            text_encoding: self.text_encoding.clone(),
        }
    }
}

/// Compatibility for application-defined complete-response loaders and
/// already resident data:/memory: input. HTTP and file: override this adapter.
pub(crate) struct BufferedURLStream {
    operation: Option<Box<dyn URLLoadOperation>>,
    response: Option<URLResponse>,
    offset: usize,
    finished: bool,
}
impl BufferedURLStream {
    pub(crate) fn new(operation: Box<dyn URLLoadOperation>) -> Self {
        Self {
            operation: Some(operation),
            response: None,
            offset: 0,
            finished: false,
        }
    }
}
impl URLStreamOperation for BufferedURLStream {
    fn Poll(&mut self, max_bytes: usize) -> io::Result<Option<URLLoadEvent>> {
        if max_bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "body budget must be positive",
            ));
        }
        if self.finished {
            return Ok(None);
        }
        if let Some(operation) = &mut self.operation {
            let Some(response) = operation.Poll()? else {
                return Ok(None);
            };
            let head = response.Head();
            self.response = Some(response);
            self.operation.take();
            return Ok(Some(URLLoadEvent::Response(head)));
        }
        let response = self.response.as_ref().expect("response precedes body");
        if self.offset < response.body.len() {
            let end = response
                .body
                .len()
                .min(self.offset.saturating_add(max_bytes));
            let data = response.body[self.offset..end].to_vec();
            self.offset = end;
            return Ok(Some(URLLoadEvent::Data(data)));
        }
        self.response.take();
        self.finished = true;
        Ok(Some(URLLoadEvent::Finished))
    }
}
