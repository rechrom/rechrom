use http_transport::*;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    (listener, url)
}

fn accept(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                // macOS accepts inherit the listener's nonblocking mode.
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                return stream;
            }
            Err(error)
                if error.kind() == io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(2))
            }
            Err(error) => panic!("HTTP server accept failed: {error}"),
        }
    }
}

fn request(stream: &mut TcpStream) -> (String, Vec<u8>) {
    let mut bytes = Vec::new();
    let end = loop {
        let mut byte = [0];
        stream.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
        if bytes.ends_with(b"\r\n\r\n") {
            break bytes.len();
        }
    };
    let headers = String::from_utf8(bytes[..end].to_vec()).unwrap();
    let length = headers
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .and_then(|value| value.trim().parse::<usize>().ok())
        })
        .unwrap_or(0);
    let mut body = vec![0; length];
    stream.read_exact(&mut body).unwrap();
    (headers, body)
}

fn response(stream: &mut TcpStream, body: &[u8], extra: &str) {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{extra}\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(body).unwrap();
}

fn wait(operation: &mut dyn HTTPRequestOperation) -> io::Result<HTTPResponseData> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(response) = operation.Poll()? {
            return Ok(response);
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "test deadline"));
        }
        thread::sleep(Duration::from_millis(2));
    }
}

fn start(transport: &mut HTTPRequestTransport, url: &str) -> Box<dyn HTTPRequestOperation> {
    transport
        .Start(&HTTPRequestData {
            method: "GET".into(),
            url: url.into(),
            ..Default::default()
        })
        .unwrap()
}

#[test]
fn reuses_connection_and_preserves_explicit_headers() {
    let (listener, url) = listener();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        for _ in 0..2 {
            let (headers, _) = request(&mut stream);
            let lower = headers.to_ascii_lowercase();
            assert!(lower.contains("user-agent: custom-agent\r\n"));
            assert!(lower.contains("accept-language: zh-cn\r\n"));
            assert!(lower.contains("accept-encoding: gzip, deflate, br\r\n"));
            response(&mut stream, b"pooled", "Content-Type: text/plain\r\n");
        }
    });
    let mut transport = HTTPRequestTransport::new(Default::default()).unwrap();
    for _ in 0..2 {
        let mut operation = transport
            .Start(&HTTPRequestData {
                method: "GET".into(),
                url: url.clone(),
                headers: vec![
                    ("User-Agent".into(), "custom-agent".into()),
                    ("Accept-Language".into(), "zh-CN".into()),
                ],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(wait(&mut *operation).unwrap().body, b"pooled");
    }
    server.join().unwrap();
}

#[test]
fn chunked_gzip_and_trailers_are_decoded() {
    let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gzip.write_all("天气预报".as_bytes()).unwrap();
    let compressed = gzip.finish().unwrap();
    let (listener, url) = listener();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        request(&mut stream);
        stream.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Encoding: gzip\r\nTrailer: X-End\r\n\r\n").unwrap();
        for chunk in compressed.chunks(3) {
            write!(stream, "{:x}\r\n", chunk.len()).unwrap();
            stream.write_all(chunk).unwrap();
            stream.write_all(b"\r\n").unwrap();
        }
        stream.write_all(b"0\r\nX-End: yes\r\n\r\n").unwrap();
    });
    assert_eq!(GetResponse(&url).unwrap().body, "天气预报".as_bytes());
    server.join().unwrap();
}

