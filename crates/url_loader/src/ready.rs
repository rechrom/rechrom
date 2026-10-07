//! A completed URL load used by data: and memory: sources.
#![allow(non_snake_case)]
use crate::{URLLoadOperation, URLResponse};
use std::io;

// cpp: resource_loader/default_resource_loader.cc:226-234
pub(crate) struct ReadyURLLoad {
    pub(crate) response: Option<URLResponse>,
}

impl URLLoadOperation for ReadyURLLoad {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.response.take())
    }
}
