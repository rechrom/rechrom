//! Registered memory: URL byte sources. This registry is not a fetched-resource cache.
#![allow(non_snake_case)]
use crate::metadata::invalid;
use crate::ready::ReadyURLLoad;
use crate::{URLLoadOperation, URLResponse};
use std::collections::HashMap;
use std::io;

#[derive(Default)]
pub(crate) struct MemoryURLs {
    responses: HashMap<String, URLResponse>,
}
impl MemoryURLs {
    // cpp: resource_loader/default_resource_loader.cc:316-332
    pub(crate) fn RegisterMemoryURL(
        &mut self,
        url: String,
        mime_type: String,
        body: Vec<u8>,
        text_encoding: String,
    ) -> io::Result<()> {
        if !url.starts_with("memory:") {
            return Err(invalid("memory URL must use memory:"));
        }
        let response = URLResponse {
            final_url: url.clone(),
            status_code: 200,
            mime_type,
            text_encoding,
            body,
        };
        match self.responses.entry(url) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(response);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(invalid("duplicate memory URL")),
        }
    }
    pub(crate) fn Load(&self, url: &str) -> io::Result<Box<dyn URLLoadOperation>> {
        let response = self.responses.get(url).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("memory URL not found: {url}"),
            )
        })?;
        Ok(Box::new(ReadyURLLoad {
            response: Some(response.clone()),
        }))
    }
}
