//! Cancellable background file: URL byte loading.
#![allow(non_snake_case)]
use crate::metadata::{invalid, MimeTypeFor, PercentDecodeData};
use crate::{URLLoadOperation, URLRequest, URLResponse};
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

// cpp: resource_loader/default_resource_loader.cc:47-73
pub(crate) fn FilePathFromURL(url: &str) -> io::Result<PathBuf> {
    let mut part = url
        .strip_prefix("file://")
        .ok_or_else(|| invalid(format!("not a file URL: {url}")))?;
    if part.starts_with("localhost/") {
        part = &part[9..];
    }
    if !part.starts_with('/') {
        return Err(invalid("file URL authority is unsupported"));
    }
    part = part.split(['?', '#']).next().unwrap_or(part);
    let decoded = PercentDecodeData(part.as_bytes());
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(PathBuf::from(std::ffi::OsString::from_vec(decoded)))
    }
    #[cfg(not(unix))]
    {
        Ok(PathBuf::from(
            String::from_utf8(decoded).map_err(|_| invalid("invalid file URL path"))?,
        ))
    }
}

// cpp: resource_loader/default_resource_loader.cc:259-291
struct FileState {
    cancelled: AtomicBool,
    response: Mutex<Option<io::Result<URLResponse>>>,
}

pub(crate) struct FileURLLoad {
    state: Arc<FileState>,
}

impl FileURLLoad {
    pub(crate) fn new(request: URLRequest, limit: usize) -> io::Result<Self> {
        let state = Arc::new(FileState {
            cancelled: AtomicBool::new(false),
            response: Mutex::new(None),
        });
        let worker_state = state.clone();
        thread::Builder::new()
            .name("layoutng-file-url".to_owned())
            .spawn(move || {
                let result = (|| -> io::Result<URLResponse> {
                    let path = FilePathFromURL(&request.url)?;
                    let mut input = std::fs::File::open(&path).map_err(|_| {
                        io::Error::other(format!("could not read file URL: {}", path.display()))
                    })?;
                    let mut response = URLResponse {
                        final_url: request.url.clone(),
                        status_code: 200,
                        mime_type: MimeTypeFor(&request).to_owned(),
                        ..URLResponse::default()
                    };
                    let mut chunk = [0_u8; 16_384];
                    while !worker_state.cancelled.load(Ordering::Relaxed) {
                        let size = input.read(&mut chunk)?;
                        if size == 0 {
                            break;
                        }
                        if size > limit.saturating_sub(response.body.len()) {
                            return Err(io::Error::other(format!(
                                "URL response exceeds configured size limit: {}",
                                request.url
                            )));
                        }
                        response.body.extend_from_slice(&chunk[..size]);
                    }
                    Ok(response)
                })();
                *worker_state
                    .response
                    .lock()
                    .expect("file URL load state poisoned") = Some(result);
            })?;
        Ok(Self { state })
    }
}

impl Drop for FileURLLoad {
    fn drop(&mut self) {
        self.state.cancelled.store(true, Ordering::Relaxed);
    }
}

impl URLLoadOperation for FileURLLoad {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        let mut response = self
            .state
            .response
            .lock()
            .expect("file URL load state poisoned");
        response.take().transpose()
    }
}
