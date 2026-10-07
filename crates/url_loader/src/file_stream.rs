#![allow(non_snake_case)]
use crate::{
    file::FilePathFromURL, metadata::MimeTypeFor, URLLoadEvent, URLRequest, URLResponseHead,
    URLStreamOperation, URLWakeCallback,
};
use std::{
    io::{self, Read},
    sync::{mpsc, Arc, Mutex},
    thread,
};
pub(crate) struct FileURLStream {
    receiver: mpsc::Receiver<io::Result<URLLoadEvent>>,
    pending: Vec<u8>,
    offset: usize,
    terminal: bool,
    wake: Arc<Mutex<Option<URLWakeCallback>>>,
}
impl FileURLStream {
    pub(crate) fn new(request: URLRequest, limit: usize) -> io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(8);
        let wake: Arc<Mutex<Option<URLWakeCallback>>> = Arc::new(Mutex::new(None));
        let worker_wake = wake.clone();
        thread::Builder::new()
            .name("layoutng-file-stream".into())
            .spawn(move || {
                let send = |event| {
                    sender
                        .send(event)
                        .map_err(|_| io::Error::other("file load cancelled"))?;
                    let callback = worker_wake.lock().expect("file wake poisoned").clone();
                    if let Some(callback) = callback {
                        callback();
                    }
                    Ok::<_, io::Error>(())
                };
                let result = (|| {
                    let mut file = std::fs::File::open(FilePathFromURL(&request.url)?)?;
                    send(Ok(URLLoadEvent::Response(URLResponseHead {
                        final_url: request.url.clone(),
                        status_code: 200,
                        mime_type: MimeTypeFor(&request).into(),
                        text_encoding: "utf-8".into(),
                    })))?;
                    let mut total = 0usize;
                    let mut chunk = [0u8; 16 * 1024];
                    loop {
                        let size = file.read(&mut chunk)?;
                        if size == 0 {
                            break;
                        }
                        total = total
                            .checked_add(size)
                            .ok_or_else(|| io::Error::other("response size overflow"))?;
                        if total > limit {
                            return Err(io::Error::other(
                                "URL response exceeds configured size limit",
                            ));
                        }
                        send(Ok(URLLoadEvent::Data(chunk[..size].to_vec())))?;
                    }
                    send(Ok(URLLoadEvent::Finished))
                })();
                if let Err(error) = result {
                    let _ = send(Err(error));
                }
            })?;
        Ok(Self {
            receiver,
            pending: vec![],
            offset: 0,
            terminal: false,
            wake,
        })
    }
}
impl URLStreamOperation for FileURLStream {
    fn Poll(&mut self, max_bytes: usize) -> io::Result<Option<URLLoadEvent>> {
        if max_bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "body budget must be positive",
            ));
        }
        if self.offset < self.pending.len() {
            let end = self
                .pending
                .len()
                .min(self.offset.saturating_add(max_bytes));
            let data = self.pending[self.offset..end].to_vec();
            self.offset = end;
            if end == self.pending.len() {
                self.pending.clear();
                self.offset = 0;
            }
            return Ok(Some(URLLoadEvent::Data(data)));
        }
        if self.terminal {
            return Ok(None);
        }
        match self.receiver.try_recv() {
            Ok(Ok(URLLoadEvent::Data(data))) if data.len() > max_bytes => {
                self.pending = data;
                self.Poll(max_bytes)
            }
            Ok(Ok(event)) => {
                self.terminal = matches!(event, URLLoadEvent::Finished);
                Ok(Some(event))
            }
            Ok(Err(error)) => {
                self.terminal = true;
                Err(error)
            }
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                self.terminal = true;
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "file stream ended without completion",
                ))
            }
        }
    }
    fn SetWakeCallback(&mut self, callback: URLWakeCallback) {
        *self.wake.lock().expect("file wake poisoned") = Some(callback.clone());
        callback();
    }
}
