//! Streaming HTTP delivery. Both compressed input and decoded output are bounded.
//! Blocking codec readers run on the executor's blocking pool, never its IO workers.
#![allow(non_snake_case)]
use crate::http_request::{
    fetch_response, network_error, protocol_error, runtime, with_stall_timeout, HTTPRequestData,
    HTTPRequestTransport, ResponseStart, Scheduler, TransferError,
};
use http_body_util::BodyExt;
use std::{
    io::{self, BufRead, Read},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;

const CHUNK_BYTES: usize = 16 * 1024;
const QUEUE_CHUNKS: usize = 8;
pub type HTTPWakeCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Debug)]
pub enum HTTPStreamEvent {
    Response {
        status: i64,
        final_url: String,
        content_type: String,
    },
    Data(Vec<u8>),
    Finished,
}
pub trait HTTPStreamOperation {
    fn Poll(&mut self, max_bytes: usize) -> io::Result<Option<HTTPStreamEvent>>;
    fn SetWakeCallback(&mut self, callback: HTTPWakeCallback);
}
type Wake = Arc<Mutex<Option<HTTPWakeCallback>>>;
type Event = Result<HTTPStreamEvent, String>;
fn wake(wake: &Wake) {
    let callback = wake.lock().expect("HTTP wake poisoned").clone();
    if let Some(callback) = callback {
        callback();
    }
}
async fn send(tx: &mpsc::Sender<Event>, event: Event, notify: &Wake) -> Result<(), TransferError> {
    tx.send(event)
        .await
        .map_err(|_| protocol_error("HTTP stream cancelled"))?;
    wake(notify);
    Ok(())
}

struct Stream {
    receiver: mpsc::Receiver<Event>,
    task: tokio::task::JoinHandle<()>,
    pending: Vec<u8>,
    offset: usize,
    wake: Wake,
    terminal: bool,
}
impl Drop for Stream {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl HTTPStreamOperation for Stream {
    fn Poll(&mut self, max_bytes: usize) -> io::Result<Option<HTTPStreamEvent>> {
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
            return Ok(Some(HTTPStreamEvent::Data(data)));
        }
        if self.terminal {
            return Ok(None);
        }
        match self.receiver.try_recv() {
            Ok(Ok(HTTPStreamEvent::Data(data))) if data.len() > max_bytes => {
                self.pending = data;
                self.Poll(max_bytes)
            }
            Ok(Ok(event)) => {
                self.terminal = matches!(event, HTTPStreamEvent::Finished);
                Ok(Some(event))
            }
            Ok(Err(error)) => {
                self.terminal = true;
                Err(io::Error::other(error))
            }
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                self.terminal = true;
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "HTTP stream ended without completion",
                ))
            }
        }
    }
    fn SetWakeCallback(&mut self, callback: HTTPWakeCallback) {
        *self.wake.lock().expect("HTTP wake poisoned") = Some(callback);
        // Also wake for events queued before the callback was installed.
        if !self.receiver.is_empty() {
            wake(&self.wake);
        }
    }
}

