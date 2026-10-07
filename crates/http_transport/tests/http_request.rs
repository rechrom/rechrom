#[cfg(test)]
mod tests {
    use http_transport::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::{io, thread, time::Duration};

    fn wait(operation: &mut dyn HTTPRequestOperation) -> io::Result<HTTPResponseData> {
        for _ in 0..500 {
            if let Some(response) = operation.Poll()? {
                return Ok(response);
            }
            thread::sleep(Duration::from_millis(10));
        }
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "HTTP test timed out",
        ))
    }

    #[test]
    fn request_operation_preserves_redirects_methods_and_size_limit() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            for _ in 0..4 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut bytes = Vec::new();
                let header_end = loop {
                    let mut chunk = [0_u8; 1024];
                    let size = stream.read(&mut chunk).unwrap();
                    assert!(size > 0);
                    bytes.extend_from_slice(&chunk[..size]);
                    if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                        break end + 4;
                    }
                };
                let headers = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length: ")
                            .and_then(|n| n.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                while bytes.len() - header_end < content_length {
                    let mut chunk = [0_u8; 1024];
                    let size = stream.read(&mut chunk).unwrap();
                    assert!(size > 0);
                    bytes.extend_from_slice(&chunk[..size]);
                }
                let first = headers.lines().next().unwrap();
                let response = if first.starts_with("GET /start ") {
                    "HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned()
                } else if first.starts_with("GET /final ") {
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello".to_owned()
                } else if first.starts_with("POST /echo ") {
                    assert_eq!(&bytes[header_end..], &[0, 255, 1]);
                    "HTTP/1.1 201 Created\r\nContent-Length: 3\r\nConnection: close\r\n\r\nnew"
                        .to_owned()
                } else if first.starts_with("GET /large ") {
                    "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nlarge"
                        .to_owned()
                } else {
                    panic!("unexpected request: {first}");
                };
                stream.write_all(response.as_bytes()).unwrap();
            }
        });

        let base = format!("http://127.0.0.1:{port}");
        let response = GetResponse(&format!("{base}/start")).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.final_url, format!("{base}/final"));
        assert_eq!(response.mime_type, "text/plain");
        assert_eq!(response.body, b"hello");

        let mut transport = HTTPRequestTransport::new(HTTPTransportOptions::default()).unwrap();
        let mut post = transport
            .Start(&HTTPRequestData {
                method: "POST".to_owned(),
                url: format!("{base}/echo"),
                body: vec![0, 255, 1],
                ..HTTPRequestData::default()
            })
            .unwrap();
        let response = wait(&mut *post).unwrap();
        assert_eq!(response.status, 201);
        assert_eq!(response.body, b"new");

        let mut options = HTTPTransportOptions::default();
        options.max_response_bytes = 2;
        options.max_retries = 0;
        let mut limited = StartHTTPRequest(
            &HTTPRequestData {
                method: "GET".to_owned(),
                url: format!("{base}/large"),
                ..HTTPRequestData::default()
            },
            &options,
        )
        .unwrap();
        assert!(wait(&mut *limited)
            .unwrap_err()
            .to_string()
            .contains("size limit"));
        server.join().unwrap();
    }
}