#[test]
fn deflate_brotli_and_decoded_size_limits() {
    let text = b"compressed response";
    let mut zlib = flate2::write::ZlibEncoder::new(Vec::new(), Default::default());
    zlib.write_all(text).unwrap();
    let mut raw = flate2::write::DeflateEncoder::new(Vec::new(), Default::default());
    raw.write_all(text).unwrap();
    let mut brotli = Vec::new();
    {
        let mut encoder = brotli::CompressorWriter::new(&mut brotli, 4096, 4, 22);
        encoder.write_all(text).unwrap();
    }
    for (encoding, bytes) in [
        ("deflate", zlib.finish().unwrap()),
        ("deflate", raw.finish().unwrap()),
        ("br", brotli),
    ] {
        let (listener, url) = listener();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            request(&mut stream);
            response(
                &mut stream,
                &bytes,
                &format!("Content-Encoding: {encoding}\r\n"),
            );
        });
        assert_eq!(GetResponse(&url).unwrap().body, text);
        server.join().unwrap();
    }
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Default::default());
    encoder.write_all(&vec![b'x'; 10_000]).unwrap();
    let bytes = encoder.finish().unwrap();
    assert!(bytes.len() < 100);
    let (listener, url) = listener();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        request(&mut stream);
        response(&mut stream, &bytes, "Content-Encoding: gzip\r\n");
    });
    let mut operation = StartHTTPRequest(
        &HTTPRequestData {
            method: "GET".into(),
            url,
            ..Default::default()
        },
        &HTTPTransportOptions {
            max_response_bytes: 100,
            max_retries: 0,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(wait(&mut *operation)
        .unwrap_err()
        .to_string()
        .contains("size limit"));
    server.join().unwrap();
}

#[test]
fn redirects_strip_cross_origin_credentials_and_reject_other_protocols() {
    let (destination, destination_url) = listener();
    let (source, source_url) = listener();
    let target = destination_url.clone();
    let source_server = thread::spawn(move || {
        let mut stream = accept(&source);
        request(&mut stream);
        write!(stream,"HTTP/1.1 302 Found\r\nLocation: {target}/final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
    });
    let target_server = thread::spawn(move || {
        let mut stream = accept(&destination);
        let (headers, _) = request(&mut stream);
        let lower = headers.to_ascii_lowercase();
        assert!(!lower.contains("authorization:"));
        assert!(!lower.contains("cookie:"));
        response(&mut stream, b"redirected", "Connection: close\r\n");
    });
    let mut operation = StartHTTPRequest(
        &HTTPRequestData {
            method: "GET".into(),
            url: source_url,
            headers: vec![
                ("Authorization".into(), "Bearer private".into()),
                ("Cookie".into(), "session=private".into()),
            ],
            ..Default::default()
        },
        &Default::default(),
    )
    .unwrap();
    let result = wait(&mut *operation).unwrap();
    assert_eq!(result.final_url, format!("{destination_url}/final"));
    assert_eq!(result.body, b"redirected");
    source_server.join().unwrap();
    target_server.join().unwrap();
    let (source, url) = listener();
    let server = thread::spawn(move || {
        let mut stream = accept(&source);
        request(&mut stream);
        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nLocation: file:///etc/hosts\r\nContent-Length: 0\r\n\r\n",
            )
            .unwrap();
    });
    assert!(GetResponse(&url)
        .unwrap_err()
        .to_string()
        .contains("expected HTTP(S)"));
    server.join().unwrap();
}

#[test]
fn retries_get_after_truncated_response() {
    let (listener, url) = listener();
    let server = thread::spawn(move || {
        let mut first = accept(&listener);
        request(&mut first);
        first
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nx")
            .unwrap();
        drop(first);
        let mut second = accept(&listener);
        request(&mut second);
        response(&mut second, b"retry", "Connection: close\r\n");
    });
    assert_eq!(GetResponse(&url).unwrap().body, b"retry");
    server.join().unwrap();
}

#[test]
fn stall_and_total_timeout_are_distinct() {
    for stall in [true, false] {
        let (listener, url) = listener();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            request(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nx")
                .unwrap();
            for _ in 0..10 {
                thread::sleep(Duration::from_millis(if stall { 150 } else { 20 }));
                if stream.write_all(b"x").is_err() {
                    break;
                }
            }
        });
        let mut operation = StartHTTPRequest(
            &HTTPRequestData {
                method: "GET".into(),
                url,
                ..Default::default()
            },
            &HTTPTransportOptions {
                stall_timeout_ms: if stall { 60 } else { 1000 },
                timeout_ms: if stall { 1000 } else { 80 },
                max_retries: 0,
                ..Default::default()
            },
        )
        .unwrap();
        let error = wait(&mut *operation).unwrap_err().to_string();
        assert!(
            error.contains(if stall {
                "no byte progress"
            } else {
                "request timeout"
            }),
            "{error}"
        );
        server.join().unwrap();
    }
}