struct QueueReader {
    receiver: mpsc::Receiver<Result<bytes::Bytes, String>>,
    chunk: bytes::Bytes,
}
impl Read for QueueReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        while self.chunk.is_empty() {
            match self.receiver.blocking_recv() {
                Some(Ok(chunk)) => self.chunk = chunk,
                Some(Err(error)) => return Err(io::Error::other(error)),
                None => return Ok(0),
            }
        }
        let size = output.len().min(self.chunk.len());
        output[..size].copy_from_slice(&self.chunk[..size]);
        self.chunk = self.chunk.slice(size..);
        Ok(size)
    }
}
struct LimitedReader {
    inner: Box<dyn Read + Send>,
    total: usize,
    limit: usize,
}
impl Read for LimitedReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let size = self.inner.read(output)?;
        self.total = self
            .total
            .checked_add(size)
            .ok_or_else(|| io::Error::other("response size overflow"))?;
        if self.total > self.limit {
            return Err(io::Error::other("resource exceeds configured size limit"));
        }
        Ok(size)
    }
}
fn decode(
    input: QueueReader,
    encodings: String,
    limit: usize,
    tx: mpsc::Sender<Event>,
    notify: Wake,
) -> io::Result<()> {
    let mut reader: Box<dyn Read + Send> = Box::new(input);
    for encoding in encodings
        .split(',')
        .rev()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        reader = match encoding.to_ascii_lowercase().as_str() {
            "identity" => reader,
            "gzip" | "x-gzip" => Box::new(flate2::read::MultiGzDecoder::new(reader)),
            "deflate" => {
                let mut buffered = io::BufReader::new(reader);
                let first = buffered.fill_buf()?;
                // fill_buf may contain a single byte from a fragmented header.
                let mut prefix = [0u8; 2];
                let count = if first.is_empty() {
                    0
                } else {
                    let mut count = 0;
                    while count < 2 {
                        let n = buffered.read(&mut prefix[count..])?;
                        if n == 0 {
                            break;
                        }
                        count += n;
                    }
                    count
                };
                let zlib =
                    count == 2 && prefix[0] & 15 == 8 && u16::from_be_bytes(prefix) % 31 == 0;
                let restored = io::Cursor::new(prefix[..count].to_vec()).chain(buffered);
                if zlib {
                    Box::new(flate2::read::ZlibDecoder::new(restored))
                } else {
                    Box::new(flate2::read::DeflateDecoder::new(restored))
                }
            }
            "br" => Box::new(brotli::Decompressor::new(reader, 4096)),
            other => {
                return Err(io::Error::other(format!(
                    "unsupported HTTP content encoding: {other}"
                )))
            }
        };
        reader = Box::new(LimitedReader {
            inner: reader,
            total: 0,
            limit,
        });
    }
    let mut reader = LimitedReader {
        inner: reader,
        total: 0,
        limit,
    };
    let mut output = [0u8; CHUNK_BYTES];
    loop {
        let size = reader.read(&mut output)?;
        if size == 0 {
            return Ok(());
        }
        tx.blocking_send(Ok(HTTPStreamEvent::Data(output[..size].to_vec())))
            .map_err(|_| io::Error::other("HTTP stream cancelled"))?;
        wake(&notify);
    }
}
async fn transfer(
    request: &HTTPRequestData,
    scheduler: &Scheduler,
    tx: &mpsc::Sender<Event>,
    notify: &Wake,
    published: &mut bool,
) -> Result<(), TransferError> {
    let ResponseStart {
        mut incoming,
        status,
        final_url,
        mime_type,
        encodings,
        head_only,
    } = fetch_response(request, scheduler).await?;
    send(
        tx,
        Ok(HTTPStreamEvent::Response {
            status,
            final_url,
            content_type: mime_type,
        }),
        notify,
    )
    .await?;
    *published = true;
    let (input_tx, input_rx) = mpsc::channel(QUEUE_CHUNKS);
    let output_tx = tx.clone();
    let output_wake = notify.clone();
    let limit = scheduler.options.max_response_bytes;
    let codec = runtime().map_err(protocol_error)?.spawn_blocking(move || {
        decode(
            QueueReader {
                receiver: input_rx,
                chunk: bytes::Bytes::new(),
            },
            if head_only { String::new() } else { encodings },
            limit,
            output_tx,
            output_wake,
        )
    });
    let mut encoded_bytes = 0usize;
    let received = async {
        loop {
            // Wait for decoder input space before beginning a network read.
            // A full bounded queue is consumer backpressure, not a network
            // stall; Chromium likewise waits for a writable body pipe first.
            let Ok(permit) = input_tx.reserve().await else {
                return Ok(());
            };
            let mut permit = Some(permit);
            let frame = with_stall_timeout(
                async {
                    incoming
                        .frame()
                        .await
                        .transpose()
                        .map_err(|e| network_error(&e))
                },
                &scheduler.options,
            )
            .await
            .map_err(|error| {
                if error.message == "transfer made no byte progress" {
                    eprintln!("http-stream-stall url={} encoded_bytes={} input_capacity={} output_capacity={}",
                        request.url, encoded_bytes, input_tx.capacity(), tx.capacity());
                }
                error
            })?;
            let Some(frame) = frame else { break; };
            if let Ok(chunk) = frame.into_data() {
                encoded_bytes = encoded_bytes
                    .checked_add(chunk.len())
                    .ok_or_else(|| protocol_error("response size overflow"))?;
                if encoded_bytes > limit {
                    return Err(protocol_error("resource exceeds configured size limit"));
                }
                for offset in (0..chunk.len()).step_by(CHUNK_BYTES) {
                    let part = chunk.slice(offset..chunk.len().min(offset + CHUNK_BYTES));
                    if let Some(permit) = permit.take() {
                        permit.send(Ok(part));
                    } else if input_tx.send(Ok(part)).await.is_err() {
                        return Ok(());
                    }
                }
            }
        }
        Ok::<_, TransferError>(())
    }
    .await;
    if let Err(error) = &received {
        let _ = input_tx.send(Err(error.message.clone())).await;
    }
    drop(input_tx);
    codec
        .await
        .map_err(protocol_error)?
        .map_err(protocol_error)?;
    received?;
    send(tx, Ok(HTTPStreamEvent::Finished), notify).await
}
impl HTTPRequestTransport {
    pub fn StartStream(
        &mut self,
        request: &HTTPRequestData,
    ) -> io::Result<Box<dyn HTTPStreamOperation>> {
        let (tx, receiver) = mpsc::channel(QUEUE_CHUNKS);
        let notify: Wake = Arc::new(Mutex::new(None));
        let worker_wake = notify.clone();
        let request = request.clone();
        let scheduler = self.scheduler.clone();
        let task = runtime()?.spawn(async move {
            let Ok(_slot) = scheduler.slots.clone().acquire_owned().await else {
                return;
            };
            let mut published = false;
            for attempt in 0.. {
                let future = transfer(&request, &scheduler, &tx, &worker_wake, &mut published);
                let result = if scheduler.options.timeout_ms > 0 {
                    tokio::time::timeout(
                        Duration::from_millis(scheduler.options.timeout_ms as u64),
                        future,
                    )
                    .await
                    .unwrap_or_else(|_| Err(TransferError::new("HTTP request timeout", true)))
                } else {
                    future.await
                };
                match result {
                    Ok(()) => return,
                    Err(error)
                        if !published
                            && matches!(request.method.as_str(), "GET" | "HEAD")
                            && error.retryable
                            && attempt < scheduler.options.max_retries =>
                    {
                        continue
                    }
                    Err(error) => {
                        let _ = send(
                            &tx,
                            Err(format!("{}: {}", request.url, error.message)),
                            &worker_wake,
                        )
                        .await;
                        return;
                    }
                }
            }
        });
        Ok(Box::new(Stream {
            receiver,
            task,
            pending: vec![],
            offset: 0,
            wake: notify,
            terminal: false,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{BufRead, Write},
        net::TcpListener,
        sync::mpsc as sync_mpsc,
        time::Instant,
    };
    fn poll(stream: &mut dyn HTTPStreamOperation, max: usize) -> HTTPStreamEvent {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(event) = stream.Poll(max).unwrap() {
                return event;
            }
            assert!(Instant::now() < deadline, "stream event timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    #[test]
    fn headers_and_gzip_output_arrive_before_network_eof() {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"<body>early</body>").unwrap();
        encoder.flush().unwrap();
        let first = encoder.get_ref().clone();
        encoder.write_all(b"<p>later</p>").unwrap();
        let whole = encoder.finish().unwrap();
        let last = whole[first.len()..].to_vec();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let (release, wait) = sync_mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut input = io::BufReader::new(socket.try_clone().unwrap());
            loop {
                let mut line = String::new();
                input.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
            }
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Encoding: gzip\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").unwrap();
            write!(socket, "{:x}\r\n", first.len()).unwrap();
            socket.write_all(&first).unwrap();
            socket.write_all(b"\r\n").unwrap();
            wait.recv_timeout(Duration::from_secs(5)).unwrap();
            write!(socket, "{:x}\r\n", last.len()).unwrap();
            socket.write_all(&last).unwrap();
            socket.write_all(b"\r\n0\r\n\r\n").unwrap();
        });
        let mut transport = HTTPRequestTransport::new(crate::HTTPTransportOptions {
            max_retries: 0,
            ..Default::default()
        })
        .unwrap();
        let mut stream = transport
            .StartStream(&HTTPRequestData {
                method: "GET".into(),
                url,
                ..Default::default()
            })
            .unwrap();
        assert!(matches!(
            poll(&mut *stream, 3),
            HTTPStreamEvent::Response { status: 200, .. }
        ));
        let mut body = Vec::new();
        while !body.ends_with(b"</body>") {
            match poll(&mut *stream, 3) {
                HTTPStreamEvent::Data(data) => {
                    assert!(data.len() <= 3);
                    body.extend(data);
                }
                other => panic!("unexpected early event: {other:?}"),
            }
        }
        assert_eq!(body, b"<body>early</body>");
        assert!(stream.Poll(3).unwrap().is_none());
        release.send(()).unwrap();
        loop {
            match poll(&mut *stream, 3) {
                HTTPStreamEvent::Data(data) => body.extend(data),
                HTTPStreamEvent::Finished => break,
                other => panic!("unexpected event: {other:?}"),
            }
        }
        assert_eq!(body, b"<body>early</body><p>later</p>");
        assert!(stream.Poll(3).unwrap().is_none());
        server.join().unwrap();
    }
    #[test]
    fn compressed_streams_preserve_bytes_and_fail_size_limits() {
        let plain = b"<body>abcdefghijklmnopqrstuvwxyz</body>";
        for encoding in ["gzip", "deflate", "raw-deflate", "br", "gzip, br"] {
            let bytes = match encoding {
                "gzip" | "gzip, br" => {
                    let mut w = flate2::write::GzEncoder::new(vec![], Default::default());
                    w.write_all(plain).unwrap();
                    w.finish().unwrap()
                }
                "deflate" => {
                    let mut w = flate2::write::ZlibEncoder::new(vec![], Default::default());
                    w.write_all(plain).unwrap();
                    w.finish().unwrap()
                }
                "raw-deflate" => {
                    let mut w = flate2::write::DeflateEncoder::new(vec![], Default::default());
                    w.write_all(plain).unwrap();
                    w.finish().unwrap()
                }
                _ => plain.to_vec(),
            };
            let bytes = if matches!(encoding, "br" | "gzip, br") {
                let mut output = vec![];
                {
                    let mut w = brotli::CompressorWriter::new(&mut output, 4096, 3, 22);
                    w.write_all(&bytes).unwrap();
                }
                output
            } else {
                bytes
            };
            for limit in [4096, 8] {
                let (tx, rx) = mpsc::channel(8);
                let (out, mut output) = mpsc::channel(8);
                let encoding = if encoding == "raw-deflate" {
                    "deflate"
                } else {
                    encoding
                }
                .to_owned();
                let codec = runtime().unwrap().spawn_blocking(move || {
                    decode(
                        QueueReader {
                            receiver: rx,
                            chunk: bytes::Bytes::new(),
                        },
                        encoding,
                        limit,
                        out,
                        Arc::new(Mutex::new(None)),
                    )
                });
                runtime().unwrap().block_on(async {
                    // Fragment even the two-byte zlib header across deliveries.
                    let input_bytes = bytes.clone();
                    let feeder = tokio::spawn(async move {
                        for byte in input_bytes {
                            if tx
                                .send(Ok(bytes::Bytes::copy_from_slice(&[byte])))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    });
                    let mut decoded = vec![];
                    while let Some(event) = output.recv().await {
                        if let Ok(HTTPStreamEvent::Data(data)) = event {
                            decoded.extend(data);
                        }
                    }
                    let result = codec.await.unwrap();
                    feeder.await.unwrap();
                    if limit == 4096 {
                        result.unwrap();
                        assert_eq!(decoded, plain);
                    } else {
                        assert!(result.is_err());
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use std::{
        io::{BufRead, Write},
        net::TcpListener,
        time::Instant,
    };
    #[test]
    fn dropping_an_unfinished_stream_releases_the_only_network_slot() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let read = |socket: &std::net::TcpStream| {
                let mut reader = io::BufReader::new(socket.try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                }
            };
            let (mut first, _) = listener.accept().unwrap();
            read(&first);
            first
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100000\r\n\r\nunfinished")
                .unwrap();
            let (mut second, _) = listener.accept().unwrap();
            read(&second);
            second
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .unwrap();
        });
        let mut transport = HTTPRequestTransport::new(crate::HTTPTransportOptions {
            max_parallel_requests: 1,
            max_retries: 0,
            ..Default::default()
        })
        .unwrap();
        let request = HTTPRequestData {
            method: "GET".into(),
            url,
            ..Default::default()
        };
        let mut first = transport.StartStream(&request).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while first.Poll(16).unwrap().is_none() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        drop(first);
        let mut second = transport.StartStream(&request).unwrap();
        let mut body = vec![];
        loop {
            match second.Poll(16).unwrap() {
                Some(HTTPStreamEvent::Data(data)) => body.extend(data),
                Some(HTTPStreamEvent::Finished) => break,
                _ => {
                    assert!(Instant::now() < deadline);
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        assert_eq!(body, b"ok");
        server.join().unwrap();
    }
}