#[test]
fn cancelling_an_active_request_releases_the_parallel_slot() {
    let (listener, url) = listener();
    let (ready_tx, ready_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut first = accept(&listener);
        request(&mut first);
        ready_tx.send(()).unwrap();
        let mut byte = [0];
        assert_eq!(
            first.read(&mut byte).unwrap(),
            0,
            "cancel should close the active HTTP/1 connection"
        );
        let mut second = accept(&listener);
        request(&mut second);
        response(&mut second, b"next", "Connection: close\r\n");
    });
    let mut transport = HTTPRequestTransport::new(HTTPTransportOptions {
        max_parallel_requests: 1,
        max_retries: 0,
        ..Default::default()
    })
    .unwrap();
    let first = start(&mut transport, &url);
    ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    let mut second = start(&mut transport, &url);
    drop(first);
    assert_eq!(wait(&mut *second).unwrap().body, b"next");
    server.join().unwrap();
}

#[test]
fn custom_method_redirects_match_curl_body_replay_rules() {
    for method in ["POST", "PUT", "GET"] {
        for status in [301, 302, 303, 307, 308] {
            let (listener, url) = listener();
            let expected = method.to_owned();
            let server = thread::spawn(move || {
                let mut stream = accept(&listener);
                let (headers, body) = request(&mut stream);
                assert!(headers.starts_with(&format!("{expected} /start HTTP/1.1\r\n")));
                assert_eq!(body, b"abc");
                assert!(headers
                    .to_ascii_lowercase()
                    .contains("content-type: application/x-www-form-urlencoded\r\n"));
                write!(stream,"HTTP/1.1 {status} Redirect\r\nLocation: /final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                drop(stream);
                let mut stream = accept(&listener);
                let (headers, body) = request(&mut stream);
                assert!(headers.starts_with(&format!("{expected} /final HTTP/1.1\r\n")));
                assert_eq!(
                    body,
                    if status >= 307 {
                        b"abc".as_slice()
                    } else {
                        b"".as_slice()
                    }
                );
                response(&mut stream, b"redirected", "Connection: close\r\n");
            });
            let mut operation = StartHTTPRequest(
                &HTTPRequestData {
                    method: method.into(),
                    url: format!("{url}/start"),
                    body: b"abc".to_vec(),
                    ..Default::default()
                },
                &Default::default(),
            )
            .unwrap();
            assert_eq!(wait(&mut *operation).unwrap().body, b"redirected");
            server.join().unwrap();
        }
    }
}

#[test]
fn post_failure_is_not_retried_and_http_errors_keep_their_body() {
    let (listener, url) = listener();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        request(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\nbroken")
            .unwrap();
        drop(stream);
        // An unsafe POST retry would arrive before the following explicit GET.
        let mut stream = accept(&listener);
        let (headers, _) = request(&mut stream);
        assert!(headers.starts_with("GET /error HTTP/1.1\r\n"));
        stream
            .write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 7\r\nConnection: close\r\n\r\nmissing",
            )
            .unwrap();
    });
    let mut transport = HTTPRequestTransport::new(Default::default()).unwrap();
    let mut post = transport
        .Start(&HTTPRequestData {
            method: "POST".into(),
            url: format!("{url}/post"),
            body: b"mutation".to_vec(),
            ..Default::default()
        })
        .unwrap();
    assert!(wait(&mut *post).is_err());
    let mut get = start(&mut transport, &format!("{url}/error"));
    let response = wait(&mut *get).unwrap();
    assert_eq!(response.status, 404);
    assert_eq!(response.body, b"missing");
    server.join().unwrap();
}

#[test]
fn cancelling_a_queued_request_never_sends_it() {
    let (listener, url) = listener();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut first = accept(&listener);
        let (headers, _) = request(&mut first);
        assert!(headers.starts_with("GET /first HTTP/1.1\r\n"));
        ready_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        response(&mut first, b"first", "Connection: close\r\n");
        drop(first);
        let mut second = accept(&listener);
        let (headers, _) = request(&mut second);
        assert!(headers.starts_with("GET /third HTTP/1.1\r\n"));
        response(&mut second, b"third", "Connection: close\r\n");
    });
    let mut transport = HTTPRequestTransport::new(HTTPTransportOptions {
        max_parallel_requests: 1,
        max_retries: 0,
        ..Default::default()
    })
    .unwrap();
    let mut first = start(&mut transport, &format!("{url}/first"));
    ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    let second = start(&mut transport, &format!("{url}/second"));
    let mut third = start(&mut transport, &format!("{url}/third"));
    drop(second);
    release_tx.send(()).unwrap();
    assert_eq!(wait(&mut *first).unwrap().body, b"first");
    assert_eq!(wait(&mut *third).unwrap().body, b"third");
    server.join().unwrap();
}
